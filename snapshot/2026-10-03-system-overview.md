# 本机系统总览：跨项目、跨 Agent 的观察入口

> 职责边界：记录 2026-10-03 用户明确要求跨项目系统 dashboard 后的本地实现、约束与证据；不是发行说明、云端/跨机能力声明或其他项目接管授权。运行说明唯一入口是 kit/live-control-plane.md。

## 用户需求与边界

用户要求现有项目级面板升为系统级，所有现在项目和 Hermes / Grok / Antigravity / Codex / Claude 同屏显示状态与报错。采用本机 Projects 目录有界只读发现与既有记录投影，保留原项目详情；不另建通用编排运行时、不改 CHARTER、不重复派工或合并检查。没有使用 Superpowers。

五种平台支持统一观察/自述，但观察种类不是执行权限。未接入、回执陈旧、只发现进程、项目未关联、源记录损坏必须分别明示，不把进程存在或 done 自述说成任务已完成。通用回执单独于派工 Host 和实际 Session Identity，不因支持 Antigravity 展示而允许未实现的派工 adapter。

只在 Agent-On 本仓改代码与自己的任务台账；其他项目不写文件、不安装 hook、不开启巡逻。项目级 oncall / 合并 / 对外授权保持原规则，不产生系统值守新角色。未承接旧 goal 的全项验收，也未授权这批发版或全局 CLI 更新。

## 落地

- `dashboard --system --root … --project …`：共用一次发现与每五秒投影；发现每六十秒、明确上限与排除目录；linked worktree / 根内 Git 别名按 common-dir 去重。项目无记录也显示，任务 ID 按项目命名空间隔离。
- 系统首页优先活动项目，默认八卡，可展开全部；项目/五种 Agent/关键词筛选；跨项目任务分栏、集中错误、分源信号、大屏与手机布局；未知不报在线。
- 保留 `/projects/<opaque-id>` 项目页面与 `/api/projects/<opaque-id>`；不给客户端输入任意路径。所有 JSON 同源 header、GET-only、localhost、textContent 与原 CSP 保持。
- `agent report` 提供 project/host/id 原子自述更新，支持状态/错误心跳，明显秘密遮盖；不改变任务状态、不授实际身份或任何 merge/dispatch 权。
- 不读私有对话数据库、命令参数或认证；进程信息不可读不冒称全部离线。Unix 可识别进程＋cwd，Windows 进程发现暂不可用。整个系统首页不对每个项目运行完整巡逻或联网取证。

## 本机观察（时点事实）

只读扫描 `/Users/chao/Projects`：30 个按 common-dir 去重的项目。IPONews 是 AgentForce 别名；otc-quote-ni-ki / otc-quote-pink 为同 common-dir 下关联 checkout，不当三个独立项目。归档和来源不明目录没有被擅自接管。

现场进程信号五类均存在，某次采样共 76 条，包括辅助进程；它不是 76 个在线 Agent。Dartify 有 Claude / Hermes / Grok 的 cwd 关联；Codex / Antigravity 的部分桌面进程没有已知项目 cwd，保留未关联。现场除 Agent-On 外的项目未见 control 台账，因此任务与原生报错仍未知，不能因总览已显示项目声称采集闭环完成。

初次现场扫描发现进程元数据略超过 256 KiB，原上限如实报 unavailable；在仍有界的情况下调整为进程专用 1 MiB 后真实读取恢复。文件记录仍各限 256 KiB。没有为展示五种模型写入伪造现场任务。

## 验证

- `cargo test --manifest-path cli/Cargo.toml --quiet`：完整 **382 passed / 0 failed**（224 unit + 158 integration），其中新增系统集成 14 项，覆盖只读发现/未接入目录、项目同名任务隔离、损坏来源、linked worktree/别名、排除与显式添加、五类自述、原任务不变、非法输入、秘密过滤/文件限制、HTTP 权限与五秒现场刷新。初次完整回归真实红灯：临时目录的 /var → /private 路径映射漏识别根内别名；修根路径 canonicalize 后十四项与完整套件转绿。
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings`、`cargo fmt --manifest-path cli/Cargo.toml --check`、`git diff --check`：exit 0。
- `python3 .github/scripts/check_docs.py`：`DOC-GATE: PASS（229 份 markdown）`，exit 0；该脚本按 Git 跟踪清单，不把新增未跟踪 snapshot 算成已被发布文档闸覆盖。
- `node cli/tests/system_dashboard_browser.cjs http://127.0.0.1:8766/ /Users/chao/.agents/skills/gstack/node_modules/playwright /tmp/agent-on-system-overview.png`：`browser=PASS, projects=30, hosts=[codex,claude,hermes,grok,antigravity], filters=PASS, dynamic_state=PASS, detail_route=PASS, XSS=PASS, mobile=PASS, console_errors=[]`。浏览器模拟响应仅用于测试，不写现场假任务。
- `node cli/tests/dashboard_browser.cjs http://127.0.0.1:8767/ /Users/chao/.agents/skills/gstack/node_modules/playwright /tmp/agent-on-project-regression.png`：原项目模式 `browser=PASS, worktrees=6, tasks=5, mixed_host_fixture=PASS, dynamic_lanes=PASS, console_errors=[]`。
- browse 真实系统页面 `goto` 返回 200；操作快照含五种 Agent 与展开 30 项目，console 无错误；已实际打开查看本地截图，非只检源码。截图布局优先活动项目、其余折叠，工作区与原项目详情保持可达。

