# 本机执行控制面

> 职责边界：把已有任务、会话回执、Git/worktree 与 landing 快照投影为同一 JSON 和浏览器面板；不拥有任务、合流或删除权限。任务/回收按 patrol-control-plane，PR 按 landing 与在班值守政策执行。

用户在原窗口继续说目标。看状态用 `agent-on dashboard`（别名 `status`），打开实时视图用：

```bash
agent-on dashboard --serve --refresh-landing
agent-on dashboard --json
# 指向另一个项目；linked worktree 自动归一到同一 primary/common git dir
agent-on dashboard --repo <项目路径> --serve --port 8765
```

按命令返回的 localhost 地址打开浏览器或桌面宿主侧栏。无需安装另一个 desktop。服务运行期间每 5 秒观察本机状态；`--refresh-landing` 才开启每 60 秒一次的联网取证，同一服务多个客户端共用一个观察器。Ctrl-C 退出后页面标断开、保留最后观察；不存在自动开机后台服务。

首页显示任务、执行窗口、worktree 库存、共享路径风险、PR 队列及值守地址。来源出错按区块显示，任务账损坏不会让旧树消失；没有任务记录就显示覆盖缺口，不能从 worktree 分支名猜出任务已完成。

三种时间分开：本机观察、宿主最后回执、PR 最后联网取证。超过两分钟的 PR 快照提示陈旧；Stop、进程退出、executor-file 的结果都不是验证通过，也不释放任务。释放仍需作者给实际证据；未确认会话沿现行保护规则保留。窗口心跳陈旧不等于可以删 checkout。

JSON `schema_version=1`，来源区块各带 `available/data` 或 `available/error`；可被后续原生壳或执行器适配复用。`daily_gc` 分开暴露登记、活动与最后运行的判断；定时登记存在但运行失败会进入关注提示，不因“active”给绿。这里不另存任务数据库，投影可以丢失重建。一次 collect 不联网、不写项目文件；联网刷新只写既有 common-dir 缓存。

HTTP 只监听 127.0.0.1，固定两个 GET 路径。状态 API 要求同源 Host/Origin 和 `X-Agent-On: dashboard`，不开放跨域，不提供命令执行、派工、合并或删除 API；浏览器只以 textContent 渲染任务文本。面板里有本机私有目标和路径，不将服务暴露到公网。

Agent-On 的持续资产是跨执行器可迁移的任务与证据、项目决策、合流审计、恢复依据和经过真实任务验证的经验。模型仍负责理解和执行；它们提升时可以继续删减流程。任何“不可替代”都需要真实使用收益支持，不能靠面板或角色命名保证。

实现：`cli/src/dashboard.rs` / `dashboard.html`。浏览器实测脚本为 `cli/tests/dashboard_browser.cjs`；单项目、文件级冲突、已有回执覆盖是本版边界。跨项目首页、跨机状态、自动恢复执行与真实 Hermes 模型闭环未在本版声明完成。
