//! SPIKE ONLY. Windowed public synthetic scene; no production capsule/release code.
//! Frame intervals measure presented application cadence, not GPU execution time.

mod input;
mod protocol;
mod quality;

use input::InputDriver;
use protocol::{
    Options, Samples, adapter_json, check_dimensions, emit, native_dpi, select_adapter, statistics,
};
use serde_json::json;
use std::{
    error::Error,
    sync::Arc,
    time::{Duration, Instant},
};

use quality::Quality;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

struct Gpu {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    params: wgpu::Buffer,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    scene: wgpu::TextureView,
    bindings: wgpu::BindGroup,
    atmosphere_bindings: wgpu::BindGroup,
    atmosphere: wgpu::RenderPipeline,
    glass: wgpu::RenderPipeline,
    adapter: wgpu::AdapterInfo,
}

impl Gpu {
    async fn new(window: Arc<Window>, options: &Options) -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window.clone())?;
        let adapters = instance.enumerate_adapters(wgpu::Backends::DX12 | wgpu::Backends::VULKAN);
        let infos: Vec<_> = adapters
            .iter()
            .map(|adapter| (adapter.get_info(), adapter.is_surface_supported(&surface)))
            .collect();
        emit(
            "adapter_inventory",
            json!({"adapters":infos.iter().map(|(info,_)|adapter_json(info)).collect::<Vec<_>>()}),
        );
        let selected = select_adapter(&infos, options.class, options.adapter_name.as_deref())?;
        let adapter = &adapters[selected];
        let info = adapter.get_info();
        if !options.class.matches(&info) {
            return Err("actual adapter class differs from requested class".into());
        }
        let actual_size = window.inner_size();
        native_dpi(&window)?;
        check_dimensions(
            options,
            actual_size.width,
            actual_size.height,
            window.scale_factor(),
        )?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Spike D"),
                ..Default::default()
            })
            .await?;
        let size = window.inner_size();
        let mut config = surface
            .get_default_config(adapter, size.width.max(1), size.height.max(1))
            .ok_or("no compatible surface configuration")?;
        config.present_mode = wgpu::PresentMode::Fifo;
        surface.configure(&device, &config);
        let params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("public animation parameters"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Spike D scene sample"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(32),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Spike D pipeline"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("experimental Temporal Glass"),
            source: wgpu::ShaderSource::Wgsl(include_str!("glass.wgsl").into()),
        });
        let pipeline = |entry, format| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(entry),
                layout: if entry == "atmosphere" {
                    None
                } else {
                    Some(&pipeline_layout)
                },
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vertex"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                multiview: None,
                cache: None,
            })
        };
        let atmosphere = pipeline("atmosphere", wgpu::TextureFormat::Rgba8Unorm);
        let atmosphere_bindings = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("atmosphere uniform only; no render target sampling"),
            layout: &atmosphere.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: params.as_entire_binding(),
            }],
        });
        let glass = pipeline("glass", config.format);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("scene diffusion sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let scene = Self::scene(&device, &config);
        let bindings = Self::bindings(&device, &layout, &params, &scene, &sampler);
        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            params,
            layout,
            sampler,
            scene,
            bindings,
            atmosphere_bindings,
            atmosphere,
            glass,
            adapter: info,
        })
    }

    fn scene(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::TextureView {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("public synthetic atmosphere"),
                size: wgpu::Extent3d {
                    width: config.width,
                    height: config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&Default::default())
    }

    fn bindings(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        params: &wgpu::Buffer,
        scene: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scene sampling bindings"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(scene),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        })
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        self.scene = Self::scene(&self.device, &self.config);
        self.bindings = Self::bindings(
            &self.device,
            &self.layout,
            &self.params,
            &self.scene,
            &self.sampler,
        );
    }

    fn draw(&self, elapsed: f32, quality: Quality, pointer: [f32; 2]) -> Result<bool> {
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.config);
                return Ok(false);
            }
            Err(wgpu::SurfaceError::Timeout) => return Ok(false),
            Err(error) => return Err(error.into()),
        };
        let values = [
            self.config.width as f32,
            self.config.height as f32,
            elapsed,
            quality.shader_value(),
            pointer[0],
            pointer[1],
            0.0,
            0.0,
        ];
        let bytes: Vec<u8> = values.into_iter().flat_map(f32::to_le_bytes).collect();
        self.queue.write_buffer(&self.params, 0, &bytes);
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        // Atmosphere must not bind its own render target as a sampled texture.
        // That entry point uses a separate uniform-only binding layout.
        for (target, pipeline, bindings) in [
            (&self.scene, &self.atmosphere, &self.atmosphere_bindings),
            (&view, &self.glass, &self.bindings),
        ] {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Spike D pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, bindings, &[]);
            pass.draw(0..3, 0..1);
        }
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        frame.present();
        Ok(true)
    }
}

