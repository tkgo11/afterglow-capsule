"""Independent public vectors using cryptography 50.0.0, not production code.

Run explicitly when reviewing the binary protocol. Do not regenerate to bless a
Rust failure. Fixed fixture keys are public and never imported by applications.
"""

import json
from pathlib import Path
import struct
from uuid import UUID

import cryptography
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
from cryptography.hazmat.primitives.kdf.hkdf import HKDF

assert cryptography.__version__ == "50.0.0"
project = UUID("00112233-4455-4677-8899-aabbccddeeff")
build = UUID("10213243-5465-4768-899a-abbccddeeff0")
object_id = UUID("21324354-6576-4879-9aab-bccddeeff001")
cek = bytes(range(32))
prefix = bytes.fromhex("1020304050607080")
plaintext = b"AFTERGLOW independent public object vector\n"
chunk_size = 16
count = (len(plaintext) + chunk_size - 1) // chunk_size
header = (b"AGOBJ1\x00\x00" + struct.pack("<HHHBB", 1, 1, 96, 0, 0)
          + project.bytes + build.bytes + object_id.bytes + prefix
          + struct.pack("<IIQQ", chunk_size, count, len(plaintext), len(plaintext)))
assert len(header) == 96
key = HKDF(algorithm=hashes.SHA256(), length=32, salt=project.bytes,
           info=b"AFTERGLOW/object/v1/" + object_id.bytes).derive(cek)
encrypted = bytearray(header)
chunks = []
for index in range(count):
    chunk = plaintext[index * chunk_size:(index + 1) * chunk_size]
    record = struct.pack("<III", index, len(chunk), len(chunk))
    nonce = prefix + struct.pack(">I", index)
    ciphertext = AESGCM(key).encrypt(nonce, chunk, header + record)
    encrypted.extend(record + ciphertext)
    chunks.append({"nonce": nonce.hex(), "aad": (header + record).hex(),
                   "ciphertext_and_tag": ciphertext.hex()})
fixture = {"format_name": "afterglow-object-crypto-public-vector",
           "format_version": 1, "minimum_reader_version": 1,
           "generator": "Python cryptography 50.0.0 HKDF(SHA256) and AESGCM",
           "cek": cek.hex(), "project_id": str(project), "build_id": str(build),
           "object_id": str(object_id), "nonce_prefix": prefix.hex(),
           "chunk_size": chunk_size, "plaintext": plaintext.hex(),
           "object_key": key.hex(), "chunks": chunks, "encrypted_object": encrypted.hex()}
Path(__file__).with_name("public-vector.json").write_text(
    json.dumps(fixture, indent=2) + "\n", encoding="utf-8")
