# 本机执行控制面

> 职责边界：把本机项目、已有任务、会话回执、可识别 Agent 进程、Git/worktree 与 landing 快照投影为文本简报、JSON 和可选网页；不是通用执行器，不拥有任务、合流或删除权限。任务/回收按 patrol-control-plane，PR 按 landing 与在班值守政策执行。新版文本简报、系统总览与通用回执是未发布开发功能，须使用当前源码构建，不能把旧安装当作已具备。

用户在原窗口继续说目标，交付过程走 [交付入口](delivery-workflow.md)。`agent-on status` 直接读目标、窗口登记、最近回报、依赖、卡点、待验收和下一步；最多展示 20 项未释放任务，其余明确提示读取完整 JSON。最后回报人不冒称负责人，缺绑定、损坏与陈旧来源明确显示。命令不写台账、不启服务、不联网。

项目事实由 `project_status` 统一收集，文本与网页消费同一份数据；`status --json` 保持既有结构。旧 `status --serve` / `--system` 参数仍兼容，网页按需使用：

```bash
agent-on dashboard --serve --refresh-landing
agent-on status --json
# 指向另一个项目；linked worktree 自动归一到同一 primary/common git dir
agent-on dashboard --repo <项目路径> --serve --port 8765
```

按命令返回的 localhost 地址打开浏览器或桌面宿主侧栏。无需安装另一个 desktop。服务运行期间每 5 秒观察本机状态；`--refresh-landing` 才开启每 60 秒一次的联网取证，同一服务多个客户端共用一个观察器。Ctrl-C 退出后页面标断开、保留最后观察；不存在自动开机后台服务。

首页显示任务、执行窗口、worktree 库存、共享路径风险、PR 队列及值守地址。来源出错按区块显示，任务账损坏不会让旧树消失；没有任务记录就显示覆盖缺口，不能从 worktree 分支名猜出任务已完成。

本地开发看板将同项目任务分为「待安排 / 进行中 / 卡住了 / 待验收」，已结项默认折叠；Codex、Claude 等共用同一页，可按工具或关键词筛选。大屏模式只突出任务卡片，详细目标、路径、回执与工作区点开再看。它仍是同一份台账的只读展示，不新增派工或合入权限。

**任务阶段不是窗口活动。** 四栏读取已登记任务；所有任务 reported 时，进行中栏可以为空，但窗口仍可能在工作。未发布源码增加独立「窗口在做什么」区，读取按实际宿主/窗口关联的活动回执；未绑定任务只显示需求观察，不从目标摘要或分支名造任务/文件范围。reported 仍待核验；Stop、本轮退出或窗口关闭不验收、不释放。

### 独立活动观察（未发布）

```bash
agent-on activity enable                 # 只在本项目开启观察，不开巡逻或模型窗口
agent-on activity bind <已有任务编号>    # 本实际窗口显式关联任务，不授所有权
agent-on activity status                 # 其他窗口按需共读同一台账
agent-on activity report --state blocked --note '等待宿主审批'
agent-on activity report --state working --note '审批已到，继续实现'
agent-on activity disable                # 停采新活动，旧回执保留为历史
```

观察开关另存 common-dir 下 `activity-config.json`，与巡逻 `config.enabled` 独立，旧 executor 回写巡逻配置不会删掉观察开关；旧安装仍不具备此功能。活动记录落私有 `activity/`；不改任务真相、合流权、模型或回收许可。`bind/report` 核实际窗口身份，不按工作目录冒认；型号不是必填字段。改绑任务先将窗口活动重置为未知，旧任务的执行/阻塞不转移到新任务；已结项任务不可新绑定，历史绑定不等于任务复活。

共用 capture 接收既有插件事件：SessionStart 只表示启动/恢复，活动未知；UserPromptSubmit、PreToolUse/PostToolUse 表示收到执行/工具信号；Stop 是 turn-finished，SessionEnd 是 closed。只采需求摘要、工具名称和事件元数据，不采工具输入/输出、完整对话或私人数据库。带 turn_id 的最近八轮重复/迟到事件不会覆盖当前一轮；缺 turn_id 的宿主只能按到达顺序观察，不保证识别任意延迟回执。原生事件不自动推断语义阻塞，等待/阻塞由执行者明确报告；阻塞不被工具流量或 Stop 清除，明确 working 才撤回自述条件。

SessionStart/UserPromptSubmit 回读最多五个窗口的简短状态，作为不可信状态数据注入原会话；不是横向派工、完整对话共享或权限。其他窗口可按需运行 activity status，无需模型两两持续聊天。值守通信路由保持不变。

