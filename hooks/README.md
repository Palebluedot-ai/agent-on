# hooks/ — Claude/Codex 共用的 plugin hook 注册

> 职责边界：本目录只有一份 canonical `hooks.json`。Claude 自动发现它；Codex 由 `.codex-plugin/plugin.json` 显式指向同一文件。执行 shim 仍住 `kit/guard/`，不在两家各养一份规则。

## 文件

| 文件 | 工具 | 状态 |
|---|---|---|
| `hooks.json` | Claude Code + Codex | `PreToolUse(Bash)` 调同一 guard；Codex plugin manifest 已接线 |

## 路径约定（可移植）

- **脚本定位（A）**：hook 命令用 `${CLAUDE_PLUGIN_ROOT}`；Codex plugin 兼容该变量并同时提供 `${PLUGIN_ROOT}`。
- **边界判定（B）**：`agent-on doctor` / `cli` 路径解析——`AGENT_ON_ROOT` → `~/.config/agent-on/config.json` → lock「本地路径」。**无 Chao 默认路径**；未登记 B 时 guard fail-open。
- **个人 scope**：可选；plugin hook 是默认。Codex 非 managed 个人 hook 首次运行可能须在 `/hooks` 信任。
- **低开销**：非 git 与 git 读命令在 Rust 解析后立即返回；只有 `commit/push` 才跑完整 lane/owns audit。

## 旧 Codex 个人 hook 迁移

若 `~/.codex/hooks.json` 仍是下面的 v0.6 旧命令：

```text
python3 "$HOME/Projects/Agent-On/kit/guard/agent-on-git-guard.sh"
```

升级后的 `.sh` 兼容入口不会再报 Python `SyntaxError`，但建议删除这条个人重复注册，交给 plugin；若仍保留则改为：

```text
bash "$HOME/Projects/Agent-On/kit/guard/agent-on-git-guard"
```

状态查询必须提示这类旧注册；Agent-On 不静默改写用户 home。

## 发版闸门

1. Claude 与 Codex plugin 均能加载 `hooks/hooks.json`。
2. 两种真实 payload 经 shim：普通命令/`git status` exit 0；lane 内 `commit/push` exit 0；越界时 exit 2 且 stderr 含 `OUT-OF-BOUNDS` 与修复命令。
3. `python3 kit/guard/agent-on-git-guard.sh` 与 `bash kit/guard/agent-on-git-guard.sh` 对同一 payload 给出同一 exit code。

## 回滚

- Claude/Codex：disable/uninstall agent-on plugin；个人 scope 条目不受影响。
