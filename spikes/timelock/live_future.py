"""SPIKE ONLY: exercise real future-round Go/Rust interoperability over HTTPS.

Run only on an environment allowed to reach a reviewed public drand relay. Never
changes pins or treats an HTTP response, clock, or status code as release authority.
Success requires the Rust CLI's exact pinned-round BLS verification and decryption.
"""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request

ROOT = Path(__file__).resolve().parents[2]


def fixture():
    text = (ROOT / "docs/external-assumptions.md").read_text(encoding="utf-8")
    start = text.index("<!-- BEGIN AFTERGLOW HISTORICAL QUICKNET FIXTURE -->")
    start = text.index("```json", start) + len("```json")
    return json.loads(text[start:text.index("```", start)])


def validate_relay(value):
    parsed = urllib.parse.urlsplit(value)
    if (parsed.scheme != "https" or not parsed.hostname or parsed.username
            or parsed.password or parsed.query or parsed.fragment
            or parsed.path not in ["", "/"]):
        raise ValueError("relay must be an HTTPS origin without credentials, path, query or fragment")
    return value.rstrip("/")


def validate_info(info, pinned):
    for name in ["public_key", "period", "genesis_time", "hash", "schemeID"]:
        if info.get(name) != pinned[name]:
            raise ValueError(f"live chain metadata disagrees with the pinned fixture: {name}")


def get_json(url):
    # Preserve normal proxy settings, platform TLS roots, and certificate checks.
    with urllib.request.urlopen(url, timeout=10) as response:
        data = response.read(16385)
    if len(data) > 16384:
        raise ValueError("relay response exceeds the spike limit")
    return json.loads(data)


def is_future_round_unavailable(status):
    # Actual relay observation: a future round may be 425 Too Early, not just
    # 404. This is test scheduling evidence only; no HTTP status grants release.
    # Authentication/server failures must not be disguised as future absence.
    return status in [404, 425]


def call(binary, *args, success=True):
    process = subprocess.run([str(binary), *map(str, args)], capture_output=True, timeout=30)
    if process.stdout:
        raise RuntimeError("spike executable printed unexpected data")
    if (process.returncode == 0) != success:
        raise RuntimeError(process.stderr.decode(errors="replace") or "unexpected spike exit code")


def run(rust, reference, relay):
    public = fixture()
    relay = validate_relay(relay)
    chain = public["chain"]
    base = relay + "/" + chain["hash"]
    validate_info(get_json(base + "/info"), chain)
    with tempfile.TemporaryDirectory(prefix="afterglow-spike-a-live-") as directory:
        folder = Path(directory)
        info = folder / "fixture.json"
        info.write_text(json.dumps(public), encoding="utf-8")
        latest_data = get_json(base + "/public/latest")
        latest = folder / "latest.json"
        latest.write_text(json.dumps(latest_data), encoding="utf-8")
        # The candidate latest is untrusted until the pinned-chain BLS check.
        call(rust, "verify", info, latest, latest_data["round"])
        target = latest_data["round"] + 10
        plain = folder / "public-synthetic.bin"
        plain.write_bytes(bytes(range(32)))
        go_cipher, rust_cipher = folder / "go.tle", folder / "rust.tle"
        call(reference, "encrypt", info, plain, go_cipher, target)
        call(rust, "encrypt", info, plain, rust_cipher, target)
        # The real relay must not have published the target yet (404/425).
        try:
            get_json(base + f"/public/{target}")
        except urllib.error.HTTPError as error:
            if not is_future_round_unavailable(error.code):
                raise
        else:
            raise RuntimeError("target was already available; rerun with a new future round")
        before = folder / "must-not-exist.bin"
        call(rust, "decrypt", info, go_cipher, before, latest, target, success=False)
        call(reference, "decrypt", info, rust_cipher, before, latest, success=False)
        if before.exists():
            raise RuntimeError("pre-round failure persisted plaintext")
        # Monotonic time bounds the test's wait; it never authorizes decryption.
        deadline = time.monotonic() + 120
        while True:
            try:
                beacon_data = get_json(base + f"/public/{target}")
                break
            except urllib.error.HTTPError as error:
                if not is_future_round_unavailable(error.code):
                    raise
            if time.monotonic() >= deadline:
                raise TimeoutError("target-round beacon did not arrive within the spike timeout")
            time.sleep(1)
        beacon = folder / "target.json"
        beacon.write_text(json.dumps(beacon_data), encoding="utf-8")
        call(rust, "verify", info, beacon, target)
        rust_output, go_output = folder / "rust-output.bin", folder / "go-output.bin"
        call(rust, "decrypt", info, go_cipher, rust_output, beacon, target)
        call(reference, "decrypt", info, rust_cipher, go_output, beacon)
        if rust_output.read_bytes() != plain.read_bytes() or go_output.read_bytes() != plain.read_bytes():
            raise RuntimeError("live interoperability plaintext mismatch")
        print(json.dumps({
            "test": "spike-a-live-future-round", "result": "passed",
            "relay": relay, "target_round": target,
            "chain_hash": chain["hash"],
            "verified_signature_sha256": hashlib.sha256(bytes.fromhex(beacon_data["signature"])).hexdigest(),
            "pre_round_rejected": True, "go_to_rust": True, "rust_to_go": True,
        }, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rust", type=Path, required=True)
    parser.add_argument("--go", type=Path, required=True)
    parser.add_argument("--relay", required=True, help="reviewed public HTTPS relay origin")
    args = parser.parse_args()
    run(args.rust.resolve(), args.go.resolve(), args.relay)
