# Snapshot — 独立窗口活动回传

> 职责边界：记录用户要求把真实任务与窗口状态接到面板后的实施范围；不是完整跨平台接线验收或发布凭证。

## 已确认的方向

2026-10-03 用户指出仅看到进程不知道在做什么的 dashboard 无法帮助调度，并在解释共用状态账本方案后说「好的，请继续」。本轮在 Agent-On 工作仓开发，保留已有未发布改动，不改 Dartify 或全局安装。

- 活动观察与巡逻开窗独立，显式启用只采状态，不启动模型、巡逻、调度或清理。
- 复用 Git common-dir 的私有台账，按实际宿主与窗口 ID 记录；没有型号依赖，没有通用工作流运行时。
- 原生事件、自述状态、任务阶段、进程存在与验证结果分开。Stop 是本轮结束，不释放任务；陈旧状态是未知，不是退出或回收授权。
- 未登记任务的窗口可显示需求摘要，但不得从摘要或分支名伪造受管任务、文件范围或授权。显式绑定现有任务才显示任务范围。
- 原生插件已支持的 SessionStart/UserPromptSubmit/PreToolUse/PostToolUse/Stop/SessionEnd 走同一 capture 入口；宿主安装、信任及事件字段必须现场验证。实际活动接线不能靠构造 hook 输入冒称已完成。
- Codex/Claude 原生事件与 Grok/Hermes 等显式回传可以共读；未装的其他平台 adapter 继续明报缺口。

## 本地实现与验证证据

- 新增独立 `activity enable/disable/status/bind/report`。观察配置、窗口回执与读回均复用 common-dir；原生 capture 可回传需求摘要、工具名称和本轮状态，但不采工具输入输出、不改任务阶段、不决定模型。
- 17 项活动边界测试涵盖独立开关、任务绑定与重绑定、阶段分离、同名跨宿主窗口隔离、近期 turn 迟到/重复、隐私限长、陈旧/异常时间、损坏/软链、显式阻塞条件、linked worktree 共读及项目/系统同源。最初 8 项先红：旧 CLI 无 `activity` 子命令；实现后最终 17 项通过。
- 最终 `cargo test --manifest-path cli/Cargo.toml --quiet` 实际输出累计 **418 passed，0 failed**。首轮受限环境的两项 HTTP 监听测试因权限拒绝失败；按本机监听权限重跑全部通过，不把环境失败隐藏为通过。
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings`、格式检查和 `git diff --check` 实际成功；文档闸输出 `DOC-GATE: PASS`（229 份已跟踪 Markdown；本快照尚未跟踪，不计入该数）。
- 最新 debug 预览重启后，`dashboard_browser.cjs` 实际输出 `browser: PASS`、6 棵 worktree、7 条含历史任务、动态分栏通过、无页面错误；`system_dashboard_browser.cjs` 实际输出 `browser: PASS`、30 个项目、过滤/动态状态/详情路由/XSS/移动端通过、无页面错误。浏览器仿真只拦截页面响应，不写用户真实台账。
- 本仓真实窗口 `codex / 01a00643-b59d-79e2-b5e0-876ef65f73f7` 绑定任务 `activity-channel-20261003`，真实 API 显示 `source: explicit-report`、`verified: false`。已现场观察超过 120 秒后变成 `unknown`；工作回传恢复后显示 1 个已观察执行窗口，同时 `running_windows: null`，不推断全量在线数。
- Dartify 原详情页只读复核：`activity.enabled: false`、窗口未知、任务 0；没有替它启用、登记假任务或补造状态。

## 2026-10-03 收尾时的未完成与授权边界

2026-10-04 用户随后 approved，本机执行面升级与真实 Codex 工具事件已有新证据；当前安装状态与 Claude 重载缺口见 [接续记录](2026-10-04-native-activity-host-upgrade.md)。以下保留首次开发收尾的历史边界。

- 本轮只验证本地源码、debug 预览和本窗口明确自述。没有升级全局实际 executor、安装插件缓存、改变宿主信任或重启宿主；**原生 hook 自动活动流仍未现场验收**。构造事件测试不是实际接线证据。
- Claude/Codex 的受支持事件入口已实现。Grok/Hermes 等可用显式回传共读，但没有声称所有平台已装好自动事件 adapter，也没有云端/跨机器消息总线。
- 迟到去重仅覆盖有 turn ID 的近期 8 轮；缺字段事件按到达顺序观察，不保证任意历史回执排序。
- 本仓单独启用活动观察；巡逻与清道夫仍未启用。关闭观察用 `agent-on activity disable`（本轮使用 `cli/target/debug/agent-on`），保留历史，不清理 worktree。
- 下一步实际安装会涉及此前同批未发布源码，不仅本模块；需明确确认范围、保留回退、按宿主信任流程验收。没有 commit、打 tag 或 push，推荐 pin 仍为 `v0.28.0`。
