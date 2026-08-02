# kit/guard — 跨仓 git 边界（机械闸）

> 职责：项目端会话对 agent-on **工作仓 B** 只写 `intake/`，禁止 add/commit/push。  
> **实现（2026-08 Rust）**：逻辑在 `cli` 二进制 `agent-on guard`；本目录 `agent-on-git-guard` 为 bash shim。

## 路径 / doctor

```bash
agent-on doctor
agent-on doctor --cwd /path/to/project
```

B 解析序：`AGENT_ON_ROOT` → `~/.config/agent-on/config.json` → lock「本地路径」→ 默认 `~/.local/share/agent-on`。  
**未登记 B 时 guard fail-open**（不拦）。

## Hook 注册

Claude（`hooks/hooks.json`）：

```json
{ "type": "command", "command": "bash \"${CLAUDE_PLUGIN_ROOT}/kit/guard/agent-on-git-guard\"" }
```

先保证：

```bash
cargo build --release --manifest-path cli/Cargo.toml
# 或
cargo install --path cli --force
```

## 最小实测

```bash
# 跨仓写 → 2（需已登记 B = 本仓）
export AGENT_ON_ROOT=/path/to/agent-on
echo '{"tool_input":{"command":"git -C '"$AGENT_ON_ROOT"' commit -m x"},"cwd":"/tmp"}' \
  | CLAUDE_PROJECT_DIR=/tmp agent-on guard; echo "expect 2"

# 自会话 → 0
echo '{"tool_input":{"command":"git commit -m x"},"cwd":"'"$AGENT_ON_ROOT"'"}' \
  | CLAUDE_PROJECT_DIR="$AGENT_ON_ROOT" agent-on guard; echo "expect 0"

# 读操作 → 0
echo '{"tool_input":{"command":"git -C '"$AGENT_ON_ROOT"' status"},"cwd":"/tmp"}' \
  | CLAUDE_PROJECT_DIR=/tmp agent-on guard; echo "expect 0"
```

回滚：从 hooks 删掉 PreToolUse 条目即可。
