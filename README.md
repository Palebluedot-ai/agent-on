# agent-on

**开箱可用的项目脚手架，辅助 Claude Code / Codex：一句话启动新项目、接管已开工的项目，并把每个项目踩的坑回流成方法论升级——用的项目越多，它越强。**

总目标与边界的唯一权威：[CHARTER.md](CHARTER.md)。版本账本：[CHANGELOG.md](CHANGELOG.md)（git tag 即版本）。

## 现在怎么用（三个入口 + 两个口令）

| 你的情况 | 对 Claude Code / Codex 说 | 它会做什么 |
|---|---|---|
| **开一个新项目** | 「读 `~/Projects/Agent-On/BOOTSTRAP.md`，初始化本项目」 | 先问 3 题定档（见下），再按档位问需求、播种骨架，一小时内开工 |
| **项目已经开工了** | 「读 `~/Projects/Agent-On/boot/adopt.md`，接管本项目」 | 考古现状 → 定档 → **增量补件**——不推倒、不重来、不回填历史，你已有的文件是项目自己的权威 |
| **换会话 / 换模型续跑** | 「读 `~/Projects/Agent-On/boot/session-handshake.md`，握手后继续」 | 复述总目标与锁定的下一步，你三选一确认后才开工 |

**日常口令**（在任何接入的项目里随时说）：

- 「**agent-on 结账**」——把这个项目攒下的教训（带证据的）回流进 agent-on（执行书 [boot/settlement.md](boot/settlement.md)）
- 「**agent-on 升级**」——把项目 pin 的方法论版本升到最新（同一执行书；升级永远显式，绝不静默）

## 会不会高射炮打蚊子？——先定档，装备按档发

不会，因为 BOOTSTRAP 开场先问三个问题：**① 有真实用户或真实数据吗？② 几天内搞完，还是持续迭代几周以上？③ 碰钱 / 安全 / 对外服务吗？**

| 档 | 适用 | 播种的东西 | 不播的东西 |
|---|---|---|---|
| **S 轻装** | 自用小工具、脚本、探索、玩具 | **三件套**：AGENTS-lite（十几行宪法）+ loop-notes + lock，一分钟播完 | phase 卡、progress.yaml、契约、run 台账，全免 |
| **M 标准** | 有真实用户，单人持续迭代 | 完整 AGENTS 骨架 + progress.yaml + phase 卡 + 三件套 | 契约 fixtures、并行装备（用到再加） |
| **L 全装** | 碰钱 / 数据 / 安全、多 agent 并行、长周期 | 全套（Euan CRM 同款） | — |

三条规则：

1. **拿不准取低档；允许升档，不许静默降档**——S 档项目开始碰真实数据的那天就是升档日（升档协议在 [boot/adopt.md](boot/adopt.md) §二，信号→动作写死了）。
2. **轻装也有三条底线**：完成要贴命令输出、暂停项写成禁令、外向操作先确认——这不是流程，是诚实，不分档。
3. **闭环不分档**：S 档也保留 lock + loop-notes，因为小项目的教训一样值钱，结账一样回流。

## 它解决什么（30 秒）

用 AI 写代码不难，难的是围绕它的一切：需求漂移、AI 谎报完成、多会话撞车、上下文丢失、每个新项目都要重新教一遍规矩。agent-on 把一个真实产品（Euan CRM：9 次多 agent 并行 run 零冲突、600+ 测试、生产在线）开发全程验证过的方法论，沉淀成可直接使用的脚手架。**每个模板都被真实使用过，没有一个是想象出来的。**

## 迭代闭环（为什么用的项目越多它越强）

```
①种(BOOTSTRAP/adopt 播骨架) → ②采(六类触发当场留痕,带证据) → ③结(口令「结账」,只写 intake/ 承接层)
    → ④消化(agent-on 仓会话分诊,落成具体文件修改) → ⑤发布(CHANGELOG + git tag) → ⑥升级(项目显式 bump pin)
```

Project A 踩的坑，消化进 kit 的 checklist 后，Project B 的下一次合流**自动执行，人都不用记得**。这个回路已经真实转过一圈：批三写完 17 张翻车案例卡，审查发现 5 张卡引用一个从未存在的「集成清单」——消化的答案是真的把它建出来（sop.md 外部服务集成清单，六条实证）。机制全文：[playbook/iteration-loop.md](playbook/iteration-loop.md)。

## 目录结构（五块资产）

