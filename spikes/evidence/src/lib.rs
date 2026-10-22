//! SPIKE ONLY. Strict evidence review, never a production release capability.
mod input;
mod spike_c;
mod spike_d;
use input::{Result, *};
use serde::Serialize;
use serde_json::Value;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Status {
    Pass,
    Fail,
    Pending,
}
#[derive(Debug, Serialize)]
pub struct Gate {
    pub status: Status,
    pub reasons: Vec<String>,
    pub audit: Vec<String>,
}
impl Default for Gate {
    fn default() -> Self {
        Self {
            status: Status::Pass,
            reasons: vec![],
            audit: vec![],
        }
    }
}
impl Gate {
    pub fn fail(&mut self, reason: impl Into<String>) {
        self.status = Status::Fail;
        self.reasons.push(reason.into());
    }
    pub fn pending(&mut self, reason: impl Into<String>) {
        if self.status != Status::Fail {
            self.status = Status::Pending;
        }
        self.reasons.push(reason.into());
    }
    pub fn check(&mut self, r: Result<()>) {
        if let Err(e) = r {
            self.fail(e);
        }
    }
    pub fn load(&mut self, root: &Path, path: &str) -> Option<Value> {
        match json(root, path) {
            Ok(v) => Some(v),
            Err(e) => {
                if !root.join(path).exists() {
                    self.pending(e);
                } else {
                    self.fail(e);
                }
                None
            }
        }
    }
    pub fn format(&mut self, v: &Value, name: &str) -> bool {
        match document(v, name) {
            Ok(()) => true,
            Err(e) => {
                self.pending(e);
                false
            }
        }
    }
    pub fn human(&mut self, v: &Value, key: &str) {
        match v.get(key) {
            Some(Value::Bool(true)) => {}
            Some(Value::Null) | None => self.pending(format!("human observation {key} is pending")),
            _ => self.fail(format!("human observation {key} failed or is invalid")),
        }
    }
}
#[derive(Debug, Serialize)]
pub struct Report {
    pub format_name: &'static str,
    pub format_version: u16,
    pub minimum_reader_version: u16,
    pub status: Status,
    pub reasons: Vec<String>,
    pub gates: std::collections::BTreeMap<String, Gate>,
}
impl Report {
    pub fn from_gates(gates: std::collections::BTreeMap<String, Gate>) -> Self {
        let status = if gates.values().any(|g| g.status == Status::Fail) {
            Status::Fail
        } else if gates.values().any(|g| g.status == Status::Pending) {
            Status::Pending
        } else {
            Status::Pass
        };
        let reasons = gates
            .iter()
            .flat_map(|(name, g)| g.reasons.iter().map(move |r| format!("{name}: {r}")))
            .collect();
        Self {
            format_name: "afterglow-phase2-validation-result",
            format_version: 2,
            minimum_reader_version: 2,
            status,
            reasons,
            gates,
        }
    }
}
fn trusted_artifact(
    g: &mut Gate,
    root: &Path,
    expected: &Value,
    manifest: &str,
    exe: &str,
    trusted: Option<&Value>,
    key: &str,
) {
    g.check(
        expected
            .get("provenance")
            .ok_or("missing artifact provenance".into())
            .and_then(provenance),
    );
    let Some(t) = trusted.and_then(|v| v.get(key)) else {
        g.pending("independently reviewed trusted artifact provenance is missing");
        return;
    };
    g.check((|| {
        same(expected, t, "provenance")?;
        let hash_key = if key == "spike_c" {
            "signed_exe_sha256"
        } else {
            "exe_sha256"
        };
        if !text(expected, hash_key)?.eq_ignore_ascii_case(text(t, "exe_sha256")?) {
            return Err("trusted executable hash mismatch".into());
        }
        digest(
            root,
            manifest,
            text(t, "expected_manifest_sha256")?,
            JSON_LIMIT,
        )?;
        digest(root, exe, text(t, "exe_sha256")?, EXE_LIMIT)?;
        Ok(())
    })());
}
fn reference(v: Option<&Value>) -> Gate {
    let mut g = Gate::default();
    let Some(v) = v else {
        g.pending("independently reviewed A/B reference missing");
        return g;
    };
    g.check((|| {
        provenance(v)?;
        if text(v, "status")? != "PASS" {
            return Err("reviewed reference did not pass".into());
        }
        let id = integer(v, "job_id")?;
        if id == 0 {
            return Err("invalid reference job ID".into());
        }
        let url = format!(
            "https://github.com/tkgo11/afterglow-capsule/actions/runs/{}/job/{id}",
            integer(v, "workflow_run_id")?
        );
        if text(v, "evidence_url")? != url {
            return Err("reference evidence URL/run/job mismatch".into());
        }
        text(v, "review_note")?;
        Ok(())
    })());
    g
}
pub fn validate(command: &str, root: &Path, trusted_path: Option<&Path>) -> Report {
    let mut gates = std::collections::BTreeMap::new();
    let mut trust_gate = Gate::default();
    let trusted = trusted_path.and_then(|path| {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            trust_gate.fail("invalid trusted provenance filename");
            return None;
        };
        match json(parent, name) {
            Ok(v) => {
                trust_gate.check(document(&v, "afterglow-phase2-trusted-provenance"));
                Some(v)
            }
            Err(e) => {
                trust_gate.fail(format!("trusted provenance: {e}"));
                None
            }
        }
    });
    if trusted_path.is_none() {
        trust_gate.pending("--trusted-provenance is required for gate acceptance; collector self-report is not a trust anchor");
    }
    gates.insert("trusted_provenance".into(), trust_gate);
    match command {
        "validate-phase2-evidence" => {
            gates.insert(
                "spike_a".into(),
                reference(trusted.as_ref().and_then(|v| v.get("spike_a"))),
            );
            gates.insert(
                "spike_b".into(),
                reference(trusted.as_ref().and_then(|v| v.get("spike_b"))),
            );
            gates.insert(
                "spike_c".into(),
                spike_c::validate(&root.join("c"), trusted.as_ref()),
            );
            gates.insert(
                "spike_d".into(),
                spike_d::validate(&root.join("d"), trusted.as_ref()),
            );
        }
        "validate-spike-c" => {
            gates.insert("spike_c".into(), spike_c::validate(root, trusted.as_ref()));
        }
        "validate-spike-d" => {
            gates.insert("spike_d".into(), spike_d::validate(root, trusted.as_ref()));
        }
        _ => {
            let mut g = Gate::default();
            g.fail("unknown command");
            gates.insert("command".into(), g);
        }
    }
    Report::from_gates(gates)
}
