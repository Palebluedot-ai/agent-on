# 2026-11-06 · 精简核心的实施与交付证据

> 职责边界：记录本轮用户授权、实现、验证、发布和安装。路线与删除范围见同日路线图；未取得实际回执的步骤不宣称完成。

## 用户批准

用户原文：「可以，按这个路线图开始实施，把这些冗余功能彻底删掉，验证通过后按仓库规则提交、打 tag 并推送。」

已确认删除自建派工、巡逻、活动采集和网页面板，保留普通对话、文字状态、Git 冲突保护和合入审计。包含路线图列出的共享基础拆分、专属测试／安装接线／配套清道夫退出、CI 与正文更新。此范围包含 `cli/src/**`、hooks、插件、CI 和配套规则；按现有硬停政策引用本次用户批准，不扩大到无关权限。

版本问题用户明确答复：「保持 v0.x，本次明确作为 breaking 特例」。因此本批为 **v0.30.0**，附迁移注记；不声称满足 v1.0 外部用户验收，不改变后续版本规则。

## 实施

- 基于远端 `5e4142c`（#44）继续；保留已有按范围 CI，固定工具链并分开下载／构建缓存。构建 key 包含实际编译器指纹、平台、lock、profile 和内容标识，兼容前缀可恢复更新后的构建；原 OS+lock 命中后不保存问题消除。
- 删除 activity、coordination、codex_model、dashboard、system_dashboard、janitor 和仅派工使用的 integration 模块；删面板 HTML、浏览器用例和专属集成套件；kit 静态面板模板也退出供货。
- 文件锁／原子写入抽到 state_io；status v2 读取当前 checkout 的真实 Git 状态、现有项目记录位置和合入缓存，不推断任务完成或窗口活动。
- SessionStart 只保留身份接续；逐工具 capture 与结束事件注册移除，旧非启动事件入口静默无操作。历史绑定仅保留路由限制，不当实时状态。
- retire-legacy 默认预览，apply 关闭项目旧采集／janitor 策略、备份配置并校验卸载旧 janitor 调度；不删除 checkout、分支、恢复 ref 或历史记录。原 report-only GC 保留。
- 方法／接入双落点：playbook/delivery.md + kit/delivery-workflow.md；boot、skill、Codex、身份接续、README/AGENTS/CHARTER 与迁移说明同步。历史执行书进 legacy/retired-v0.30。

## 本地验证（发布前）

- `cargo test --manifest-path cli/Cargo.toml --locked`：16 套，**347 passed / 0 failed**。
- `cargo clippy --manifest-path cli/Cargo.toml --locked --all-targets -- -D warnings`：exit 0；fmt 通过。
- Python 独立合入审计：**101 tests OK**；CI 选择器：**9 tests OK**。
- 文档闸：`DOC-GATE: PASS（251 份 markdown：职责边界棘轮 / 相对链接 / 推荐 pin 三处一致）`（实施版本更新前）。
- 新回归覆盖退场命令拒绝执行、旧非启动 hook 不采集、坏历史任务不阻塞 status、当前 worktree 语义、迁移可预览／重复且保全历史、坏配置与软链拒绝；身份与合入保护回归保留。
- 本仓 `retire-legacy --apply`：关闭 1 份旧活动配置，janitor timer absent；历史保留。只读进程核对未发现 agent-on 面板／巡逻运行进程。已有 worktree-gc 报告任务不属于退场清道夫，保留。

## 待补实际回执

线上候选、最低 Rust、缓存复用与隔离安装的实际证据已补在下文。用户已确认合入 PR #45 并完成发布；随后要求再研究每日一次 CI，本轮按最新选择收口。合入审计、封版 tag 推送和实际全局执行器升级尚未执行。已安装的 Claude/Codex 插件均关闭，不自动开启。夜间触发延迟不因删功能宣称已解决。

## 历史 #44 的本次追认

合入前 report 实际发现 #44 为 UNVERIFIED_HARDSTOP：10 月 29 日已合入，但缺可核验批准指针。已向用户说明该 PR 的文档免编译、测试范围精简内容及链接，并询问“确认保留并追加本次追认，保留原历史，不伪造当时批准”；用户答复：「按你建议」。据此于 2026-11-06 追加追认记录，不改写此前账本，也不声称 10 月 29 日已有批准。

最低支持版本验证：`cargo +1.89.0 check --manifest-path cli/Cargo.toml --locked --all-targets` exit 0；默认工具链未更改。

## 批次审查补修

需求与约定两轴审查定位并修复：迁移锁文件软链可能截断目标（先红后绿回归）；首次备份改为完整写入／同步后原子发布，已有不完整备份拒绝覆盖原配置；历史绑定扫描取消 256 条截断，保留超上限的已知窗口路由限制；无旧调度器支持的平台跳过 Unix HOME 探测。修复保留在相关保护回归中。

首个线上 PR #45 run 42178335421 全部必需检查 success；此为修补前候选，最终候选需另核，不以旧绿代替新提交验证。

## 线上 CI 与重复采样

