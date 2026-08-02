#!/usr/bin/env python3
"""Assert agent-on open-box defaults: skill routing, demotion protocol, demotions.

Scans real file paths. Exit 0 only if required phrases exist and forbidden
patterns (default Superpowers implementer; resurrected lock-token ritual as
recommended default) are absent.

Usage:
  python3 scripts/check-skill-routing.py
  python3 scripts/check-skill-routing.py --with-agent-memory
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def die(msg: str) -> None:
    print(f"FAIL: {msg}", file=sys.stderr)
    sys.exit(1)


def read(path: Path) -> str:
    if not path.is_file():
        die(f"missing file: {path}")
    return path.read_text(encoding="utf-8")


def must_contain(path: Path, text: str, patterns: list[str]) -> None:
    for pat in patterns:
        if re.search(pat, text, flags=re.IGNORECASE | re.MULTILINE) is None:
            die(f"{path.relative_to(ROOT) if path.is_relative_to(ROOT) else path}: missing /{pat}/")


def must_not_contain(path: Path, text: str, patterns: list[str]) -> None:
    for pat in patterns:
        if re.search(pat, text, flags=re.IGNORECASE | re.MULTILINE):
            die(f"{path.relative_to(ROOT) if path.is_relative_to(ROOT) else path}: forbidden /{pat}/")


def check_agent_on() -> None:
    lite = ROOT / "kit" / "AGENTS-lite.md"
    skel = ROOT / "kit" / "AGENTS-skeleton.md"
    bootstrap = ROOT / "BOOTSTRAP.md"
    readme = ROOT / "README.md"
    ledger = ROOT / "ledger" / "run-card-logging.md"
    mrd = ROOT / "snapshot" / "2026-08-02-light-hard-premium-mrd.md"
    adopt = ROOT / "boot" / "adopt.md"
    schemas = ROOT / "kit" / "schemas" / "README.md"
    phase_gates = ROOT / "playbook" / "phase-gates.md"
    audit = ROOT / "snapshot" / "2026-08-03-research-residual-audit.md"

    lite_t = read(lite)
    skel_t = read(skel)
    boot_t = read(bootstrap)
    readme_t = read(readme)
    ledger_t = read(ledger)
    mrd_t = read(mrd)
    adopt_t = read(adopt)
    schemas_t = read(schemas)
    phase_t = read(phase_gates)
    audit_t = read(audit)

    # lite: institution + suppress Superpowers default
    must_contain(
        lite,
        lite_t,
        [
            r"制度在 agent-on",
            r"不默认 Superpowers",
            r"brainstorming",
            r"subagent-driven-development",
        ],
    )
    # skeleton must NOT present Superpowers as the filled-in implementer example default
    must_not_contain(
        skel,
        skel_t,
        [
            r"实现执行\s*\|\s*\[?如 Superpowers subagent-driven-development",
            r"实现执行\s*\|\s*Superpowers",
        ],
    )
    must_contain(
        skel,
        skel_t,
        [
            r"不.*默认 Superpowers|不默认.*subagent-driven-development",
            r"主会话",
            r"brainstorming",
        ],
    )

    must_contain(
        bootstrap,
        boot_t,
        [
            r"默认心态偏 S|拿不准取低档",
            r"不默认 Superpowers",
            r"制度",
            r"adopt\.md.*§三|降档.*§三|§三.*降档",
        ],
    )
    must_contain(
        readme,
        readme_t,
        [
            r"默认心态偏 S|拿不准取低档",
            r"旁路|尚未在真实项目验证|零真实项目跑通|开箱勿启用",
            r"Superpowers 退出默认|不默认 Superpowers",
        ],
    )
    must_contain(
        ledger,
        ledger_t,
        [
            r"旁路",
            r"未.*真实项目跑通|尚未在真实项目验证",
        ],
    )
    must_contain(
        mrd,
        mrd_t,
        [
            r"B1",
            r"C1",
            r"已拍板",
        ],
    )

    # (A) demotion protocol — equal footing with promote
    must_contain(
        adopt,
        adopt_t,
        [
            r"## 三、降档",
            r"禁止静默降档",
            r"只删不用的件",
            r"不重播",
            r"local_deviations",
            r"显式批准",
        ],
    )

    # (B) schemas + ledger class demotion visible at schemas entry
    must_contain(
        schemas,
        schemas_t,
        [
            r"旁路",
            r"开箱.*勿启用|勿启用",
            r"未.*真实项目跑通|尚未在真实项目验证",
        ],
    )

    # lock-token ritual: historical death list, not recommended default
    must_contain(
        phase_gates,
        phase_t,
        [
            r"每轮口令复述",
            r"不要照搬|扔掉的是.*形式|禁止复活",
            r"fail-closed",
        ],
    )
    # must not re-introduce "every reply must paste Global/In-Scope" as current required default
    must_not_contain(
        phase_gates,
        phase_t,
        [
            r"开箱默认.*每轮.*Global\s*/\s*In-Scope",
            r"推荐路径.*每轮口令复述硬拦",
        ],
    )

    # residual audit inventory present
    must_contain(
        audit,
        audit_t,
        [
            r"三不变量|完成=证据|单一状态写者",
            r"jsonl|旁路",
            r"锁口令|每轮口令复述",
            r"下次顺手|降档协议",
        ],
    )

    print(
        "OK agent-on:",
        lite.name,
        skel.name,
        bootstrap.name,
        readme.name,
        ledger.name,
        mrd.name,
        adopt.name,
        schemas.name,
        phase_gates.name,
        audit.name,
    )


def check_agent_memory(home: Path) -> None:
    claude = home / "agent-memory" / "dotfiles" / "claude" / "CLAUDE.md"
    routing = home / "agent-memory" / "memory" / "project_skill_routing.md"
    mem = home / "agent-memory" / "memory" / "MEMORY.md"

    claude_t = read(claude)
    routing_t = read(routing)
    mem_t = read(mem)

    must_contain(
        claude,
        claude_t,
        [
            r"不默认 Superpowers|退出默认",
            r"agent-on.*制度|制度层",
            r"完成.*=.*验证|贴验证|实际输出",
            r"brainstorming",
        ],
    )
    # must not still require Superpowers as default implementer
    must_not_contain(
        claude,
        claude_t,
        [
            r"用 Superpowers subagent-driven-development 的执行引擎",
            r"Implementation → Superpowers subagent-driven-development",
        ],
    )
    must_contain(
        routing,
        routing_t,
        [
            r"Superpowers",
            r"not default|不默认|off default|退出默认",
            r"agent-on",
        ],
    )
    must_not_contain(
        mem,
        mem_t,
        [
            r"impl=Superpowers",
        ],
    )
    must_contain(mem, mem_t, [r"project_skill_routing\.md", r"Superpowers off default|off default|不默认"])

    print("OK agent-memory:", claude, routing.name, mem.name)


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--with-agent-memory",
        action="store_true",
        help="Also check ~/agent-memory Claude routing files",
    )
    ap.add_argument(
        "--home",
        type=Path,
        default=Path.home(),
        help="Home directory for agent-memory (default: ~)",
    )
    args = ap.parse_args()

    check_agent_on()
    if args.with_agent_memory:
        check_agent_memory(args.home)
    print("ALL CHECKS PASSED")


if __name__ == "__main__":
    main()
