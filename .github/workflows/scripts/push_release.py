#!/usr/bin/env python3
"""Push Android release outcome to Feishu webhook."""
import json
import os
import sys
import urllib.request


def main() -> int:
    webhook = os.environ.get("FEISHU_WEBHOOK")
    if not webhook:
        return 0
    tag = os.environ.get("TAG", "")
    run_url = os.environ.get("RUN_URL", "")
    release_url = os.environ.get("RELEASE_URL", "")
    status = (os.environ.get("STATUS", "") or "").lower()
    success = status == "success"
    title = (
        f"🔔 【通知】Android Release {tag} 已发布"
        if success
        else f"⚠️ 【告警】Android Release {tag} 失败"
    )
    template = "green" if success else "orange"
    content = (
        f"**Tag:** `{tag}`\n"
        f"**状态:** {'✅ 成功' if success else '❌ 失败'}\n"
        f"**Run:** {run_url}\n"
        f"**Release:** {release_url}\n\n"
        + ("APK 已附在 Release assets 下载使用。" if success else "请查看 Run 日志定位错误。")
    )
    payload = {
        "msg_type": "interactive",
        "card": {
            "header": {
                "title": {"tag": "plain_text", "content": title},
                "template": template,
            },
            "elements": [
                {"tag": "markdown", "content": content},
                {"tag": "hr"},
                {
                    "tag": "note",
                    "elements": [
                        {
                            "tag": "plain_text",
                            "content": "来源：GitHub Actions android.yml | Release pipeline",
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


if __name__ == "__main__":
    sys.exit(main())
