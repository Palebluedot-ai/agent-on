# intake · 2026-07-31 · inbox-radar（首次结账，21 卡）

> 范围：2026-07-12 接入 Agent-On 后尚未回流的 AI 协作、证据、发布与恢复教训。业务规则、团队口径、模型选择和客户数据均留在项目端。

### phantom-backlog-needs-evidence-sync（计划状态不跟证据收口会制造幽灵 P0）
- source:inbox-radar @ 3d45b23 | pin v0.3.0
- evidence:`3d45b23 docs: archive stale transition todos` 将已有代码与测试证据收口的旧 TODO/PARTIAL 改为历史问题，并把当前状态指向权威状态源。
- confidence:high（同一项目多份阶段计划反复把已完成事项重新抬成 P0）
- claim:阶段计划的状态表必须随验收证据同步；历史问题可保留原貌，但当前状态只能写可验证事实并指向唯一权威状态源。
- suggested_landing:playbook/truth-hierarchy 的状态面喂养规则；kit/phase-card-template 的收口检查行
- rollback:revert 对应 agent-on 落地 commit
- trace:loop-notes.md §2026-07-21「计划文档也会制造 phantom P0」
- 状态:landed@同批(第十七次消化·包1:truth-hierarchy §五¾ + merge 7b + dashboard ⑥ + phase 收口行)

### durable-status-must-not-self-pin-live-head（长期状态文件不要追逐当前 HEAD）
- source:inbox-radar @ 36c7c15 | pin v0.3.0
- evidence:`36c7c15 docs: snapshot runtime preflight status` 暴露“最新 main 是某 SHA”会被下一次 status-only commit 立即打破；后续合同测试改为要求 live git/GitHub 查询。
- confidence:high（同型漂移同时出现在 HEAD SHA 与 ahead/behind 计数）
- claim:长期状态文件只记录具名历史观测，不 self-pin 当前 HEAD、分支计数或最新 CI；这些易变事实必须由 live 命令在决策时查询。
- suggested_landing:playbook/truth-hierarchy 的动态事实边界；kit/dashboard-template 的“当前值来源”提示
- rollback:revert 对应 agent-on 落地 commit
- trace:loop-notes.md §2026-07-21「状态提交自指漂移」+ §2026-07-18「证据避免自指」
- 状态:landed@同批(第十七次消化·包1:truth-hierarchy §五¾ + merge 7b + dashboard ⑥ + phase 收口行)

### release-prerequisites-need-machine-readable-preflight（生产前置要变成机器可判的 preflight）
- source:inbox-radar @ 36c7c15 | pin v0.3.0
- evidence:`36c7c15` 引入只读 runtime preflight 状态；后续 `53e7101` 将外部门与 load wrapper 接通，报告只给 JSON pass/block 且不输出 secret。
- confidence:high（人工 checklist 曾允许单项本机检查被误读成生产 load 授权）
- claim:生产前置必须收敛为只读、机器可判、默认 fail-closed 的 preflight；输出明确列出每个 pass/block，且不得联网探测或泄露 secret。
- suggested_landing:playbook/sop 的发布阶段；kit/phase-card-template 的 production preflight 行
- rollback:revert 对应 agent-on 落地 commit
- trace:loop-notes.md §2026-07-21「生产前置需要机器门禁」
- 状态:pending

### safe-block-is-progress-evidence（安全阻塞本身也是发布进展证据）
- source:inbox-radar @ 36c7c15 | pin v0.3.0
- evidence:runtime preflight 对 Git/floor/plist/unloaded 给出通过，同时对缺失 env/Graph/LLM 给出精确阻塞；pipeline 未被启动。
- confidence:medium（单项目实证，但适用于有多前置条件的发布）
- claim:把“为什么不能上线”产品化为可重复的结构化检查；已通过项和正确阻塞项都应被记录，避免团队把 fail-closed 误判为没有进展。
- suggested_landing:playbook/sop 发布状态；kit/progress-template 的 blocker 证据格式
- rollback:revert 对应 agent-on 落地 commit
- trace:loop-notes.md §2026-07-21「安全阻塞是进展证据」
- 状态:pending

