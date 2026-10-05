"""Generate public PE resource fixtures; never signing keys or private content."""

import argparse
from pathlib import Path
import struct
import zlib


def fixtures(folder):
    folder.mkdir(parents=True, exist_ok=True)
    capsule = b"AFTERGLOW public Spike C test resource v1\0"
    (folder / "capsule.bin").write_bytes(capsule)
    # PNG-backed Windows icon, one transparent pixel; public test artwork only.
    def png_chunk(kind, data):
        body = kind + data
        return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body) & 0xffffffff)
    icon = b"\x89PNG\r\n\x1a\n"
    icon += png_chunk(b"IHDR", struct.pack(">IIBBBBB", 1, 1, 8, 6, 0, 0, 0))
    icon += png_chunk(b"IDAT", zlib.compress(bytes(5)))
    icon += png_chunk(b"IEND", b"")
    (folder / "icon.bin").write_bytes(icon)
    group = struct.pack("<HHH", 0, 1, 1) + struct.pack("<BBBBHHIH", 1, 1, 0, 0, 1, 32, len(icon), 101)
    (folder / "group-icon.bin").write_bytes(group)
    # VS_VERSION_INFO containing a VS_FIXEDFILEINFO for file/product 1.0.0.0.
    key = "VS_VERSION_INFO\0".encode("utf-16-le")
    prefix = struct.pack("<HHH", 0, 52, 0) + key
    prefix += bytes((-len(prefix)) % 4)
    fixed = struct.pack("<13I", 0xFEEF04BD, 0x10000, 0x10000, 0, 0x10000, 0,
                        0x3F, 0, 0x40004, 1, 0, 0, 0)
    version = prefix + fixed
    version = struct.pack("<H", len(version)) + version[2:]
    (folder / "version.bin").write_bytes(version)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("folder", type=Path)
    fixtures(parser.parse_args().folder)
