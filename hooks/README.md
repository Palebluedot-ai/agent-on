# hooks/ — Claude/Codex 共用的 plugin hook 注册

> 职责边界：本目录只有一份 canonical `hooks.json`。Claude 自动发现它；Codex 由 `.codex-plugin/plugin.json` 显式指向同一文件。执行 shim 仍住 `kit/guard/`，不在两家各养一份规则。

## 文件

| 文件 | 工具 | 状态 |
|---|---|---|
| `hooks.json` | Claude Code + Codex | `PreToolUse(Bash)` 与 `PreToolUse(SendMessage)` 调**同一** guard；Codex plugin manifest 已接线 |

## 两个授权 matcher，一个 guard

| matcher | 判什么 | 拦什么 |
|---|---|---|
| `Bash` / Codex `exec_command` | 跨仓 git 写入与值守合并/对外通信路由；实际提交/推送范围由原生 Git hooks 检查，不在这里重复审整个脏树 | 越界 git 写；非值守窗口的 `gh pr merge`、`update-branch`、tag push、PR/Issue 评论、chat webhook |
| `SendMessage` / `send_message_to_thread` | 核对另一个已登记的实际窗口或宿主绑定 ID，不按共享工作目录冒认 | 非值守窗口发给另一个窗口的横向消息；给值守交单/回执及会话内部地址照旧放行 |

路由闸只在**有人 `agent-on oncall claim` 且登记还活着**（默认 90 分钟内有心跳）时生效，无人在班或登记过期一律 fail-open；协议全文见 [`kit/babysit/ROUTING.md`](../kit/babysit/ROUTING.md)。
**MCP 外发工具不在这两个 matcher 内**（Telegram / Slack MCP 等按各自工具名注册）——要机械兜住，自己加一条 matcher 指向同一 shim；否则那条通道只有纪律层兜着。

## 观察 hook（未发布扩展，不授权）

共用 capture 记录 SessionStart/UserPromptSubmit/PreToolUse/PostToolUse/Stop/SessionEnd；工具观察 matcher 为 `.*`，只存工具名，不存工具输入/输出。原授权 guard 注册保持独立，记录失败不拦工作，也不代替 guard。

`activity enable` 仅启本项目独立观察，不启动巡逻/模型或采其他项目。SessionStart/提交时回读短状态供原窗口共读；其他窗口可 `activity status` 按需读同一私有台账，不是向其他模型发消息或授权派工。真实接线需升级实际 executor 与插件、按宿主流程信任，并验证现场事件；源码和构造 payload 测试不等于真实宿主已接通。用法与覆盖限制见 [面板执行书](../kit/live-control-plane.md)。

## 路径约定（可移植）

- **脚本定位（A）**：hook 命令用 `${CLAUDE_PLUGIN_ROOT}`；Codex plugin 兼容该变量并同时提供 `${PLUGIN_ROOT}`。
- **边界判定（B）**：`agent-on doctor` / `cli` 路径解析——`AGENT_ON_ROOT` → `~/.config/agent-on/config.json` → lock「本地路径」。**无 Chao 默认路径**；未登记 B 时 guard fail-open。
- **个人 scope**：可选；plugin hook 是默认。Codex 非 managed 个人 hook 首次运行可能须在 `/hooks` 信任。
- **低开销**：授权 guard 对普通非 git/只读 Git 命令快速返回；实际提交/推送检查留在原生 Git hooks。活动观察另有显式开关、私有原子回执与数量/大小上限，不调用模型。

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
4. 路由闸三态：无人在班 → 任何 payload exit 0；值守在班 → 值守窗口 exit 0、功能窗口 `gh pr merge` exit 2 且 stderr 含「转投」模板与在班地址。

## 回滚

- Claude/Codex：disable/uninstall agent-on plugin；个人 scope 条目不受影响。