### verify-remote-state-through-authoritative-api（远端状态先查权威 API，不靠页面感觉）
- source:inbox-radar @ abcdc89 | pin v0.3.0
- evidence:`abcdc89 docs: record PR and runtime gate status` 记录 PR #36 已 MERGED 与 main CI run `29773533843`；结论来自 `gh pr view` / `gh run view`，而非仍显示进行中的页面感知。
- confidence:high（页面状态与远端 API 事实直接冲突）
- claim:PR、CI、部署等远端对象在采取下一步动作前，必须通过权威 API/CLI fresh read-back；缓存页面和会话记忆只能做线索，不能做动作依据。
- suggested_landing:playbook/sop 外部系统取证；kit/merge-checklist 的 fresh remote state 行
- rollback:revert 对应 agent-on 落地 commit
- trace:loop-notes.md §2026-07-21「PR 状态不要靠页面感觉」
- 状态:pending

### historical-docs-need-drift-quarantine（历史文档必须隔离，不能继续伪装当前规则）
- source:inbox-radar @ 6dd9100 | pin v0.3.0
- evidence:`6dd9100 fix: prevent stale roster prompt drift` 将旧 roster 文档标成 history/disabled，并用合同测试要求当前 roster 只从结构化真相源生成。
- confidence:high（旧业务说明虽不再直接进 prompt，仍持续误导后续 agent）
- claim:被替代的规则文档必须显式标记 history/disabled 并从当前生成链退出；若仍保留作证据，应加 drift test 防它再次被当成现行真相。
- suggested_landing:playbook/truth-hierarchy 的历史证据层；kit/AGENTS-skeleton 的 superseded-doc 纪律
- rollback:revert 对应 agent-on 落地 commit
- trace:loop-notes.md §2026-07-21「历史业务文档也会污染未来判断」
- 状态:landed@同批(第十七次消化·包1:truth-hierarchy §五¾ + merge 7b + dashboard ⑥ + phase 收口行)

### skipped-production-check-must-stay-red（跳过生产检查不能返回绿色）
- source:inbox-radar @ 2df1cf5 | pin v0.3.0
- evidence:`2df1cf5 fix: prevent skipped launchd preflight success` 让 `--skip-launchd` 报告保持 `ok=false`，防 CI/non-mac dry-run 被误当生产 load 证据。
- confidence:high（原行为在配置齐全时可 exit 0，形成明确误授权路径）
- claim:任何 skip/unsupported 的生产检查都必须保持整体红灯并标明“不适用于生产授权”；未知或未执行不能折算为通过。
- suggested_landing:playbook/sop 发布门；kit/preflight/checklist 的三态语义
- rollback:revert 对应 agent-on 落地 commit
- trace:loop-notes.md §2026-07-21「跳过检查必须保持红灯」
- 状态:pending

### release-gate-must-wrap-the-execution-entry（发布门必须包住唯一执行入口）
- source:inbox-radar @ 53e7101 | pin v0.3.0
- evidence:`53e7101 fix: enforce external gate before launchd load` 让 load wrapper 重新派生外部门并拒绝手工翻绿；此前 runbook 有门，但执行入口只检查本机条件。
- confidence:high（存在可直接绕过文档门禁的真实 load 路径）
- claim:发布规则必须接到唯一执行入口并由代码重新计算，不能只写在 runbook；手工状态字段不得单独把门翻绿。
- suggested_landing:playbook/sop 发布执行面；kit/phase-card-template 的“门禁接线到入口”验收行
- rollback:revert 对应 agent-on 落地 commit
- trace:loop-notes.md §2026-07-21「文档门禁必须接到执行入口」
- 状态:pending

