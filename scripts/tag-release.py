#!/usr/bin/env python3
"""tag-release.py — 消化收尾发版硬门的机械助手

职责边界:在 agent-on 工作仓、工作区干净且 HEAD 已超前最新 tag 时:
  - 计算下一 semver(patch|minor|major)
  - 创建 annotated tag(默认不自动 push,打印命令;--push 才推)
  - 不改 CHANGELOG 正文(封版文案由消化会话先写好再跑本脚本)

用法:
  python3 scripts/tag-release.py --level minor --title "一句话说明"
  python3 scripts/tag-release.py --level patch --title "措辞" --push

退出码:0 成功;1 用法/状态错误;2 已与最新 tag 齐平(无需发版)
"""
from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path


def run(cmd: list[str], check: bool = True) -> str:
    r = subprocess.run(cmd, capture_output=True, text=True, cwd=ROOT)
    if check and r.returncode != 0:
        sys.stderr.write(r.stderr or r.stdout or f"fail: {cmd}\n")
        sys.exit(1)
    return (r.stdout or "").strip()


ROOT = Path(__file__).resolve().parents[1]
SEMVER = re.compile(r"^v?(\d+)\.(\d+)\.(\d+)$")


def latest_tag() -> str:
    out = run(["git", "tag", "--sort=-v:refname"], check=False)
    for line in out.splitlines():
        t = line.strip()
        if SEMVER.match(t.lstrip("v") if False else t) or SEMVER.match(
            t[1:] if t.startswith("v") else t
        ):
            if re.match(r"^v\d+\.\d+\.\d+$", t):
                return t
    sys.stderr.write("找不到 vX.Y.Z 形态 tag\n")
    sys.exit(1)


def parse_ver(tag: str) -> tuple[int, int, int]:
    m = re.match(r"^v(\d+)\.(\d+)\.(\d+)$", tag)
    if not m:
        sys.stderr.write(f"无法解析 tag: {tag}\n")
        sys.exit(1)
    return int(m.group(1)), int(m.group(2)), int(m.group(3))


def bump(level: str, major: int, minor: int, patch: int) -> str:
    if level == "major":
        return f"v{major + 1}.0.0"
    if level == "minor":
        return f"v{major}.{minor + 1}.0"
    if level == "patch":
        return f"v{major}.{minor}.{patch + 1}"
    sys.stderr.write("--level 须为 major|minor|patch\n")
    sys.exit(1)


def main() -> int:
    ap = argparse.ArgumentParser(description="agent-on 消化收尾: annotated tag")
    ap.add_argument("--level", required=True, choices=("patch", "minor", "major"))
    ap.add_argument("--title", required=True, help="tag 说明一句话")
    ap.add_argument(
        "--push",
        action="store_true",
        help="创建后执行 git push origin HEAD 与 git push origin <tag>",
    )
    ap.add_argument(
        "--allow-dirty",
        action="store_true",
        help="允许脏工作区(默认拒绝——先 commit 再发版)",
    )
    args = ap.parse_args()

    status = run(["git", "status", "--porcelain"])
    if status and not args.allow_dirty:
        sys.stderr.write("工作区不干净,先 commit 再发版(或 --allow-dirty):\n")
        sys.stderr.write(status + "\n")
        return 1

    tag = latest_tag()
    ahead = run(["git", "rev-list", "--count", f"{tag}..HEAD"])
    if ahead == "0":
        sys.stderr.write(f"HEAD 已与 {tag} 齐平,无需发版\n")
        return 2

    maj, mino, pat = parse_ver(tag)
    new_tag = bump(args.level, maj, mino, pat)
    head = run(["git", "rev-parse", "--short", "HEAD"])

    if args.level == "major":
        sys.stderr.write(
            "WARNING: major 必须在 CHANGELOG 写清迁移注记;无注记禁止打 tag(人工自检)\n"
        )

    msg = f"{new_tag} — {args.level}\n\n{args.title}\n\n基于 {tag} + {ahead} commit (HEAD {head})"
    run(["git", "tag", "-a", new_tag, "-m", msg])
    print(f"created annotated tag {new_tag} (was {tag}, +{ahead} commits)")
    print(f"  HEAD: {run(['git', 'rev-parse', 'HEAD'])}")

    if args.push:
        run(["git", "push", "origin", "HEAD"])
        run(["git", "push", "origin", new_tag])
        print(f"pushed origin HEAD and {new_tag}")
    else:
        print("下一步(须执行,否则下游仍升不了):")
        print(f"  git push origin HEAD && git push origin {new_tag}")
        print("并确认 README/AGENTS 推荐 pin 已改为", new_tag)

    return 0


if __name__ == "__main__":
    sys.exit(main())
