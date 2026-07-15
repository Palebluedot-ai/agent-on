# hooks/ — plugin 生命周期内的 hook 注册

> 职责边界：本目录是 **plugin 打包形态** 下的 hook 入口（Claude 自动发现 `hooks/hooks.json`；Codex 须在 `.codex-plugin/plugin.json` 的 `hooks` 字段显式指向）。脚本本体仍住 `kit/guard/`（宪章「轻量校验脚本」额度内、canonical 不双头）。个人 scope 手工注册（`~/.claude/settings.json` / `~/.codex/hooks.json`）与本目录**并存过渡**，见 [kit/guard/README.md](../kit/guard/README.md)。

## 文件

| 文件 | 工具 | 状态 |
|---|---|---|
| `hooks.json` | Claude Code | **阶段 2 启用**——PreToolUse 调 guard；命令用 `${CLAUDE_PLUGIN_ROOT}` |
| `hooks-codex.json` | Codex CLI | **阶段 3 备件**——已写好，但 `.codex-plugin/plugin.json` 仍写 `hooks:{}` 抑制误注册，待 openai/codex#16430 实测通过再接线 |

## 路径约定（可移植）

- **脚本定位（A）**：`${CLAUDE_PLUGIN_ROOT}` / `${PLUGIN_ROOT}` → plugin 装机面，OS 无关。
- **边界判定（B）**：`kit/guard/agent_on_paths.py`——`AGENT_ON_ROOT` → `~/.config/agent-on/config.json` → lock「本地路径」。**无 Chao 默认路径**；未登记 B 时 guard fail-open。
- **个人 scope**：可选；不要在对外文档写死 `Projects/Agent-On`。

## 闸门（发版前必过）

1. **Claude install + hook 注册**（**2026-07-16 已过**，证据见 `snapshot/2026-07-15-v05-plugin-scoping.md`「Claude 闸门实测」）：摘 symlink → marketplace add 本仓 → install → details 含 PreToolUse；debug 见 `Loading hooks from plugin: agent-on`；`${CLAUDE_PLUGIN_ROOT}` 路径合成 PreToolUse 真拦。
2. **Claude 交互会话真拦**（**仍开**）：已登录的 Claude Code 会话里，非 agent-on 项目触发 `git -C <agent-on> commit`，应被拦。CLI `-p` 本机曾 `Not logged in` 未完成。
3. **Codex**：验证 plugin 内 hook **是否真执行**（#16430）。若 exit 2 从不出现 → 保持 `hooks:{}` + 个人 scope 手工注册，不接线 `hooks-codex.json`。

## 回滚

- Claude：disable/uninstall agent-on plugin，或删本目录后重装旧版；个人 scope 条目不受影响。
- Codex：保持 `hooks:{}` 即未接线状态。
