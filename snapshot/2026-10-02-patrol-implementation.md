# 2026-10-02 · 巡逻、派工、清道夫与默认流程减负实施

> 职责边界：记录本次实施范围、用户授权来源和实际验证。当前使用说明只有 `kit/patrol-control-plane.md`；10-01 两份快照保留当时的评审草案，不作为现行执行书。
> 基线：`v0.25.2` / `124303d`。实现分支 `codex/coordination-and-reclaim`；本文件首次落盘时未提交、未合入、未发布，推荐 pin 仍指已发布版本。
> 后续原清单补齐、提交授权与最终验证见 [同日核心清单收口](2026-10-02-core-checklist-completion.md)；本页保留第一次实施时的证据与限制。

## 用户决定与核心

用户要求保留有效的值守；增加提前发现共享改动的巡逻、统一入口、自动独立记录窗口、可调用 Claude/Codex/Grok 的 Agent OS 派工和定期清道夫；并明确「直接开工」。产品核心保留上下文、实际证据、共享协作与经验回流，成长包含减法。该指令授权本批实现，不意味着启用当前项目的巡逻、替用户登录模型、删除现有树或跳过发布硬停。

十个窗口仍用一个日常入口。共享 Card、token、路由和锁文件由一个任务收口，其余任务等共享成果集成。窗口数量不决定 worktree 数量，读任务复用项目，确实同时写的独立任务按需隔离。值守在班仍唯一负责合入、发布、对外通信和跨窗口中转。

## 这批具体落地

| 需求 | 落地 | 边界 |
|---|---|---|
| 自动独立巡逻 | Terminal/tmux 启动已有 CLI；桌面 skill 桥接原生 create/wait/bind | 新聊天不保证第二个系统窗口；登录和权限提示不算 ready |
| 自动记录与任务列表 | opt-in hooks 捕获需求摘要、真实 session id 与结果；共同 git dir 跨树回执 | 不复制全量对话，未接 hook 的窗口覆盖不全；摘要自述不算验证 |
| 提前调和 | 声明路径 + 实际 committed/index/working/untracked 对表，派工前返回重叠归属 | 机械路径检测不能证明语义/视觉都一致；不改原提交闸 |
| Agent OS | dispatch/agent-os 的三家 CLI 薄适配器；默认模型继承、可显式选择 | 不托管认证、不装工具、不自建模型服务、不 bypass |
| 清道夫 | 有限受管策略、独立日任务、report/apply/restore | 不纳管旧未知树，不删除分支；native worktree 用宿主 archive |
| 减负 | 明确续接直接推进、风险验证、共享先行、模板按需、原型可复用、短报告、消化可淘汰规则 | 不跳 required CI，不削凭据/数据/权限保护，不宣称已经量化提速 |

实现为 `cli/src/coordination.rs`、`cli/src/janitor.rs`，复用 `worktree_schedule.rs`，不引入运行库、常驻模型循环或通用工作流引擎。Rust 1.89+ 的进程文件锁在进程退出时释放；临时文件原子替换，私有回执 0600。版本 manifest 与源码候选为 0.26.0，版本真相仍以实际发布 tag 为准。

失败留下任务与 checkout；无真实 host id 的启动可用同回执 retry，有 id 的退出会话精确 resume，状态未知先明确 close。不得按 `--last` 猜。桌面 --worktree 由宿主造树后绑定实际路径，避免 CLI 与宿主各造一棵。

巡逻观察器 30 秒刷新文件证据，只对问题集合变化给本机提示。独立模型初次自动给报告，此后读最新台账回答查询；不把缓存更新当每轮模型都已思考。主窗口下一条需求会得到记录位置与重叠数量；原生后台持续巡逻复用宿主 heartbeat。

## 追加：原窗口压缩续接

用户本轮补充上下文耗尽的痛点，并明确偏好「在一个窗口里面 compact，不需要 handoff 到第二个新的窗口」。默认复用宿主原生自动 compact，主动压缩也在同一会话，不新增窗口、checkout 或接管模型上下文的运行时。

