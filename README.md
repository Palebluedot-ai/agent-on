# agent-on

**开箱可用的项目脚手架，辅助 Claude Code / Codex 启动和推进新项目：clone 即得经过实战验证的方法论，不再每个新项目都从零铺垫规则。**

总目标与边界的唯一权威：[CHARTER.md](CHARTER.md)。

## 30 秒了解

用 AI 写代码不难，难的是围绕它的一切：需求漂移、AI 谎报完成、多会话撞车、上下文丢失、每个新项目都要重新教一遍规矩。

agent-on 把一个真实产品（Euan CRM：9 次多 agent 并行 run 零冲突、600+ 测试、生产在线）开发全程验证过的方法论，沉淀成可直接使用的脚手架。**每个模板都被真实使用过，没有一个是想象出来的。**

## 快速开始

在你的新项目目录里，对 Claude Code 或 Codex 说一句：

> 读 ~/Projects/agent-on/BOOTSTRAP.md，按它初始化本项目。

AI 会问你 6 个问题，然后搭好全部骨架（项目宪法、状态文件、phase 卡体系），并把纪律讲给你听。从 clone 到第一张卡开工，一小时以内。

## 结构（五块资产）

| 目录 | 块 | 内容 |
|---|---|---|
| [BOOTSTRAP.md](BOOTSTRAP.md) + `boot/` | **Boot** | 新项目冷启动指令 + 深挖版新场景问卷 + 会话续接握手（第 N 次接续怎么开场） |
| `kit/` | **Kit** | 模板层：AGENTS 骨架、phase 卡、派工词、审查词、合流 checklist（含 DoD 门禁）、状态文件、四张卡 JSON Schema（schemas/）、ABDC 决策四模板（abdc/）、commit 分层、PRD / 需求澄清包 / milestone 模板 |
| `playbook/` | **Playbook** | 方法论十三篇：SOP、防幻觉、二车道、模型无关化、混编经济学、多人协作、架构师透镜、前置追问、真相源治理、阶段闸门、元原则、ABDC 决策、沉淀分层 + 机制七篇（mechanisms/） |
| `bench/` | **Bench** | 能力探针 + 能力真相表（防「假装已实现」）+ 修正闭环 + 翻车案例集（批三结构化） |
| `ledger/` | **Ledger** | run 台账模板 + jsonl 落盘规范 + audit-lint 状态机校验脚本 + Euan 九次 run 实测数据 |
| `snapshot/` | — | 本仓决策快照（工具定义、融合地图等） |
| `legacy/` | — | 前身仓考古层（kickoff-os 原文归档） |

## 它从哪来（三代演化，2026-04 → 07）

这套东西不是设计出来的，是三代试错演化出来的，主线一句话：**从「锁住 AI」到「和 AI 对齐」**。

1. **一代（4-5 月）· Harness Engineering**：想自建编排运行时（Router / Lane / Policy 注入）。5200 行设计文档，0 行落地代码——教训：工具能力已覆盖时，造引擎不如立纪律。活下来的是契约 schema、DoD 门禁、「默认隔离 + 单一写者」原则。
2. **二代（5-6 月）· 沟通治理**：用锁口令和状态机堵漂移。防漂移框架自己漂进了文档洁癖（14 篇文档零实现）——教训：治理文档 ≠ 治理执行。活下来的是真相源分层仲裁、「申请切阶段 ≠ 切阶段」的三态语义。
3. **三代（6-7 月）· Loop Engineering 实战**：在真实产品上跑出来的完整方法论——契约先行并行、自包含 phase 卡、TDD + 完成贴证据、对抗式审查，9 次并行 run 零冲突验证。

完整家谱与融合裁决：[snapshot/2026-07-07-fusion-map.md](snapshot/2026-07-07-fusion-map.md)。

## 与其他工具的关系

不冲突，分工明确：GStack / Superpowers 这类强 skill 管**某个环节怎么做**（评审、QA、发布、调试）；agent-on 管**项目怎么启动、怎么推进、怎么不漂**。相关外部参照：[lipingtababa/agents-zone-skillset](https://github.com/lipingtababa/agents-zone-skillset)（Harness Engineering Playbook 的参照实现）。

## 状态与路线

- **v0.2（当前）**：批一批二完成——五块骨架 + 三代资产迁入 + CHARTER / BOOTSTRAP 就位 + 前身仓精选移植（20 文件：五篇方法论、四张卡 schema、audit-lint、ABDC 四模板、握手/问卷/真相表/修正闭环/DoD 门禁）
- 批三：Bench 案例集结构化
- 批四：对外收尾（旧仓归档标头、push）
- **v0.3 门槛**：Euan 之外至少 1 个真实项目 dogfood 全流程走完
