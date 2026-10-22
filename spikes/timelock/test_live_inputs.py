"""Pure-input tests for the isolated live spike; no fake network/beacon evidence."""

import copy
import unittest
from unittest.mock import patch
import urllib.error
from live_future import fixture, get_json, is_future_round_unavailable, validate_info, validate_relay


class LiveInputs(unittest.TestCase):
    def test_three_server_failures_abort_instead_of_becoming_release_evidence(self):
        error = urllib.error.HTTPError("https://relay.example", 500, "test server failure", {}, None)
        with patch("live_future.urllib.request.urlopen", side_effect=error) as request, patch("live_future.time.sleep"):
            with self.assertRaises(urllib.error.HTTPError):
                get_json("https://relay.example")
        self.assertEqual(request.call_count, 3)

    def test_transport_retries_do_not_mask_access_denial_or_future_responses(self):
        for status in [401, 403, 404, 425]:
            error = urllib.error.HTTPError("https://relay.example", status, "public test status", {}, None)
            with patch("live_future.urllib.request.urlopen", side_effect=error) as request, patch("live_future.time.sleep") as sleep:
                with self.assertRaises(urllib.error.HTTPError):
                    get_json("https://relay.example")
            self.assertEqual(request.call_count, 1)
            sleep.assert_not_called()

    def test_future_statuses_do_not_hide_authentication_or_server_failures(self):
        for status in [404, 425]:
            self.assertTrue(is_future_round_unavailable(status))
        for status in [200, 401, 403, 429, 500, 503]:
            self.assertFalse(is_future_round_unavailable(status))

    def test_transport_configuration_rejects_credentials_and_non_https(self):
        for value in ["http://relay.example", "https://user:secret@relay.example",
                      "https://relay.example/path", "https://relay.example?q=1",
                      "https://relay.example#fragment", "https://"]:
            with self.assertRaises(ValueError):
                validate_relay(value)
        self.assertEqual(validate_relay("https://relay.example/"), "https://relay.example")

    def test_every_chain_pin_must_match_before_live_testing(self):
        pinned = fixture()["chain"]
        validate_info(pinned, pinned)
        for name in ["public_key", "period", "genesis_time", "hash", "schemeID"]:
            altered = copy.deepcopy(pinned)
            altered[name] = "different"
            with self.assertRaises(ValueError):
                validate_info(altered, pinned)


if __name__ == "__main__":
    unittest.main()