### truth-surfaces-and-code-must-close-together（最终合流要同时核代码与状态面）
- source:inbox-radar @ c37c9aa | pin v0.3.0
- evidence:`c37c9aa docs: adopt project truth and release controls` 取证到代码已前进 32 commit，而 progress/phase/dashboard 仍显示已关闭 P0；状态面此前保持未跟踪。
- confidence:high（同一仓 exact HEAD 与三张状态面发生显著分叉）
- claim:最终合流检查必须同时核对 exact HEAD、测试证据和所有 canonical 状态面；代码提交完成但状态面未落盘，不算环节收口。
- suggested_landing:playbook/truth-hierarchy；kit/merge-checklist 的状态面 closure 行
- rollback:revert 对应 agent-on 落地 commit
- trace:loop-notes.md §2026-07-18「真相漂移」
- 状态:landed@同批(第十七次消化·包1:truth-hierarchy §五¾ + merge 7b + dashboard ⑥ + phase 收口行)

### production-runtime-needs-clean-persistent-checkout（生产调度不要运行在开发 checkout）
- source:inbox-radar @ c37c9aa | pin v0.3.0
- evidence:共享 checkout 的 dirty guard 在 09:07 以 exit 14 阻止 pipeline；项目随后迁移到 `/Users/chao/Projects/inbox-radar-runtime` 并由 preflight 锁定 branch/upstream/clean/head。
- confidence:high（共享 checkout 已真实造成定时任务中断）
- claim:本机生产调度也要使用干净、持久、用途单一的 runtime checkout；开发工作树只能作为构建源，不能直接承载定时生产。
- suggested_landing:playbook/sop 部署运行态；kit/deployment checklist 的 runtime isolation 行
- rollback:revert 对应 agent-on 落地 commit
- trace:loop-notes.md §2026-07-18「运行态隔离」
- 状态:pending

### final-gate-evidence-must-follow-the-last-edit（最终门禁证据必须晚于最后一次改动）
- source:inbox-radar @ c37c9aa | pin v0.3.0
- evidence:exact-HEAD gate 已在会话中通过，但 tracked progress 仍保留未来时态；修文档后原 gate 已不再覆盖最终 HEAD。
- confidence:high（同一收口同时出现“测试已过”和“最终提交未被测试覆盖”）
- claim:每次最终门禁必须绑定 exact SHA，并在最后一处代码或文档变更后重跑；会话里的旧通过记录不能证明后续提交。
- suggested_landing:playbook/sop 验证时序；kit/merge-checklist 的 exact-HEAD final gate 行
- rollback:revert 对应 agent-on 落地 commit
- trace:loop-notes.md §2026-07-18「证据闭环」
- 状态:pending

### do-not-fabricate-retroactive-run-ledgers（漏记实时台账时不要事后伪造）
- source:inbox-radar @ c37c9aa | pin v0.3.0
- evidence:接管后的首次正式多-agent loop 已开始后才接入 run ledger；项目只补人读 Run #1 摘要，并在 lock 登记“不回填 JSONL”偏离。
- confidence:medium（单项目一次，但时间线真实性问题可泛化）
- claim:运行台账必须从首次派工实时采集；若中途才发现漏记，只补明确标注的人工摘要，不事后伪造 task 时间、token 或 agent 轨迹。
- suggested_landing:playbook/iteration-loop；kit/run-ledger schema 的 retroactive-data 禁令
- rollback:revert 对应 agent-on 落地 commit
- trace:loop-notes.md §2026-07-18「编排账本」+ agent-on.lock.md local_deviations
- 状态:pending

### config-truth-needs-nonsecret-executable-probe（运行配置不要从叙述文档推断）
- source:inbox-radar @ c37c9aa | pin v0.3.0
- evidence:历史说明把 live provider 误认成本地 HTTP 组合；只输出布尔分类的探针证明实际两路均为合规 HTTPS，且未打印 endpoint 或 secret。
- confidence:high（叙述与真实配置直接冲突，探针给出可重复反证）
- claim:迁移或审计运行配置时，优先写不泄密的可执行探针验证协议、位置和能力；叙述性文档只做线索，不得反推出 secret、endpoint 或当前 provider。
- suggested_landing:playbook/anti-hallucination 的配置取证；kit/preflight 的 secret-safe 输出约定
- rollback:revert 对应 agent-on 落地 commit
- trace:loop-notes.md §2026-07-18「配置真相」
- 状态:pending