struct Probe {
    options: Options,
    gpu: Option<Gpu>,
    quality: Quality,
    launched: Instant,
    warmup_start: Option<Instant>,
    last: Option<Instant>,
    samples: Samples,
    driver: InputDriver,
    pointer: [f32; 2],
    focused: bool,
    collecting: bool,
    windows: u32,
    failed: bool,
    completed: bool,
}

impl Probe {
    fn fail(&mut self, event_loop: &ActiveEventLoop, reason: impl ToString) {
        if !self.completed {
            let reason = reason.to_string();
            emit("failure", json!({"reason":reason}));
            emit(
                "complete",
                json!({"status":"FAIL","windows_collected":self.windows,"reason":reason}),
            );
            self.failed = true;
            self.completed = true;
        }
        event_loop.exit();
    }
    fn sample(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(reason) = self.samples.validate(self.options.drive_input) {
            self.fail(event_loop, reason);
            return;
        }
        let gpu = self.gpu.as_ref().unwrap();
        let measured_dpi = match native_dpi(&gpu.window) {
            Ok(value) => value,
            Err(reason) => {
                self.fail(event_loop, reason);
                return;
            }
        };
        let (average, p95) = statistics(&self.samples.intervals).unwrap();
        let (present_average, present_p95) = statistics(&self.samples.event_to_present).unwrap();
        let ack = statistics(&self.samples.acknowledgements);
        let next = self.quality.degrade(average);
        self.windows += 1;
        emit(
            "sample",
            json!({"index":self.windows,"frames":self.samples.intervals.len(),
            "width":gpu.config.width,"height":gpu.config.height,"actual_dpi_percent":gpu.window.scale_factor()*100.0,"native_dpi":measured_dpi,"dpi_awareness":"PerMonitorAware",
            "mode":self.quality.name(),"next_mode":next.name(),"interval_avg_ms":average,"interval_p95_ms":p95,
            "pointer_events":self.samples.pointer_events,"keyboard_events":self.samples.keyboard_events,
            "input_ack_count":self.samples.acknowledgements.len(),"input_ack_avg_ms":ack.map(|x|x.0),"input_ack_p95_ms":ack.map(|x|x.1),
            "event_to_present_count":self.samples.event_to_present.len(),"event_to_present_avg_ms":present_average,"event_to_present_p95_ms":present_p95}),
        );
        self.quality = next;
        self.samples = Samples::default();
        if self.windows == self.options.windows {
            emit(
                "complete",
                json!({"status":"PASS","windows_collected":self.windows,"reason":"machine observations complete; physical provenance and human responsiveness/foreground observations require separate acceptance"}),
            );
            self.completed = true;
            event_loop.exit();
        }
    }
}

