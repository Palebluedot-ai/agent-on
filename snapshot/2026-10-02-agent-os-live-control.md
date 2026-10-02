# 2026-10-02 · 从八月设想到可见的本机控制面

> 职责边界：记录本轮需求、现状、实现范围与实测；现行执行入口仍为 kit。不是新的合并、回收或全局配置授权。

用户要求重读八月对话、pull 仓库、补齐未完成的 worktree/并行 PR 管理，并评估 desktop、实时返回与 Hermes 接入。`git pull --ff-only origin main` 实报 `Already up to date.`，基线 `v0.26.0 / 0c6e5dc`。已有及本轮期间出现的其他会话未跟踪素材保持原样，不纳入本批。

## 现状与交付范围

| 八月需求 | 当前证据 | 本轮处理 |
|---|---|---|
| commit/push 的机械保护 | 本机 hooks status healthy，两个 Git hooks 已启用 | 保留现行操作范围判据 |
| PR 统一取证与排序 | landing 已实现、v0.13.0 已合入 | 修同 SHA 下 CI/审批变化被冻结；复用文件证据，批量更新易变状态 |
| 多 worktree 生命周期 | landing/gc/janitor 已实现 | 面板统一列出现存树与保护原因，不擅自回收旧树 |
| 任务、窗口和真实返回 | patrol/task/dispatch 已实现，但本仓 config.enabled=false、任务/会话回执为空 | 加本机实时投影视图；没有捕获时明确显示覆盖缺口 |
| desktop 面板 | 项目静态 dashboard 模板不是本机执行面 | 先做可在浏览器/Codex 侧栏运行的按需面板；复用同一 JSON，之后才评估原生壳 |
| Hermes | 本机 v0.21.3，`hermes chat --help` 有 --query、--in、--resume | 加薄 CLI 适配；不代装、不修改 Hermes hooks/认证，缺少 host 回执时不冒充 ready |
| 装机可用 | Codex 实际支持 plugin add；setup 仍 install 且吞失败 | 修真实命令、先构建再装插件、失败回执与 pin 失败处理 |

## 产品结构

执行器负责模型、工具调用、认证和宿主会话。Agent-On 负责跨执行器共享的任务/依赖/证据、Git 与 PR 事实、合流责任、回收恢复和项目经验回流。面板只投影已有事实，不成为第二份任务数据库或新的通用编排运行时。

实时状态要区分：本机观察时间、host 最后回执、PR 最后联网取证、用户明确释放。进程退出、Stop、文件变动均不能冒充完成或验证通过。断开时保留最后证据并标时间；未知不得显示为正常。

本轮先实现单项目视图、共享观察器、JSON 状态与按需 HTTP 服务；多个浏览器读同一份状态，PR 联网刷新显式开启。面板没有 merge、删除、shell 或派工写接口。已有操作沿原 CLI 和值守权限执行。

可持续资产是可迁移的任务与证据协议、真实项目的经验、状态与恢复能力；模型能力增长会减少流程需求，这些能力须持续以真实使用成本衡量，不能承诺“不会被替代”。

## 验收（2026-10-03，香港时间）

实际运行的是本批构建的 `cli/target/debug/agent-on`；全局 `~/.cargo/bin/agent-on` 仍为 v0.26.0。未改全局配置、未安装新版插件、未删除 checkout 或分支、未提交/推送本批，推荐 pin 保留 v0.26.0。

