//! Versioned machine observations for this isolated spike only.
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Instant,
};

use crate::quality::Quality;

pub fn emit(event: &str, mut fields: Value) {
    let object = fields.as_object_mut().expect("event fields are an object");
    object.insert("format_name".into(), json!("afterglow-spike-d-event"));
    object.insert("format_version".into(), json!(2));
    object.insert("minimum_reader_version".into(), json!(2));
    object.insert("event".into(), json!(event));
    println!("{fields}");
}

pub fn adapter_json(info: &wgpu::AdapterInfo) -> Value {
    json!({"name":info.name,"vendor":info.vendor,"device":info.device,
        "backend":format!("{:?}", info.backend),"driver":info.driver,
        "driver_info":info.driver_info,"device_type":format!("{:?}", info.device_type)})
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuClass {
    Integrated,
    Discrete,
}
impl GpuClass {
    pub fn name(self) -> &'static str {
        match self {
            Self::Integrated => "integrated",
            Self::Discrete => "discrete",
        }
    }
    pub fn matches(self, info: &wgpu::AdapterInfo) -> bool {
        info.device_type
            == match self {
                Self::Integrated => wgpu::DeviceType::IntegratedGpu,
                Self::Discrete => wgpu::DeviceType::DiscreteGpu,
            }
    }
}

/// Select among explicit compatible adapters. Equivalent backend views are
/// deduplicated by physical identity, with DX12 preferred on Windows. Distinct
/// matching devices remain ambiguous instead of depending on enumeration order.
pub fn select_adapter(
    infos: &[(wgpu::AdapterInfo, bool)],
    class: GpuClass,
    name: Option<&str>,
) -> Result<usize, String> {
    let mut devices = BTreeMap::<(u32, u32, String), usize>::new();
    let mut seen_backends = BTreeSet::new();
    for (index, (info, compatible)) in infos.iter().enumerate() {
        if !compatible
            || !class.matches(info)
            || name.is_some_and(|filter| !info.name.to_lowercase().contains(&filter.to_lowercase()))
        {
            continue;
        }
        let key = (info.vendor, info.device, info.name.to_lowercase());
        if !seen_backends.insert((key.clone(), format!("{:?}", info.backend))) {
            return Err("multiple physical adapters have indistinguishable name/vendor/device on one backend; do not silently collapse them".into());
        }
        if let Some(previous) = devices.get_mut(&key) {
            let rank = |backend| match backend {
                wgpu::Backend::Dx12 => 0,
                wgpu::Backend::Vulkan => 1,
                _ => 2,
            };
            if rank(info.backend) < rank(infos[*previous].0.backend) {
                *previous = index;
            }
        } else {
            devices.insert(key, index);
        }
    }
    if devices.is_empty() {
        return Err(format!(
            "no surface-compatible {} adapter matches the requested class/name; no fallback is allowed",
            class.name()
        ));
    }
    if devices.len() != 1 {
        return Err(format!(
            "{} distinct {} adapters match; supply --adapter-name to resolve ambiguity",
            devices.len(),
            class.name()
        ));
    }
    Ok(*devices.values().next().unwrap())
}