判断依据：[Claude Code 当前机制](https://code.claude.com/docs/en/how-claude-code-works) 说明自动压缩先清理旧工具输出，必要时总结历史，同时承认早期聊天细节可能丢失；[Codex 配置](https://learn.chatgpt.com/docs/config-file/config-reference) 有原生自动压缩阈值，[App Server](https://learn.chatgpt.com/docs/app-server) 的 thread/compact/start 在同一 threadId 完成。[Anthropic 2026-03-24 的工程实验](https://www.anthropic.com/engineering/harness-design-long-running-apps) 记录早期 Sonnet 4.5 需要上下文重置，而 Opus 4.5 可去掉重置、连续会话使用自动压缩。后者是特定模型与实验的证据，不能推成所有模型无限压缩无损。

本机核对到 Codex CLI 0.147.0、Claude Code 2.1.201；这不代表桌面后端或当前选用模型的版本。未做真实多轮压缩与新会话的质量对照，不能声称已量化本项目遗忘率或提速。

正文 `boot/session-handshake.md` 和 `playbook/model-playbook.md`、模板 `kit/AGENTS-skeleton.md` 与巡逻执行书同步：自然收口或准备主动压缩时更新已有任务记录，保留目标、用户决定、未完项与证据；压缩后按需重读原文件与实际 Git 状态。压缩后的摘要不是单一真相，不对上一份摘要反复加工作为唯一交接材料。校正后仍持续遗忘、重复工作或绕圈时才考虑新会话，不按百分比或次数硬切换。

本次仅文档与模板规则，没有自动 compact 前写卡、compact 后注入的跨宿主 hook 接线，也没有实际压缩当前聊天。不会把巡逻需求摘要当成全量任务记忆，不新增每轮审批或强制交接文档。

## 本轮开发与清理清单（2026-10-02 核对）

本节响应用户「列一个清单，准备开始开发：删掉过去的内容、明确要开发的点以及新开发的点」。它是本批工作清单，不替代宪章、功能执行书或运行状态。现有第一版代码与模板已在本地，后续从清理和接线验收继续，不从零重复实现。

核对时 HEAD 为 `124303d` / `v0.25.2`，分支 `codex/coordination-and-reclaim`；本批 44 个文件改动尚未提交、合入或发布，Cargo/plugin 的 0.26.0 只是候选。上一轮测试实际输出共 300 passed、0 failed；实际宿主与窗口链仍未验证。

### A. 删除旧规则与精简默认流程

“删掉过去的内容”落实到下表中的旧要求、重复入口与过时说法。历史决策、事故证据、intake 和可回放案例保留；过时的展望从执行入口退出并归档，不把历史原文改成今天的结论。

| 清理项 | 处理方式 | 核对状态 |
|---|---|---|
| 先 claim/owns 才能开工、续跑反复三选一 | 删前置与重复确认，明确指令直接推进 | 主要规则已改；README 示例仍待对齐 |
| 碰真实用户或外部服务就整包升档 | 装备按协作规模选择，护栏按具体风险选择 | BOOTSTRAP/adopt 已改；README 仍有旧的“上全套”说法 |
| MRD/PRD/任务卡/仪表盘成套必建 | 退出默认，复用已有目标与状态，需要时才展开 | 本批主入口已改，交付前再查模板残留 |
| 样式小改强制全量 TDD、原型必须重写 | 去掉绝对要求，按实际改动验证并复用合格部分 | 主要正文与模板已改；旧展望仍有旧表格 |
| 普通回复强制六段报告 | 退出普通回复默认，复杂交付按需保留结构 | 本批 output-contract 与模板已改 |
| 固定模型品牌套餐、换模型必跑能力考试、几题推成 95% 可靠性 | 删除品牌绑定和无依据阈值，探针仅作可选诊断 | model-playbook/bench 已改；握手模型提示和 orchestration-future 仍待清理 |
| 复发两项目自动升硬门、每次消化必须新增或改正文 | 去掉强制增长，支持修正、合并、删除、保留案例与有理由拒绝 | 正文已改；回流卡的“claim 必为新规则”示例待调整 |
| 不同页面自动各开树和 PR | 去掉按页面拆轨，共享组件/token/路由/锁文件先归一个任务 | 本批派工与协作规则已改 |
| 每轮 compact 都新开窗口、交接文档或 worktree | 从本轮方案退出，默认原窗口原生 compact | 规则已写，未添加自动接线或新压缩引擎 |

具体残留落点：`README.md` 的装备/示例/模型表述、`CHARTER.md` 的旧模型“保费”措辞、`boot/session-handshake.md` 的换模型必探针提示、`kit/AGENTS-skeleton.md` 的环节表 TDD 旧称、`kit/promotion-card-template.md` 的 claim 示例、`playbook/orchestration-future.md` 的过时套餐与迁移要求。后者建议归档，现行模型策略只认 `playbook/model-playbook.md`；归档前逐项核引用，保留仍有价值的原始来源。

### B. 核心保留与本轮新增

核心保留：可复用的项目上下文、完成证据、经验采集/消化/淘汰、版本化升级；有效的值守与合并审计继续使用。案例与验证按需调用，不把它们扩成每个任务的必走流程。

| 开发点 | 当前已落地 | 交付前缺口 |
|---|---|---|
| 统一入口与共享改动协调 | 原窗口接目标；共享任务先行，派发前检查路径重叠与依赖集成 | 用两个实际页面任务验证共享修改的归属、顺序与结果回传；当前路径检查不能证明语义或视觉一致 |
| 独立巡逻与自动记录窗口 | patrol 启动/复用/状态/扫描、需求摘要与任务回执；Terminal/tmux 和桌面桥接规则 | Claude/Codex 真实启动、登录/权限/缺 hook 情况、记录覆盖、只提示变化；桌面新聊天与第二个系统窗口需分别验收 |
| Agent OS 跨模型派工 | 三家本机 CLI 薄适配、默认模型继承、明确路径/依赖、失败重试与精确续接 | Claude→Codex/Grok 等实际接单与结果回传；缺安装/认证时真实报错，不以替身测试报厂商链已通 |
| 清道夫 | 有限受管回收、独立日任务、保全 ref 与 restore；临时 Git 仓测试已覆盖 | 受控临时项目的系统调度、实际占用保护和恢复验收；不先对旧项目树批量启用 |
| 原窗口 compact 续跑 | 原生压缩优先、已有任务文件保存关键事实、按需重读原文件 | 用一次真实长任务观察是否遗忘决定或重复工作；当前仅规则，无自动压缩前写卡/后注入 |
| 消化支持减法 | 允许规则删除/收缩/合并、保留案例或拒绝，不为指标制造新规则 | 用一张真实摩擦卡完成“退出默认或删除规则”的回流验收，检查卡片模板与供货说明一致 |

### C. 执行顺序与暂不扩项

1. **清理收口**：处理 A 表残留、归档过时展望并修引用，README/宪章/模板与实际第一版行为一致。先做这一项。
2. **真实接线**：在受控项目验收两个宿主开窗与记录、跨模型派工回传、一个共享任务加两个页面的协调过程，以及清道夫调度和恢复。先小规模，不以同时十个窗口当初次验收。
3. **减负验收**：观察原窗口 compact 的续跑与一张实际回流卡；只对观察到的缺口补机制。
4. **供货收口**：按实际变更检查 CLI/plugin/模板的执行版本与升级说明；具体结果批准后按本仓规则封版、提交、annotated tag 与原子推送。

本轮暂不新增通用编排引擎、记忆数据库、自写压缩器、强制自动 handoff、全仓 CI 缓存或绕过 required checks 的机制；现有同文件提交闸保留，出现真实误拦反例后另列切片。恢复和审批事实不能用自述摘要替代。

## 回收必须同时成立

CLI 自己创建的 checkout、明确 release 与证据、释放后 HEAD 没变化、成果已集成到主树、无 live/unconfirmed 会话或活依赖、Git 干净、无锁/进行中的 Git 操作/子模块/嵌套仓库、本地资源与 `lsof` 进程盘点完整、闲置期完成。默认 24 小时；ignored 默认保护，明确 regenerable 缓存中仍保护 env/数据库/密钥。不能盘点就保留。

先钉 `refs/agent-on/recovery/<task-id>` 和回执，执行前复核，再无 force `git worktree remove`。分支与引用保留；恢复到原路径不覆盖现有目录。旧 daily-gc 与新 janitor 的名称、状态、配置文件分别归属，旧报告任务不升级权限。disable 先撤授权再卸载；遗留任务再次运行也读新策略。

## 文档删改与单一权威

- 正文：BOOTSTRAP、adopt/handshake/settlement、freedom-vs-discipline、anti-hallucination/sop、model-playbook、iteration-loop/meta-principles、multi-contributor-protocol。
- 模板：AGENTS-skeleton、track-prompt、merge-checklist、output-contract；旧 claim 前置、品牌脚手架套餐、强制全量 TDD/原型重写、整包文档和无意义空段退出默认。
- 值守仅修漂移：canonical 文档不自动先问，地址只认 common git dir 当前 oncall 登记。原合并政策、审计和三权不改。
- 新功能执行书只有 `kit/patrol-control-plane.md`；两个宿主复用同一 `skill/SKILL.md` 与 plugin hook manifest。

## 验证与现场覆盖

测试使用临时 Git 仓、假 provider 与启动器，不使用真实账户调用模型，也不对本项目工作树执行回收。最终实现校验：

- `cargo test --manifest-path cli/Cargo.toml --quiet`：各套件 `211 + 20 + 10 + 7 + 7 + 11 + 9 + 3 + 3 + 11 + 8 = 300` 项全过，0 failed，exit 0；本地完整输出 `/private/tmp/agent-on-coordination-tests-final.log`。
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings`：exit 0。`cargo fmt --manifest-path cli/Cargo.toml --check`：exit 0。
- 源码构建的 `agent-on intake-lint`：`承接层校验通过:208 张卡,六项齐全。`，exit 0。
- `.github/scripts/check_docs.py`：`DOC-GATE: PASS（220 份 markdown：职责边界棘轮 / 相对链接 / 推荐 pin 三处一致）`，exit 0。已将本批新文件标记 intent-to-add，正常 git ls-files 能覆盖它们；用户 intake 不纳入补丁。
- 入口 skill：基础 validator 通过；保留原有 Claude 扩展字段，单独确认 disable-model-invocation 为 true、argument-hint 为字符串，未为迎合基础校验删调用政策或安装包。
- 本项目只读巡检：`oncall status --json` = `{"present":false,"self_is_oncall":false,"stale":false}`，`worktree status` = `ok`；`patrol scan` 在主树解析 base `124303ddd9308c437afa18a62960e4f92ee9f88c`，0 重叠、0 unknown。
- `git diff --check`：exit 0。用户已有 intake 的 SHA256 仍是 `85273f542f18f54cfee06cecd836b5d971011a20966f46da8fb4ffd46955321d`。

已覆盖：并发巡逻复用、disable/start 重新启用、声明与已提交重叠、依赖先集成、只读不挡单写者、失败 retry 不重复造树、三家实际 argv/子进程、用户目标不进入 shell、精确会话 ID、原生树绑定、Stop 结果非验证、oncall 的 --repo 冒充阻断、释放后的新 HEAD 保护、merged-PR head 覆盖、占用句柄/dirty/locked/ignored 保护、缓存白名单与数据库保护、恢复 ref/分支保留、旧 GC 与 janitor 定时任务隔离。

未现场验证：真实 Terminal 弹窗与系统通知、Claude/Codex/Grok 认证与模型任务、Codex 桌面 create→wait→hook→bind、系统调度安装与按时执行。未改本机权限、插件缓存、全局 hook 配置或自动化；未动用户已有 `intake/2026-09-29-dartify.md`。

## 公开接口依据

实际本机 CLI --help 与官方说明对表，不用深链接预填聊天冒充派发：

- [Claude Code headless](https://code.claude.com/docs/en/headless)、[hooks](https://code.claude.com/docs/en/hooks)
- [Codex hooks](https://learn.chatgpt.com/docs/hooks)、[非交互模式](https://learn.chatgpt.com/docs/non-interactive-mode)
- [Grok CLI](https://docs.x.ai/build/cli/headless-scripting)、[Git worktree](https://git-scm.com/docs/git-worktree)

本批含 CLI、hook、插件与文档棘轮基线，属于现行合入硬停目录。正式合入/发布按用户对具体结果的批准执行，交付 commit 仍须同批 annotated tag + 原子 push；不修改硬停清单规避该步骤。
