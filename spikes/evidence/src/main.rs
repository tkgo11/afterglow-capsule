use afterglow_phase2_evidence::{Status, validate};
use std::{path::Path, process::ExitCode};
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 && !(args.len() == 5 && args[3] == "--trusted-provenance") {
        eprintln!(
            "usage: phase2-evidence validate-phase2-evidence|validate-spike-c|validate-spike-d <directory> [--trusted-provenance <reviewed-file>]"
        );
        return ExitCode::from(1);
    }
    let report = validate(&args[1], Path::new(&args[2]), args.get(4).map(Path::new));
    match serde_json::to_string_pretty(&report) {
        Ok(s) => println!("{s}"),
        Err(e) => {
            eprintln!("serialization failure: {e}");
            return ExitCode::from(1);
        }
    }
    ExitCode::from(match report.status {
        Status::Pass => 0,
        Status::Fail => 1,
        Status::Pending => 2,
    })
}
