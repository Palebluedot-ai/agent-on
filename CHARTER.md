# CHARTER — agent-on 项目宪章

> 职责边界：本文件是 agent-on 总目标与边界的唯一权威。其他文档（README / BOOTSTRAP / playbook）与本文件冲突时，以本文件为准；改本文件 = 改产品方向，需用户拍板。
> 拍板链：2026-07-01 工具定义（[snapshot/agent-on-tool-definition.md](snapshot/agent-on-tool-definition.md)）→ 2026-07-07 融合裁决（[snapshot/2026-07-07-fusion-map.md](snapshot/2026-07-07-fusion-map.md)）→ 2026-07-07 总目标确认（本文件，用户拍板）→ 2026-07-07 迭代闭环拍板（用户：通过项目不断迭代 agent-on 自身；2026-07-08 落盘 [playbook/iteration-loop.md](playbook/iteration-loop.md)）。

## 总目标（一句话）

**agent-on 是一个开箱可用的项目脚手架，辅助 Claude Code / Codex 启动和推进新项目：clone 即得经过实战验证的方法论与启动机制，不再每个新项目都从零铺垫规则。**

## 「开箱可用」的定义（三条承诺，可验收）

1. **零铺垫启动**：新项目里用户只说一句「读 agent-on 的 BOOTSTRAP.md 并初始化本项目」，AI 就能完成项目骨架、规则、状态文件的初始化——从 clone 到第一张 phase 卡开工 ≤ 1 小时。
2. **方法论内置**：Loop Engineering 的不变量（契约先行并行 / 单一状态写者 / 自包含 phase 卡 / TDD + 完成贴证据 / 对抗式审查）随骨架直接生效，用户不需要先懂原理。
3. **工具与模型无关**：Claude Code 和 Codex 都能吃（规则载体 = AGENTS.md 跨工具标准，CLAUDE.md 只做一行指针）；模型可换（脚手架「保费」旋钮按模型能力调档，见 playbook/model-playbook.md）。
4. **越用越强**（2026-07-07 拍板）：每个项目的翻车与心得经「agent-on 结账」回流、消化成方法论的具体文件修改（版本化、可审计），Project A 的坑变成 Project B 的免疫——机制见 playbook/iteration-loop.md 六站闭环。

## 服务对象

先是 Chao 自己的新项目（dogfood）；产品化后给任何用 AI 协作开发的个人 / 小团队——尤其是非全职工程师背景的 builder。

## 五块资产

| 块 | 职责 |
|---|---|
| **Boot** | 冷启动：BOOTSTRAP.md 入口 + 新场景问卷 + 会话续接握手 |
| **Kit** | 模板层：宪法骨架 / phase 卡 / 派工词 / 审查词 / 合流 checklist / 状态文件 / 契约 schema |
| **Playbook** | 方法论：SOP、防幻觉、二车道、混编经济学、多轨决策、真相源治理、机制七篇等 |
| **Bench** | 能力探针 + 翻车案例集 + 修正闭环（模型会过时，案例集只增值） |
| **Ledger** | run 数据台账：每次编排的时长 / 冲突 / 返工 / 成本 |

## 边界（不做的事）

- **不做编排运行时 / 框架**——一代仓 5200 行文档 0 行落地代码的教训：工具能力已覆盖时，造引擎不如立纪律
- **不与 GStack / Superpowers 等强 skill 冲突**——agent-on 管「项目怎么启动和推进」，不管「某个环节怎么做」
- **不先做平台**——先文档形态跑通 dogfood，再考虑 skill 化 / 自动化（先固化规则 → 再抽象模板 → 最后自动化）
- **不为通用牺牲实战有效性**——每个模板必须来自真实项目的真实使用，没有一个是想象出来的

## 成功标准

- 第二个新项目启动明显更快：clone → 第一张 phase 卡开工 ≤ 1 小时
- 文档更少漂移、模型更少幻觉、沟通更少来回（继承 kickoff-os 愿景，依然成立）
- Euan-Flutter 之外至少 1 个真实项目全流程走完——dogfood 通过才算 v0.3

## 自举

本仓的开发遵守本仓的方法论：决策入 snapshot、完成贴证据、单一权威文件、卡片化推进。
