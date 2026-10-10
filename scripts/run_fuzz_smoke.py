"""Bounded instrumented core/preparation parser smoke; not accepted hardening."""

import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
NIGHTLY = "nightly-2025-09-15"
OUTPUT = ROOT / "fuzz/target/smoke-evidence"
TARGETS = {"capsule_header": 4096, "manifest": 65536, "object_table": 65536, "workspace": 65536,
           "prepared_encrypted_object": 1024 * 1024}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    OUTPUT.mkdir(parents=True, exist_ok=True)
    locks = {path: digest(path) for path in [ROOT / "Cargo.lock", ROOT / "fuzz/Cargo.lock", ROOT / "preparation/Cargo.lock"]}
    version = subprocess.check_output(["cargo", "fuzz", "--version"], cwd=ROOT, text=True).strip()
    if version != "cargo-fuzz 0.13.2":
        raise RuntimeError("Install the pinned cargo-fuzz 0.13.2")
    subprocess.run(["cargo", f"+{NIGHTLY}", "fetch", "--manifest-path", "fuzz/Cargo.toml", "--locked"], cwd=ROOT, check=True, timeout=120)
    results = []
    for target, max_length in TARGETS.items():
        log = OUTPUT / f"{target}.log"
        environment = dict(os.environ, CARGO_NET_OFFLINE="true")
        corpus = []
        if target == "prepared_encrypted_object":
            # Copy the independent public vector into an ignored, mutable corpus.
            # libFuzzer never writes to the committed vector or to creator files.
            seeds = ROOT / "fuzz/corpus/prepared_encrypted_object"
            seeds.mkdir(parents=True, exist_ok=True)
            vector = json.loads((ROOT / "preparation/object-crypto/tests/public-vector.json").read_text())
            (seeds / "independent-public-vector").write_bytes(bytes.fromhex(vector["encrypted_object"]))
            corpus = [str(seeds)]
        with log.open("wb") as stream:
            process = subprocess.run([
                "cargo", f"+{NIGHTLY}", "fuzz", "run", target, *corpus, "--",
                f"-max_len={max_length}", "-max_total_time=30", "-timeout=10",
            ], cwd=ROOT, env=environment, stdout=stream, stderr=subprocess.STDOUT, timeout=300)
        content = log.read_text(errors="replace")
        counters = re.search(r"Loaded \d+ modules\s+\((\d+) inline", content)
        runs = re.search(r"Done (\d+) runs in (\d+) second", content)
        passed = process.returncode == 0 and counters is not None and int(counters[1]) > 0 and runs is not None
        result = {"target": target, "status": "PASS" if passed else "FAIL", "exit_code": process.returncode,
                  "instrumented_counters": int(counters[1]) if counters else None,
                  "runs": int(runs[1]) if runs else None, "seconds": int(runs[2]) if runs else None,
                  "log": log.name, "log_sha256": digest(log)}
        results.append(result)
        print(json.dumps(result), flush=True)
    unchanged = all(digest(path) == value for path, value in locks.items())
    report = {"format_name": "afterglow-fuzz-smoke", "format_version": 1, "minimum_reader_version": 1,
              "nightly": NIGHTLY, "cargo_fuzz": version, "lockfiles_unchanged": unchanged, "results": results,
              "limit": "30 seconds per core/preparation target; not exhaustive hardening or phase acceptance"}
    (OUTPUT / "summary.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    if not unchanged or any(result["status"] != "PASS" for result in results):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