#[derive(Clone, Debug)]
pub struct Options {
    pub width: u32,
    pub height: u32,
    pub quality: Quality,
    pub class: GpuClass,
    pub adapter_name: Option<String>,
    pub dpi: u32,
    pub windows: u32,
    pub warmup: u64,
    pub timeout: u64,
    pub drive_input: bool,
}
impl Options {
    pub fn parse(args: &[String]) -> Result<Option<Self>, String> {
        if args == ["--list-adapters"] {
            return Ok(None);
        }
        if args.len() < 3 {
            return Err("usage: WIDTH HEIGHT full|reduced|opaque --gpu integrated|discrete --expected-dpi 100|150|200 [--adapter-name SUBSTRING] [--sample-windows 3] [--warmup-seconds 3] [--timeout-seconds 120] [--drive-input]".into());
        }
        let width = args[0].parse().map_err(|_| "invalid width")?;
        let height = args[1].parse().map_err(|_| "invalid height")?;
        if !matches!((width, height), (960, 640) | (1440, 900) | (1920, 1080)) {
            return Err("unsupported matrix resolution".into());
        }
        let quality = match args[2].as_str() {
            "full" => Quality::Full,
            "reduced" => Quality::Reduced,
            "opaque" => Quality::Opaque,
            _ => return Err("unsupported effect mode".into()),
        };
        let mut fields = BTreeMap::new();
        let mut drive_input = false;
        let mut i = 3;
        while i < args.len() {
            let key = args[i].as_str();
            if key == "--drive-input" {
                if drive_input {
                    return Err("duplicate --drive-input".into());
                }
                drive_input = true;
                i += 1;
                continue;
            }
            if !matches!(
                key,
                "--gpu"
                    | "--expected-dpi"
                    | "--adapter-name"
                    | "--sample-windows"
                    | "--warmup-seconds"
                    | "--timeout-seconds"
            ) {
                return Err(format!("unknown option {key}"));
            }
            let value = args
                .get(i + 1)
                .ok_or_else(|| format!("missing value for {key}"))?;
            if fields.insert(key, value.as_str()).is_some() {
                return Err(format!("duplicate option {key}"));
            }
            i += 2;
        }
        let class = match fields.get("--gpu").copied() {
            Some("integrated") => GpuClass::Integrated,
            Some("discrete") => GpuClass::Discrete,
            _ => return Err("--gpu integrated|discrete is required".into()),
        };
        let number = |key, default| -> Result<u64, String> {
            fields.get(key).map_or(Ok(default), |v| {
                v.parse().map_err(|_| format!("invalid {key}"))
            })
        };
        let dpi = number("--expected-dpi", 0)?;
        if !matches!(dpi, 100 | 150 | 200) {
            return Err("--expected-dpi 100|150|200 is required".into());
        }
        let windows = number("--sample-windows", 3)?;
        let warmup = number("--warmup-seconds", 3)?;
        let timeout = number("--timeout-seconds", 120)?;
        if !(3..=10).contains(&windows)
            || !(3..=30).contains(&warmup)
            || !(30..=600).contains(&timeout)
            || timeout <= warmup
        {
            return Err(
                "sampling bounds: windows 3..10, warmup 3..30 seconds, timeout 30..600 seconds"
                    .into(),
            );
        }
        let adapter_name = fields.get("--adapter-name").map(|v| v.to_string());
        if adapter_name.as_ref().is_some_and(|v| v.trim().is_empty()) {
            return Err("adapter-name cannot be empty".into());
        }
        Ok(Some(Self {
            width,
            height,
            quality,
            class,
            adapter_name,
            dpi: dpi as u32,
            windows: windows as u32,
            warmup,
            timeout,
            drive_input,
        }))
    }
}

/// Borderless fullscreen preserves the existing display mode and provides an
/// exact monitor-sized client area without decoration/work-area clamping.
/// Smaller cells use a borderless ordinary window on the same primary display.
pub fn fullscreen_cell(
    width: u32,
    height: u32,
    monitor_width: u32,
    monitor_height: u32,
) -> Result<bool, String> {
    if width > monitor_width || height > monitor_height || monitor_width == 0 || monitor_height == 0
    {
        return Err(format!(
            "physical primary display {monitor_width}x{monitor_height} cannot fit required client area {width}x{height}; no resolution/scaling simulation is allowed"
        ));
    }
    Ok(width == monitor_width && height == monitor_height)
}

pub fn check_dimensions(
    options: &Options,
    width: u32,
    height: u32,
    scale: f64,
) -> Result<f64, String> {
    let dpi = scale * 100.0;
    if width != options.width || height != options.height {
        return Err(format!(
            "actual client size {width}x{height} differs from required {}x{}",
            options.width, options.height
        ));
    }
    if !dpi.is_finite() || (dpi - f64::from(options.dpi)).abs() > 0.01 {
        return Err(format!(
            "actual DPI {dpi}% differs from required {}%",
            options.dpi
        ));
    }
    Ok(dpi)
}

