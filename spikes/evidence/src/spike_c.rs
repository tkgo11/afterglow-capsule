use crate::{Gate, input::*, trusted_artifact};
use serde_json::Value;
use std::path::Path;

pub fn validate(root: &Path, trusted: Option<&Value>) -> Gate {
    let mut g = Gate::default();
    let Some(expected) = g.load(root, "spike-c-expected.json") else {
        return g;
    };
    if !g.format(&expected, "afterglow-spike-c-evidence") {
        return g;
    }
    trusted_artifact(
        &mut g,
        root,
        &expected,
        "spike-c-expected.json",
        "SpikeC-Standalone.exe",
        trusted,
        "spike_c",
    );
    g.check((|| {
        let exe = digest(
            root,
            "SpikeC-Standalone.exe",
            text(&expected, "signed_exe_sha256")?,
            EXE_LIMIT,
        )?;
        architecture(&exe)?;
        if text(&expected, "file_version")? != "2.0.0.0" {
            return Err("expected native version must be 2.0.0.0".into());
        }
        resources(&expected)?;
        Ok(())
    })());
    let Some(report) = g.load(root, "spike-c-clean-vm-result.json") else {
        return g;
    };
    if !g.format(&report, "afterglow-spike-c-clean-vm-result") {
        return g;
    }
    g.check((|| {
        same(&report, &expected, "provenance")?;
        same(&report, &expected, "file_version")?;
        for key in ["capsule", "icon", "group-icon", "version"] {
            if !text(&report["resource_sha256"], key)?.eq_ignore_ascii_case(text(&expected["resource_sha256"], key)?) {
                return Err(format!("resource hash mismatch: {key}"));
            }
        }
        if !text(&report,"exe_sha256")?.eq_ignore_ascii_case(text(&expected,"signed_exe_sha256")?) {return Err("recipient executable hash mismatch".into());}
        if text(&report,"architecture")?!="x86_64" || text(&report,"one_file_execution")?!="passed" {return Err("recipient architecture or one-file execution failed".into());}
        resources(&report)?;
        let env=report.get("environment").ok_or("missing recipient environment")?;
        if integer(env,"product_type")?!=1 {return Err("Windows Server/development runner does not qualify as clean recipient Windows client".into());}
        yes(env,"process_64_bit")?;no(env,"hosted_ci_detected")?;yes(env,"virtual_machine_detected")?;
        for key in ["os_caption","os_version","manufacturer","model"] {text(env,key)?;}
        if !text(env,"os_caption")?.contains("Windows") {return Err("recipient is not Windows".into());}
        if !matches!(text(env,"os_architecture")?,"64-bit"|"x86_64"|"AMD64") {return Err("recipient OS architecture is not x64".into());}
        let build=text(env,"os_build")?;if build.parse::<u32>().ok().filter(|v|*v>=10240).is_none() {return Err("recipient Windows build is invalid".into());}
        if !array(env,"developer_tools")?.is_empty() {return Err("developer tooling is installed on recipient VM".into());}
        Ok(())
    })());
    match report
        .get("environment")
        .and_then(|env| env.get("qualifies_clean"))
    {
        Some(Value::Bool(true)) => {}
        Some(Value::Bool(false))
            if report.get("machine_status").and_then(Value::as_str) == Some("PENDING")
                && report.get("clean_recipient_vm_attested") == Some(&Value::Bool(false)) =>
        {
            g.pending("clean recipient environment awaits genuine VM provenance");
        }
        _ => g.fail("environment does not qualify as clean"),
    }
    match report.get("clean_recipient_vm_attested") {
        Some(Value::Bool(true)) => {}
        Some(Value::Bool(false)) | Some(Value::Null) | None => {
            g.pending("clean recipient VM attestation is missing")
        }
        _ => g.fail("invalid clean VM attestation type"),
    }
    if report
        .get("vm_provenance_sha256")
        .is_none_or(Value::is_null)
        && report.get("clean_recipient_vm_attested") == Some(&Value::Bool(false))
    {
        g.pending("VM provenance file/hash is missing");
    } else {
        g.check((|| {
            let path = text(&report, "vm_provenance_file")?;
            let bytes = digest(
                root,
                path,
                text(&report, "vm_provenance_sha256")?,
                JSON_LIMIT,
            )?;
            let vm = parse(&bytes)?;
            document(&vm, "afterglow-spike-c-vm-provenance")?;
            for key in ["origin", "base_image", "note"] {
                text(&vm, key)?;
            }
            if text(&vm, "installation_kind")? != "clean-windows-vm" {
                return Err("VM is not a clean Windows installation".into());
            }
            yes(&vm, "clean_recipient_attested")?;
            Ok(())
        })());
    }
    g.check((|| {
        let bytes = digest(
            root,
            text(&report, "shell_icon_file")?,
            text(&report, "shell_icon_sha256")?,
            JSON_LIMIT,
        )?;
        if bytes.len() < 45 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" || &bytes[12..16] != b"IHDR" {
            return Err("shell icon evidence is not a PNG image".into());
        }
        let width = u32::from_be_bytes(bytes[16..20].try_into().map_err(|_| "invalid PNG width")?);
        let height =
            u32::from_be_bytes(bytes[20..24].try_into().map_err(|_| "invalid PNG height")?);
        if width == 0 || height == 0 || width > 8192 || height > 8192 {
            return Err("invalid shell icon evidence dimensions".into());
        }
        Ok(())
    })());
    if let Some(human) = report.get("shell_icon_observation") {
        g.human(human, "correct");
        if human.get("correct") == Some(&Value::Bool(true)) {
            g.check(text(human, "note").map(|_| ()));
        }
    } else {
        g.pending("shell icon visual observation missing");
    }
    match report.get("machine_status").and_then(Value::as_str) {
        Some("FAIL") => g.fail("recipient collector reported machine failure"),
        Some("PENDING") => g.pending("recipient collector machine checks are pending"),
        Some("PASS") => {}
        _ => g.fail("invalid recipient machine status"),
    }
    match array(&report, "failures") {
        Ok(v) if v.is_empty() => {}
        Ok(v) => g.fail(format!("recipient failures: {v:?}")),
        Err(e) => g.fail(e),
    }
    match array(&report, "pending") {
        Ok(v) if v.is_empty() => {}
        Ok(v) => g.pending(format!("recipient pending: {v:?}")),
        Err(e) => g.fail(e),
    }
    g
}
fn resources(v: &Value) -> Result<()> {
    let resources = v
        .get("resource_sha256")
        .and_then(Value::as_object)
        .ok_or("missing resource SHA-256 map")?;
    if resources.len() != 4 {
        return Err("exactly four resource hashes are required".into());
    }
    for key in ["capsule", "icon", "group-icon", "version"] {
        hex(
            resources
                .get(key)
                .and_then(Value::as_str)
                .ok_or(format!("missing resource hash {key}"))?,
            64,
        )?;
    }
    Ok(())
}
fn architecture(bytes: &[u8]) -> Result<()> {
    if bytes.len() < 64 || &bytes[..2] != b"MZ" {
        return Err("executable DOS header is invalid".into());
    }
    let offset =
        u32::from_le_bytes(bytes[60..64].try_into().map_err(|_| "invalid PE offset")?) as usize;
    let end = offset
        .checked_add(6)
        .ok_or("PE header arithmetic overflow")?;
    let header = bytes.get(offset..end).ok_or("PE header is out of bounds")?;
    if &header[..4] != b"PE\0\0" || header[4..6] != [0x64, 0x86] {
        return Err("executable is not native PE x64".into());
    }
    Ok(())
}
