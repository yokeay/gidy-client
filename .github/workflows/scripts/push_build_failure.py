#!/usr/bin/env python3
"""Push Android CI failure tail to Feishu webhook."""
import json
import os
import sys
import urllib.request


def main() -> int:
    webhook = os.environ.get("FEISHU_WEBHOOK")
    if not webhook:
        print("FEISHU_WEBHOOK env missing, skip push")
        return 0
    err = (os.environ.get("ERR", "") or "")[:6000] or "(none matched)"
    tail = (os.environ.get("TAIL", "") or "")[-12000:]
    run_url = os.environ.get("RUN_URL", "")
    sha = (os.environ.get("SHA", "") or "")[:7]
    content = (
        "**❌ Android assembleDebug 失败**\n\n"
        f"**Run:** {run_url}\n"
        f"**Commit:** `{sha}`\n\n"
        "**Critical errors:**\n```\n" + err + "\n```\n\n"
        "**Build log tail (150 lines):**\n```\n" + tail + "\n```"
    )
    payload = {
        "msg_type": "interactive",
        "card": {
            "header": {
                "title": {
                    "tag": "plain_text",
                    "content": "⚠️ 【告警】Android CI 失败 · assembleDebug",
                },
                "template": "orange",
            },
            "elements": [
                {"tag": "markdown", "content": content[:28000]},
                {"tag": "hr"},
                {
                    "tag": "note",
                    "elements": [
                        {
                            "tag": "plain_text",
                            "content": "来源：GitHub Actions android.yml | 自动错误日志",
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
        return 0  # don't fail the diagnostic step itself
    return 0


if __name__ == "__main__":
    sys.exit(main())