/// Native per-monitor window DPI must agree with winit, rather than assuming
/// command-line DPI or process scaling reflects the physical display state.
#[cfg(windows)]
pub fn native_dpi(window: &winit::window::Window) -> Result<u32, String> {
    use windows_sys::Win32::UI::HiDpi::{
        DPI_AWARENESS_PER_MONITOR_AWARE, GetAwarenessFromDpiAwarenessContext, GetDpiForWindow,
        GetWindowDpiAwarenessContext,
    };
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let handle = window.window_handle().map_err(|e| e.to_string())?;
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return Err("expected Win32 window handle".into());
    };
    let hwnd = handle.hwnd.get() as windows_sys::Win32::Foundation::HWND;
    // SAFETY: winit owns this live HWND on the current event loop thread.
    unsafe {
        if GetAwarenessFromDpiAwarenessContext(GetWindowDpiAwarenessContext(hwnd))
            != DPI_AWARENESS_PER_MONITOR_AWARE
        {
            return Err("window is not per-monitor DPI-aware; virtualized DPI cannot validate physical scaling".into());
        }
        let dpi = GetDpiForWindow(hwnd);
        if dpi == 0 || (f64::from(dpi) / 96.0 - window.scale_factor()).abs() > 0.0001 {
            return Err("native per-monitor DPI differs from winit scale".into());
        }
        Ok(dpi)
    }
}
#[cfg(not(windows))]
pub fn native_dpi(window: &winit::window::Window) -> Result<u32, String> {
    Ok((window.scale_factor() * 96.0).round() as u32)
}

pub fn statistics(values: &[f64]) -> Option<(f64, f64)> {
    if values.is_empty()
        || values
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0)
    {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let average = values.iter().sum::<f64>() / values.len() as f64;
    if !average.is_finite() {
        return None;
    }
    Some((
        average,
        sorted[(sorted.len() * 95 / 100).min(sorted.len() - 1)],
    ))
}

