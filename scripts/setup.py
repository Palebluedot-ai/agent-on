#!/usr/bin/env python3
"""agent-on 跨平台一键装机：固定默认目录放好工作仓 B + 写 config + 可选 plugin/symlink。

用法:
  python3 scripts/setup.py
  python3 scripts/setup.py --pin v0.5.0
  python3 scripts/setup.py --work-root D:\\dev\\agent-on
  python3 scripts/setup.py --with-plugins
  python3 scripts/setup.py --with-symlinks
  py -3 scripts\\setup.py          # Windows

默认目录（产品钉死）:
  macOS/Linux: ~/.local/share/agent-on
  Windows:     %LOCALAPPDATA%\\agent-on

依赖: python3 标准库 + 本机 git。不改 shell RC。
"""
from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path

# 允许从仓内任意 cwd 调用
_REPO_HINT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(_REPO_HINT / "kit" / "guard"))

from agent_on_paths import (  # noqa: E402
    DEFAULT_PIN,
    OFFICIAL_HTTPS,
    config_path,
    default_work_root,
    doctor_report,
    looks_like_agent_on,
    write_config_work_root,
)


def run(cmd: list[str], cwd: Path | None = None, check: bool = True) -> subprocess.CompletedProcess:
    print("+", " ".join(cmd), flush=True)
    return subprocess.run(cmd, cwd=str(cwd) if cwd else None, check=check)


def which(name: str) -> str | None:
    return shutil.which(name)


def ensure_git() -> None:
    if not which("git"):
        print("ERROR: 需要 git 在 PATH 中。", file=sys.stderr)
        sys.exit(2)


def clone_or_update(work_root: Path, pin: str, remote: str) -> None:
    git_dir = work_root / ".git"
    if not git_dir.exists():
        work_root.parent.mkdir(parents=True, exist_ok=True)
        if work_root.exists() and any(work_root.iterdir()):
            print(f"ERROR: {work_root} 非空且不是 git 仓。换 --work-root 或清空后重试。", file=sys.stderr)
            sys.exit(2)
        run(["git", "clone", remote, str(work_root)])
    else:
        run(["git", "fetch", "--tags", "--force", "origin"], cwd=work_root, check=False)

    # pin: tag 优先，失败则保持当前分支并警告
    r = subprocess.run(
        ["git", "checkout", "--detach", pin],
        cwd=str(work_root),
        capture_output=True,
        text=True,
    )
    if r.returncode != 0:
        print(f"WARN: checkout {pin} 失败，尝试 origin/main …", flush=True)
        run(["git", "checkout", "main"], cwd=work_root, check=False)
        run(["git", "pull", "--ff-only", "origin", "main"], cwd=work_root, check=False)
    else:
        print(f"checked out {pin}", flush=True)

    if not looks_like_agent_on(work_root):
        print(f"ERROR: {work_root} 不像 agent-on 仓（缺 CHARTER/BOOTSTRAP）。", file=sys.stderr)
        sys.exit(1)


def try_plugin_claude(work_root: Path) -> None:
    claude = which("claude")
    if not claude:
        print("skip claude plugin: claude 不在 PATH")
        return
    # 远端 marketplace（与本地 path 二选一；远端失败再试本地）
    r1 = subprocess.run(
        [claude, "plugin", "marketplace", "add", "Palebluedot-ai/agent-on"],
        capture_output=True,
        text=True,
    )
    if r1.returncode != 0:
        subprocess.run(
            [claude, "plugin", "marketplace", "add", str(work_root)],
            check=False,
        )
    subprocess.run(
        [claude, "plugin", "install", "agent-on@agent-on", "-s", "user"],
        check=False,
    )
    print("claude plugin: 已尝试 install agent-on@agent-on（失败可手动重跑）")


def try_plugin_codex(work_root: Path) -> None:
    codex = which("codex")
    if not codex:
        print("skip codex plugin: codex 不在 PATH")
        return
    r1 = subprocess.run(
        [codex, "plugin", "marketplace", "add", "Palebluedot-ai/agent-on"],
        capture_output=True,
        text=True,
    )
    if r1.returncode != 0:
        subprocess.run(
            [codex, "plugin", "marketplace", "add", str(work_root)],
            check=False,
        )
    subprocess.run(
        [codex, "plugin", "install", "agent-on@agent-on"],
        check=False,
    )
    print("codex plugin: 已尝试 install agent-on@agent-on")


