#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""本仓文档不变量闸：把 AGENTS.md 里靠自觉执行的三条纪律变成 CI 可红。

职责边界：只钉**能机械判定**的三件——文档职责边界声明（棘轮）、相对链接可解析、
推荐 pin 三处一致。不判断内容对错，不做单一权威判定（那要人读），也不碰 lane
控制面（那是 `agent-on worktree check` 的事）。

设计遵 kit/ledger-ratchet-pattern.md 三条：
- **逃生门**：`AGENT_ON_DOC_GATE_OFF=1` 一键关闸；棘轮基线补一行即绿。
- **假红纪律**：取证失败（读不到文件 / git 不可用）退 2 并明说「取证失败」，
  与「真违规」退 1 分开——假红训练「红了当没看见」，比没有闸更糟。
- **元动作自涵盖**：本文件与它的基线文件都受同一套闸约束。

用法：`python3 .github/scripts/check_docs.py [--fix-baseline]`

放在 `.github/` 而不是 `scripts/`：`scripts/README.md` 写死「本目录不再放 Python，
可执行物在 cli/」。本文件是**本仓自己的 CI 件**，不是产品交付面，塞进 `agent-on`
子命令会平白扩大产品表面（AGENTS.md「不做的事」）。
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BASELINE = Path(__file__).resolve().parent / "doc-boundary-baseline.txt"

# 要求声明职责边界的目录：canonical 正文层。
# bench/cases/ 不在内——案例卡是固定五节格式，职责边界由 bench/cases/README.md 统一声明。
BOUNDARY_DIRS = ("playbook", "kit", "boot", "docs", "scripts")
BOUNDARY_MARKER = "职责边界"
BOUNDARY_HEAD_LINES = 12

LINK_RE = re.compile(r"(?<!\!)\[[^\]]*\]\(([^)\s]+)\)")
# 冻结层：snapshot/ 是决策快照、intake/ 是承接素材、legacy/ 是考古件。它们记的是
# 「当时是这样」，改它们等于改历史；死链闸只管还在被人当说明书读的 canonical 层。
FROZEN_DIRS = ("snapshot/", "intake/", "legacy/")
# 只有「长得像路径」的才算链接。模板里 `[先读它](验收标准逐条打勾)` 这种把括号当
# 注解用的写法不是链接，拿它报红就是假红。
PATH_LIKE = re.compile(r"/|\.(md|html|py|rs|toml|json|yml|yaml|sh|txt)$", re.I)
PIN_RE = re.compile(r"推荐 pin[^`\n]*`(v\d+\.\d+\.\d+)`")


class Evidence(Exception):
    """取证失败：闸自己没法判，必须与「真违规」区分开。"""