API `activity` 分开给 enabled、available、windows、errors、stale_after_seconds；缺数据、损坏、异常时间、超两分钟未回传或关闭观察时显示 unknown。新鲜回执也只标 partial，`running_windows` 始终 null；`observed_working_windows` 只计成功读取的 working 状态记录，错误时 null，不是全部在线窗口。自述标 explicit-report/未核验，真实 hook 标 native-hook，进程存在和 legacy ready 不升级成活动证明。等待/阻塞自述与工具事件各有时间，超时不因后台流量变成新鲜判断。单记录最多 256 KiB、窗口最多 256，达到上限/软链/特殊文件明确报不可用。陈旧或关闭窗口不成为删 checkout 的依据。

**源码接收能力不等于现场已接线。** 初次开发只用真实当前窗口的明确回传 dogfood，临时仓构造事件不是现场证明。2026-10-04 用户批准本机 executor/两边插件升级后，本 Codex 窗口已真实回传 native-hook/PostToolUse，项目与系统 API 共读；Claude 待原窗口重载，完整事件生命周期仍需验收。新增/变更 hook 按宿主流程由用户确认信任，不直接写信任 hash。详见 [本机安装证据](../snapshot/2026-10-04-native-activity-host-upgrade.md)。Dartify 与其他项目没有擅自启用采集；Hermes/Grok/Antigravity 自动 adapter 仍未接齐。

`task add … --title '短标题'` 和 `task progress … --title '短标题'` 可设置 1–80 字展示名，不改完整目标和范围。新执行器在显式登记、进度及未释放任务的结果回传时记录实际宿主身份为 `last_reporter`。有派工会话时展示关联执行工具；只有手动任务时标「最近登记 / 回传」，不是当前在线或独占负责的证明。旧任务缺来源显示「未关联工具」，不从编号猜模型。旧执行器仍能读取记录，但再次写入可能丢掉新增展示字段；跨工具使用本轮新字段前需各自升级实际执行器，未发布源码不能当作已安装版本。自动捕获仍须原有显式启用和正常 hooks 信任，不能为显示 Claude 卡片伪造任务或会话。

三种时间分开：本机观察、宿主最后回执、PR 最后联网取证。超过两分钟的 PR 快照提示陈旧；Stop、进程退出、executor-file 的结果都不是验证通过，也不释放任务。释放仍需作者给实际证据；未确认会话沿现行保护规则保留。窗口心跳陈旧不等于可以删 checkout。

未发布的用户指令型阶段交接会在下一任务保存可选 `handoff`：来源任务/结果、审阅的结果时间、目标摘要/范围/来源树与 HEAD、固定摘要和确认依据。项目 JSON 与系统的项目详情 JSON 同台账共读；系统跨项目任务摘要暂不投影 handoff。创建后源报告更新不改写快照，不冒充源任务已验收/释放或执行者在线。当前 HTML 没有专用阶段流程图、批准按钮或自动启动接口；交接入口仍是现有 dispatch，按 patrol-control-plane 的明确用户指令执行。

JSON `schema_version=1`，来源区块各带 `available/data` 或 `available/error`；可被后续原生壳或执行器适配复用。`daily_gc` 分开暴露登记、活动与最后运行的判断；定时登记存在但运行失败会进入关注提示，不因“active”给绿。这里不另存任务数据库，投影可以丢失重建。一次 collect 不联网、不写项目文件；联网刷新只写既有 common-dir 缓存。

HTTP 只监听 127.0.0.1，固定两个 GET 路径。状态 API 要求同源 Host/Origin 和 `X-Agent-On: dashboard`，不开放跨域，不提供命令执行、派工、合并或删除 API；浏览器只以 textContent 渲染任务文本。面板里有本机私有目标和路径，不将服务暴露到公网。

## 系统总览（本机跨项目，未发布）

```bash
# 默认只在 ~/Projects 有界发现项目，不扫描整个 home
agent-on dashboard --system --serve
# 可重复 root；被排除或 Git 项目内的嵌套项目用 project 显式加入
agent-on dashboard --system --root /path/to/projects --root /other/projects --project /outside/repo --serve
agent-on dashboard --system --root /path/to/projects --json
```

同一页展示所有已发现项目，按项目、Codex / Claude / Hermes / Grok / Antigravity 或关键词筛选；活动项目优先，默认八张摘要卡，其余一键展开。跨项目任务四栏与集中错误读取已有记录，项目编号＋任务编号作复合键，不把不同项目的同名任务混在一起。单项目详情保留，顶部可返回系统总览。单个记录损坏只标其来源不可读，其他项目继续显示；统计只计成功读取的记录，不证明未登记来源没有任务或错误。

