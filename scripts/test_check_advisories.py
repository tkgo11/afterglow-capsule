"""Regressions against false success in advisory tooling; no physical evidence."""

import copy
import unittest

from scripts.check_advisories import inspect_report


def empty_report():
    return {"vulnerabilities": {"found": False, "count": 0, "list": []}, "warnings": {},
            "database": {"advisory-count": 1}, "lockfile": {"dependency-count": 1},
            "settings": {"ignore": [], "target_arch": [], "target_os": [], "severity": None,
                         "informational_warnings": ["unmaintained", "unsound", "notice"]}}


class AdvisoryFailures(unittest.TestCase):
    def test_nonzero_tool_exit_cannot_pass_with_empty_vulnerabilities(self):
        self.assertTrue(inspect_report(empty_report(), 1)[0])

    def test_missing_yank_metadata_error_blocks_even_with_zero_exit(self):
        actual_error = "error: couldn't check if the package is yanked: not found: No such crate in crates.io index: chrono"
        self.assertTrue(inspect_report(empty_report(), 0, actual_error)[0])

    def test_failed_or_missing_report_cannot_pass(self):
        for value in (None, {}, {"error": "network failed"}, []):
            self.assertTrue(inspect_report(value, 0)[0])

    def test_empty_coverage_or_disabled_unsoundness_cannot_pass(self):
        for section, field in (("database", "advisory-count"), ("lockfile", "dependency-count")):
            value = empty_report()
            value[section][field] = 0
            self.assertTrue(inspect_report(value, 0)[0])
        for settings in (None, {}, {**empty_report()["settings"], "informational_warnings": []},
                         {**empty_report()["settings"], "informational_warnings": ["unsound"]}):
            value = empty_report()
            value["settings"] = settings
            self.assertTrue(inspect_report(value, 0)[0])

    def test_found_and_count_cannot_disagree(self):
        for change in ({"found": True}, {"count": 1}, {"list": [{}]}, {"count": True}):
            value = empty_report()
            value["vulnerabilities"].update(change)
            self.assertTrue(inspect_report(value, 0)[0])

    def test_actual_vulnerability_blocks_even_with_zero_exit(self):
        value = empty_report()
        value["vulnerabilities"] = {"found": True, "count": 1, "list": [{}]}
        self.assertTrue(inspect_report(value, 0)[0])

    def test_unsound_warning_blocks_even_with_zero_exit(self):
        value = empty_report()
        value["warnings"]["unsound"] = [{"advisory": {"id": "TEST"}, "package": {"name": "fixture", "version": "1"}}]
        self.assertTrue(inspect_report(value, 0)[0])

    def test_informational_findings_remain_visible(self):
        value = empty_report()
        value["warnings"]["unmaintained"] = [{"advisory": {"id": "TEST"}, "package": {"name": "fixture", "version": "1"}}]
        reasons, findings = inspect_report(value, 0)
        self.assertEqual(reasons, [])
        self.assertEqual(findings, [{"kind": "unmaintained", "id": "TEST", "package": "fixture", "version": "1"}])

    def test_ignored_and_filtered_audits_cannot_pass(self):
        for field, changed in (("ignore", ["TEST"]), ("target_os", ["windows"]), ("target_arch", ["x86_64"]), ("severity", "critical")):
            value = copy.deepcopy(empty_report())
            value["settings"][field] = changed
            self.assertTrue(inspect_report(value, 0)[0])


if __name__ == "__main__":
    unittest.main()
