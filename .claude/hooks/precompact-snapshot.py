#!/usr/bin/env python3
"""PreCompact hook：在上下文压缩前，把在途进度快照到持久文件，避免压缩丢失未落盘的工作。

由 .claude/settings.json 的 PreCompact 钩子触发（auto + manual）。
"""
import os
import subprocess
import sys
from datetime import datetime

project = os.environ.get("CLAUDE_PROJECT_DIR") or os.getcwd()


def sh(args):
    try:
        return subprocess.run(
            args, cwd=project, capture_output=True, text=True, timeout=10
        ).stdout.strip()
    except Exception:
        return ""


branch = sh(["git", "branch", "--show-current"])
last_commit = sh(["git", "log", "--oneline", "-1"])
status = sh(["git", "status", "--short"])

task = ""
task_script = os.path.join(project, ".trellis", "scripts", "task.py")
if os.path.exists(task_script):
    task_out = sh(["python3", task_script, "current"])
    task = task_out.splitlines()[0] if task_out else ""

ts = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
snapshot = (
    f"\n### PreCompact 快照 {ts}\n"
    f"- branch: {branch or 'n/a'}\n"
    f"- last commit: {last_commit or 'n/a'}\n"
    f"- active task: {task or 'n/a'}\n"
    f"- 未提交改动:\n```\n{status or '(clean)'}\n```\n"
)

out_path = os.path.join(project, ".claude", "auto-progress.md")
try:
    with open(out_path, "a", encoding="utf-8") as f:
        f.write(snapshot)
except Exception as exc:  # noqa: BLE001
    print(f"precompact-snapshot: 写入快照失败 {exc}", file=sys.stderr)

# 提示压缩后的自己：把未落盘的进度补进 PROGRESS.md / 当前任务 progress.md
print(
    "压缩前已快照 git 状态到 .claude/auto-progress.md；"
    "若有未落盘进度，请更新 PROGRESS.md 与当前 Trellis 任务的 progress.md。"
)
