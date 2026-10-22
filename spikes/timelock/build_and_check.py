"""Build and run the historical-only differential suite on Linux/Windows CI."""

from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
suffix = ".exe" if sys.platform == "win32" else ""
rust = ROOT / "spikes/target/debug" / ("afterglow-spike-a" + suffix)
reference = ROOT / "spikes/target" / ("spike-a-reference" + suffix)
subprocess.run(["cargo", "build", "--manifest-path", "spikes/Cargo.toml", "--locked", "-p", "afterglow-spike-a"], cwd=ROOT, check=True)
subprocess.run(["go", "build", "-mod=readonly", "-o", str(reference), "."], cwd=ROOT / "spikes/timelock/go", check=True)
subprocess.run([sys.executable, "spikes/timelock/check_historical.py", "--rust", str(rust), "--go", str(reference)], cwd=ROOT, check=True)
