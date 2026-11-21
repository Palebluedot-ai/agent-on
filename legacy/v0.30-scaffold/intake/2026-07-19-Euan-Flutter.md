# intake · 2026-07-19 · Euan-Flutter（客户安全异议 → D18 混合留存拍板会话）

> 锚：loop-notes「2026-07-12 · ⇄ agent-on 结账回执 #3」之后无新散文条目；本批 = **本会话**过程教训 2 卡。  
> **域判据**：产品口径本身（截图默认不囤 / 证据 opt-in / 客户页结构化）已落项目 `docs/requirements.md` **D18**，**不出仓**。出仓的是「隐私叙事 vs 产品回看」冲突时的协作判据，与「暂停项局部解禁」落账纪律。

### privacy-feature-tension-hybrid-optin（敏感原件：默认结构化真相 + 证据 opt-in，禁止双全叙事）
- source:Euan-Flutter @ f0369a6 | pin v0.3.0
- evidence:本会话用户确认方案 3；项目 `docs/requirements.md` D18 落账（原图默认不长期囤 +「证据保留」opt-in + 客户页默认结构化/摘要）；对照 Typeless 公开口径（云端实时处理、ZDR、历史在端）与 CRM「客户页回看聊天截图」硬需求的结构性张力——已在会话中显式拆穿「两边都讲满 = 自相矛盾」。实现挂 TODOS T31；「不用于训练」仍依赖 T26 供应商条款核实。
- confidence:medium（单项目一次拍板；机理对「AI 处理敏感媒体 + 还要历史回看」类产品通用）
- claim:当隐私叙事（零/最短留存、不训练）与产品能力（云端回看原件/相册）打架时，**禁止同时承诺两者**；默认业务真相 = 用户确认后的**结构化事实（+可编辑摘要）**，敏感原件（截图/录音）默认一次性燃料；长期存原件必须**显式 opt-in（证据保留）**并配私有存储/访问控制。竞品 ZDR slogan 先读隐私政策技术含义再抄，勿抄营销面。
- suggested_landing:playbook/elicitation-protocol.md 或 architect-lens 增「隐私×功能冲突」判据半节；kit/requirement-pack 或 prd-template 加一行「敏感原件：默认丢 / 证据 opt-in / 禁双全叙事」
- rollback:revert 落地 commit
- trace:Euan 会话 2026-07-19 安全架构讨论 → 用户「选 3」；D18 落账 diff
- 状态:landed@同批(第十二次消化:elicitation-protocol §五 + prd-template 第10条)

### partial-constitution-unpause-dtable（暂停项可局部解禁：D 表划界 + 多面同步，禁止聊天默示全解）
- source:Euan-Flutter @ f0369a6 | pin v0.3.0
- evidence:本会话前「加密策略」整块 = AGENTS 暂停项；拍板后只解 **原图留存/证据 opt-in/客户页渲染口径**，**真·E2E/字段级/KMS 仍暂停**——同批改 `requirements.md` D18、`AGENTS.md`/`Claude.md` 暂停表述收窄、`TODOS.md` T31、`dashboard.html`、`docs/security-threat-model.md` §五、`docs/AGENT-HANDOFF.md`、`docs/architecture/README.md`，避免只改聊天/只改一处导致执行会话误以为加密全开。
- confidence:high（一次完整落账闭环；与项目既有 D-表动态需求协议 §9 同族）
- claim:宪法级「以后再聊」暂停项允许**局部解禁**：必须用决策表（D 编号）写清「已拍什么 / 仍禁什么」；同批同步宪法摘要、人读仪表盘、TODOS 落地项、威胁模型相关句——**禁止**会话口头同意后只改业务 docs 不改暂停表述，也禁止把一刀局部解禁当成整栈加密/状态机解禁。
- suggested_landing:playbook/truth-hierarchy 或 sop 动态需求协议段；kit/AGENTS-skeleton 暂停项行补「局部解禁 = D 表划界 + 多面同步」
- rollback:revert 落地 commit
- trace:Euan AGENTS.md §1 暂停项 → D18 同批多文件同步
- 状态:landed@同批(第十二·卡2:AGENTS-skeleton 暂停项局部解禁 + truth-hierarchy §六)
