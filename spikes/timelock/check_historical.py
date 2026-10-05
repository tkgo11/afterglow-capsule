"""Historical/public Spike A subset. Success does not complete the phase gate."""

import argparse
import copy
import hashlib
import json
import math
from pathlib import Path
import subprocess
import tempfile
import time
import unittest

ROOT = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--rust", type=Path, required=True)
parser.add_argument("--go", type=Path)
args = parser.parse_args()
RUST = args.rust.resolve()
GO = args.go.resolve() if args.go else None


def public_fixture():
    text = (ROOT / "docs/external-assumptions.md").read_text(encoding="utf-8")
    start = text.index("<!-- BEGIN AFTERGLOW HISTORICAL QUICKNET FIXTURE -->")
    start = text.index("```json", start) + len("```json")
    end = text.index("```", start)
    return json.loads(text[start:end])


class HistoricalInteroperability(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="afterglow-spike-a-")
        self.addCleanup(self.temp.cleanup)
        self.folder = Path(self.temp.name)
        self.fixture = public_fixture()
        self.info = self.write_json("fixture.json", self.fixture)
        self.beacon_data = self.fixture["historical_beacon"].copy()
        self.beacon_data["randomness"] = hashlib.sha256(bytes.fromhex(self.beacon_data["signature"])).hexdigest()
        self.beacon = self.write_json("beacon.json", self.beacon_data)
        self.round = self.beacon_data["round"]
        self.plain = self.folder / "public-synthetic-payload.bin"
        self.plain.write_bytes(bytes(range(32)))

    def write_json(self, name, value):
        path = self.folder / name
        path.write_text(json.dumps(value), encoding="utf-8")
        return path

    def call(self, binary, *arguments, success=True):
        process = subprocess.run([str(binary), *map(str, arguments)], capture_output=True, timeout=30)
        self.assertEqual(process.stdout, b"", "spike must not print plaintext")
        if success:
            self.assertEqual(process.returncode, 0, process.stderr.decode(errors="replace"))
        else:
            self.assertNotEqual(process.returncode, 0, "invalid material was accepted")
        return process

    def encrypt_rust(self, round_number=None):
        path = self.folder / "rust.tle"
        self.call(RUST, "encrypt", self.info, self.plain, path, round_number or self.round)
        return path

    def reject_rust(self, cipher, beacon=None, fixture=None, round_number=None):
        output = self.folder / "must-not-exist.bin"
        self.call(RUST, "decrypt", fixture or self.info, cipher, output,
                  beacon or self.beacon, round_number or self.round, success=False)
        self.assertFalse(output.exists(), "failed decryption persisted plaintext")

    def test_historical_beacon_verification_and_rust_round_trip(self):
        self.call(RUST, "verify", self.info, self.beacon, self.round)
        cipher = self.encrypt_rust()
        output = self.folder / "rust.bin"
        self.call(RUST, "decrypt", self.info, cipher, output, self.beacon, self.round)
        self.assertEqual(output.read_bytes(), self.plain.read_bytes())

    def test_published_official_go_reference_ciphertext_decrypts_in_rust(self):
        cipher = ROOT / "test-vectors/timelock/reference-historical.tle"
        output = self.folder / "published-reference.bin"
        self.call(RUST, "decrypt", self.info, cipher, output, self.beacon, self.round)
        self.assertEqual(output.read_bytes(), b"hello world")

    @unittest.skipIf(GO is None, "Go executable not supplied; differential evidence missing")
    def test_go_encrypt_rust_decrypt(self):
        cipher = self.folder / "go.tle"
        self.call(GO, "encrypt", self.info, self.plain, cipher, self.round)
        output = self.folder / "rust-from-go.bin"
        self.call(RUST, "decrypt", self.info, cipher, output, self.beacon, self.round)
        self.assertEqual(output.read_bytes(), self.plain.read_bytes())

    @unittest.skipIf(GO is None, "Go executable not supplied; differential evidence missing")
    def test_rust_encrypt_go_decrypt(self):
        cipher = self.encrypt_rust()
        output = self.folder / "go-from-rust.bin"
        self.call(GO, "decrypt", self.info, cipher, output, self.beacon)
        self.assertEqual(output.read_bytes(), self.plain.read_bytes())

    def test_future_round_ciphertext_rejects_available_historical_beacon(self):
        chain = self.fixture["chain"]
        # Build-time choice only: no fake clock/beacon exists in production code.
        future = math.ceil((time.time() + 600 - chain["genesis_time"]) / chain["period"]) + 1
        cipher = self.encrypt_rust(future)
        self.reject_rust(cipher, round_number=future)
        if GO:
            go_cipher = self.folder / "go-future.tle"
            self.call(GO, "encrypt", self.info, self.plain, go_cipher, future)
            self.reject_rust(go_cipher, round_number=future)
        # No after-round live decryption is claimed by this offline test.

    def test_wrong_round_and_wrong_chain_are_rejected(self):
        cipher = self.encrypt_rust()
        self.reject_rust(cipher, round_number=self.round + 1)
        altered = copy.deepcopy(self.fixture)
        altered["chain"]["hash"] = "ff" * 32
        wrong_chain = self.write_json("wrong-chain.json", altered)
        self.reject_rust(cipher, fixture=wrong_chain)
        if GO:
            output = self.folder / "go-wrong-chain.bin"
            self.call(GO, "decrypt", wrong_chain, cipher, output, self.beacon, success=False)
            self.assertFalse(output.exists())

    def test_wrong_beacon_round_signature_randomness_and_public_key_are_rejected(self):
        cipher = self.encrypt_rust()
        for name, changes in [
            ("wrong-round", {"round": self.round + 1}),
            ("invalid-signature", {"signature": "00" * 48}),
            ("invalid-randomness", {"randomness": "00" * 32}),
        ]:
            beacon = self.write_json(name + ".json", self.beacon_data | changes)
            self.reject_rust(cipher, beacon=beacon)
        altered = copy.deepcopy(self.fixture)
        altered["chain"]["public_key"] = "00" * 96
        self.reject_rust(cipher, fixture=self.write_json("bad-public-key.json", altered))

    def test_foreign_chain_beacon_at_the_same_round_is_rejected(self):
        own = self.fixture["additional_quicknet_beacon"].copy()
        own["randomness"] = hashlib.sha256(bytes.fromhex(own["signature"])).hexdigest()
        foreign = self.fixture["historical_foreign_beacon"].copy()
        foreign["randomness"] = hashlib.sha256(bytes.fromhex(foreign["signature"])).hexdigest()
        self.assertEqual(own["round"], foreign["round"])
        good_beacon = self.write_json("round-1000.json", own)
        bad_beacon = self.write_json("foreign-chain-round-1000.json", foreign)
        cipher = self.encrypt_rust(own["round"])
        output = self.folder / "round-1000.bin"
        self.call(RUST, "decrypt", self.info, cipher, output, good_beacon, own["round"])
        self.assertEqual(output.read_bytes(), self.plain.read_bytes())
        self.reject_rust(cipher, beacon=bad_beacon, round_number=foreign["round"])

    def test_ciphertext_modification_and_truncation_do_not_persist_plaintext(self):
        cipher = self.encrypt_rust()
        original = cipher.read_bytes()
        for offset in [0, len(original) // 2, len(original) - 1]:
            altered = bytearray(original)
            altered[offset] ^= 1
            corrupt = self.folder / "corrupt.tle"
            corrupt.write_bytes(altered)
            self.reject_rust(corrupt)
            if GO:
                output = self.folder / "go-corrupt.bin"
                self.call(GO, "decrypt", self.info, corrupt, output, self.beacon, success=False)
                self.assertFalse(output.exists())
        truncated = self.folder / "truncated.tle"
        truncated.write_bytes(original[:-1])
        self.reject_rust(truncated)


if __name__ == "__main__":
    unittest.main(argv=["historical-spike-a"], verbosity=2)
