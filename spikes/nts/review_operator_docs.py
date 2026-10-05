"""SPIKE ONLY: collect public operator provenance; never select/change endpoints.

Live TLS/NTP measurements remain separate requirements. If documentation no longer
lists a candidate, fail and leave the decision for review instead of adopting a
different endpoint automatically.
"""

import hashlib
import html
import json
from pathlib import Path
import re
import sys
import urllib.request

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from run_live import endpoints


def main():
    failed = False
    for entry in endpoints()["nts_operators"]:
        try:
            request = urllib.request.Request(entry["documentation_url"], headers={
                "User-Agent": "AFTERGLOW Phase 2 public documentation review",
            })
            with urllib.request.urlopen(request, timeout=15) as response:
                body = response.read(1048577)
                final_url = response.url
            if len(body) > 1048576:
                raise ValueError("documentation exceeds the review limit")
            text = html.unescape(body.decode("utf-8"))
            match = re.search(r"(?<![A-Za-z0-9.-])" + re.escape(entry["host"]) + r"(?![A-Za-z0-9.-])", text)
            if not match:
                # Public nearby NTS hostnames help review stale candidates without
                # silently changing the configured endpoint or accepting evidence.
                candidates = sorted(set(re.findall(r"\b[a-z0-9.-]*(?:nts|ntp)[a-z0-9.-]*\.(?:se|com|net)\b", text)))
                raise ValueError(f"configured host absent from official documentation; public mentions={candidates}")
            snippet = re.sub(r"<[^>]+>", " ", text[max(0, match.start()-120):match.end()+120])
            print(json.dumps({
                "operator": entry["operator"], "host": entry["host"],
                "source": final_url, "document_sha256": hashlib.sha256(body).hexdigest(),
                "public_excerpt": " ".join(snippet.split()),
            }))
        except Exception as error:
            failed = True
            print(f"{entry['operator']}: documentation review failed: {error}", file=sys.stderr)
    if failed:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