#[derive(Default)]
pub struct Samples {
    pub intervals: Vec<f64>,
    pub pointer_events: u64,
    pub keyboard_events: u64,
    pub acknowledgements: Vec<f64>,
    pub event_to_present: Vec<f64>,
    pending_received: Vec<Instant>,
}
impl Samples {
    /// Called only for actual winit WindowEvents, never by the input driver.
    pub fn received(&mut self, pointer: bool, now: Instant) {
        if pointer {
            self.pointer_events += 1;
        } else {
            self.keyboard_events += 1;
        }
        self.pending_received.push(now);
    }
    /// CPU event-dispatch to successful frame submission/present call, not
    /// optical input latency, GPU completion, or a claim about human perception.
    pub fn presented(&mut self, now: Instant) {
        for event in self.pending_received.drain(..) {
            self.event_to_present
                .push(now.duration_since(event).as_secs_f64() * 1000.0);
        }
    }
    pub fn validate(&self, driven: bool) -> Result<(), String> {
        if self.intervals.len() != 300 || statistics(&self.intervals).is_none() {
            return Err("required 300 finite frame intervals missing".into());
        }
        if self.pointer_events == 0 || self.keyboard_events == 0 {
            return Err("each sample requires actual received pointer and keyboard events".into());
        }
        if self.event_to_present.len() as u64 != self.pointer_events + self.keyboard_events
            || statistics(&self.event_to_present).is_none()
        {
            return Err("received events have missing CPU frame-submission latency samples".into());
        }
        if driven
            && (self.acknowledgements.is_empty() || statistics(&self.acknowledgements).is_none())
        {
            return Err("driven input has no actual event acknowledgements".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn info(
        name: &str,
        device: u32,
        class: wgpu::DeviceType,
        backend: wgpu::Backend,
    ) -> wgpu::AdapterInfo {
        wgpu::AdapterInfo {
            name: name.into(),
            vendor: 1,
            device,
            device_type: class,
            driver: "test".into(),
            driver_info: "test".into(),
            backend,
        }
    }
    fn options() -> Options {
        Options::parse(
            &[
                "960",
                "640",
                "full",
                "--gpu",
                "integrated",
                "--expected-dpi",
                "100",
            ]
            .map(str::to_string),
        )
        .unwrap()
        .unwrap()
    }
    #[test]
    fn selects_exact_class_and_deduplicates_backends() {
        let infos = vec![
            (
                info(
                    "NVIDIA",
                    2,
                    wgpu::DeviceType::DiscreteGpu,
                    wgpu::Backend::Dx12,
                ),
                true,
            ),
            (
                info(
                    "Intel",
                    1,
                    wgpu::DeviceType::IntegratedGpu,
                    wgpu::Backend::Vulkan,
                ),
                true,
            ),
            (
                info(
                    "Intel",
                    1,
                    wgpu::DeviceType::IntegratedGpu,
                    wgpu::Backend::Dx12,
                ),
                true,
            ),
        ];
        assert_eq!(
            select_adapter(&infos, GpuClass::Integrated, None).unwrap(),
            2
        );
        assert_eq!(select_adapter(&infos, GpuClass::Discrete, None).unwrap(), 0);
    }
    #[test]
    fn absent_incompatible_ambiguous_and_software_fail() {
        let software = vec![(
            info("CPU", 1, wgpu::DeviceType::Cpu, wgpu::Backend::Vulkan),
            true,
        )];
        assert!(select_adapter(&software, GpuClass::Integrated, None).is_err());
        let infos = vec![
            (
                info(
                    "Intel A",
                    1,
                    wgpu::DeviceType::IntegratedGpu,
                    wgpu::Backend::Dx12,
                ),
                true,
            ),
            (
                info(
                    "Intel B",
                    2,
                    wgpu::DeviceType::IntegratedGpu,
                    wgpu::Backend::Dx12,
                ),
                true,
            ),
        ];
        assert!(select_adapter(&infos, GpuClass::Integrated, None).is_err());
        assert_eq!(
            select_adapter(&infos, GpuClass::Integrated, Some("b")).unwrap(),
            1
        );
        assert!(
            select_adapter(&[(infos[0].0.clone(), false)], GpuClass::Integrated, None).is_err()
        );
    }
    #[test]
    fn indistinguishable_physical_adapters_are_not_collapsed() {
        let same = info(
            "Intel",
            1,
            wgpu::DeviceType::IntegratedGpu,
            wgpu::Backend::Dx12,
        );
        let mut vulkan = same.clone();
        vulkan.backend = wgpu::Backend::Vulkan;
        assert!(
            select_adapter(
                &[(same.clone(), true), (same, true)],
                GpuClass::Integrated,
                None
            )
            .is_err()
        );
        assert!(
            select_adapter(
                &[(vulkan.clone(), true), (vulkan, true)],
                GpuClass::Integrated,
                None
            )
            .is_err()
        );
    }
    #[test]
    fn monitor_sized_cells_use_borderless_fullscreen_without_rescaling() {
        assert!(fullscreen_cell(1920, 1080, 1920, 1080).unwrap());
        assert!(!fullscreen_cell(960, 640, 1920, 1080).unwrap());
        assert!(!fullscreen_cell(1920, 1080, 2560, 1440).unwrap());
        assert!(fullscreen_cell(1920, 1080, 1366, 768).is_err());
    }
    #[test]
    fn arguments_and_dpi_fail_closed() {
        let opts = options();
        assert!(check_dimensions(&opts, 960, 640, 1.5).is_err());
        assert!(check_dimensions(&opts, 959, 640, 1.0).is_err());
        assert!(check_dimensions(&opts, 960, 640, f64::NAN).is_err());
        assert!(check_dimensions(&opts, 960, 640, 1.0).is_ok());
        assert!(Options::parse(&["960", "640", "full"].map(str::to_string)).is_err());
        assert!(
            Options::parse(
                &[
                    "960",
                    "640",
                    "full",
                    "--gpu",
                    "discrete",
                    "--expected-dpi",
                    "100",
                    "--timeout-seconds",
                    "0"
                ]
                .map(str::to_string)
            )
            .is_err()
        );
    }
    #[test]
    fn input_requires_received_events_and_submission() {
        let mut sample = Samples {
            intervals: vec![16.0; 300],
            ..Default::default()
        };
        assert!(sample.validate(false).is_err());
        let now = Instant::now();
        sample.received(true, now);
        sample.received(false, now);
        assert!(sample.validate(false).is_err());
        sample.presented(now);
        assert!(sample.validate(false).is_ok());
        assert!(sample.validate(true).is_err());
        sample.acknowledgements.push(2.0);
        assert!(sample.validate(true).is_ok());
    }
    #[test]
    fn invalid_statistics_are_rejected() {
        assert!(statistics(&[]).is_none());
        assert!(statistics(&[f64::NAN]).is_none());
        assert!(statistics(&[f64::MAX, f64::MAX]).is_none());
        assert!(statistics(&[-1.0]).is_none());
        assert_eq!(statistics(&[1.0, 3.0]), Some((2.0, 3.0)));
    }
}