def link_skill(work_root: Path, dest: Path) -> None:
    src = work_root / "skill"
    if not src.is_dir():
        print(f"skip symlink: 无 {src}")
        return
    dest.parent.mkdir(parents=True, exist_ok=True)
    if dest.is_symlink() or dest.exists():
        if dest.is_symlink() and dest.resolve() == src.resolve():
            print(f"symlink ok: {dest}")
            return
        print(f"skip symlink: 已存在 {dest}（不覆盖）")
        return
    os.symlink(src, dest, target_is_directory=True)
    print(f"symlink: {dest} -> {src}")


def run_intake_lint(work_root: Path) -> None:
    lint = work_root / "ledger" / "intake-lint.py"
    intake = work_root / "intake"
    if not lint.is_file():
        return
    cards = list(intake.glob("*.md")) if intake.is_dir() else []
    cards = [c for c in cards if c.name != "README.md"]
    if not cards:
        print("intake-lint: 无卡文件，跳过")
        return
    r = subprocess.run([sys.executable, str(lint)], cwd=str(work_root))
    if r.returncode == 0:
        print("intake-lint: 通过")
    else:
        print("intake-lint: 有问题（exit %s）——贡献前请修好" % r.returncode)


def print_next_steps(work_root: Path) -> None:
    print()
    print("=" * 60)
    print("agent-on setup 完成")
    print(f"  work_root (B) = {work_root}")
    print(f"  config        = {config_path()}")
    print()
    print("开工：")
    print("  Claude Code  →  /agent-on init   或「初始化本项目」")
    print("  Codex        →  $agent-on init  或「初始化本项目」")
    print("  Grok         →  「初始化本项目」（全局 AGENT.md 需有 Agent-On 路由）")
    print()
    print("自检：")
    print(f"  python3 {work_root / 'kit' / 'guard' / 'agent_on_paths.py'}")
    print(f"  python3 {work_root / 'ledger' / 'intake-lint.py'}")
    print()
    print("文档：README「给朋友的 5 分钟装机」")
    print("=" * 60)


def main() -> int:
    parser = argparse.ArgumentParser(description="agent-on 一键装机（默认目录 + config）")
    parser.add_argument(
        "--work-root",
        type=str,
        default=None,
        help=f"覆盖默认 B 目录（默认: {default_work_root()}）",
    )
    parser.add_argument("--pin", type=str, default=DEFAULT_PIN, help=f"git tag/ref（默认 {DEFAULT_PIN}）")
    parser.add_argument(
        "--remote",
        type=str,
        default=OFFICIAL_HTTPS,
        help="git remote URL",
    )
    parser.add_argument(
        "--with-plugins",
        action="store_true",
        help="尝试 claude/codex plugin marketplace add + install",
    )
    parser.add_argument(
        "--with-symlinks",
        action="store_true",
        help="挂 skill symlink 到 ~/.claude/skills 与 ~/.agents/skills",
    )
    parser.add_argument(
        "--config-only",
        action="store_true",
        help="不 clone：仅当 work_root 已是合法仓时写 config",
    )
    args = parser.parse_args()

    ensure_git()
    work_root = Path(args.work_root).expanduser().resolve() if args.work_root else default_work_root()

    print(f"platform     = {sys.platform}")
    print(f"work_root    = {work_root}")
    print(f"pin          = {args.pin}")
    print(f"remote       = {args.remote}")

    if args.config_only:
        if not looks_like_agent_on(work_root):
            print(f"ERROR: --config-only 但 {work_root} 不是合法 agent-on 仓", file=sys.stderr)
            return 1
    else:
        clone_or_update(work_root, args.pin, args.remote)

    cfg = write_config_work_root(work_root)
    print(f"wrote config = {cfg}")

    if args.with_plugins:
        try_plugin_claude(work_root)
        try_plugin_codex(work_root)

    if args.with_symlinks:
        home = Path.home()
        link_skill(work_root, home / ".claude" / "skills" / "agent-on")
        link_skill(work_root, home / ".agents" / "skills" / "agent-on")

    print()
    print("--- doctor ---")
    # 临时让 doctor 看到 config
    sys.path.insert(0, str(work_root / "kit" / "guard"))
    print(doctor_report())

    run_intake_lint(work_root)
    print_next_steps(work_root)
    return 0


if __name__ == "__main__":
    sys.exit(main())
