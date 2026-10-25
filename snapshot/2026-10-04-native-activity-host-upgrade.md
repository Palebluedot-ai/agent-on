# Snapshot — 2026-10-04 · 本机活动执行面升级

> 职责边界：记录用户 approved 后的本机安装与真实事件取证，不是新版本发布、所有平台接线、其他项目采集或自动任务验收的证明。

## 授权与范围

用户明确批准上轮提出的“升级本机执行器和插件，先只接 Agent-On，按宿主流程验证 Claude/Codex 自动回传”。沿用原窗口，不启动其他模型，不选择型号，不重启用户聊天，不修改 Dartify。全局执行器及两个既有插件会更新；活动观察仍仅在 Agent-On 项目启用，巡逻和清道夫未启用。

此前本地开发的 418 项测试及两个面板的浏览器证据见 [实现记录](2026-10-03-activity-channel.md)。本次安装同一份 CLI 源码，并非重新跑完这些测试或发布新 tag。

## 备份与执行

- 动手前已给回退命令：`bash /Users/chao/.local/share/agent-on/host-backups/20261004-activity/rollback.sh`。
- 备份目录为 0700；旧全局/仓内执行器、两边精确版本缓存均已备份，缓存用 APFS 写时复制。Codex 配置与个人 hooks、Claude 插件登记只作私有备份。回退脚本恢复执行器与缓存、保留被替换下来的缓存；不覆盖后来配置或手写信任记录。脚本语法检查通过，未在活环境强制演练回退。
- `cargo build --manifest-path cli/Cargo.toml --release --offline --locked` 实际成功；`cargo install --path cli --locked --offline --force` 实际替换全局执行器。
- `codex plugin add agent-on@agent-on --json` 实际返回安装路径 `/Users/chao/.codex/plugins/cache/agent-on/agent-on/0.28.0`；已从现有本地 marketplace 更新这一插件。
- `claude plugin update agent-on@agent-on --scope user` 返回同版本已最新，但源码指纹和 hooks 校验仍为旧缓存，不能据此宣称升级完成。整仓缓存同步被权限检查拒绝，未执行；改为核对差异后只同步 8 个变动的 CLI 源文件、hook、入口与 4 份必要执行书，以及执行器。没有复制项目素材或用户配置。
- 原 localhost 8765 debug 服务经精确 PID 核对后停止，改由 `/Users/chao/.cargo/bin/agent-on dashboard --system --root /Users/chao/Projects --serve --port 8765` 提供原地址只读预览。不是开新模型窗口。

## 实际核验

- 全局、仓内 release、Codex 缓存、Claude 缓存四个执行器均返回 `source_fingerprint: fnv1a64:9fa66d9b5bc58c27`、`source_dirty: true`、`release_tag: null`；这是基于 v0.28.0 的未发布本地源码，不冒充新发布。
- 两个缓存的 `hooks/hooks.json` 与源码 SHA256 都为 `5068771c046265996ee7e22ef76be9c4a24f5eaf5fd8b6aaac2a2b8b312c8cd2`。
- `agent-on doctor` 最终输出“执行面与 READ_ROOT 一致”，Git executor 同源。Claude 插件 manifest 验证成功，登记仍 enabled；Codex 同一插件仍 installed/enabled。
- Codex 配置及个人 hooks 与备份 SHA256 完全相同，未更改模型或信任标记。
- **真实 Codex 事件**：未输入构造 capture payload，本窗口正常工具调用后，CLI 及面板 API 同时读到 `host: codex`、`session_id: 01a00643-b59d-79e2-b5e0-876ef65f73f7`、`source: native-hook`、`event: PostToolUse`、`stale: false`。现场回执时间 `2026-10-03T16:39:17+00:00` 即香港 2026-10-04 00:39；证实该工具事件自动回传链，不推断所有事件与所有窗口都已覆盖。
- Dartify 仍 `activity.enabled: false`、窗口未知、回执 0；未擅自接入。Agent-On 巡逻及清道夫仍 disabled。

## 需要用户操作／未完成

1. Claude 原 Agent-On 窗口执行 `/reload-plugins`，再进行一次正常需求/工具调用；没有真实 Claude 回执前不报接通。
2. Codex 若提示新增/变更 hook 待审，按宿主原生流程检查 Agent-On 定义并信任。CLI 入口为 `/hooks`；不直接写 trust hash，不使用绕过信任的选项。安装不等于信任，见 [OpenAI hook 文档](https://learn.chatgpt.com/docs/hooks)。已有真实 PostToolUse 回传不代替新定义的信任或其他事件验收。
3. 新一轮提交、停止、恢复的自动共读，以及 Claude 原生事件仍需现场续验；本轮未制造 Stop/SessionStart 输入来补完成。

尚未实现的 Grok/Hermes/Antigravity 自动事件 adapter、跨机运输、无人值守阶段流水线不因本次安装变成已完成。没有 commit、tag、push 或清理 worktree；推荐 pin 仍为 v0.28.0。