### cross-runtime-contracts-need-adversarial-matrices（跨运行时契约要锁可观察语义）
- source:inbox-radar @ cd7d694 | pin v0.3.0
- evidence:`cd7d694 docs: settle release recovery truth` 记录 Python `posixpath.normpath` 与 Worker URL/path 对双前导 slash、编码 separator、重复 slash、dot segment 的行为不一致；两边用同类 adversarial matrix 锁定。
- confidence:high（两个真实运行时对同一输入族给出不同默认行为）
- claim:跨语言/运行时边界不要共享“应该一样”的假设；先写 decode→canonicalize→accept/reject 的可观察契约，再在每个实现上跑同一 adversarial matrix。
- suggested_landing:playbook/anti-hallucination 的边界契约；kit/review-prompt 的 cross-runtime matrix 行
- rollback:revert 对应 agent-on 落地 commit
- trace:loop-notes.md §2026-07-18「跨运行时路径语义」
- 状态:pending

### validate-at-producer-and-consumer-boundaries（不可信数据要生产者和消费者双门）
- source:inbox-radar @ cd7d694 | pin v0.3.0
- evidence:只在卡片渲染时拦截会让坏 URL 留在 cache，只在写入时拦截又不能保护历史/手工数据；最终 producer 净化且 Python/Worker consumer 独立 fail-closed。
- confidence:high（两种单门方案均有可构造失效面）
- claim:对会持久化并跨运行时消费的不可信字段，producer 写入前净化，所有 consumer 仍独立 fail-closed；单边校验不能替代另一边。
- suggested_landing:playbook/sop 数据边界；kit/review-prompt 的 producer/consumer 双门检查
- rollback:revert 对应 agent-on 落地 commit
- trace:loop-notes.md §2026-07-18「生产者/消费者双门」
- 状态:pending

### new-review-findings-invalidate-inflight-gates（新 P1 会让正在跑的旧门禁过期）
- source:inbox-radar @ cd7d694 | pin v0.3.0
- evidence:完整 gate 运行期间独立 reviewer 发现路径边界 P1；旧 gate 即使继续完成，也不覆盖修复后的 HEAD。流程改为中止旧 gate、补 RED/GREEN、reviewer 清零后只跑一次最终 exact-HEAD gate。
- confidence:high（真实并行复审与门禁时序冲突）
- claim:门禁运行中出现新的边界反例或 P1，应立即取消旧门禁；先修复并复审清零，再对最终 exact HEAD 运行唯一有效门禁。
- suggested_landing:playbook/workflow-orchestration 的并行复审时序；kit/merge-checklist 的 stale gate 规则
- rollback:revert 对应 agent-on 落地 commit
- trace:loop-notes.md §2026-07-18「并行复审时序」
- 状态:pending

### operations-docs-are-executable-release-surface（运维文档里的命令也是执行面）
- source:inbox-radar @ 3283821 | pin v0.3.0
- evidence:`3283821 docs: close scheduler recovery gate` 修复排错章节中可绕过正文发布门的 `launchctl load -w`；每个 start/load 示例就近重申完整前置。
- confidence:high（复制单条文档命令即可真实绕门）
- claim:把 runbook 中可复制执行的 start/load/deploy 命令视为代码入口审计；每条命令必须就近带齐前置，不能依赖读者记住正文远处的门禁。
- suggested_landing:playbook/sop 运维文档审查；kit/review-prompt 的 executable docs 行
- rollback:revert 对应 agent-on 落地 commit
- trace:loop-notes.md §2026-07-18「运维文档也是执行面」
- 状态:pending

