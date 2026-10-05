"""Independent format checks for public Windows test fixtures."""

import struct
import tempfile
import unittest
import zlib
from pathlib import Path

from fixtures import fixtures


class ResourceFixtureTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.folder = Path(self.temp.name)
        fixtures(self.folder)

    def test_png_chunks_crc_and_group_icon_reference(self):
        png = (self.folder / "icon.bin").read_bytes()
        self.assertEqual(png[:8], b"\x89PNG\r\n\x1a\n")
        cursor = 8
        kinds = []
        while cursor < len(png):
            length = struct.unpack_from(">I", png, cursor)[0]
            body = png[cursor+4:cursor+8+length]
            crc = struct.unpack_from(">I", png, cursor+8+length)[0]
            self.assertEqual(zlib.crc32(body) & 0xffffffff, crc)
            kinds.append(body[:4])
            cursor += length + 12
        self.assertEqual(kinds, [b"IHDR", b"IDAT", b"IEND"])
        self.assertEqual(cursor, len(png))
        group = (self.folder / "group-icon.bin").read_bytes()
        self.assertEqual(struct.unpack_from("<HHH", group), (0, 1, 1))
        record = struct.unpack_from("<BBBBHHIH", group, 6)
        self.assertEqual(record, (32, 32, 0, 0, 1, 32, len(png), 101))

    def test_fixed_version_header_alignment_and_signature(self):
        version = (self.folder / "version.bin").read_bytes()
        self.assertEqual(struct.unpack_from("<HHH", version), (len(version), 52, 0))
        key_end = 6 + len("VS_VERSION_INFO\0".encode("utf-16-le"))
        self.assertEqual(version[6:key_end].decode("utf-16-le"), "VS_VERSION_INFO\0")
        fixed_offset = (key_end + 3) // 4 * 4
        fixed = struct.unpack_from("<13I", version, fixed_offset)
        self.assertEqual(fixed[:4], (0xFEEF04BD, 0x10000, 0x20000, 0))
        self.assertEqual(fixed_offset + 52, len(version))

    def test_stock_resources_are_distinct_and_project_icon_is_visible(self):
        baseline = self.folder / "stock"
        fixtures(baseline, "baseline")
        for name in ["capsule.bin", "icon.bin", "group-icon.bin", "version.bin"]:
            self.assertNotEqual((baseline / name).read_bytes(), (self.folder / name).read_bytes())
        png = (self.folder / "icon.bin").read_bytes()
        self.assertEqual(struct.unpack_from(">II", png, 16), (32, 32))
        cursor = 8
        while png[cursor+4:cursor+8] != b"IDAT":
            cursor += struct.unpack_from(">I", png, cursor)[0] + 12
        length = struct.unpack_from(">I", png, cursor)[0]
        pixels = zlib.decompress(png[cursor+8:cursor+8+length])
        self.assertEqual(len(pixels), 32 * (1 + 32 * 4))
        for row in range(32):
            self.assertEqual(pixels[row * 129], 0)
            self.assertTrue(all(value == 255 for value in pixels[row*129+4:(row+1)*129:4]))


if __name__ == "__main__":
    unittest.main()
