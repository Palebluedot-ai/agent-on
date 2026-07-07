#!/usr/bin/env python3
"""audit-lint.py — 编排审计事件流(jsonl)的状态机校验器。

职责:读一条 run 的 audit_event jsonl,逐行校验 ——
  1. 合法迁移(ALLOWED_TRANSITIONS):state_from→state_to 必须是允许的边;
  2. 连续性(state_from 必须等于同 (run_id,task_id) 上一次的 state_to);
  3. 非法跳转检测(跳过中间态,如 queued 直接 running);
  4. 时间戳格式(ISO-8601,末尾 Z 或 +00:00);
  5. 必填字段齐全(对齐 kit/schemas/audit-event.schema.json)。
纯标准库零依赖。报错信息中文。退出码:全绿 0,有错 1。

用法:  python3 audit-lint.py <run>.jsonl

源流:一代 agent-orchestration-playbook/scripts/execution_audit_lint_v1.py,
2026-07-07 批二移植 —— 字段名对齐去平台化后的 audit-event schema
(channel/workspace/profile → worktree_path/phase_card),CLI 与报错改中文。
"""
import json
import sys
from datetime import datetime
from pathlib import Path

# 固定状态机:每个状态允许迁到的下一状态集合。done/failed 为终态。
ALLOWED_TRANSITIONS = {
    "none": {"queued"},
    "queued": {"triaged", "failed"},
    "triaged": {"planned", "failed"},
    "planned": {"running", "failed"},
    "running": {"reviewing", "failed"},
    "reviewing": {"done", "failed"},
    "done": set(),
    "failed": set(),
}

# 必填字段,对齐 kit/schemas/audit-event.schema.json 的 required
REQUIRED = {
    "run_id", "task_id", "timestamp",
    "state_from", "state_to", "actor_role", "action",
}


def parse(path: Path):
    events = []
    for i, line in enumerate(path.read_text().splitlines(), start=1):
        if not line.strip():
            continue
        try:
            events.append((i, json.loads(line)))
        except Exception as e:
            events.append((i, {"_parse_error": str(e)}))
    return events


def lint(events):
    errors = []
    last_state = {}  # (run_id, task_id) -> 上一次的 state_to
    for line_no, ev in events:
        if "_parse_error" in ev:
            errors.append((line_no, f"JSON 解析失败:{ev['_parse_error']}"))
            continue

        miss = sorted(REQUIRED - set(ev.keys()))
        if miss:
            errors.append((line_no, f"缺必填字段:{', '.join(miss)}"))
            continue

        # 时间戳格式
        try:
            datetime.fromisoformat(str(ev["timestamp"]).replace("Z", "+00:00"))
        except Exception:
            errors.append((line_no, f"时间戳格式非法(需 ISO-8601):{ev['timestamp']}"))

        # schema 规定 state_from 是字符串枚举,首个事件写 "none";JSON null 直接点破,别让报错里出现 'None' 迷惑人
        if ev["state_from"] is None:
            errors.append(
                (line_no, 'state_from 是 null:按 schema(audit-event.schema.json)应为字符串,'
                          '首个事件请写 "none"')
            )
            continue

        key = (ev["run_id"], ev["task_id"])
        prev = last_state.get(key, "none")

        # 连续性:state_from 必须接上一次的 state_to
        if ev["state_from"] != prev:
            errors.append(
                (line_no, f"state_from 断裂:该 task 上一状态应为 '{prev}',"
                          f"本行却写 '{ev['state_from']}'")
            )

        # 合法迁移 / 非法跳转
        allowed = ALLOWED_TRANSITIONS.get(ev["state_from"], set())
        if ev["state_to"] not in allowed:
            legal = ", ".join(sorted(allowed)) or "(终态,不可再迁移)"
            errors.append(
                (line_no, f"非法跳转:'{ev['state_from']}' → '{ev['state_to']}';"
                          f"从 '{ev['state_from']}' 只允许迁到 {legal}")
            )
        else:
            last_state[key] = ev["state_to"]

        # failed 应带 error_type
        if ev["state_to"] == "failed" and not ev.get("error_type"):
            errors.append((line_no, "迁到 failed 却未记 error_type"))

    return errors


def main():
    if len(sys.argv) != 2:
        print("用法:python3 audit-lint.py <run>.jsonl", file=sys.stderr)
        sys.exit(2)

    path = Path(sys.argv[1])
    if not path.exists():
        print(f"文件不存在:{path}", file=sys.stderr)
        sys.exit(2)

    events = parse(path)
    errors = lint(events)

    if not errors:
        print(f"审计通过:{len(events)} 个事件,状态机全合法。")
        sys.exit(0)

    print(f"审计不通过:{len(events)} 个事件,发现 {len(errors)} 处问题:")
    for line_no, msg in errors:
        print(f"  第 {line_no} 行:{msg}")
    sys.exit(1)


if __name__ == "__main__":
    main()