| 目录 | 块 | 内容 |
|---|---|---|
| [BOOTSTRAP.md](BOOTSTRAP.md) + `boot/` | **Boot** | 新项目冷启动（含定档三问）· 存量项目接入书 adopt · 会话续接握手 · 结账/升级执行书 · 深挖版问卷 |
| `kit/` | **Kit** | 模板层：AGENTS 骨架 + **AGENTS-lite 轻装版**、phase 卡、派工词、审查词、合流 checklist（含 DoD 门禁）、状态文件、lock 模板、Promotion Card、四张卡 JSON Schema（schemas/）、ABDC 决策四模板（abdc/）、commit 分层、PRD / 需求澄清包 / milestone 模板 |
| `playbook/` | **Playbook** | 方法论十五篇：SOP（含外部服务集成清单）、防幻觉、二车道、模型无关化、混编经济学、多人协作、架构师透镜、前置追问、真相源治理、阶段闸门、元原则、ABDC 决策、沉淀分层、**迭代闭环**、**工作流编排**（确定性扇出防幻觉七件）+ 机制七篇（mechanisms/） |
| `bench/` | **Bench** | 翻车案例集 17 卡（[bench/cases/](bench/cases/README.md)）+ 能力探针 + 能力真相表 + 修正闭环 |
| `ledger/` | **Ledger** | run 台账模板 + jsonl 落盘规范 + audit-lint 状态机校验脚本 + Euan 九次 run 实测数据 |
| `intake/` | — | 承接层：各项目「结账」回流的落点（目录即仪表盘，`ls` 一眼见积压） |
| `snapshot/` `legacy/` | — | 决策快照（工具定义、融合地图）· 前身仓考古层 |

## 常见问题

**Q：简单项目也要走全流程吗？**
不。定档三问把你路由到 S 轻装：三件套一分钟播完，phase 卡 / 状态文件 / 契约全免。唯一不可省的是「完成贴证据」这类底线——那不是流程，是诚实。

**Q：我有项目已经写了一半，怎么接入？**
说「读 boot/adopt.md 接管本项目」。它先考古（读你现有的 README / 规则 / 近期 commit，产出一句话现状 + 能力真相表），再定档，然后**只补缺的件**。你已有的规则文件不会被替换（有则合并，没有才新建），历史不回填。

**Q：方法论更新了，我的项目会被动改吗？**
不会。项目 pin 具体版本（tag + commit），升级是显式口令；只有 major（不动手会坏）需要动你项目里的文件，而且以 diff 提案呈报、你逐条批准。

**Q：口令是显性的吗？每次都要念吗？**
只显性三次：每台机器 clone 一次仓、每个项目接入时说一句口令（要问你定档和需求，显性是设计）、之后只剩「结账」「升级」两个口令。日常开发零操作——接入时种下的 AGENTS.md 会被 Claude Code / Codex 自动读取，规则自动生效。

**Q：换电脑 / 别人的电脑路径不一样怎么办？会自动建文件夹吗？**
不会自动建——它是个 git 仓，`git clone` 到**任意路径**都行，口令里的路径换成你 clone 的位置即可（仓内全部相对引用，位置无关）。路径只锚在两处填空位：你全局 CLAUDE.md 的路由节、各项目 lock 文件的「本地路径」行。将来 skill 化可以消掉路径问题（Backlog，dogfood 后决定）。

**Q：Codex 也能用吗？**
能。规则载体是 AGENTS.md（Claude Code 与 Codex 的共同标准），CLAUDE.md 只是一行指针。模型也可换——脚手架的「保费」旋钮按模型能力调档（playbook/model-playbook.md）。

**Q：agent-on 自己怎么进化？**
你在项目里说「agent-on 结账」，带证据的教训回流进 intake/，消化会话把它落成 playbook / kit 的具体文件修改并记 CHANGELOG、打 tag——每次消化必须产出至少一处文件改动，只读不改算消化失败（防「文档写了没人执行」的二代翻车）。

**Q：为什么信这套东西？**
它不是设计出来的，是三代试错演化出来的：一代想自建编排引擎（5200+ 行设计物，引擎目录 0 行代码）、二代用锁堵漂移（防漂移框架自己漂进文档洁癖）、三代在真实产品上实战出 Loop Engineering。主线一句话：**从「锁住 AI」到「和 AI 对齐」**。完整家谱与裁决：[snapshot/2026-07-07-fusion-map.md](snapshot/2026-07-07-fusion-map.md)。

## 与其他工具的关系

不冲突，分工明确：GStack / Superpowers 这类强 skill 管**某个环节怎么做**（评审、QA、发布、调试）；agent-on 管**项目怎么启动、怎么推进、怎么不漂、怎么反哺**。相关外部参照：[lipingtababa/agents-zone-skillset](https://github.com/lipingtababa/agents-zone-skillset)。

## 状态与路线

- **v0.2（当前）**：五块骨架、三代资产合流、Bench 案例集 17 卡、迭代闭环六站机制、S/M/L 档位路由、存量项目接入书
- **v0.3 门槛**：Euan 仓内 agent-on/ 倒仓首结账跑通（闭环验收测试）+ 一个新项目 BOOTSTRAP dogfood 全流程
- **v1.0**：两个以上项目跑完完整闭环（Project A 的坑确实变成了 Project B 的免疫）