- PR 候选 `a2edc1d`：run [42178655935](https://github.com/Palebluedot-ai/agent-on/actions/runs/42178655935) 必需检查通过，CLI job 49 秒。
- 同候选手动全量：run [42178741068](https://github.com/Palebluedot-ai/agent-on/actions/runs/42178741068) 通过，CLI 66 秒；后续热缓存 run [42179034270](https://github.com/Palebluedot-ai/agent-on/actions/runs/42179034270) 通过，CLI 58 秒。构建缓存约 58 MB（原约 158 MB）；热运行只编译本仓 crate，依赖构建得到复用。
- 第三次全量 run [42179233004](https://github.com/Palebluedot-ai/agent-on/actions/runs/42179233004) **失败**：既有 `persisted_schedule_survives_caller_path_and_executable_change` 在启动刚复制的二进制时出现 Linux `ETXTBSY`，不是断言失败。不能丢弃此样本后宣称三次稳定。
- 测试夹具改用与原程序同文件系统的临时硬链接，仍验证不同的规范化可执行路径和 PATH；不再写入即将启动的二进制。产品调度实现未改变。修补后重新采样，实际回执另补。

### 修补后三次全量样本

同一提交 `c2949e55d80a1402c53b66e14ceae08f4d8fd735`，相同 ubuntu-latest、Rust 1.97.1、兼容构建缓存；三次均运行全套 Rust、clippy、审计、范围选择器、文档和 intake。

| 运行 | CLI job 时长 | 结论 |
|---|---:|---|
| [run 42180020399](https://github.com/Palebluedot-ai/agent-on/actions/runs/42180020399) | 44 秒 | 全量 success |
| [run 42180254924](https://github.com/Palebluedot-ai/agent-on/actions/runs/42180254924) | 62 秒 | 全量 success |
| [run 42180393049](https://github.com/Palebluedot-ai/agent-on/actions/runs/42180393049) | 45 秒 | 全量 success |

中位数 **45 秒**，范围 **44–62 秒**；相较此前 81 秒单次全量基线，中位数低约 44%。样本小且 runner 波动存在，不报告 P95，也不声称修复计划触发延迟。修补前失败样本仍如实保留。本地相关 13 项测试连续 5 轮通过，未加入重试或跳过断言。

### 隔离安装与升级实测

从当前候选 `cargo install --offline --locked --path cli --root <临时安装目录>` 成功。`build-info` 为 c2949e5、source_dirty=false、source_fingerprint=fnv1a64:af6583aa3ffdc8b8；尚未封版，package 仍 0.29.0，release_tag=null，不冒称最终发布。

隔离项目将真实 v0.29.0 执行器替换为候选后：预览不改配置，两次 apply 成功；首次备份、历史 task、已有分支／worktree 和未提交文件逐一保留；status schema v2 可读；旧 capture 事件不写记录，退场命令均拒绝。实际本机两宿主插件均未启用；全局 CLI 和 repo release 的备份已备好，正式替换待封版后核 build-info。

## 主线合入与发布批准

2026-11-06 用户对“将 PR #45 合入 main，再封 v0.30.0、打 annotated tag 并推送；范围为已确认的功能删除、迁移清理、保留保护逻辑和 CI 优化”明确答复：「确认合入 main 并完成发布」。

随后用户提出：「利用 matt copark skill 再想一下怎么继续搞优化 CI 时间，我上次提到可否不用每次 merge 都 CI，每天 CI 一次可以么？」本轮使用 Matt Pocock research 方法核一手事实、比较每日集中与 PR 轻闸；研究建议不冒充已采纳配置。

## 最新 CI 节奏：每日一次

依据用户追加的“可否不用每次 merge 都 CI，每天 CI 一次”要求，采用 [研究](2026-11-06-daily-ci-research.md) 的方案 A；上述按范围 PR CI 是第一阶段证据，最终交付节奏在此更新：

- 仓库 Actions 仅每日香港 01:23 与按需手动全量；PR、push、tag 均不触发，不用空 job 冒充测试通过。
- 本仓 required_checks 清空，与不再上报的 PR job 同步；外部真实失败／待决、评审、draft/hold、冲突、权限与逐次 report/record 仍保留。GitHub 实查 main protection 404、rulesets=[]，未改远端保护。
- 本地按受影响范围验证，普通 tag 不另行强制全量，高风险或证据不足才手动补跑。外部贡献只许 intake 的范围由合入人核完整文件列表，外部作者硬停仍在。
- 现有 master 不提供夜间红灯自动停合入联动；维护者发现代码回归优先修复并暂停受影响发布。计划时间不保证准点执行，不能说每日绿覆盖之后的新提交。
- 新回归验证“无 PR 检查且政策无 required 可 READY；仍有 required 时 WAIT；实际失败仍 FIX”。独立审计 102 项、范围选择器 9 项、本地文档闸与 YAML 触发结构通过。后续手动运行只验证本次工作流调整，不变成每批默认步骤。

用户对每日模式的追加要求与此前“确认合入 main 并完成发布”一并用于本批；不扩大到关闭外部安全检查或取消审计。
