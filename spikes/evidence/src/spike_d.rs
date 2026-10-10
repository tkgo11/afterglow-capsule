use crate::{Gate, Status, input::*, trusted_artifact};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

fn coordinates(v: &Value) -> Result<String> {
    let class = text(v, "requested_gpu_class")?;
    let dpi = integer(v, "requested_dpi_percent")?;
    let width = integer(v, "width")?;
    let height = integer(v, "height")?;
    let mode = text(v, "requested_mode")?;
    if !matches!(class, "integrated" | "discrete")
        || !matches!(dpi, 100 | 150 | 200)
        || !matches!((width, height), (960, 640) | (1440, 900) | (1920, 1080))
        || !matches!(mode, "full" | "reduced" | "opaque")
    {
        return Err("cell is outside the required 54-cell matrix".into());
    }
    Ok(format!("{class}/{dpi}/{width}x{height}/{mode}"))
}
fn actual_dpi(v: &Value, expected: u64) -> Result<()> {
    if v.get("actual_dpi_percent").and_then(Value::as_f64) != Some(expected as f64) {
        return Err("actual DPI differs from required DPI".into());
    }
    Ok(())
}
fn adapter(v: &Value, class: &str) -> Result<()> {
    let ty = match class {
        "integrated" => "IntegratedGpu",
        "discrete" => "DiscreteGpu",
        _ => return Err("unknown requested GPU class".into()),
    };
    if text(v, "device_type")? != ty {
        return Err("actual GPU class differs from requested GPU class".into());
    }
    for field in ["name", "backend"] {
        text(v, field)?;
    }
    for field in ["driver", "driver_info"] {
        if v.get(field).and_then(Value::as_str).is_none() {
            return Err(format!("missing adapter {field}"));
        }
    }
    for field in ["vendor", "device"] {
        if integer(v, field)? > u32::MAX as u64 {
            return Err(format!("invalid adapter {field}"));
        }
    }
    Ok(())
}
fn timing(v: &Value, key: &str, positive: bool) -> Result<f64> {
    let n = v
        .get(key)
        .and_then(Value::as_f64)
        .ok_or(format!("missing/non-number timing {key}"))?;
    if !n.is_finite() || n < 0.0 || (positive && n == 0.0) || n > 120_000.0 {
        return Err(format!("invalid timing {key}"));
    }
    Ok(n)
}
fn next_mode(mode: &str, avg: f64) -> Result<&str> {
    match mode {
        "opaque" | "static" => Ok(mode),
        "full" | "reduced" if avg > 25.0 => Ok("static"),
        "full" if avg > 16.7 => Ok("reduced"),
        "full" | "reduced" => Ok(mode),
        _ => Err("invalid sample mode".into()),
    }
}
// PowerShell can reserialize 100.0 as 100. Compare numeric JSON values without
// accepting strings/booleans or rounding large integer identifiers/counters.
fn equivalent(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            x == y
                || matches!((x.as_f64(), y.as_f64()), (Some(a), Some(b)) if a == b && a.abs() <= (1u64 << 53) as f64)
        }
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(a, b)| equivalent(a, b))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, a)| y.get(k).is_some_and(|b| equivalent(a, b)))
        }
        _ => a == b,
    }
}
fn raw(root: &Path, cell: &Value) -> Result<()> {
    let bytes = digest(
        root,
        text(cell, "raw_log")?,
        text(cell, "raw_log_sha256")?,
        LOG_LIMIT,
    )?;
    digest(
        root,
        text(cell, "stderr_log")?,
        text(cell, "stderr_log_sha256")?,
        LOG_LIMIT,
    )?;
    let source = std::str::from_utf8(&bytes).map_err(|_| "probe JSONL is not UTF-8")?;
    let mut start = None;
    let mut samples = vec![];
    let mut complete = None;
    let mut seen_inventory = false;
    for line in source.lines().filter(|line| !line.trim().is_empty()) {
        if line.len() > 65536 {
            return Err("probe record exceeds 64 KiB".into());
        }
        let event = parse(line.as_bytes())?;
        document(&event, "afterglow-spike-d-event")?;
        match text(&event, "event")? {
            "adapter_inventory" if !seen_inventory && start.is_none() => {
                seen_inventory = true;
                let a = array(&event, "adapters")?;
                if a.is_empty() || a.len() > 64 {
                    return Err("invalid adapter inventory".into());
                }
            }
            "start" if start.is_none() && samples.is_empty() && complete.is_none() => {
                start = Some(event);
            }
            "sample" if start.is_some() && complete.is_none() && samples.len() < 3 => {
                samples.push(event)
            }
            "complete" if start.is_some() && complete.is_none() => complete = Some(event),
            "failure" => return Err(format!("probe failure: {}", text(&event, "reason")?)),
            _ => return Err("duplicate, out-of-order or unknown probe event".into()),
        }
    }
    let start = start.ok_or("probe start event missing")?;
    for key in [
        "requested_gpu_class",
        "requested_dpi_percent",
        "width",
        "height",
        "requested_mode",
    ] {
        same(&start, cell, key)?;
    }
    let class = text(cell, "requested_gpu_class")?;
    let info = start.get("adapter").ok_or("probe adapter missing")?;
    adapter(info, class)?;
    if Some(info) != cell.get("actual_adapter") {
        return Err("collector adapter differs from raw probe adapter".into());
    }
    let dpi = integer(cell, "requested_dpi_percent")?;
    actual_dpi(&start, dpi)?;
    if start.get("native_dpi").and_then(Value::as_u64) != Some(dpi * 96 / 100)
        || text(&start, "dpi_awareness")? != "PerMonitorAware"
    {
        return Err(
            "native effective DPI/per-monitor awareness proof missing or inconsistent".into(),
        );
    }
    let driven = start
        .get("drive_input")
        .and_then(Value::as_bool)
        .ok_or("drive_input must be boolean")?;
    if text(cell, "input_source")? == "windows-sendinput" && !driven {
        return Err("collector claims input driver absent from raw probe".into());
    }
    if samples.len() != 3
        || !equivalent(
            &Value::Array(array(cell, "samples")?.clone()),
            &Value::Array(samples.clone()),
        )
    {
        return Err("exactly three raw sample windows matching collector are required".into());
    }
    let mut mode = text(cell, "requested_mode")?.to_owned();
    for (index, sample) in samples.iter().enumerate() {
        if integer(sample, "index")? != index as u64 + 1 || integer(sample, "frames")? != 300 {
            return Err("sample windows must be indexed 1..3 with 300 frames each".into());
        }
        for key in ["width", "height"] {
            same(sample, cell, key)?;
        }
        actual_dpi(sample, dpi)?;
        if integer(sample, "native_dpi")? != dpi * 96 / 100
            || text(sample, "dpi_awareness")? != "PerMonitorAware"
        {
            return Err("sample native DPI/awareness proof differs from required DPI".into());
        }
        if text(sample, "mode")? != mode {
            return Err("mode progression does not match automatic degradation policy".into());
        }
        let avg = timing(sample, "interval_avg_ms", true)?;
        let p95 = timing(sample, "interval_p95_ms", true)?;
        // A fast mean cannot hide unstable frame cadence in the upper tail.
        // This mirrors the conservative policy in the physical probe/collector.
        let expected_next = next_mode(&mode, avg.max(p95))?;
        if text(sample, "next_mode")? != expected_next {
            return Err("slow frames did not follow automatic degradation policy".into());
        }
        let pointer = integer(sample, "pointer_events")?;
        let keyboard = integer(sample, "keyboard_events")?;
        if pointer == 0 || keyboard == 0 {
            return Err("every window requires actual received pointer and keyboard events".into());
        }
        let received = pointer
            .checked_add(keyboard)
            .ok_or("input counter overflow")?;
        let acknowledged = integer(sample, "input_ack_count")?;
        if acknowledged > received || (driven && acknowledged == 0) {
            return Err("input acknowledgement count is missing or inconsistent".into());
        }
        if integer(sample, "event_to_present_count")? != received {
            return Err("event-to-present count differs from actual received events".into());
        }
        for key in [
            "input_ack_avg_ms",
            "input_ack_p95_ms",
            "event_to_present_avg_ms",
            "event_to_present_p95_ms",
        ] {
            timing(sample, key, false)?;
        }
        mode = expected_next.to_owned();
    }
    let complete = complete.ok_or("probe completion event missing")?;
    if text(&complete, "status")? != "PASS" || integer(&complete, "windows_collected")? != 3 {
        return Err("probe did not report three successful windows".into());
    }
    Ok(())
}
fn cell(root: &Path, v: &Value, expected: &Value) -> Gate {
    let mut g = Gate::default();
    g.check((|| {
        document(v, "afterglow-spike-d-cell-result")?;
        coordinates(v)?;
        if Some(text(v, "cell_id")?) != root.file_name().and_then(|name| name.to_str()) {
            return Err("cell ID does not match its immutable attempt directory".into());
        }
        text(v, "session_id")?;
        same(v, expected, "provenance")?;
        same(v, expected, "exe_sha256")?;
        if v.get("exit_code").and_then(Value::as_i64) != Some(0) {
            return Err("probe exited unsuccessfully".into());
        }
        if text(v, "machine_status")? != "PASS" || !array(v, "failures")?.is_empty() {
            return Err("collector reported machine failure".into());
        }
        if !matches!(text(v, "input_source")?, "windows-sendinput" | "human") {
            return Err("input source must be explicit".into());
        }
        adapter(
            v.get("actual_adapter").ok_or("actual adapter missing")?,
            text(v, "requested_gpu_class")?,
        )?;
        actual_dpi(v, integer(v, "requested_dpi_percent")?)?;
        let hardware = v.get("hardware").ok_or("hardware provenance missing")?;
        text(hardware, "os_caption")?;
        text(hardware, "os_build")?;
        text(hardware, "power_note")?;
        let rate = hardware
            .get("refresh_rate")
            .ok_or("display refresh rate missing")?;
        let rates = match rate {
            Value::Array(rates) => rates.as_slice(),
            _ => std::slice::from_ref(rate),
        };
        if !rates.iter().any(|rate| {
            rate.as_f64()
                .is_some_and(|n| n.is_finite() && (1.0..=1000.0).contains(&n))
        }) || rates.iter().any(|rate| {
            !rate.is_null()
                && !rate
                    .as_f64()
                    .is_some_and(|n| n.is_finite() && (0.0..=1000.0).contains(&n))
        }) {
            return Err("invalid/missing active display refresh rate metadata".into());
        }
        if array(hardware, "gpu_driver")?.is_empty() {
            return Err("GPU driver provenance is missing".into());
        }
        raw(root, v)?;
        Ok(())
    })());
    if let Some(human) = v.get("human_observation") {
        g.human(human, "responsive_input");
        g.human(human, "foreground_preserved");
        if human.get("responsive_input") == Some(&Value::Bool(true))
            || human.get("foreground_preserved") == Some(&Value::Bool(true))
        {
            g.check(text(human, "note").map(|_| ()));
        }
    } else {
        g.pending("human foreground/input observations missing");
    }
    g
}
pub fn validate(root: &Path, trusted: Option<&Value>) -> Gate {
    let mut g = Gate::default();
    let Some(expected) = g.load(root, "spike-d-expected.json") else {
        return g;
    };
    if !g.format(&expected, "afterglow-spike-d-evidence") {
        return g;
    }
    trusted_artifact(
        &mut g,
        root,
        &expected,
        "spike-d-expected.json",
        "SpikeD-Glass.exe",
        trusted,
        "spike_d",
    );
    g.check(
        digest(
            root,
            "SpikeD-Glass.exe",
            text(&expected, "exe_sha256").unwrap_or(""),
            EXE_LIMIT,
        )
        .map(|_| ()),
    );
    let path = match safe_path(root, "cells") {
        Ok(p) => p,
        Err(e) => {
            g.fail(e);
            return g;
        }
    };
    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(e) => {
            g.pending(format!("matrix cells missing: {e}"));
            return g;
        }
    };
    let mut all = BTreeMap::new();
    for (index, entry) in entries.enumerate() {
        if index >= 4096 {
            g.fail("evidence attempt count exceeds bound");
            return g;
        }
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                g.fail(e.to_string());
                continue;
            }
        };
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            g.fail("cell directory name is not UTF-8");
            continue;
        };
        let relative = format!("cells/{name}/cell.json");
        let Some(v) = g.load(root, &relative) else {
            continue;
        };
        if !g.format(&v, "afterglow-spike-d-cell-result") {
            continue;
        }
        let coordinate = match coordinates(&v) {
            Ok(c) => c,
            Err(e) => {
                g.fail(format!("{relative}: {e}"));
                continue;
            }
        };
        let cell_root = match safe_path(root, &format!("cells/{name}")) {
            Ok(p) => p,
            Err(e) => {
                g.fail(e);
                continue;
            }
        };
        let result = cell(&cell_root, &v, &expected);
        all.insert(relative, (coordinate, result));
    }
    let mut selected = BTreeSet::new();
    let mut excluded = BTreeMap::new();
    let explicit_selection = root.join("matrix-selection.json").exists();
    if explicit_selection {
        if let Some(selection) = g.load(root, "matrix-selection.json") {
            g.check((|| {
                document(&selection, "afterglow-spike-d-matrix-selection")?;
                for item in array(&selection, "selections")? {
                    let path = text(item, "cell_path")?;
                    safe_path(root, path)?;
                    if !all.contains_key(path) || !selected.insert(path.to_owned()) {
                        return Err("unknown or duplicate selected cell path".into());
                    }
                }
                for item in array(&selection, "exclusions")? {
                    let path = text(item, "cell_path")?;
                    safe_path(root, path)?;
                    let reason = text(item, "reason")?;
                    if !all.contains_key(path)
                        || selected.contains(path)
                        || excluded
                            .insert(path.to_owned(), reason.to_owned())
                            .is_some()
                    {
                        return Err(
                            "unknown, duplicated or simultaneously selected exclusion".into()
                        );
                    }
                }
                Ok(())
            })());
        }
        for path in all.keys() {
            if !selected.contains(path) && !excluded.contains_key(path) {
                g.pending(format!(
                    "unresolved attempt {path}; explicit selection/exclusion with reason required"
                ));
            }
        }
    } else {
        selected.extend(all.keys().cloned());
    }
    let mut matrix = BTreeMap::<String, String>::new();
    for path in selected {
        if let Some((coordinate, result)) = all.get(&path) {
            if let Some(other) = matrix.insert(coordinate.clone(), path.clone()) {
                let reason = format!(
                    "duplicate cell {coordinate}: {other} and {path}; explicit attempt selection required"
                );
                if explicit_selection {
                    g.fail(reason);
                } else {
                    g.pending(reason);
                }
            }
            for reason in &result.reasons {
                match result.status {
                    Status::Fail => g.fail(format!("{path}: {reason}")),
                    Status::Pending => g.pending(format!("{path}: {reason}")),
                    Status::Pass => {}
                }
            }
        }
    }
    for (path, reason) in excluded {
        if let Some((_, result)) = all.get(&path) {
            g.audit.push(format!("excluded attempt {path}; reason={reason}; original_status={:?}; original_reasons={:?}",result.status,result.reasons));
        }
    }
    for class in ["integrated", "discrete"] {
        for dpi in [100, 150, 200] {
            for (w, h) in [(960, 640), (1440, 900), (1920, 1080)] {
                for mode in ["full", "reduced", "opaque"] {
                    let coordinate = format!("{class}/{dpi}/{w}x{h}/{mode}");
                    if !matrix.contains_key(&coordinate) {
                        g.pending(format!("required cell missing: {coordinate}"));
                    }
                }
            }
        }
    }
    if matrix.len() != 54 {
        g.pending(format!(
            "required exactly 54 unique matrix cells, found {}",
            matrix.len()
        ));
    }
    g
}
