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

## 分类器/闸拒诊断（命令字面 ≠ 目标）

> 源流:Dartify 2026-08-08——`reset --hard` 拒、同命令间歇拒、带管道拒;换 `ff-only` / 去管道 / 重试即过。

安全闸(PreToolUse / auto classifier)拒绝的是**这条命令字符串**,不是用户目标,且可能间歇误拒。

1. 换**更保守**的等价手段(`reset --hard` → `merge --ff-only`、管道 → 裸跑、整目录 → 显式路径)  
2. **原样重试一次**  
3. 两步不过 → 向用户说明意图并请求授权  

**禁止**:一次拒绝就缩减交付范围或改口「做不到」;也禁止为绕闸升级破坏性。见 anti-hallucination 第六型#17。
