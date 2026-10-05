"""Pure-input tests for the isolated live spike; no fake network/beacon evidence."""

import copy
import unittest
from live_future import fixture, is_future_round_unavailable, validate_info, validate_relay


class LiveInputs(unittest.TestCase):
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
