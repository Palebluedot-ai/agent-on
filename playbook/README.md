# Playbook — 方法论层索引

> 职责边界：本目录是 agent-on 的方法论正文（「为什么这么做」）。模板在 `../kit/`，冷启动入口在 `../BOOTSTRAP.md`。
> 「我们的『酷』更多的是在对话过程中想法的沉淀，而不是 skills 的数量。」——2026-04-27，agent-on 的初心（全文见 [meta-principles.md](meta-principles.md) 引言）。

## 八篇方法论（三代 Euan 实战迁入，2026-07-07 批一）

| 文件 | 核心论点 |
|---|---|
| [sop.md](sop.md) | 完整开发 SOP：需求整理 → MRD/PRD → 评审 → 切片 → 细颗粒执行 → 并行 → 审查 QA → 沉淀 |
| [anti-hallucination.md](anti-hallucination.md) | 防幻觉 12 机制：幻觉 = f(信息缺口 × 自由度)；接口/状态/完成/决策四类幻觉的机械封堵 |
| [freedom-vs-discipline.md](freedom-vs-discipline.md) | 二车道：Explore（错误不算错误）× Ship（全纪律），两道不许串；品味前置四层 |
| [orchestration-future.md](orchestration-future.md) | 模型无关化：脚手架分「保费」（随模型变强调低）与「资产」（模型越强越值钱）；prompt 易挥发，文件系统持久 |
| [model-playbook.md](model-playbook.md) | 混编经济学：贵模型买判断（~15%），便宜模型配七脚手架买执行（~70%）；减少返工四杠杆 |
| [multi-contributor-protocol.md](multi-contributor-protocol.md) | 多人协作：agent 轨道 ↔ 人类 contributor 同构；单写者换写者不换位置 |
| [architect-lens.md](architect-lens.md) | 架构师六维透镜：域边界 → 真相源 → 信任钱流 → 生命周期 → 故障半径 → 演进自由 |
| [elicitation-protocol.md](elicitation-protocol.md) | 前置追问协议：偏好缺口 = 幻觉第五类；2×2 判据（偏好方差 × 返工成本） |

## 机制七篇（mechanisms/）

Loop Engineering 的执行机制定义：自包含文件系统、三固化剂、审查者规则与触发流、监控与 Error Signal、文件系统归档。这七篇是 kit 模板背后的规则原文。

## 五篇前身仓移植（2026-07-07 批二，源流与裁决见 [../snapshot/2026-07-07-fusion-map.md](../snapshot/2026-07-07-fusion-map.md) §5）

| 文件 | 核心论点 | 源 |
|---|---|---|
| [truth-hierarchy.md](truth-hierarchy.md) | 单一真相源不够，还要层级（Canonical/承接/举证）+ 仲裁次序 + 跨仓 pin 映射 | 二代 + 支线 |
| [phase-gates.md](phase-gates.md) | 申请切阶段 ≠ 切阶段：三态语义 + 6 个 block 理由码 + 冻结物双步解锁 fail-closed | 二代 |
| [meta-principles.md](meta-principles.md) | 方法论的方法论：self-dogfooding / 治理进 Git / 反思必须产出协议升级 / 单 agent 优先 / 不许「感觉更好」过门禁 / 失败预定义降级 | 三仓合成 |
| [abdc-decision.md](abdc-decision.md) | 多轨决策：三轨论证（反例义务）→ 加权评审（强制记异议）→ 最小可证伪实验（三态判定+熔断） | 支线 |
| [memory-layering.md](memory-layering.md) | 沉淀分层 L0–L4：什么东西该沉到五块的哪一块；高频失败优先沉淀为 anti-pattern | 支线 |
