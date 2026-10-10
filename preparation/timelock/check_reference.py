"""Public 32-byte CEK differential tests using the pinned official Go wrapper.

This does not replace the mandatory future-round Spike A or accept Phase 2.
"""

import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--rust", type=Path, required=True)
parser.add_argument("--go", type=Path, required=True)
args = parser.parse_args()
text = (ROOT / "docs/external-assumptions.md").read_text(encoding="utf-8")
fixture = json.loads(text.split("<!-- BEGIN AFTERGLOW HISTORICAL QUICKNET FIXTURE -->")[1]
                     .split("```json")[1].split("```")[0])
chain = fixture["chain"]
material = fixture["additional_quicknet_beacon"]
round_number = material["round"]
signature = bytes.fromhex(material["signature"])


def timestamp(seconds):
    return datetime.fromtimestamp(seconds, timezone.utc).isoformat()


release = {"utc": timestamp(chain["genesis_time"] + (round_number - 1) * chain["period"]),
           "timezone": "UTC", "target_round": round_number, "timelock_profile": "quicknet-v1",
           "network": {"profile_id": "quicknet-v1", "chain_hash": chain["hash"],
                       "public_key": chain["public_key"], "genesis_utc": timestamp(chain["genesis_time"]),
                       "period_seconds": chain["period"], "scheme_id": chain["schemeID"],
                       "relays": ["https://relay-one.example", "https://relay-two.example"]}}


def call(binary, *arguments, success=True):
    result = subprocess.run([str(binary.resolve()), *map(str, arguments)],
                            capture_output=True, timeout=30)
    if result.stdout or (result.returncode == 0) != success:
        raise RuntimeError(f"Differential command failed: {result.stderr.decode(errors='replace')}")


with tempfile.TemporaryDirectory(prefix="afterglow-prepared-public-cek-") as folder:
    path = Path(folder)
    info = path / "fixture.json"
    info.write_text(json.dumps(fixture), encoding="utf-8")
    pinned = path / "release.json"
    pinned.write_text(json.dumps(release), encoding="utf-8")
    beacon = path / "beacon.json"
    beacon.write_text(json.dumps({**material, "randomness": hashlib.sha256(signature).hexdigest()}), encoding="utf-8")
    cek = path / "public-cek.bin"
    cek.write_bytes(bytes(range(32)))
    rust_cipher = path / "rust.tle"
    go_cipher = path / "go.tle"
    call(args.rust, "lock", pinned, cek, rust_cipher)
    call(args.go, "encrypt", info, cek, go_cipher, round_number)
    for name, cipher, decryptor in [("go-to-rust", go_cipher, args.rust),
                                    ("rust-to-go", rust_cipher, args.go)]:
        output = path / (name + ".bin")
        if decryptor == args.rust:
            call(decryptor, "unlock", pinned, cipher, beacon, output)
        else:
            call(decryptor, "decrypt", info, cipher, output, beacon)
        if output.read_bytes() != cek.read_bytes():
            raise RuntimeError("Interoperability recovered a different CEK")
    for original in (rust_cipher, go_cipher):
        data = original.read_bytes()
        variants = [data[:-1]]
        for offset in (0, len(data) // 2, len(data) - 1):
            changed = bytearray(data)
            changed[offset] ^= 1
            variants.append(bytes(changed))
        for index, variant in enumerate(variants):
            corrupted = path / f"corrupted-{index}.tle"
            corrupted.write_bytes(variant)
            output = path / "must-not-exist.bin"
            call(args.rust, "unlock", pinned, corrupted, beacon, output, success=False)
            if output.exists():
                raise RuntimeError("Failed decryption wrote plaintext")
    print(json.dumps({"status": "PASS", "scope": "historical public CEK differential only",
                      "directions": ["Go to prepared Rust", "prepared Rust to Go"],
                      "phase2_accepted": False}))
