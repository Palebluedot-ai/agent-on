# Playbook — 方法论层索引

> 职责边界：本目录是 agent-on 的方法论正文（「为什么这么做」）。模板在 `../kit/`，冷启动入口在 `../BOOTSTRAP.md`。全部内容从 Euan-Flutter 实战沉淀迁入（2026-07-07 批一）。

## 八篇方法论

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

## 批二将新增

truth-hierarchy（真相源三层仲裁）、phase-gates（阶段闸门三态语义 + block 理由码）、meta-principles（self-dogfooding / 治理进 Git / 反思产出协议升级）、ABDC 多轨决策、Soul Memory L0–L4 记忆分层。来源与裁决见 [../snapshot/2026-07-07-fusion-map.md](../snapshot/2026-07-07-fusion-map.md) §5。