来源严格分开：既有任务账是任务真相；宿主回执显示最后观察与是否陈旧；通用回执标自述/未核验；进程与辅助进程仅证明存在，不证明在工作。五类工具都可使用通用回执，但只有已接入的真实 hook 才自动产生宿主回执。不会为显示某个模型伪造任务。桌面常驻进程可能没有可关联工作目录，此时标「项目未关联」，不从窗口标题或对话猜项目。

状态登记按项目、执行工具、实际窗口 ID 和任务 ID 关联，**不要求型号**，也不查询模型目录。模型即使每轮不同，不改变任务身份、不得导致重复登记或丢失进度。型号以后可作为有来源的可选展示信息；缺失就是未知，不能拿型号作状态、身份或授权的必填条件。规划、执行与验收使用谁、哪款模型由用户决定；Agent-On 仅执行明确调度指令，不从任务状态、失败或目录推荐推导改派授权。未指定型号沿用宿主当次设置；显式选择原样保留，不自动换号或档位。可选只读诊断按 patrol-control-plane，不参与启动选型。

项目发现每 60 秒；任务/回执/进程每 5 秒。本机 Unix 进程发现只读取可执行文件名称和已识别进程的工作目录，不读命令参数、会话 transcript、私人平台数据库、认证或密钥文件。macOS 用本机进程元数据，Linux 用可见名称与 /proc cwd；封装在通用 node/python 的 Agent 可能无法识别。进程观察受系统权限或读取限制影响时明确 unavailable，不能当作全部离线。Windows 进程发现尚未实现，已有项目台账和通用回执仍可读。

扫描深度最多三层、目录 512 个、项目 128 个，每类账本最多 256 条、单记录 256 KiB，达到上限明报覆盖缺口。Git 项目及 linked worktree 按 common dir 归一；扫描根内的直接 Git 项目别名可归一显示，其他软链不递归。隐藏、`_` 开头的归档/参考目录、生成与依赖目录排除；未识别 manifest 的一级目录也保留为未接入。发现 Git 项目后不继续递归其内部，嵌套独立项目须显式添加。只读发现不在其他项目启用巡逻、不装 hook、不创建记录目录。

系统 API `schema_version=2`，`scope=system`；项目详情保持 schema 1。一个本地服务共享一个总览观察器，不为每个浏览器重扫；系统模式不对所有仓重复运行完整 Git 检查或联网刷新。项目详情按需观察，短期缓存共用。系统模式与 `--refresh-landing` 冲突；要更新 PR 证据仍在目标项目运行现有 refresh。项目合并权不因系统页面集中展示而集中给新角色。

系统服务另有固定形状的 `/projects/<opaque-id>` 和 `/api/projects/<opaque-id>`，只接受已发现项目 ID，不接受客户端提供 repo 路径；所有 JSON API 沿用同源与自定义 header 保护，没有写 API。

### 通用状态回执（不是宿主身份或授权）

```bash
agent-on agent report --host antigravity --id window-one --repo /path/to/repo --state working --note '正在修登录界面'
# 关联现有任务可额外写 --task TASK_ID，不修改原任务
agent-on agent report --host hermes --id window-one --repo /path/to/repo --state blocked --note '账户拒绝，等待处理'
```

host 支持 codex / claude / hermes / grok / antigravity；状态 working / waiting / blocked / done。同一项目、host、id 的再次报告原子更新同一私有回执，可作为 harness 的心跳/完成/失败通知入口；两分钟没有更新标陈旧。回执保存在 common-dir 的 `agent-on/control/agents/`，不新增全局任务数据库。仅存短注释并遮盖明显凭据（不能保证发现全部秘密），不要提交密钥或完整对话。显式 task 必须已有，但 done 不验收、不释放、不清作者阻塞。自述不授实际宿主身份、值守权、派工权或合并权。

当前未自动安装其他项目的回执接线，未完成 Hermes / Grok / Antigravity 全部原生事件采集；通用命令需由实际 harness 调用。Antigravity 的只读观察/通用回执不等于新增可执行派工 adapter。本版不包含跨机/云端总线、任意宿主窗口枚举或自动恢复执行。

Agent-On 的持续资产是跨执行器可迁移的任务与证据、项目决策、合流审计、恢复依据和经过真实任务验证的经验。模型仍负责理解和执行；它们提升时可以继续删减流程。任何“不可替代”都需要真实使用收益支持，不能靠面板或角色命名保证。

实现：`cli/src/dashboard.rs` / `dashboard.html`、`system_dashboard.rs` / `system_dashboard.html`。浏览器实测脚本为 `cli/tests/dashboard_browser.cjs` / `system_dashboard_browser.cjs`；系统回归为 `cli/tests/system_overview.rs`。跨机状态、自动恢复执行与真实 Hermes 模型闭环未声明完成；现场证据见 [系统总览开发记录](../snapshot/2026-10-03-system-overview.md)。