- `cargo test --quiet`：319 passed / 0 failed，14 组结果。首次最终回归发现原有兼容 wrapper 测试会继承其他测试临时环境；单独重跑通过，补共用环境锁后整套通过。生产 guard 判据未改。
- `cargo clippy --all-targets -- -D warnings`、`cargo fmt --check`、`git diff --check`：exit 0。
- 文档闸：tracked 223 份 PASS；使用同一检查函数追加本批两份新 Markdown，225 份 PASS。不是关闭闸或预写新版本。
- 真实 Chrome headless（独立配置，非替身 DOM）：`{"browser":"PASS","worktrees":6,"tasks":1,"console_errors":[]}`。桌面与 390px 布局、筛选、任务读取、断线保留最后观察通过；状态接口缺 header / 跨源请求退 403，POST 退 405。
- 实盘六棵树完整显示：有未提交内容的 orchestration 树保留，angry-sinoussi 的 1 个本地独有 commit 明示；其余回收候选只报告，没有接管或删除。本轮任务登记在主树共享账，先显示 working 进度，最终真实回传后 API 显示 reported、results=1、released_at=null，没有把待发布结果冒充释放。
- 开始时 PR 快照距今约 47 天。面板显式开启一个联网刷新器后重新取得 origin main 与 open PR 状态；当前 open PR 为 0。因此真实并行 PR 压力现场尚未发生，不能用空队列宣布大规模调度验收。
- Rust 回归覆盖：linked/primary 共读、任务 progress/result 的 reported 状态且不释放、损坏任务账不隐藏物理树；同 SHA 的 CI pending→green、审批打回、mergeable 冲突均更新队列，文件详情调用仍为 1 次。
- Hermes 接口依据本机 `hermes --version` / `hermes chat --help`（v0.21.3）和官方 CLI 文档；隔离执行器测试覆盖字面 prompt、精确 resume/model/cwd、exit 7、不绕过授权，以及只读拒绝先于创建任务。替身 argv 测试不等于真实认证模型任务。
- setup 隔离安装测试覆盖构建先于插件、Codex add 优先/旧 install 兼容、cargo 或插件失败非零、missing pin 不改用 main；没有拿用户全局配置做实验。

命令日志 `/private/tmp/agent-on-live-final-tests.log`，真实面板截图 `/private/tmp/agent-on-dashboard.png`（临时验收产物，不是发布文件）。发布候选为 minor v0.27.0；本批含 `cli/src/**`，按本仓权限硬停在最终合入/发布批准，不沿用八月旧版本批准。

10-03 goal 续接发现并补两处真实漏检：Codex command workdir 被 session cwd 覆盖；调度登记存在但最后执行 exit 78 仍显示 active。另修损坏 PreToolUse payload 静默放行。源码修正与实际 macOS 既有报告调度恢复、未达成的旧目标条款，逐项见 [旧 goal 验收审计](2026-10-03-worktree-goal-audit.md)。新增五项回归后完整测试 324 passed / 0 failed（`/private/tmp/agent-on-goal-final-tests.log`），clippy/fmt exit 0；未宣称已安装的 v0.26.0 自动含这些未发布修正。

真实 Hermes 认证模型任务与自动 hook、宿主自动回执全覆盖、多个真实 PR 压力现场、跨项目首页、跨机控制、原生 desktop、收益回访仍待完成。面板是本机只读入口与机器事实投影，不是这些能力的替代验收。

## 发布授权（2026-10-03）

用户明确批准「v0.27.0 发布到 main，并原子推送 main＋annotated tag」。本批据此封版、同步推荐 pin 与 CLI/插件 manifest，并通过本仓 tag-release 一次原子推送；此前的未发布验收记录保留其时间边界。授权不包含全局 CLI/插件升级、旧 worktree 删除或扩大 Hermes/desktop 验收结论；不据此宣称整个旧 goal 完成。最终提交与发布身份以 v0.27.0 annotated tag 及其指向的 main commit 为准。

发布前按 ship 的测试/独立复核/分层提交流程检查，不使用 Superpowers；版本格式、main 与 annotated tag 原子推送及授权范围以本仓纪律和用户本次批准为准，不另建 PR、不安装新的全局工具。独立复核发现并修两项：① PR 文件取证失败后批量绿 CI 可误升 NOW 并缓存空文件表，改为 unavailable、禁止可合/未知重叠结论并重试；② PR 联网刷新占任务账本锁，改专用仓库刷新锁。两项回归均先因该原因失败，再修实现转绿，复核确认不再阻塞。

最终 `cargo test --quiet`：326 passed / 0 failed（14 组，日志 `/private/tmp/agent-on-v027-tests.log`）；`cargo clippy --all-targets -- -D warnings`、`cargo fmt --check`、`git diff --check`、`cargo build --release` 均 exit 0。发布元数据首次同步时 default pin 与 crate 版本自检明确报错，补齐 `paths::DEFAULT_PIN` 后整套重新跑绿，没有绕过检查。未运行覆盖率工具，不把测试数量换算为覆盖率。
