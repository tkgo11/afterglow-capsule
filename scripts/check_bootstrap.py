"""Check bootstrap topology and version pinning, not production security."""

import json
import pathlib
import subprocess
import tomllib
import unittest

ROOT = pathlib.Path(__file__).resolve().parent.parent
CRATES = {
    "ag-schema", "ag-project", "ag-capsule", "ag-object-store", "ag-crypto",
    "ag-timelock", "ag-time", "ag-theme", "ag-motion", "ag-render",
    "ag-validation", "ag-pe-packager", "ag-windows",
}


def read_toml(path):
    with path.open("rb") as source:
        return tomllib.load(source)


class BootstrapTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.metadata = json.loads(subprocess.check_output(
            ["cargo", "metadata", "--locked", "--format-version", "1"], cwd=ROOT,
        ))
        cls.packages = {p["name"]: p for p in cls.metadata["packages"]}

    def test_all_specified_crates_and_separate_apps_are_workspace_members(self):
        members = set(self.metadata["workspace_members"])
        actual = {p["name"] for p in self.metadata["packages"] if p["id"] in members}
        self.assertEqual(actual, CRATES | {"afterglow-builder", "afterglow-viewer"})

    def test_viewer_dependency_closure_has_no_creator_application_or_packager(self):
        nodes = {n["id"]: n for n in self.metadata["resolve"]["nodes"]}
        pending = [self.packages["afterglow-viewer"]["id"]]
        visited = set()
        while pending:
            package_id = pending.pop()
            if package_id not in visited:
                visited.add(package_id)
                pending.extend(nodes[package_id]["dependencies"])
        creator_only = {self.packages[n]["id"] for n in
                        ("afterglow-builder", "ag-pe-packager")}
        self.assertTrue(visited.isdisjoint(creator_only))

    def test_runtime_template_pin_matches_viewer_and_remains_a_scaffold(self):
        template = read_toml(ROOT / "viewer-runtime/template/runtime-template.toml")
        self.assertEqual(template["format_name"], "afterglow-runtime-template")
        self.assertEqual(template["format_version"], 1)
        self.assertEqual(template["minimum_reader_version"], 1)
        viewer = self.packages[template["runtime_package"]]
        self.assertEqual(template["runtime_version"], viewer["version"])
        self.assertEqual(template["target"], "x86_64-pc-windows-msvc")
        self.assertEqual(template["status"], "scaffold")

    def test_frontend_workspace_has_a_lock_and_only_the_builder_package(self):
        manifest = json.loads((ROOT / "package.json").read_text(encoding="utf-8"))
        lock = json.loads((ROOT / "package-lock.json").read_text(encoding="utf-8"))
        self.assertTrue(manifest["private"])
        self.assertEqual(manifest["workspaces"], ["packages/builder-ui"])
        self.assertEqual(lock["packages"]["packages/builder-ui"]["name"],
                         "@afterglow/builder-ui")


if __name__ == "__main__":
    unittest.main()
