# intake — hk-sfc-licensees · 2026-08-01 首次结账

> 项目端会话只写本文件，不 add/commit/push agent-on。  
> pin: agent-on @ v0.5.1

### local-dev-constraint-is-not-product-endstate
- source: hk-sfc-licensees @ pending-main-push | pin v0.5.1
- evidence: docs/product/cloud-website-architecture.md §0; requirements D3 vs D20 纠偏叙事; session snapshot docs/snapshots/2026-08-01-174803.md
- confidence: high
- claim: 把「crawl 必须本机/常驻、礼貌限速」写成「产品只能本机安装」是错误合并；交付物终局与生产线运行时必须分栏写进 requirements/AGENTS，避免后续选型全盘本机化。
- suggested_landing: playbook 产品/运行时双层 + kit/AGENTS-skeleton 一句「runtime ≠ product surface」
- rollback: revert 落地 commit
- trace: snapshot 2026-08-01-174803 Key Decisions D19/D20
- 状态: pending

### loop-cadence-is-not-deadline
- source: hk-sfc-licensees @ pending-main-push | pin v0.5.1
- evidence: user asked whether 30m loop means finish all code in 30m; scheduler prompt used interval as fire spacing with stop condition separate
- confidence: medium
- claim: 向用户说明 recurring loop 时必须显式区分「开火间隔」与「任务截止/完成条件」，禁止让人以为 cadence = 工时预算。
- suggested_landing: kit 或 skill 里 /loop 用户可见说明半句
- rollback: revert 落地 commit
- trace: conversation 2026-08-01 loop clarification
- 状态: pending

### do-not-block-on-competitor-private-stack
- source: hk-sfc-licensees @ pending-main-push | pin v0.5.1
- evidence: docs/product/competitor-thesfcnetwork-teardown.md §6; docs/product/tech-selection-lock.md §1 — public surface cannot prove DB engine; selection proceeded on product constraints
- confidence: high
- claim: 对标竞品技术栈时，合法公开面挖不到的引擎名不得阻塞本项目选型；用可验证的产品约束 + 可逆分层锁定。
- suggested_landing: playbook/anti-hallucination 或 tech-selection 短则
- rollback: revert 落地 commit
- trace: teardown §6 confidence table
- 状态: pending

### scheduler-stop-must-match-real-product-scope
- source: hk-sfc-licensees @ loop task 019fbce71e3a | pin v0.5.1
- evidence: detached fire marked DONE after docs+L1 status and deleted scheduler while W1 cloud impl and L4 network still open; user later asked for more implementation
- confidence: medium
- claim: 自动 loop 的 stop condition 必须对齐用户心中的「做完」范围；若 stop 只覆盖文档/局部 UI，须在回报里写明「云上线/数据轨未完成」且默认不自删，或拆成命名清晰的多 loop。
- suggested_landing: boot 或 skill loop 模板 stop condition checklist
- rollback: revert 落地 commit
- trace: fire DONE message track1–3 vs user follow-up W1 skeleton request
- 状态: pending
