# kit/guard/ — 机械强制层（不赌 agent 读没读规则）

> 职责边界：存放跨仓边界等硬规矩的**强制执行脚本**（宪章「轻量校验脚本」额度内）。文档层（AGENTS.md/AGENT.md）负责讲道理，本目录负责「讲不通也过不去」。设计依据：deep-research run wf_9c47f385-3e2（2026-07-13，Anthropic 官方 advisory/deterministic 裁决 + Euan 越界事故 5b4ecdd）；卡 `enforcement-layer-design`。

## agent-on-git-guard.sh（L-动作层，PreToolUse hook）

只拦一件事：**会话根不在 agent-on 仓内，却对 agent-on 仓执行 git 写操作**（add/commit/push/reset/tag 创建等）。读操作（status/log/diff/`tag -l`，握手对表在用）与 agent-on 自己的会话（含 `.claude/worktrees/` 下的）一律放行。拦截时 exit 2，stderr 回灌给模型讲清正确动作（落盘 intake 即止，提示用户切会话）。

### 注册（两路并存过渡，v0.5）

**路 A · 个人 scope 手工注册**（v0.4 起已挂，换机可靠；路径可改 `AGENT_ON_ROOT`）：

**Codex**（`~/.codex/hooks.json`）与 **Claude Code**（`~/.claude/settings.json` → 真身 `~/agent-memory/dotfiles/claude/settings.json`）均已于 2026-07-13 注册（Claude 侧首次写入被用户 soft_deny 拦下、经用户明示确认后落笔——护栏对 agent 自己生效的活例）。对**新开会话**生效：

```json
"PreToolUse": [
  {
    "matcher": "Bash",
    "hooks": [
      { "type": "command", "command": "python3 \"$HOME/Projects/Agent-On/kit/guard/agent-on-git-guard.sh\"" }
    ]
  }
]
```

（Codex 侧省略 `"matcher"` 行——其 hooks.json 的既有条目即无 matcher 形态，脚本对非 git 命令零干扰、快速 exit 0。）

**路 B · Plugin 生命周期注册**（v0.5 阶段 2+）：

- Claude：仓内 [`hooks/hooks.json`](../../hooks/hooks.json) 随 plugin 启用自动挂载；命令为 `python3 "${CLAUDE_PLUGIN_ROOT}/kit/guard/agent-on-git-guard.sh"`。
- Codex：备件在 [`hooks/hooks-codex.json`](../../hooks/hooks-codex.json)，**尚未接线**——`.codex-plugin/plugin.json` 仍 `hooks:{}`，等 #16430 实测通过再改（防 auto-discovery 误注册，见 superpowers 6.1.0→6.1.1）。
- 边界目标仓可用环境变量 **`AGENT_ON_ROOT`** 覆盖（默认 `~/Projects/Agent-On`）；与「脚本从哪加载」的 `CLAUDE_PLUGIN_ROOT` 是两回事。
- 两路同时挂时 guard 可能跑两次：双放行 / 双拦截语义不变，仅多一次廉价 exit。

### 实测矩阵（2026-07-13，14/14 通过，直接管道合成 PreToolUse JSON）

| 姿势 | 期望 | 结果 |
|---|---|---|
| 事故原型：项目会话 cwd 在 agent-on 直接 `git commit` | 拦 | ✅ |
| `cd <绝对路径> && git commit` / `cd ../Agent-On && git add`（相对路径） | 拦 | ✅✅ |
| `git -C <路径> commit --no-verify`（tool 层拦截，git 层旗标无效） | 拦 | ✅ |
| `git -C ~/... push` / `add+commit` 复合 / `tag v9`、`tag -a` 创建 | 拦 | ✅✅✅✅ |
| agent-on 主会话与 worktree 会话的 commit/push（消化/收件） | 放 | ✅✅ |
| 项目自己仓 commit（message 提到 agent-on 不误伤） | 放 | ✅ |
| 项目会话读 agent-on（`log`/`tag -l` 握手对表） | 放 | ✅ |
| 项目仓读操作 / 非 git 命令 | 放 | ✅✅ |

### 已知边界（研究实证的失效面，诚实列出）

- Claude Code：`--dangerously-skip-permissions` 下 hooks 不生效（本机不用该 flag）；Task/subagent 工具有 exit-2 失效记录（issue #26923）
- Codex：会话根变量回退用 cwd——项目会话若先 `cd` 进 agent-on 再单独发 `git commit`（跨两次调用），Codex 侧可能漏拦（Claude 侧有 CLAUDE_PROJECT_DIR 不受影响）
- 两家 hooks 均快速迭代，JSON 字段可能数月内变化；guard 解析失败时**放行不报错**（不当故障点）——变更后跑一遍下方回归
- git 原生 pre-commit hook 兜底（防 hooks 全关的终极层）本轮研究零验证证据，deferred

### 回归与回滚

```bash
# 回归（12+2 用例脚本见会话 scratchpad,或手工三发）:
echo '{"tool_name":"Bash","tool_input":{"command":"git -C ~/Projects/Agent-On commit -m x"},"cwd":"/tmp"}' \
  | CLAUDE_PROJECT_DIR=/tmp python3 ~/Projects/Agent-On/kit/guard/agent-on-git-guard.sh; echo "exit=$? (期望2)"
# AGENT_ON_ROOT 覆盖:
echo '{"tool_name":"Bash","tool_input":{"command":"git -C /tmp/my-agent-on commit -m x"},"cwd":"/tmp"}' \
  | CLAUDE_PROJECT_DIR=/tmp AGENT_ON_ROOT=/tmp/my-agent-on python3 ~/Projects/Agent-On/kit/guard/agent-on-git-guard.sh; echo "exit=$? (期望2)"
# 回滚路 A:从 ~/.claude/settings.json 与 ~/.codex/hooks.json 删掉 PreToolUse 里本脚本条目
# 回滚路 B:claude plugin disable/uninstall agent-on(或 Codex 侧保持 hooks:{} 即未接线)
```

## 未建层（等 L-动作跑两周再议）

- **L-进场**：SessionStart 注入硬规矩摘要（治「没读」；注入≠遵循，边际价值待 L-动作数据）
- **L-收尾**：Stop hook 结账校验（agent-on 仓有 intake/ 之外改动不许收工）
