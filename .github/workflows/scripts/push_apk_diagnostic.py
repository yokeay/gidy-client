#!/usr/bin/env python3
"""Push APK output diagnostic to Feishu webhook so we can see real path."""
import json
import os
import sys
import urllib.request


def main() -> int:
    webhook = os.environ.get("FEISHU_WEBHOOK")
    if not webhook:
        print("FEISHU_WEBHOOK env missing, skip push")
        return 0
    diag = (os.environ.get("DIAG", "") or "")[-15000:] or "(empty)"
    run_url = os.environ.get("RUN_URL", "")
    sha = (os.environ.get("SHA", "") or "")[:7]
    content = (
        "**🔎 Android APK 路径诊断**\n\n"
        f"**Run:** {run_url}\n"
        f"**Commit:** `{sha}`\n\n"
        "**Diagnostic output:**\n```\n" + diag + "\n```"
    )
    payload = {
        "msg_type": "interactive",
        "card": {
            "header": {
                "title": {
                    "tag": "plain_text",
                    "content": "📌 【诊断】Android APK 输出路径",
                },
                "template": "blue",
            },
            "elements": [
                {"tag": "markdown", "content": content[:28000]},
                {"tag": "hr"},
                {
                    "tag": "note",
                    "elements": [
                        {
                            "tag": "plain_text",
                            "content": "来源：GitHub Actions android.yml | 路径诊断",
                        }
                    ],
                },
            ],
        },
    }
    req = urllib.request.Request(
        webhook,
        data=json.dumps(payload).encode("utf-8"),
        headers={"Content-Type": "application/json"},
    )
    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            print(resp.read().decode())
    except Exception as exc:
        print(f"feishu push failed: {exc}", file=sys.stderr)
        return 0
    return 0


if __name__ == "__main__":
    sys.exit(main())
