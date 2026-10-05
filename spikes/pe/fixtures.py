"""Generate public PE resource fixtures; never signing keys or private content."""

import argparse
from pathlib import Path
import struct
import zlib


def fixtures(folder, variant="project"):
    folder.mkdir(parents=True, exist_ok=True)
    if variant not in ["baseline", "project"]:
        raise ValueError("unknown public fixture variant")
    baseline = variant == "baseline"
    capsule = ("AFTERGLOW public Spike C " + variant + " resource\0").encode("ascii")
    (folder / "capsule.bin").write_bytes(capsule)
    # Visible PNG-backed Windows icon. Different dimensions/color prove real
    # replacement of existing stock resources, not only insertion into an EXE.
    dimension = 16 if baseline else 32
    background = bytes([180, 90, 40, 255] if baseline else [20, 170, 220, 255])
    def png_chunk(kind, data):
        body = kind + data
        return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body) & 0xffffffff)
    icon = b"\x89PNG\r\n\x1a\n"
    icon += png_chunk(b"IHDR", struct.pack(">IIBBBBB", dimension, dimension, 8, 6, 0, 0, 0))
    pixels = bytearray()
    for y in range(dimension):
        pixels.append(0)  # PNG filter None.
        for x in range(dimension):
            pixels.extend(bytes([255, 255, 255, 255]) if x == y else background)
    icon += png_chunk(b"IDAT", zlib.compress(pixels))
    icon += png_chunk(b"IEND", b"")
    (folder / "icon.bin").write_bytes(icon)
    group = struct.pack("<HHH", 0, 1, 1) + struct.pack("<BBBBHHIH", dimension, dimension, 0, 0, 1, 32, len(icon), 101)
    (folder / "group-icon.bin").write_bytes(group)
    # Stock 1.0.0.0 → project 2.0.0.0, using the same native resource IDs.
    key = "VS_VERSION_INFO\0".encode("utf-16-le")
    prefix = struct.pack("<HHH", 0, 52, 0) + key
    prefix += bytes((-len(prefix)) % 4)
    version_ms = (1 if baseline else 2) << 16
    fixed = struct.pack("<13I", 0xFEEF04BD, 0x10000, version_ms, 0, version_ms, 0,
                        0x3F, 0, 0x40004, 1, 0, 0, 0)
    version = prefix + fixed
    version = struct.pack("<H", len(version)) + version[2:]
    (folder / "version.bin").write_bytes(version)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("folder", type=Path)
    parser.add_argument("--variant", choices=["baseline", "project"], default="project")
    args = parser.parse_args()
    fixtures(args.folder, args.variant)
