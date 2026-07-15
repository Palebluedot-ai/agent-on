#!/usr/bin/env python3
# agent_on_paths — 可移植路径解析（A 运行包 / B 工作仓）
# 职责边界：唯一实现「哪里是 agent-on」的解析序；guard / doctor / 文档同此。
# 产品约定（2026-07-16）：~/Projects/Agent-On 不是产品默认；B 必须显式登记。
#
# A · 运行包根（只读）：CLAUDE_PLUGIN_ROOT / PLUGIN_ROOT（plugin 装机面）
# B · 工作仓（可写）：AGENT_ON_ROOT → ~/.config/agent-on/config.json work_root
#                    → 自 cwd 上溯 agent-on.lock.md 的「本地路径」
# 无 B：settle/digest 应拒绝；guard fail-open（不拦）
from __future__ import annotations

import json
import os
import re
import sys
from pathlib import Path
from typing import Optional, Tuple

CONFIG_REL = Path(".config") / "agent-on" / "config.json"
MARKERS = ("CHARTER.md", "BOOTSTRAP.md")
LOCK_NAME = "agent-on.lock.md"
# lock 行：本地路径:… 或 本地路径：…（全角冒号）
LOCK_PATH_RE = re.compile(
    r"^\s*本地路径\s*[:：]\s*`?([^\s`]+)`?\s*$",
    re.MULTILINE,
)


def expand(p: str) -> Path:
    return Path(os.path.realpath(os.path.expanduser(p.strip().strip("\"'"))))


def looks_like_agent_on(root: Path) -> bool:
    try:
        return root.is_dir() and all((root / m).is_file() for m in MARKERS)
    except OSError:
        return False


def config_path() -> Path:
    return Path.home() / CONFIG_REL


def read_config_work_root() -> Optional[Path]:
    cfg = config_path()
    if not cfg.is_file():
        return None
    try:
        data = json.loads(cfg.read_text(encoding="utf-8"))
    except (OSError, ValueError, UnicodeDecodeError):
        return None
    if not isinstance(data, dict):
        return None
    wr = data.get("work_root")
    if not isinstance(wr, str) or not wr.strip():
        return None
    return expand(wr)


def find_lock_up(start: Optional[Path] = None) -> Optional[Path]:
    cur = (start or Path.cwd()).resolve()
    for _ in range(24):
        cand = cur / LOCK_NAME
        if cand.is_file():
            return cand
        if cur.parent == cur:
            break
        cur = cur.parent
    return None


def work_root_from_lock(lock_file: Path) -> Optional[Path]:
    try:
        text = lock_file.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError):
        return None
    m = LOCK_PATH_RE.search(text)
    if not m:
        return None
    return expand(m.group(1))


def resolve_work_root(cwd: Optional[str] = None) -> Tuple[Optional[Path], str]:
    """返回 (B 路径或 None, 来源标签)。路径存在且像 agent-on 仓才算。"""
    env = os.environ.get("AGENT_ON_ROOT")
    if env and env.strip():
        p = expand(env)
        if looks_like_agent_on(p):
            return p, "AGENT_ON_ROOT"
        return None, "AGENT_ON_ROOT(invalid)"

    cfg = read_config_work_root()
    if cfg is not None:
        if looks_like_agent_on(cfg):
            return cfg, "config:work_root"
        return None, "config:work_root(invalid)"

    start = Path(cwd).resolve() if cwd else None
    lock = find_lock_up(start)
    if lock is not None:
        from_lock = work_root_from_lock(lock)
        if from_lock is not None and looks_like_agent_on(from_lock):
            return from_lock, f"lock:{lock}"
        if from_lock is not None:
            return None, f"lock(invalid):{lock}"

    return None, "unset"


def resolve_read_root(cwd: Optional[str] = None) -> Tuple[Optional[Path], str]:
    """读模板/执行书：plugin 优先，其次 B。"""
    for key in ("CLAUDE_PLUGIN_ROOT", "PLUGIN_ROOT"):
        v = os.environ.get(key)
        if v and v.strip():
            p = expand(v)
            if looks_like_agent_on(p):
                return p, key
    wr, src = resolve_work_root(cwd)
    if wr is not None:
        return wr, f"work_root via {src}"
    return None, "unset"


def doctor_report(cwd: Optional[str] = None) -> str:
    lines = []
    rr, rs = resolve_read_root(cwd)
    wr, ws = resolve_work_root(cwd)
    lines.append(f"read_root  = {rr or '(none)'}  [{rs}]")
    lines.append(f"work_root  = {wr or '(none)'}  [{ws}]")
    lines.append(f"config     = {config_path()}  exists={config_path().is_file()}")
    for k in ("AGENT_ON_ROOT", "CLAUDE_PLUGIN_ROOT", "PLUGIN_ROOT"):
        lines.append(f"env {k} = {os.environ.get(k) or '(unset)'}")
    if wr is None:
        lines.append("")
        lines.append("未登记可写工作仓 (work_root)。init/adopt/handshake 可只靠 plugin。")
        lines.append("settle/digest 需要可写 clone，任选其一登记：")
        lines.append("  1) export AGENT_ON_ROOT=<clone绝对路径>")
        lines.append(
            '  2) mkdir -p ~/.config/agent-on && '
            'echo \'{"work_root":"<clone绝对路径>"}\' > ~/.config/agent-on/config.json'
        )
        lines.append("  3) 项目 agent-on.lock.md 的「本地路径」填本机 clone 路径")
        lines.append("clone 示例：git clone <repo-url> <任意路径>（不必叫 Projects/Agent-On）")
    if rr is None:
        lines.append("")
        lines.append("未找到可读运行包。请：")
        lines.append("  claude plugin marketplace add Palebluedot-ai/agent-on")
        lines.append("  claude plugin install agent-on@agent-on")
        lines.append("或登记 work_root 指向完整 clone。")
    return "\n".join(lines) + "\n"


def main(argv: list[str]) -> int:
    cwd = None
    if len(argv) >= 2 and argv[1] == "--cwd" and len(argv) >= 3:
        cwd = argv[2]
    sys.stdout.write(doctor_report(cwd))
    wr, _ = resolve_work_root(cwd)
    rr, _ = resolve_read_root(cwd)
    # doctor 总是 0 出口，方便人读；缺 B 用文案表达
    return 0 if (rr is not None or wr is not None) else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