临时验收服务使用 8766/8767，验完关闭；8765 原项目预览由本轮已构建的系统模式替换。仅替换本窗口自己持有的服务，未终止任何平台 Agent。预览用源码开发二进制，不代表全局 executor 或插件升级。未 commit/tag/push；推荐发行 pin 不变。

## 未完成 / 未扩张

Hermes / Grok / Antigravity 自动原生事件接线、任意宿主所有窗口枚举、全部任务/错误覆盖、真实跨模型协作、跨机云端、自动执行恢复、常驻开机服务、全局安装与发布不在本轮完成声明中。五类工具可调用通用回执，仍需要各自真实 harness 的接线和现场验证。

## 同日现场反馈：进行中为空不等于没有窗口在工作

用户指出当前 Agent-On 正在开发，进行中却显示「暂无任务」。14:46 香港时间附近的实际观察：四个未释放任务均 reported，项目 capture disabled，宿主 Codex 原生工具读取本窗口确为 active；面板五秒请求仍有新时间，故不是 HTTP 刷新停止。根因是任务阶段与当前窗口活动被混用，且 capture 在巡逻关闭时提前返回，不产生新的活动回执。现有 ready 宿主回执本身也不区分 active/idle。

尝试按本机随宿主提供的官方只读接口读取原生状态，实际本地连接在 tools/list 即断开；官方 SDK 的独立同环境探针也报 pipe closed。没有把合成传输测试当作真实接线通过；原生传输实验已撤下，不留一个永远失败的五秒后台调用，不查私有数据库或绕开宿主授权。

本次只落地诚实的状态展示修复：

- 系统与项目 API 单列 activity，未接入实时源则 available=false/state=unknown/running_windows=null，而非零。
- 两种页面都常显窗口活动未知，进行中标「已登记」；任务空栏和来源损坏不推出没有活跃窗口。
- reported 卡片统一显示等待核验；旧「正在…」进度移到历史详情。旧任务不批量改 working、不自动释放。
- 当前反馈确实触发新一轮修复，因此作者显式重开 system-overview 的进度，并将接线待批准状态如实登记；这不是自动采集完成的证据。

回归新增一个真实 Git 场景：即使同时有 working/reported 任务和新鲜 ready 会话，也不伪报原生窗口活动。该用例先红（缺 activity.available 字段），后绿。最终完整 `cargo test --manifest-path cli/Cargo.toml --quiet` **383 passed / 0 failed**（224 unit +159 integration）；第一次普通沙箱运行的两项 HTTP 用例被本机监听权限拒绝，获准以本地临时端口复跑后通过，不隐藏环境失败。clippy all-targets -D warnings、fmt --check、diff --check 均 exit 0；DOC-GATE PASS（229 已跟踪 markdown，新 snapshot 未计入）。

两种真实浏览器脚本均 PASS，系统页 projects=30、五类 host、过滤/动态分栏/详情/XSS/mobile 全过，console_errors=[]；原项目页 worktrees=6/tasks=5、mixed_host_fixture/dynamic_lanes 通过。合成响应只用于浏览器回归，不写现场假任务。系统空栏测试另发现损坏来源提示遮掉活动未知，修后同验收转绿；测试脚本自身的异步读取错误也修正后复跑。实际查看截图 `/tmp/agent-on-state-coverage.png`，并核真实 API 明报 activity unknown。只重启本窗口自己的 8765 预览，临时 8767 验收服务关闭。

未完成：独立于巡逻的活动 hook、实际 executor/插件升级与信任、真实活动/结束/断线/恢复闭环。已向用户请求本机升级与接线授权，未得到回复前不安装；授权不能代替功能实现和现场验收。没有 commit/tag/push，也没有改其他项目、全局配置、合并政策或 CHARTER。任务展示纠偏不算自动状态问题全部解决。
