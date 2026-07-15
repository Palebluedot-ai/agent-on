# kit/guard/ — 机械强制层（不赌 agent 读没读规则）

> 职责边界：存放跨仓边界等硬规矩的**强制执行脚本**（宪章「轻量校验脚本」额度内）。文档层负责讲道理，本目录负责「讲不通也过不去」。
> **可移植（2026-07-16）**：不再默认 `~/Projects/Agent-On`。B 工作仓经 `agent_on_paths.py` 解析；未登记 B 时 guard **fail-open**（不拦），避免外机/Windows 误伤。

## 路径模块 `agent_on_paths.py`

| 面 | 含义 | 解析 |
|---|---|---|
| A 运行包 | plugin 只读根 | `CLAUDE_PLUGIN_ROOT` / `PLUGIN_ROOT` |
| B 工作仓 | 可写 clone（intake/消化） | `AGENT_ON_ROOT` → `~/.config/agent-on/config.json` 的 `work_root` → 上溯 `agent-on.lock.md`「本地路径」 |

```bash
python3 kit/guard/agent_on_paths.py          # doctor 报告
python3 kit/guard/agent_on_paths.py --cwd /path/to/project
```

示例配置：[`../agent-on-user-config.example.json`](../agent-on-user-config.example.json) → 拷到 `~/.config/agent-on/config.json`。

## agent-on-git-guard.sh（L-动作层，PreToolUse hook）

只拦：**会话根不在已登记的 B 内，却对 B 执行 git 写**。读操作与 B 内会话一律放行。未登记 B → exit 0（fail-open）。

### 注册

**路 B · Plugin（推荐，路径可移植）**：仓内 [`hooks/hooks.json`](../../hooks/hooks.json) 随 Claude plugin 启用：

```json
{ "type": "command", "command": "python3 \"${CLAUDE_PLUGIN_ROOT}/kit/guard/agent-on-git-guard.sh\"" }
```

**路 A · 个人 scope（可选 fallback）**：可挂 `~/.claude/settings.json` / `~/.codex/hooks.json`。命令优先 `${CLAUDE_PLUGIN_ROOT}/kit/guard/...`（若环境会注入）；否则指向本机 **B** 下的脚本绝对路径。**不要**在对外文档写死 `Projects/Agent-On`。更稳：Claude 只走路 B（plugin）；个人 scope 主要留给 Codex #16430 未通时。

### 回归

```bash
# 先登记 B
export AGENT_ON_ROOT=/path/to/agent-on-clone   # 或写 config.json

echo '{"tool_input":{"command":"git -C '"$AGENT_ON_ROOT"' commit -m x"},"cwd":"/tmp"}' \
  | CLAUDE_PROJECT_DIR=/tmp python3 kit/guard/agent-on-git-guard.sh; echo "expect 2"

# 未登记 B → fail-open
env -u AGENT_ON_ROOT HOME=/tmp/no-config \
  python3 kit/guard/agent-on-git-guard.sh <<< '{"tool_input":{"command":"git commit -m x"},"cwd":"/tmp"}'
# expect 0
```

## 未建层

- L-进场 / L-收尾（等 L-动作跑稳）
- git 原生 pre-commit 兜底（deferred）
