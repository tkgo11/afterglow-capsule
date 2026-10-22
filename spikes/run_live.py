"""Isolated live experiments using centralized candidate inputs, never release code."""

import argparse
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def endpoints():
    text = (ROOT / "docs/external-assumptions.md").read_text(encoding="utf-8")
    start = text.index("<!-- BEGIN AFTERGLOW SPIKE ENDPOINT CANDIDATES -->")
    start = text.index("```json", start) + len("```json")
    data = json.loads(text[start:text.index("```", start)])
    if (data["format_name"] != "afterglow-spike-endpoint-candidates"
            or data["format_version"] != 1 or data["minimum_reader_version"] != 1):
        raise ValueError("unsupported endpoint candidate document")
    return data


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("spike", choices=["timelock", "nts"])
    args = parser.parse_args()
    data = endpoints()
    suffix = ".exe" if sys.platform == "win32" else ""
    binaries = ROOT / "spikes/target/debug"
    if args.spike == "timelock":
        command = [sys.executable, str(ROOT / "spikes/timelock/live_future.py"),
                   "--rust", str(binaries / ("afterglow-spike-a" + suffix)),
                   "--go", str(ROOT / "spikes/target" / ("spike-a-reference" + suffix)),
                   "--relay", data["drand_relay"]]
    else:
        subprocess.run(["cargo", "build", "--manifest-path", "spikes/Cargo.toml",
                        "--locked", "-p", "afterglow-spike-b"], cwd=ROOT, check=True)
        command = [str(binaries / ("afterglow-spike-b" + suffix)),
                   *[entry["host"] for entry in data["nts_operators"]]]
    subprocess.run(command, cwd=ROOT, check=True, timeout=200)


if __name__ == "__main__":
    main()
