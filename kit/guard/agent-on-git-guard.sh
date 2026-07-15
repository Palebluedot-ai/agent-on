#!/usr/bin/env python3
# agent-on-git-guard — PreToolUse hook:跨仓 git 边界的机械强制层(L-动作)
# 职责边界:只拦一件事——「会话根不在 agent-on 工作仓(B)内,却对 B 执行 git 写操作」。
#   项目端会话对 agent-on 仓只写 intake/ 素材文件,不 add/commit/push(AGENT.md 硬规矩 2026-07-13)。
#   读操作(status/log/diff/tag 对表)一律放行;agent-on 仓自己的会话(含其 worktrees)一律放行。
# B 解析:kit/guard/agent_on_paths.py(AGENT_ON_ROOT → config → lock);未登记 B 时 fail-open 不拦。
# CLAUDE_PLUGIN_ROOT 只用于「本脚本从哪加载」,不用于边界判定。
# 语义:exit 0 = 放行;exit 2 = 拦截,stderr 回灌给模型说明原因(两家等价)。
# 回滚:从 hooks 注册里删掉本脚本条目即回 advisory 现状;脚本本体无副作用可留。
# 来源:deep-research run wf_9c47f385-3e2(2026-07-13)+ Euan 越界事故(5b4ecdd);可移植路径 2026-07-16。
import json
import os
import re
import shlex
import sys

# 与本脚本同目录的路径解析模块
sys.path.insert(0, os.path.dirname(os.path.realpath(__file__)))
try:
    from agent_on_paths import resolve_work_root  # type: ignore
except ImportError:
    resolve_work_root = None  # type: ignore

# git 写操作子命令(动历史/工作树的);读操作 status/log/diff/show/fetch/describe/tag -l 不在列
WRITE_OPS = {
    "add", "commit", "push", "reset", "revert", "rebase", "merge",
    "cherry-pick", "stash", "tag", "rm", "mv", "restore", "checkout",
    "switch", "clean", "am", "apply", "update-ref",
}
# 命令文本里可能指向工作仓的路径片段(仅作候选;最终以 B realpath 前缀判定)
PATH_RE = re.compile(r"[~/][\w@.\-/\\]*agent-on[\w@.\-/\\]*", re.IGNORECASE)


def norm(p: str, base: str) -> str:
    p = os.path.expanduser(p)
    if not os.path.isabs(p):
        p = os.path.join(base, p)
    return os.path.realpath(p)


def main() -> int:
    try:
        data = json.loads(sys.stdin.read() or "{}")
    except (ValueError, UnicodeDecodeError):
        return 0  # 解析不了就不拦(guard 只管边界,不当故障点)

    ti = data.get("tool_input") or {}
    cmd = ti.get("command") if isinstance(ti, dict) else None
    if not isinstance(cmd, str) or "git" not in cmd:
        return 0

    cwd = data.get("cwd") or os.getcwd()

    # B 未登记 → fail-open(外机/Windows 无 Chao 路径时不瞎拦)
    agent_on = None
    if resolve_work_root is not None:
        wr, _src = resolve_work_root(cwd)
        if wr is not None:
            agent_on = str(wr)
    if not agent_on:
        return 0

    def inside_agent_on(p: str) -> bool:
        # macOS 大小写不敏感;Windows 亦常不敏感——统一 lower 比较
        return p.lower().startswith(agent_on.lower())

    try:
        toks = shlex.split(cmd)
    except ValueError:
        toks = cmd.split()

    write_hit = False
    targets = []  # 本命令里 git 写操作的目标目录候选
    cd_base = cwd  # 顺序模拟 cd 链
    i = 0
    while i < len(toks):
        t = toks[i]
        if t == "cd" and i + 1 < len(toks):
            cd_base = norm(toks[i + 1], cd_base)
            i += 2
            continue
        if t == "git":
            git_dir = cd_base
            j = i + 1
            while j < len(toks):
                tj = toks[j]
                if tj == "-C" and j + 1 < len(toks):
                    git_dir = norm(toks[j + 1], cd_base)
                    j += 2
                    continue
                if tj.startswith("-"):
                    j += 1
                    continue
                if tj in WRITE_OPS:
                    # 特例:`git tag`(裸)与 `git tag -l/--list/...` 是读操作(握手对表在用),只有创建/删除才算写
                    if tj == "tag":
                        nxt = toks[j + 1] if j + 1 < len(toks) else None
                        READ_TAG_FLAGS = {
                            "-l", "--list", "-n", "--contains", "--points-at",
                            "--sort", "--format", "--merged", "--no-merged", "--column",
                        }
                        is_sep = nxt in (None, "&&", "||", ";", "|")
                        if is_sep or (nxt or "").split("=")[0] in READ_TAG_FLAGS:
                            break  # 读形态,不算写
                    write_hit = True
                    targets.append(git_dir)
                break  # 第一个非选项 token 即子命令,判完就走
            i = j + 1
            continue
        i += 1

    if not write_hit:
        return 0

    # 命令文本里显式出现的 agent-on 路径(cd/-C 之外的形态)也算目标
    for m in PATH_RE.findall(cmd):
        targets.append(norm(m, cwd))

    if not any(inside_agent_on(t) for t in targets):
        return 0

    # 会话根:Claude Code 注入 CLAUDE_PROJECT_DIR;Codex 侧回退 cwd(残留风险见 README)
    sess = (
        os.environ.get("CLAUDE_PROJECT_DIR")
        or os.environ.get("CODEX_PROJECT_DIR")
        or cwd
    )
    if inside_agent_on(os.path.realpath(os.path.expanduser(sess))):
        return 0  # agent-on 工作仓自己的会话(含 worktree),放行

    sys.stderr.write(
        "⛔ 跨仓 git 边界拦截(agent-on-git-guard):项目端会话对 agent-on 工作仓只写 intake/ 素材文件,"
        "不 add / 不 commit / 不 push(AGENT.md 硬规矩 2026-07-13)。\n"
        f"工作仓(B) {agent_on}\n"
        f"会话根 {sess} 不在 B 内,却试图对 B 执行 git 写操作。\n"
        "正确动作:落盘 intake 文件即止;git 收件/消化由用户切到 agent-on 工作仓会话执行。\n"
        f"被拦命令:{cmd}\n"
    )
    return 2


if __name__ == "__main__":
    sys.exit(main())
