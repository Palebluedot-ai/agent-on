# kit/guard — 跨仓与 worktree 边界（机械闸）

> 职责：①项目端会话对 agent-on **工作仓 B** 只写 `intake/`，禁止 add/commit/push；②在 Claude/Codex 发起 `git commit/push` 前执行 lane/owns 严格检查。
> **实现**：逻辑在 Rust CLI 的 `agent-on guard`；本目录 extensionless 文件是 canonical Bash shim，`.sh` 仅为旧个人 hook 的 Bash/Python 双兼容入口。

## 路径 / doctor

```bash
agent-on doctor
agent-on doctor --cwd /path/to/project
```

B 解析序：`AGENT_ON_ROOT` → `~/.config/agent-on/config.json` → lock「本地路径」→ 默认 `~/.local/share/agent-on`。  
**未登记 B 时 guard fail-open**（不拦）。

## Hook 注册与触发成本

Claude（`hooks/hooks.json`）：

```json
{ "type": "command", "command": "bash \"${CLAUDE_PLUGIN_ROOT}/kit/guard/agent-on-git-guard\"" }
```

Codex plugin manifest 指向**同一份** `hooks/hooks.json`，不另养副本。guard 对非 git、git 读命令立即放行；完整 `agent-on worktree check` 只在 `commit/push` 前运行。

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

# 当前 repo 的 commit/push → 自动跑 lane/owns strict check
echo '{"tool_input":{"command":"git commit -m probe"},"cwd":"'"$PWD"'"}' \
  | CLAUDE_PROJECT_DIR="$PWD" agent-on guard; echo "expect 0, or 2 with actionable audit"
```

若 stderr 含 `OUT-OF-BOUNDS`，把文件移回所属 lane，或由控制轨重新划分 `owns`；若是 `ERROR/unknown`，先修检查器，不以跳过 hook 当修复。

### v0.6 Codex 旧注册

`python3 .../agent-on-git-guard.sh` 曾因 v0.7 把脚本换成 Bash 而产生 `SyntaxError`。当前 `.sh` 兼容入口已同时支持 `python3` 与 `bash`，但长期建议删除个人重复 hook、使用 plugin；Agent-On 状态检查只提醒，不擅自改 `~/.codex/hooks.json`。

回滚：从 hooks 删掉 PreToolUse 条目即可。

## 分类器/闸拒诊断（命令字面 ≠ 目标）

> 源流:Dartify 2026-08-08——`reset --hard` 拒、同命令间歇拒、带管道拒;换 `ff-only` / 去管道 / 重试即过。

安全闸(PreToolUse / auto classifier)拒绝的是**这条命令字符串**,不是用户目标,且可能间歇误拒。

1. 换**更保守**的等价手段(`reset --hard` → `merge --ff-only`、管道 → 裸跑、整目录 → 显式路径)  
2. **原样重试一次**  
3. 两步不过 → 向用户说明意图并请求授权  

**禁止**:一次拒绝就缩减交付范围或改口「做不到」;也禁止为绕闸升级破坏性。见 anti-hallucination 第六型#17。
