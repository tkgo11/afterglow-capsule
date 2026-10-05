//! SPIKE ONLY. Windowed public synthetic scene; no production capsule/release code.
//! Frame intervals measure presented application cadence, not GPU execution time.

mod quality;

use std::{error::Error, sync::Arc, time::Instant};

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
}

impl Gpu {
    async fn new(window: Arc<Window>) -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window.clone())?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
            })
            .await?;
        let info = adapter.get_info();
        println!(
            "adapter={info:?}; OS={}; scale={}",
            std::env::consts::OS,
            window.scale_factor()
        );
        if !matches!(
            info.device_type,
            wgpu::DeviceType::IntegratedGpu | wgpu::DeviceType::DiscreteGpu
        ) {
            return Err("reference hardware must be an integrated or discrete GPU; software/unknown adapters cannot satisfy Spike D".into());
        }
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Spike D"),
                ..Default::default()
            })
            .await?;
        let size = window.inner_size();
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
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
    size: PhysicalSize<u32>,
    gpu: Option<Gpu>,
    quality: Quality,
    start: Instant,
    last: Option<Instant>,
    frames: Vec<f64>,
    pointer: [f32; 2],
    inputs: u64,
    paused: bool,
    failed: bool,
}

impl ApplicationHandler for Probe {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }
        let result = event_loop.create_window(Window::default_attributes()
            .with_title("SPIKE D — public synthetic scene; 1 Full, 2 Reduced, 3 Opaque, Escape Exit")
            .with_inner_size(self.size))
            .map_err(|error| -> Box<dyn Error> { error.into() })
            .and_then(|window| pollster::block_on(Gpu::new(Arc::new(window))));
        match result {
            Ok(gpu) => {
                gpu.window.request_redraw();
                self.gpu = Some(gpu);
            }
            Err(error) => {
                eprintln!("Spike D: {error}");
                self.failed = true;
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(gpu) = self.gpu.as_mut() else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                self.paused = size.width == 0 || size.height == 0;
                gpu.resize(size);
                self.last = None;
                self.frames.clear();
                if !self.paused {
                    gpu.window.request_redraw();
                }
            }
            WindowEvent::Focused(focused) => {
                self.paused = !focused;
                self.last = None;
                self.frames.clear();
                if focused {
                    gpu.window.request_redraw();
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                println!(
                    "actual DPI scale={scale_factor}; rerun this matrix cell after DPI change"
                );
                self.last = None;
                self.frames.clear();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.pointer = [
                    (position.x / f64::from(gpu.config.width)) as f32,
                    (position.y / f64::from(gpu.config.height)) as f32,
                ];
                self.inputs += 1;
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                match event.logical_key {
                    Key::Named(NamedKey::Escape) => event_loop.exit(),
                    Key::Character(ref value) => {
                        match value.as_str() {
                            "1" => self.quality = Quality::Full,
                            "2" => self.quality = Quality::Reduced,
                            "3" => self.quality = Quality::Opaque,
                            _ => return,
                        }
                        self.frames.clear();
                        self.last = None;
                        self.inputs += 1;
                    }
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested if !self.paused => {
                match gpu.draw(
                    self.start.elapsed().as_secs_f32(),
                    self.quality,
                    self.pointer,
                ) {
                    Ok(true) => {
                        let now = Instant::now();
                        if let Some(last) = self.last {
                            self.frames
                                .push(now.duration_since(last).as_secs_f64() * 1000.0);
                        }
                        self.last = Some(now);
                        if self.frames.len() >= 300 {
                            let average =
                                self.frames.iter().sum::<f64>() / self.frames.len() as f64;
                            self.frames.sort_by(f64::total_cmp);
                            let p95 = self.frames[self.frames.len() * 95 / 100];
                            let next = self.quality.degrade(average);
                            println!(
                                "physical={}x{} DPI={} mode={:?} frames={} interval_avg_ms={average:.3} interval_p95_ms={p95:.3} input_events={} next={next:?}",
                                gpu.config.width,
                                gpu.config.height,
                                gpu.window.scale_factor(),
                                self.quality,
                                self.frames.len(),
                                self.inputs
                            );
                            self.quality = next;
                            self.frames.clear();
                        }
                    }
                    Ok(false) => {
                        self.last = None;
                    }
                    Err(error) => {
                        eprintln!("Spike D: {error}");
                        self.failed = true;
                        event_loop.exit();
                        return;
                    }
                }
                gpu.window.request_redraw();
            }
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        return Err("usage: spike-d width height full|reduced|opaque (set actual Windows display DPI separately)".into());
    }
    let width: u32 = args[1].parse()?;
    let height: u32 = args[2].parse()?;
    if !matches!((width, height), (960, 640) | (1440, 900) | (1920, 1080)) {
        return Err("unsupported spike matrix resolution".into());
    }
    let quality = match args[3].as_str() {
        "full" => Quality::Full,
        "reduced" => Quality::Reduced,
        "opaque" => Quality::Opaque,
        _ => return Err("unsupported effect mode".into()),
    };
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut probe = Probe {
        size: PhysicalSize::new(width, height),
        gpu: None,
        quality,
        start: Instant::now(),
        last: None,
        frames: Vec::new(),
        pointer: [0.5, 0.5],
        inputs: 0,
        paused: false,
        failed: false,
    };
    event_loop.run_app(&mut probe)?;
    if probe.failed {
        return Err("GPU spike did not run successfully".into());
    }
    Ok(())
}