impl ApplicationHandler for Probe {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }
        let result = event_loop.create_window(Window::default_attributes()
            .with_title("SPIKE D — public synthetic input probe; F8 harmless input; Escape fails incomplete cell")
            .with_resizable(false)
            .with_inner_size(PhysicalSize::new(self.options.width,self.options.height)))
            .map_err(|error| -> Box<dyn Error> { error.into() })
            .and_then(|window| pollster::block_on(Gpu::new(Arc::new(window), &self.options)));
        match result {
            Ok(gpu) => {
                let measured_dpi = match native_dpi(&gpu.window) {
                    Ok(value) => value,
                    Err(reason) => {
                        self.fail(event_loop, reason);
                        return;
                    }
                };
                emit(
                    "start",
                    json!({"requested_gpu_class":self.options.class.name(),"requested_dpi_percent":self.options.dpi,
                    "width":gpu.config.width,"height":gpu.config.height,"requested_mode":self.options.quality.name(),
                    "actual_dpi_percent":gpu.window.scale_factor()*100.0,"adapter":adapter_json(&gpu.adapter),
                    "native_dpi":measured_dpi,"dpi_awareness":"PerMonitorAware",
                    "drive_input":self.options.drive_input,"sample_windows":self.options.windows,"warmup_seconds":self.options.warmup,
                    "latency_definition":"CPU SendInput call to matching received winit event; event dispatch to frame submission/present call; not photon latency"}),
                );
                self.focused = gpu.window.has_focus();
                if self.focused {
                    self.warmup_start = Some(Instant::now());
                }
                gpu.window.request_redraw();
                self.gpu = Some(gpu);
            }
            Err(error) => self.fail(event_loop, error),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.launched.elapsed() >= Duration::from_secs(self.options.timeout) {
            self.fail(
                event_loop,
                "bounded cell timeout before required windows completed",
            );
            return;
        }
        // Keep timeout observable even when minimized/unfocused. No busy drawing
        // or input injection while another application has foreground focus.
        event_loop.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_millis(100),
        ));
        if self.focused
            && let Some(gpu) = &self.gpu
        {
            gpu.window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(gpu) = self.gpu.as_mut() else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => self.fail(event_loop, "window closed before completion"),
            WindowEvent::Resized(size) => {
                if let Err(reason) = check_dimensions(
                    &self.options,
                    size.width,
                    size.height,
                    gpu.window.scale_factor(),
                ) {
                    self.fail(event_loop, reason);
                    return;
                }
                gpu.resize(size);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if let Err(reason) = check_dimensions(
                    &self.options,
                    gpu.config.width,
                    gpu.config.height,
                    scale_factor,
                ) {
                    self.fail(event_loop, reason);
                }
            }
            WindowEvent::Focused(focused) => {
                self.focused = focused;
                if !focused && self.collecting {
                    self.fail(event_loop, "focus lost during a required sample window");
                    return;
                }
                self.last = None;
                self.samples = Samples::default();
                self.driver = InputDriver::default();
                self.warmup_start = focused.then(Instant::now);
                if focused {
                    gpu.window.request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.pointer = [
                    (position.x / f64::from(gpu.config.width)) as f32,
                    (position.y / f64::from(gpu.config.height)) as f32,
                ];
                let now = Instant::now();
                let acknowledgement = self.driver.acknowledge_pointer(position, now);
                if self.collecting {
                    self.samples.received(true, now);
                    if let Some(ms) = acknowledgement {
                        self.samples.acknowledgements.push(ms);
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                if event.logical_key == Key::Named(NamedKey::Escape) {
                    self.fail(event_loop, "Escape interrupted required cell");
                    return;
                }
                let now = Instant::now();
                let acknowledgement = if event.logical_key == Key::Named(NamedKey::F8) {
                    self.driver.acknowledge_key(now)
                } else {
                    None
                };
                if self.collecting {
                    self.samples.received(false, now);
                    if let Some(ms) = acknowledgement {
                        self.samples.acknowledgements.push(ms);
                    }
                }
            }
            WindowEvent::RedrawRequested if self.focused => {
                if let Err(reason) = native_dpi(&gpu.window) {
                    self.fail(event_loop, reason);
                    return;
                }
                if let Err(reason) = check_dimensions(
                    &self.options,
                    gpu.window.inner_size().width,
                    gpu.window.inner_size().height,
                    gpu.window.scale_factor(),
                ) {
                    self.fail(event_loop, reason);
                    return;
                }
                let now = Instant::now();
                if !self.collecting
                    && self.warmup_start.is_some_and(|start| {
                        now.duration_since(start) >= Duration::from_secs(self.options.warmup)
                    })
                {
                    self.collecting = true;
                    self.samples = Samples::default();
                    self.last = None;
                    self.driver = InputDriver::default();
                }
                if self.options.drive_input
                    && let Err(reason) = self.driver.pump(&gpu.window, now)
                {
                    self.fail(event_loop, reason);
                    return;
                }
                match gpu.draw(
                    self.launched.elapsed().as_secs_f32(),
                    self.quality,
                    self.pointer,
                ) {
                    Ok(true) => {
                        let now = Instant::now();
                        if self.collecting {
                            self.samples.presented(now);
                            if let Some(last) = self.last {
                                self.samples
                                    .intervals
                                    .push(now.duration_since(last).as_secs_f64() * 1000.0);
                            }
                            self.last = Some(now);
                            if self.samples.intervals.len() == 300 {
                                self.sample(event_loop);
                            }
                        }
                    }
                    Ok(false) => {
                        self.fail(
                            event_loop,
                            "surface acquisition failed; uninterrupted sample cannot be counted",
                        );
                        return;
                    }
                    Err(error) => {
                        self.fail(event_loop, error);
                        return;
                    }
                }
                if !self.completed {
                    self.gpu.as_ref().unwrap().window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(options) = Options::parse(&args)? else {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapters = instance.enumerate_adapters(wgpu::Backends::DX12 | wgpu::Backends::VULKAN);
        emit(
            "adapter_inventory",
            json!({"adapters":adapters.iter().map(|adapter|adapter_json(&adapter.get_info())).collect::<Vec<_>>()}),
        );
        return Ok(());
    };
    if !cfg!(windows) {
        return Err("physical matrix execution requires Windows; inventory and unit tests are available here".into());
    }
    let event_loop = EventLoop::new()?;
    let mut probe = Probe {
        quality: options.quality,
        options,
        gpu: None,
        launched: Instant::now(),
        warmup_start: None,
        last: None,
        samples: Samples::default(),
        driver: InputDriver::default(),
        pointer: [0.5, 0.5],
        focused: false,
        collecting: false,
        windows: 0,
        failed: false,
        completed: false,
    };
    event_loop.run_app(&mut probe)?;
    if probe.failed {
        return Err("GPU cell failed; inspect prior failure record".into());
    }
    if !probe.completed {
        return Err("event loop exited without completed evidence".into());
    }
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        emit("failure", json!({"reason":error.to_string()}));
        std::process::exit(1);
    }
}