### ephemeral-divergence-belongs-in-live-commands（分支计数等易变证据只在动作前 fresh 查询）
- source:inbox-radar @ 3283821 | pin v0.3.0
- evidence:把“当前分支 ahead N”写进该分支后，提交本身立即改变 N；项目改为只持久化具名 SHA 观测点，push 前 fresh fetch 再计算。
- confidence:high（写入证据的动作本身机械改变被记录值）
- claim:ahead/behind、open PR 数、pending checks 等易变计数只用于动作前 fresh decision，不写成长期真相；持久记录改用具名对象与当时观测时间。
- suggested_landing:与 `durable-status-must-not-self-pin-live-head` 语义归并；kit/merge-checklist fresh fetch 行
- rollback:revert 对应 agent-on 落地 commit
- trace:loop-notes.md §2026-07-18「证据避免自指」
- 状态:landed@同批(第十七次消化·包1:truth-hierarchy §五¾ + merge 7b + dashboard ⑥ + phase 收口行)

### contract-tests-should-lock-invariants-not-old-phrases（合同测试锁不变量，不锁旧措辞）
- source:inbox-radar @ 6f76d35 | pin v0.3.0
- evidence:`6f76d35 test: enforce complete scheduler release gate` 将旧 phrase oracle 改为同时要求完整外部/runtime gate、单项测试不构成授权、排错不得直接 load；旧测试此前阻止更安全的文档语义。
- confidence:high（旧 phrase test 可通过恢复误导句“换绿灯”）
- claim:文档合同测试应断言安全不变量、必需概念和禁止路径，不逐字冻结旧句；语义升级时先保留 RED 证据，再把 oracle 改成可判别结构。
- suggested_landing:playbook/anti-hallucination 的测试锚；kit/review-prompt 的 phrase-oracle 检查
- rollback:revert 对应 agent-on 落地 commit
- trace:loop-notes.md §2026-07-18「合同测试锁不变量」
- 状态:pending

### capability-scoped-release-gates-enable-narrow-rollout（全局门未闭环时按能力做窄发布）
- source:inbox-radar @ b6fbc39 | pin v0.5.1
- evidence:`b6fbc39 feat: allow manager-only runtime release gate` 新增 `release_scope=manager-only`；真实 preflight 在 `external_release_gate=false` 时仍要求 Git/env/Graph/LLM/manager recipient/runner contract 全绿，随后只向 manager 发送。
- confidence:high（真实 smoke 证明窄能力可用，未批准的 team/action 能力仍被机械排除）
- claim:不要让一个全局 Boolean 把独立能力绑死；按 release scope 列出必须满足与明确不适用的门，并机械验证该 scope 的收件人、执行入口和禁用能力。
- suggested_landing:playbook/sop 渐进发布；kit/phase-card-template 的 capability scope / excluded capabilities 字段
- rollback:revert 对应 agent-on 落地 commit
- trace:loop-notes.md §2026-07-31「按能力拆发布门」
- 状态:pending

### ambiguous-external-effects-require-readback-before-retry（外部副作用不明时先查实况再重试）
- source:inbox-radar @ 467f0ac | pin v0.5.1
- evidence:`run-2026-07-30T232310_0800` 中 7 次 `lark-cli` 实际返回 `{ok:true,data.message_id}` 并送达，但旧解析器误报 `transport_failed`；recovery journal 保持 `delivery_attempting`、拒绝重发。`lark-cli im +chat-messages-list` 读回 7 个 message_id 后才对账 manifest；`467f0ac fix: accept lark cli v1 send receipts` 加 34 项相关测试。
- confidence:high（外部消息历史、7 个唯一 message_id、恢复 journal 与修复测试四路证据一致）
- claim:解析失败不能自动等同传输失败；遇到可能已产生外部副作用的 ambiguous result，必须冻结 replay，先用只读 API 按时间窗/收件人/幂等标识 read-back，再决定对账或重试。
- suggested_landing:playbook/sop 恢复与幂等；kit/review-prompt 的 ambiguous side-effect read-back 行；bench 案例
- rollback:revert 对应 agent-on 落地 commit
- trace:loop-notes.md §2026-07-31「外部副作用不明时先 read-back，禁止盲重放」
- 状态:pending
