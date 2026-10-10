//! Synthetic validator fixtures ONLY. These temporary files are not physical evidence.
use afterglow_phase2_evidence::{Status, validate};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn write(root: &Path, name: &str, v: &Value) {
    let p = root.join(name);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, serde_json::to_vec(v).unwrap()).unwrap();
}
fn read(root: &Path, name: &str) -> Value {
    serde_json::from_slice(&fs::read(root.join(name)).unwrap()).unwrap()
}
fn doc(name: &str) -> Value {
    json!({"format_name":name,"format_version":2,"minimum_reader_version":2})
}
fn provenance() -> Value {
    json!({"repository":"tkgo11/afterglow-capsule","source_commit":"a".repeat(40),"workflow_run_id":42,"workflow_run_attempt":1,"workflow_path":".github/workflows/spikes.yml"})
}
struct Fixture {
    root: PathBuf,
    trusted: PathBuf,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(self.root.parent().unwrap());
    }
}
impl Fixture {
    fn new() -> Self {
        let base = std::env::temp_dir().join(format!(
            "afterglow-synthetic-validator-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let root = base.join("evidence");
        let trusted = base.join("trusted.json");
        fs::create_dir_all(&root).unwrap();
        let c = root.join("c");
        let d = root.join("d");
        fs::create_dir_all(&c).unwrap();
        fs::create_dir_all(&d).unwrap();
        let mut exe = vec![0; 128];
        exe[..2].copy_from_slice(b"MZ");
        exe[60..64].copy_from_slice(&64u32.to_le_bytes());
        exe[64..70].copy_from_slice(b"PE\0\0\x64\x86");
        fs::write(c.join("SpikeC-Standalone.exe"), &exe).unwrap();
        fs::write(d.join("SpikeD-Glass.exe"), &exe).unwrap();
        let mut ce = doc("afterglow-spike-c-evidence");
        ce["provenance"] = provenance();
        ce["signed_exe_sha256"] = hash(&exe).into();
        ce["file_version"] = "2.0.0.0".into();
        ce["resource_sha256"] = json!({"capsule":hash(b"capsule"),"icon":hash(b"icon"),"group-icon":hash(b"group"),"version":hash(b"version")});
        write(&c, "spike-c-expected.json", &ce);
        let mut vm = doc("afterglow-spike-c-vm-provenance");
        vm["origin"] = "synthetic unit test".into();
        vm["base_image"] = "synthetic test only".into();
        vm["note"] = "never physical evidence".into();
        vm["installation_kind"] = "clean-windows-vm".into();
        vm["clean_recipient_attested"] = true.into();
        write(&c, "vm-provenance.json", &vm);
        // Synthetic valid PNG container, never a real shell screenshot.
        let hex = "89504e470d0a1a0a0000000d4948445200000020000000200806000000737a7af40000003049444154789cedce2101000008033042d03f0099e802316e26e6573d7b49252020202020202020202020202020900e3c52ab6497c1c055c80000000049454e44ae426082";
        let png: Vec<u8> = hex
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect();
        fs::write(c.join("shell-icon.png"), &png).unwrap();
        let mut cr = doc("afterglow-spike-c-clean-vm-result");
        cr["provenance"] = provenance();
        cr["exe_sha256"] = hash(&exe).into();
        cr["resource_sha256"] = ce["resource_sha256"].clone();
        cr["file_version"] = "2.0.0.0".into();
        cr["architecture"] = "x86_64".into();
        cr["one_file_execution"] = "passed".into();
        cr["environment"] = json!({"os_caption":"Windows 11 test","os_build":"26100","os_version":"10.0.26100","os_architecture":"64-bit","os_native_architecture":"AMD64","product_type":1,"process_64_bit":true,"developer_tools":[],"qualifies_clean":true,"hosted_ci_detected":false,"virtual_machine_detected":true,"manufacturer":"synthetic","model":"synthetic"});
        cr["vm_provenance_file"] = "vm-provenance.json".into();
        cr["vm_provenance_sha256"] = hash(&fs::read(c.join("vm-provenance.json")).unwrap()).into();
        cr["clean_recipient_vm_attested"] = true.into();
        cr["shell_icon_file"] = "shell-icon.png".into();
        cr["shell_icon_sha256"] = hash(&png).into();
        cr["shell_icon_observation"] = json!({"correct":true,"note":"synthetic test observation"});
        cr["machine_status"] = "PASS".into();
        cr["failures"] = json!([]);
        cr["pending"] = json!([]);
        write(&c, "spike-c-clean-vm-result.json", &cr);
        let mut de = doc("afterglow-spike-d-evidence");
        de["provenance"] = provenance();
        de["exe_sha256"] = hash(&exe).into();
        write(&d, "spike-d-expected.json", &de);
        let mut index = 0;
        for class in ["integrated", "discrete"] {
            for dpi in [100, 150, 200] {
                for (w, h) in [(960, 640), (1440, 900), (1920, 1080)] {
                    for mode in ["full", "reduced", "opaque"] {
                        let dir = d.join(format!("cells/{index}"));
                        fs::create_dir_all(&dir).unwrap();
                        let adapter = json!({"name":format!("synthetic {class}"),"vendor":32902,"device":123,"backend":"Dx12","driver":"test","driver_info":"synthetic","device_type":if class=="integrated" {"IntegratedGpu"} else {"DiscreteGpu"}});
                        let mut start = doc("afterglow-spike-d-event");
                        start["event"] = "start".into();
                        start["requested_gpu_class"] = class.into();
                        start["requested_dpi_percent"] = dpi.into();
                        start["width"] = w.into();
                        start["height"] = h.into();
                        start["requested_mode"] = mode.into();
                        start["actual_dpi_percent"] = dpi.into();
                        start["native_dpi"] = (dpi * 96 / 100).into();
                        start["dpi_awareness"] = "PerMonitorAware".into();
                        start["adapter"] = adapter.clone();
                        start["drive_input"] = true.into();
                        let samples: Vec<Value> = (1..=3)
                            .map(|i| {
                                let mut s = doc("afterglow-spike-d-event");
                                s["event"] = "sample".into();
                                s["index"] = i.into();
                                s["frames"] = 300.into();
                                s["width"] = w.into();
                                s["height"] = h.into();
                                s["actual_dpi_percent"] = dpi.into();
                                s["native_dpi"] = (dpi * 96 / 100).into();
                                s["dpi_awareness"] = "PerMonitorAware".into();
                                s["mode"] = mode.into();
                                s["next_mode"] = mode.into();
                                s["interval_avg_ms"] = 16.0.into();
                                s["interval_p95_ms"] = 16.5.into();
                                s["pointer_events"] = 10.into();
                                s["keyboard_events"] = 5.into();
                                s["input_ack_count"] = 15.into();
                                s["input_ack_avg_ms"] = 1.0.into();
                                s["input_ack_p95_ms"] = 2.0.into();
                                s["event_to_present_count"] = 15.into();
                                s["event_to_present_avg_ms"] = 3.0.into();
                                s["event_to_present_p95_ms"] = 5.0.into();
                                s
                            })
                            .collect();
                        let mut complete = doc("afterglow-spike-d-event");
                        complete["event"] = "complete".into();
                        complete["status"] = "PASS".into();
                        complete["windows_collected"] = 3.into();
                        let mut events = vec![start];
                        events.extend(samples.clone());
                        events.push(complete);
                        let raw = events
                            .iter()
                            .map(|e| serde_json::to_string(e).unwrap())
                            .collect::<Vec<_>>()
                            .join("\n");
                        fs::write(dir.join("probe.jsonl"), &raw).unwrap();
                        fs::write(dir.join("stderr.log"), b"").unwrap();
                        let mut cell = doc("afterglow-spike-d-cell-result");
                        cell["cell_id"] = index.to_string().into();
                        cell["session_id"] = "synthetic-session".into();
                        cell["provenance"] = provenance();
                        cell["exe_sha256"] = de["exe_sha256"].clone();
                        cell["requested_gpu_class"] = class.into();
                        cell["requested_dpi_percent"] = dpi.into();
                        cell["width"] = w.into();
                        cell["height"] = h.into();
                        cell["requested_mode"] = mode.into();
                        cell["actual_dpi_percent"] = dpi.into();
                        cell["actual_adapter"] = adapter;
                        cell["samples"] = samples.into();
                        cell["input_source"] = "windows-sendinput".into();
                        cell["machine_status"] = "PASS".into();
                        cell["exit_code"] = 0.into();
                        cell["failures"] = json!([]);
                        cell["raw_log"] = "probe.jsonl".into();
                        cell["raw_log_sha256"] = hash(raw.as_bytes()).into();
                        cell["stderr_log"] = "stderr.log".into();
                        cell["stderr_log_sha256"] = hash(b"").into();
                        cell["human_observation"] = json!({"responsive_input":true,"foreground_preserved":true,"note":"synthetic test only"});
                        cell["hardware"] = json!({"os_caption":"Windows test","os_build":"26100","gpu_driver":[{"name":"synthetic"}],"refresh_rate":60,"power_note":"synthetic test only"});
                        write(&dir, "cell.json", &cell);
                        index += 1;
                    }
                }
            }
        }
        let mut tr = doc("afterglow-phase2-trusted-provenance");
        for key in ["spike_a", "spike_b"] {
            let mut reference = provenance();
            reference["status"] = "PASS".into();
            reference["job_id"] = 7.into();
            reference["evidence_url"] =
                "https://github.com/tkgo11/afterglow-capsule/actions/runs/42/job/7".into();
            reference["review_note"] = "synthetic test only".into();
            tr[key] = reference;
        }
        for (key, manifest, folder) in [
            ("spike_c", "spike-c-expected.json", &c),
            ("spike_d", "spike-d-expected.json", &d),
        ] {
            tr[key] = json!({"provenance":provenance(),"expected_manifest_sha256":hash(&fs::read(folder.join(manifest)).unwrap()),"exe_sha256":hash(&exe)});
        }
        fs::write(&trusted, serde_json::to_vec(&tr).unwrap()).unwrap();
        Self { root, trusted }
    }
    fn status(&self) -> Status {
        validate("validate-phase2-evidence", &self.root, Some(&self.trusted)).status
    }
    fn mutate(&self, path: &str, f: impl FnOnce(&mut Value)) {
        let mut v = read(&self.root, path);
        f(&mut v);
        write(&self.root, path, &v);
    }
    fn raw_mutate(&self, f: impl FnOnce(&mut Vec<Value>)) {
        let dir = self.root.join("d/cells/0");
        let raw = fs::read_to_string(dir.join("probe.jsonl")).unwrap();
        let mut events = raw
            .lines()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect();
        f(&mut events);
        let raw = events
            .iter()
            .map(|e| serde_json::to_string(e).unwrap())
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(dir.join("probe.jsonl"), &raw).unwrap();
        let mut c = read(&dir, "cell.json");
        c["raw_log_sha256"] = hash(raw.as_bytes()).into();
        c["samples"] = events
            .iter()
            .filter(|e| e["event"] == "sample")
            .cloned()
            .collect::<Vec<_>>()
            .into();
        write(&dir, "cell.json", &c);
    }
}
#[test]
fn complete_synthetic_fixture_validates() {
    assert_eq!(Fixture::new().status(), Status::Pass);
}
#[test]
fn no_trusted_provenance_never_accepts() {
    let f = Fixture::new();
    assert_eq!(
        validate("validate-phase2-evidence", &f.root, None).status,
        Status::Pending
    );
}
#[test]
fn human_observation_null_is_pending() {
    let f = Fixture::new();
    f.mutate("d/cells/0/cell.json", |v| {
        v["human_observation"]["responsive_input"] = Value::Null
    });
    assert_eq!(f.status(), Status::Pending);
}
#[test]
fn human_observation_false_fails() {
    let f = Fixture::new();
    f.mutate("d/cells/0/cell.json", |v| {
        v["human_observation"]["foreground_preserved"] = false.into()
    });
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn wrong_gpu_class_fails() {
    let f = Fixture::new();
    f.mutate("d/cells/0/cell.json", |v| {
        v["actual_adapter"]["device_type"] = "DiscreteGpu".into()
    });
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn wrong_dpi_fails() {
    let f = Fixture::new();
    f.mutate("d/cells/0/cell.json", |v| {
        v["actual_dpi_percent"] = 150.into()
    });
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn zero_input_fails_even_if_raw_and_summary_agree() {
    let f = Fixture::new();
    f.raw_mutate(|e| e[1]["pointer_events"] = 0.into());
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn no_keyboard_fails() {
    let f = Fixture::new();
    f.raw_mutate(|e| e[1]["keyboard_events"] = 0.into());
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn bool_numeric_rejected() {
    let f = Fixture::new();
    f.raw_mutate(|e| e[1]["frames"] = true.into());
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn negative_timing_rejected() {
    let f = Fixture::new();
    f.raw_mutate(|e| e[1]["event_to_present_avg_ms"] = (-0.1).into());
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn sample_counter_rejected() {
    let f = Fixture::new();
    f.raw_mutate(|e| e[1]["event_to_present_count"] = 16.into());
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn no_native_dpi_proof_rejected() {
    let f = Fixture::new();
    f.raw_mutate(|e| {
        e[0].as_object_mut()
            .unwrap()
            .remove("native_dpi")
            .map(|_| ())
            .unwrap()
    });
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn missing_window_rejected() {
    let f = Fixture::new();
    f.raw_mutate(|e| {
        e.remove(1);
    });
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn unhidden_probe_failure_rejected() {
    let f = Fixture::new();
    f.raw_mutate(|e| {
        e.last_mut().unwrap()["status"] = "FAIL".into();
    });
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn failed_process_never_accepted() {
    let f = Fixture::new();
    f.mutate("d/cells/0/cell.json", |v| v["exit_code"] = 1.into());
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn raw_log_tamper_rejected() {
    let f = Fixture::new();
    fs::write(f.root.join("d/cells/0/probe.jsonl"), b"{}").unwrap();
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn missing_raw_log_rejected() {
    let f = Fixture::new();
    fs::remove_file(f.root.join("d/cells/0/probe.jsonl")).unwrap();
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn missing_required_cell_pending() {
    let f = Fixture::new();
    fs::remove_dir_all(f.root.join("d/cells/0")).unwrap();
    assert_eq!(f.status(), Status::Pending);
}
#[test]
fn duplicate_cell_never_implicitly_latest() {
    let f = Fixture::new();
    let p = f.root.join("d/cells/extra");
    fs::create_dir(&p).unwrap();
    for name in ["cell.json", "probe.jsonl", "stderr.log"] {
        fs::copy(f.root.join("d/cells/0").join(name), p.join(name)).unwrap();
    }
    f.mutate("d/cells/extra/cell.json", |v| v["cell_id"] = "extra".into());
    assert_eq!(f.status(), Status::Pending);
}
#[test]
fn trusted_manifest_hash_rejects_relabeling() {
    let f = Fixture::new();
    f.mutate("d/spike-d-expected.json", |v| {
        v["provenance"]["source_commit"] = "b".repeat(40).into()
    });
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn duplicate_json_keys_rejected() {
    let f = Fixture::new();
    fs::write(f.root.join("d/cells/0/cell.json"),br#"{"format_name":"afterglow-spike-d-cell-result","format_version":2,"format_version":2,"minimum_reader_version":2}"#).unwrap();
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn relative_path_escape_rejected() {
    let f = Fixture::new();
    f.mutate("d/cells/0/cell.json", |v| {
        v["raw_log"] = "../../../c/vm-provenance.json".into()
    });
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn old_v1_artifact_pending_and_preserved() {
    let f = Fixture::new();
    f.mutate("d/spike-d-expected.json", |v| {
        v["format_version"] = 1.into()
    });
    assert_eq!(f.status(), Status::Pending);
    assert!(f.root.join("d/spike-d-expected.json").exists());
}
#[test]
fn developer_vm_rejected() {
    let f = Fixture::new();
    f.mutate("c/spike-c-clean-vm-result.json", |v| {
        v["environment"]["developer_tools"] = json!(["rustc"])
    });
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn hosted_ci_not_clean_recipient() {
    let f = Fixture::new();
    f.mutate("c/spike-c-clean-vm-result.json", |v| {
        v["environment"]["hosted_ci_detected"] = true.into()
    });
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn server_not_recipient() {
    let f = Fixture::new();
    f.mutate("c/spike-c-clean-vm-result.json", |v| {
        v["environment"]["product_type"] = 3.into()
    });
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn missing_shell_observation_pending() {
    let f = Fixture::new();
    f.mutate("c/spike-c-clean-vm-result.json", |v| {
        v["shell_icon_observation"]["correct"] = Value::Null
    });
    assert_eq!(f.status(), Status::Pending);
}
#[test]
fn wrong_exe_hash_rejected() {
    let f = Fixture::new();
    fs::write(f.root.join("c/SpikeC-Standalone.exe"), b"wrong").unwrap();
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn unsigned_different_resource_hash_rejected() {
    let f = Fixture::new();
    f.mutate("c/spike-c-clean-vm-result.json", |v| {
        v["resource_sha256"]["capsule"] = "0".repeat(64).into()
    });
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn bad_reference_job_rejected() {
    let f = Fixture::new();
    let mut v: Value = serde_json::from_slice(&fs::read(&f.trusted).unwrap()).unwrap();
    v["spike_a"]["job_id"] = 8.into();
    fs::write(&f.trusted, serde_json::to_vec(&v).unwrap()).unwrap();
    assert_eq!(f.status(), Status::Fail);
}
#[cfg(unix)]
#[test]
fn symlink_log_rejected() {
    let f = Fixture::new();
    let p = f.root.join("d/cells/0/probe.jsonl");
    let raw = fs::read(&p).unwrap();
    fs::remove_file(&p).unwrap();
    fs::write(f.root.join("outside.log"), raw).unwrap();
    std::os::unix::fs::symlink(f.root.join("outside.log"), p).unwrap();
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn missing_vm_provenance_is_pending_not_fabricated_pass() {
    let f = Fixture::new();
    f.mutate("c/spike-c-clean-vm-result.json", |v| {
        v["clean_recipient_vm_attested"] = false.into();
        v["vm_provenance_sha256"] = Value::Null;
        v["environment"]["qualifies_clean"] = false.into();
        v["machine_status"] = "PENDING".into();
        v["pending"] = json!(["supply genuine VM provenance"]);
    });
    fs::remove_file(f.root.join("c/vm-provenance.json")).unwrap();
    assert_eq!(f.status(), Status::Pending);
}
#[test]
fn dirty_vm_still_fails_when_provenance_is_pending() {
    let f = Fixture::new();
    f.mutate("c/spike-c-clean-vm-result.json", |v| {
        v["clean_recipient_vm_attested"] = false.into();
        v["vm_provenance_sha256"] = Value::Null;
        v["environment"]["qualifies_clean"] = false.into();
        v["machine_status"] = "PENDING".into();
        v["environment"]["developer_tools"] = json!(["SDK"]);
    });
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn zero_frame_timing_rejected() {
    let f = Fixture::new();
    f.raw_mutate(|e| e[1]["interval_avg_ms"] = 0.into());
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn slow_frames_require_real_degradation_policy() {
    let f = Fixture::new();
    f.raw_mutate(|e| e[1]["interval_avg_ms"] = 30.into());
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn corrupted_or_zero_size_shell_image_rejected() {
    let f = Fixture::new();
    let mut bytes = fs::read(f.root.join("c/shell-icon.png")).unwrap();
    bytes[16..20].copy_from_slice(&0u32.to_be_bytes());
    fs::write(f.root.join("c/shell-icon.png"), &bytes).unwrap();
    f.mutate("c/spike-c-clean-vm-result.json", |v| {
        v["shell_icon_sha256"] = hash(&bytes).into()
    });
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn summary_counter_tamper_without_raw_change_rejected() {
    let f = Fixture::new();
    f.mutate("d/cells/0/cell.json", |v| {
        v["samples"][0]["pointer_events"] = 12.into()
    });
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn windows_numeric_json_reserialization_keeps_meaning() {
    let f = Fixture::new();
    f.mutate("d/cells/0/cell.json", |v| {
        v["samples"][0]["interval_avg_ms"] = 16.into()
    });
    assert_eq!(f.status(), Status::Pass);
}
#[test]
fn json_nested_duplicate_field_rejected() {
    let f = Fixture::new();
    let p = f.root.join("d/cells/0/cell.json");
    let old = fs::read_to_string(&p).unwrap();
    let altered = old.replace(
        "\"foreground_preserved\":true",
        "\"foreground_preserved\":false,\"foreground_preserved\":true",
    );
    assert_ne!(altered, old);
    fs::write(p, altered).unwrap();
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn explicit_retry_selection_preserves_failed_attempt_audit() {
    let f = Fixture::new();
    let p = f.root.join("d/cells/retry");
    fs::create_dir(&p).unwrap();
    for name in ["cell.json", "probe.jsonl", "stderr.log"] {
        fs::copy(f.root.join("d/cells/0").join(name), p.join(name)).unwrap();
    }
    f.mutate("d/cells/retry/cell.json", |v| v["cell_id"] = "retry".into());
    f.mutate("d/cells/0/cell.json", |v| v["exit_code"] = 1.into());
    let mut selection = doc("afterglow-spike-d-matrix-selection");
    let mut paths = (1..54)
        .map(|i| json!({"cell_path":format!("cells/{i}/cell.json")}))
        .collect::<Vec<_>>();
    paths.push(json!({"cell_path":"cells/retry/cell.json"}));
    selection["selections"] = paths.into();
    selection["exclusions"] = json!([{"cell_path":"cells/0/cell.json","reason":"Human reviewed preserved failed attempt; rerun on corrected physical setup."}]);
    write(&f.root, "d/matrix-selection.json", &selection);
    let report = validate("validate-phase2-evidence", &f.root, Some(&f.trusted));
    assert_eq!(report.status, Status::Pass);
    assert_eq!(report.gates["spike_d"].audit.len(), 1);
    assert!(report.gates["spike_d"].audit[0].contains("original_status=Fail"));
    assert!(f.root.join("d/cells/0/cell.json").exists());
}
#[test]
fn selection_cannot_hide_attempt_without_explanation() {
    let f = Fixture::new();
    let mut selection = doc("afterglow-spike-d-matrix-selection");
    selection["selections"] = (1..54)
        .map(|i| json!({"cell_path":format!("cells/{i}/cell.json")}))
        .collect::<Vec<_>>()
        .into();
    selection["exclusions"] = json!([]);
    write(&f.root, "d/matrix-selection.json", &selection);
    assert_eq!(f.status(), Status::Pending);
}
#[test]
fn selection_duplicate_coordinate_fails() {
    let f = Fixture::new();
    let p = f.root.join("d/cells/retry");
    fs::create_dir(&p).unwrap();
    for name in ["cell.json", "probe.jsonl", "stderr.log"] {
        fs::copy(f.root.join("d/cells/0").join(name), p.join(name)).unwrap();
    }
    f.mutate("d/cells/retry/cell.json", |v| v["cell_id"] = "retry".into());
    let mut selection = doc("afterglow-spike-d-matrix-selection");
    let mut paths = (0..54)
        .map(|i| json!({"cell_path":format!("cells/{i}/cell.json")}))
        .collect::<Vec<_>>();
    paths.push(json!({"cell_path":"cells/retry/cell.json"}));
    selection["selections"] = paths.into();
    selection["exclusions"] = json!([]);
    write(&f.root, "d/matrix-selection.json", &selection);
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn metadata_bom_is_supported() {
    let f = Fixture::new();
    let p = f.root.join("c/spike-c-clean-vm-result.json");
    let mut data = vec![0xef, 0xbb, 0xbf];
    data.extend(fs::read(&p).unwrap());
    fs::write(p, data).unwrap();
    assert_eq!(f.status(), Status::Pass);
}
#[test]
fn raw_jsonl_nan_is_invalid() {
    let f = Fixture::new();
    let p = f.root.join("d/cells/0/probe.jsonl");
    let data = fs::read_to_string(&p)
        .unwrap()
        .replace("\"interval_avg_ms\":16.0", "\"interval_avg_ms\":NaN");
    fs::write(&p, &data).unwrap();
    f.mutate("d/cells/0/cell.json", |v| {
        v["raw_log_sha256"] = hash(data.as_bytes()).into()
    });
    assert_eq!(f.status(), Status::Fail);
}
#[cfg(unix)]
#[test]
fn symlink_evidence_ancestor_rejected() {
    let f = Fixture::new();
    let alias = f.root.parent().unwrap().join("alias");
    std::os::unix::fs::symlink(&f.root, &alias).unwrap();
    assert_eq!(
        validate("validate-phase2-evidence", &alias, Some(&f.trusted)).status,
        Status::Fail
    );
}
#[test]
fn fast_mean_cannot_hide_slow_p95_without_degradation() {
    let f = Fixture::new();
    f.raw_mutate(|events| events[1]["interval_p95_ms"] = 50.into());
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn high_p95_automatic_static_fallback_is_honestly_accepted() {
    let f = Fixture::new();
    f.raw_mutate(|events| {
        events[1]["interval_p95_ms"] = 50.into();
        events[1]["next_mode"] = "static".into();
        for sample in &mut events[2..=3] {
            sample["mode"] = "static".into();
            sample["next_mode"] = "static".into();
        }
    });
    assert_eq!(f.status(), Status::Pass);
}
#[test]
fn moderate_p95_requires_and_accepts_reduced_fallback() {
    let f = Fixture::new();
    f.raw_mutate(|events| {
        events[1]["interval_p95_ms"] = 20.into();
        events[1]["next_mode"] = "reduced".into();
        for sample in &mut events[2..=3] {
            sample["mode"] = "reduced".into();
            sample["next_mode"] = "reduced".into();
        }
    });
    assert_eq!(f.status(), Status::Pass);
}
#[test]
fn standalone_complete_machine_evidence_is_not_a_trust_anchor() {
    let f = Fixture::new();
    assert_eq!(
        validate("validate-spike-d", &f.root.join("d"), None).status,
        Status::Pending
    );
    assert_eq!(
        validate("validate-spike-c", &f.root.join("c"), None).status,
        Status::Pending
    );
}
#[test]
fn contradictory_selection_and_exclusion_fails() {
    let f = Fixture::new();
    let mut selection = doc("afterglow-spike-d-matrix-selection");
    selection["selections"] = (0..54)
        .map(|i| json!({"cell_path":format!("cells/{i}/cell.json")}))
        .collect::<Vec<_>>()
        .into();
    selection["exclusions"] = json!([{"cell_path":"cells/0/cell.json","reason":"This cannot both be selected and excluded"}]);
    write(&f.root, "d/matrix-selection.json", &selection);
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn exclusion_without_reviewed_reason_fails() {
    let f = Fixture::new();
    let mut selection = doc("afterglow-spike-d-matrix-selection");
    selection["selections"] = (1..54)
        .map(|i| json!({"cell_path":format!("cells/{i}/cell.json")}))
        .collect::<Vec<_>>()
        .into();
    selection["exclusions"] = json!([{"cell_path":"cells/0/cell.json","reason":" "}]);
    write(&f.root, "d/matrix-selection.json", &selection);
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn arm64_emulation_cannot_qualify_as_clean_x64() {
    let f = Fixture::new();
    f.mutate("c/spike-c-clean-vm-result.json", |v| {
        v["environment"]["os_native_architecture"] = "ARM64".into()
    });
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn windows_other_than_10_11_client_rejected() {
    let f = Fixture::new();
    f.mutate("c/spike-c-clean-vm-result.json", |v| {
        v["environment"]["os_caption"] = "Windows 12 Client".into()
    });
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn truncated_png_is_rejected_even_with_matching_report_hash() {
    let f = Fixture::new();
    let path = f.root.join("c/shell-icon.png");
    let mut bytes = fs::read(&path).unwrap();
    bytes.truncate(bytes.len() - 5);
    fs::write(path, &bytes).unwrap();
    f.mutate("c/spike-c-clean-vm-result.json", |v| {
        v["shell_icon_sha256"] = hash(&bytes).into()
    });
    assert_eq!(f.status(), Status::Fail);
}
#[test]
fn corrupt_png_crc_is_rejected_even_with_matching_report_hash() {
    let f = Fixture::new();
    let path = f.root.join("c/shell-icon.png");
    let mut bytes = fs::read(&path).unwrap();
    bytes[50] ^= 1;
    fs::write(path, &bytes).unwrap();
    f.mutate("c/spike-c-clean-vm-result.json", |v| {
        v["shell_icon_sha256"] = hash(&bytes).into()
    });
    assert_eq!(f.status(), Status::Fail);
}
