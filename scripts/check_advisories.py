"""Review existing Rust locks against one recorded RustSec snapshot, without fixes."""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
LOCKS = {"root": "Cargo.lock", "spikes": "spikes/Cargo.lock", "fuzz": "fuzz/Cargo.lock"}
VERSION = "cargo-audit 0.22.2"


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def inspect_report(report, exit_code):
    """A tool failure, inconsistent report or unsoundness cannot become PASS."""
    reasons = []
    if exit_code != 0:
        reasons.append(f"cargo-audit exited {exit_code}")
    if not isinstance(report, dict):
        return reasons + ["Missing JSON audit report"], []
    for section, field in (("database", "advisory-count"), ("lockfile", "dependency-count")):
        value = report.get(section)
        if not isinstance(value, dict) or type(value.get(field)) is not int or value[field] <= 0:
            reasons.append(f"Missing or empty {section} coverage")
    vulnerabilities = report.get("vulnerabilities")
    if (not isinstance(vulnerabilities, dict)
            or type(vulnerabilities.get("found")) is not bool
            or type(vulnerabilities.get("count")) is not int
            or not isinstance(vulnerabilities.get("list"), list)):
        reasons.append("Malformed vulnerability results")
    else:
        count = vulnerabilities["count"]
        if count != len(vulnerabilities["list"]) or vulnerabilities["found"] != (count > 0):
            reasons.append("Inconsistent vulnerability results")
        if count != 0 or vulnerabilities["found"]:
            reasons.append("Reported vulnerable dependencies")
    warnings = report.get("warnings")
    findings = []
    if not isinstance(warnings, dict):
        reasons.append("Missing warning results")
    else:
        for kind, entries in warnings.items():
            if not isinstance(entries, list):
                reasons.append("Malformed warning results")
                continue
            for entry in entries:
                try:
                    findings.append({"kind": kind, "id": entry["advisory"]["id"],
                                     "package": entry["package"]["name"], "version": entry["package"]["version"]})
                except (KeyError, TypeError):
                    reasons.append("Malformed warning entry")
            if kind == "unsound" and entries:
                reasons.append("Reported unsound dependency")
    settings = report.get("settings", {})
    if not isinstance(settings, dict):
        return reasons + ["Malformed audit settings"], findings
    if settings.get("ignore") != [] or settings.get("target_arch") != [] or settings.get("target_os") != []:
        reasons.append("Advisories ignored or target-filtered, or settings missing")
    if "severity" not in settings or settings["severity"] is not None:
        reasons.append("Severity-filtered audit or missing severity settings")
    enabled = settings.get("informational_warnings")
    if (not isinstance(enabled, list) or not all(type(kind) is str for kind in enabled)
            or not {"unmaintained", "unsound", "notice"}.issubset(enabled)):
        reasons.append("Required informational warnings not enabled")
    return reasons, findings


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--db", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--audit-bin", default="cargo-audit")
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    database = args.db.resolve()
    version = subprocess.check_output([args.audit_bin, "--version"], text=True, timeout=10).strip()
    if version != VERSION:
        raise RuntimeError(f"Expected {VERSION}, got {version}")
    revision = subprocess.check_output(["git", "-C", str(database), "rev-parse", "HEAD"], text=True, timeout=10).strip()
    origin = subprocess.check_output(["git", "-C", str(database), "remote", "get-url", "origin"], text=True, timeout=10).strip()
    if origin.rstrip("/").removesuffix(".git") != "https://github.com/RustSec/advisory-db":
        raise RuntimeError("Use a separately cloned official RustSec advisory database")
    if subprocess.check_output(["git", "-C", str(database), "status", "--porcelain"], text=True, timeout=10).strip():
        raise RuntimeError("Advisory database has uncommitted changes")
    lock_hashes = {relative: digest(ROOT / relative) for relative in LOCKS.values()}
    results = []
    for graph, relative in LOCKS.items():
        lock = ROOT / relative
        before = lock_hashes[relative]
        log = output / f"{graph}-audit.json"
        error = output / f"{graph}-audit.stderr.log"
        exit_code = -1
        with log.open("wb") as stdout, error.open("wb") as stderr:
            try:
                process = subprocess.run([args.audit_bin, "audit", "--file", str(lock), "--db", str(database),
                                          "--no-fetch", "--deny", "unsound", "--json"],
                                         cwd=ROOT, stdout=stdout, stderr=stderr, timeout=180)
                exit_code = process.returncode
            except subprocess.TimeoutExpired:
                stderr.write(b"Audit process exceeded the 180-second deadline\n")
        try:
            report = json.loads(log.read_bytes())
        except (ValueError, OSError):
            report = None
        reasons, findings = inspect_report(report, exit_code)
        if digest(lock) != before:
            reasons.append("Audit changed the lockfile")
        result = {"graph": graph, "status": "FAIL" if reasons else "PASS", "exit_code": exit_code,
                  "lockfile": relative, "lockfile_sha256": before, "reasons": reasons, "warnings": findings,
                  "report": log.name, "report_sha256": digest(log), "stderr": error.name, "stderr_sha256": digest(error)}
        results.append(result)
        print(json.dumps(result), flush=True)
    unchanged = all(digest(ROOT / relative) == expected for relative, expected in lock_hashes.items())
    summary = {"format_name": "afterglow-dependency-advisory-result", "format_version": 1, "minimum_reader_version": 1,
               "status": "PASS" if unchanged and all(item["status"] == "PASS" for item in results) else "FAIL",
               "lockfiles_unchanged": unchanged,
               "source_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True, timeout=10).strip(),
               "tool": version, "advisory_database": {"origin": origin, "revision": revision}, "results": results,
               "scope": "Published RustSec snapshot, no target/severity/advisory exclusions. Informational warnings retained; not a comprehensive audit or phase acceptance."}
    (output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    if summary["status"] != "PASS":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