def tracked_markdown() -> list[Path]:
    try:
        out = subprocess.run(
            ["git", "ls-files", "*.md"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=True,
        ).stdout
    except (OSError, subprocess.CalledProcessError) as exc:
        raise Evidence(f"git ls-files 跑不动：{exc}") from exc
    return [ROOT / line for line in out.splitlines() if line]


def read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except OSError as exc:
        raise Evidence(f"读不到 {path.relative_to(ROOT)}：{exc}") from exc


def load_baseline() -> set[str]:
    if not BASELINE.exists():
        return set()
    return {
        line.strip()
        for line in read(BASELINE).splitlines()
        if line.strip() and not line.startswith("#")
    }


def boundary_violators(files: list[Path]) -> set[str]:
    missing = set()
    for path in files:
        rel = path.relative_to(ROOT).as_posix()
        if not rel.startswith(BOUNDARY_DIRS):
            continue
        head = "\n".join(read(path).splitlines()[:BOUNDARY_HEAD_LINES])
        if BOUNDARY_MARKER not in head:
            missing.add(rel)
    return missing


def check_boundary(files: list[Path]) -> list[str]:
    """棘轮：存量欠账进基线，新增违规立即红；补齐后必须从基线里划掉，不许倒退。"""
    baseline = load_baseline()
    missing = boundary_violators(files)
    problems = []
    for rel in sorted(missing - baseline):
        problems.append(
            f"职责边界缺失：{rel} 前 {BOUNDARY_HEAD_LINES} 行没有「{BOUNDARY_MARKER}」声明"
            f"（文档纪律第 5 条）。出口：在文件开头加一行 `> 职责边界：本文件管…，不管…`"
        )
    for rel in sorted(baseline - missing):
        problems.append(
            f"棘轮倒退：{rel} 已经补上职责边界，但还挂在 {BASELINE.relative_to(ROOT)} 里。"
            f"出口：把这一行从基线删掉（`python3 .github/scripts/check_docs.py --fix-baseline` 可代劳）"
        )
    for rel in sorted(baseline):
        if not (ROOT / rel).exists():
            problems.append(
                f"基线指向不存在的文件：{rel}。出口：从 {BASELINE.relative_to(ROOT)} 删掉这一行"
            )
    return problems


def check_links(files: list[Path]) -> list[str]:
    problems = []
    for path in files:
        rel = path.relative_to(ROOT).as_posix()
        if rel.startswith(FROZEN_DIRS):
            continue
        for lineno, line in enumerate(read(path).splitlines(), 1):
            for target in LINK_RE.findall(line):
                if re.match(r"^[a-z][a-z0-9+.-]*:", target) or target.startswith("#"):
                    continue
                # `~/…` 是使用者机器上的家目录，不是仓内路径。
                if target.startswith("~"):
                    continue
                if not PATH_LIKE.search(target):
                    continue
                bare = target.split("#", 1)[0]
                if not bare:
                    continue
                resolved = (path.parent / bare).resolve()
                if not resolved.exists():
                    problems.append(
                        f"死链：{rel}:{lineno} 指向 {target}，解析成 "
                        f"{os.path.relpath(resolved, ROOT)} 不存在。"
                        f"出口：改成真实路径，或把目标文件补上"
                    )
    return problems


def check_pin() -> list[str]:
    """推荐 pin 三处一致 + 该 tag 真存在：本仓自举纪律第 6 条的机械化。"""
    problems = []
    pins = {}
    for name in ("AGENTS.md", "README.md"):
        found = PIN_RE.findall(read(ROOT / name))
        if not found:
            problems.append(
                f"{name} 里找不到「推荐 pin：`vX.Y.Z`」。"
                f"出口：补回该行，或改本闸的判据（两者都要有人拍板）"
            )
            continue
        if len(set(found)) > 1:
            problems.append(
                f"{name} 自己就有 {len(set(found))} 个不同的推荐 pin：{sorted(set(found))}。"
                f"出口：统一成同一个版本号"
            )
        pins[name] = found[0]
    if len(set(pins.values())) > 1:
        detail = "，".join(f"{k}={v}" for k, v in sorted(pins.items()))
        problems.append(f"推荐 pin 跨文件漂移：{detail}。出口：把落后的一处改成当前版本")
    if pins:
        pin = sorted(pins.values())[0]
        changelog = read(ROOT / "CHANGELOG.md")
        if not re.search(rf"^## {re.escape(pin)}[（(]", changelog, re.M):
            problems.append(
                f"CHANGELOG.md 里没有 `## {pin}（…）` 这一节，"
                f"但 AGENTS/README 已经把它当推荐 pin。"
                f"出口：先封 CHANGELOG 的 `[未发布]` 段，再改推荐 pin"
            )
        try:
            tags = subprocess.run(
                ["git", "tag", "--list", pin],
                cwd=ROOT,
                capture_output=True,
                text=True,
                check=True,
            ).stdout.split()
        except (OSError, subprocess.CalledProcessError) as exc:
            raise Evidence(f"git tag 跑不动：{exc}") from exc
        if pin not in tags:
            problems.append(
                f"推荐 pin `{pin}` 没有对应的 git tag（版本真相 = git tag）。"
                f"出口：`agent-on tag-release --level … --title \"…\" --push`；"
                f"CI 上若是 fetch 深度不够导致看不到 tag，用 `fetch-depth: 0`"
            )
    return problems


def fix_baseline(files: list[Path]) -> int:
    missing = sorted(boundary_violators(files))
    header = (
        "# 职责边界欠账基线（棘轮，只许变短）\n"
        "#\n"
        "# 每一行 = 一份还没写职责边界声明的 canonical 文档。闸只拦「新增违规」与「补齐后没划掉」，\n"
        "# 不逼任何人一次补完存量。补一份就从这里删一行；`python3 .github/scripts/check_docs.py --fix-baseline` 代劳。\n"
        "# 反向加行 = 放松闸，需要在 PR 里写明理由。\n"
    )
    BASELINE.write_text(header + "\n".join(missing) + "\n", encoding="utf-8")
    print(f"基线已重写：{len(missing)} 份欠账 → {BASELINE.relative_to(ROOT)}")
    return 0


def main() -> int:
    if os.environ.get("AGENT_ON_DOC_GATE_OFF", "").lower() in ("1", "true", "yes"):
        print("DOC-GATE: OFF（AGENT_ON_DOC_GATE_OFF 已设）——闸被显式关掉，不代表文档合格")
        return 0
    try:
        files = tracked_markdown()
        if "--fix-baseline" in sys.argv:
            return fix_baseline(files)
        problems = check_boundary(files) + check_links(files) + check_pin()
    except Evidence as exc:
        print(f"DOC-GATE: 取证失败 —— {exc}")
        print("这不是「文档违规」，是闸自己没法判。修好取证再重跑，别当红灯忽略。")
        return 2
    if problems:
        print(f"DOC-GATE: FAIL（{len(problems)} 条）")
        for item in problems:
            print(f"  - {item}")
        print("\n每条都带出口。全仓一键关闸：AGENT_ON_DOC_GATE_OFF=1")
        return 1
    print(f"DOC-GATE: PASS（{len(files)} 份 markdown：职责边界棘轮 / 相对链接 / 推荐 pin 三处一致）")
    return 0


if __name__ == "__main__":
    sys.exit(main())
