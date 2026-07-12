# agent-on

**开箱可用的项目脚手架，辅助 Claude Code / Codex：一句话启动新项目、接管已开工的项目，并把每个项目踩的坑回流成方法论升级——用的项目越多，它越强。**

*Agent-on is a ready-to-use project scaffold for AI coding agents (Claude Code / Codex / Grok): bootstrap a new project with one sentence, adopt an in-flight one without rebuilding, and flow every lesson back into the methodology — the more projects use it, the stronger it gets.*

总目标与边界的唯一权威：[CHARTER.md](CHARTER.md)。版本账本：[CHANGELOG.md](CHANGELOG.md)（git tag 即版本）。

## 指令速查 · Command Reference（中英）

**三条触发路径等价 / Three equivalent trigger routes**：中文口令（三家工具通用 · Chinese voice commands, all tools）· `/agent-on <cmd>`（Claude Code）· `$agent-on <cmd>`（Codex，经 `~/.agents/skills`）——背后是**同一份内核** [skill/SKILL.md](skill/SKILL.md)（one shared kernel）。

| 场景 Scenario | 中文口令 Say (CN) | 命令 Command | What it does (EN) |
|---|---|---|---|
| **新项目初始化** | 「读 `~/Projects/Agent-On/BOOTSTRAP.md`，初始化本项目」 | `/agent-on init` | Bootstrap a new project: 3 tier questions (S/M/L) → seed the skeleton, start within the hour |
| **接管半路项目** | 「接管本项目」 | `/agent-on adopt` | Adopt an in-flight project: archaeology → tier → incremental fit; never rebuild, never backfill history |
| **换会话续跑** | 「握手后继续」 | `/agent-on handshake` | Resume in a fresh session: 3-step re-alignment (goal → current phase → your pick) before any work |
| **沉淀回流** | 「**agent-on 结账**」 | `/agent-on settle` | Settle: flow evidence-backed lessons from this project into agent-on's `intake/` |
| **消化落地** | 「消化」（须在本仓会话） | `/agent-on digest` | Digest: triage intake cards, land them as concrete file changes, tag（agent-on repo session only） |
| **升级 pin** | 「**agent-on 升级**」 | `/agent-on upgrade` | Upgrade: explicitly bump the project's pinned methodology version — never silent |
| **整理想法** | 「整理想法」 | — | Organize the thoughts inbox（速记区 → dated, categorized entries with dispositions） |
| **更新仪表盘** | 「更新仪表盘」 | — | Redraw `dashboard.html` from truth sources — never hand-edit numbers（M/L tiers） |

**入口怎么选 / Which entry?** 全新项目 → `init`；已开工但从没接入过 agent-on → `adopt`（**不是** handshake）；接入过的项目每次换会话 → `handshake`。One-time per project: init/adopt. Every new session after that: handshake.

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

Project A 踩的坑，消化进 kit 的 checklist 后，Project B 的下一次合流**自动执行，人都不用记得**。这个回路在 v0.2 自建过程中内部验证过一次：批三写完 17 张翻车案例卡，审查发现 5 张卡引用一个从未存在的「集成清单」——消化的答案是真的把它建出来（sop.md 外部服务集成清单，六条实证）。首个下游项目的真实结账（Euan 倒仓）是 v0.3 门槛件，不算内部验证那一圈。机制全文：[playbook/iteration-loop.md](playbook/iteration-loop.md)。

## 目录结构（五块资产）

| 目录 | 块 | 内容 |
|---|---|---|
| [BOOTSTRAP.md](BOOTSTRAP.md) + `boot/` | **Boot** | 新项目冷启动（含定档三问）· 存量项目接入书 adopt · 会话续接握手 · 结账/升级执行书 · 深挖版问卷 |
| `kit/` | **Kit** | 模板层：AGENTS 骨架 + **AGENTS-lite 轻装版**、phase 卡、派工词、审查词、合流 checklist（含 DoD 门禁）、状态文件、lock 模板、Promotion Card、四张卡 JSON Schema（schemas/）、ABDC 决策四模板（abdc/）、commit 分层、PRD / 需求澄清包 / milestone 模板、**项目仪表盘 dashboard**（M/L）、**想法收集箱 thoughts-and-ideas**（全档） |
| `playbook/` | **Playbook** | 方法论十五篇：SOP（含外部服务集成清单）、防幻觉、二车道、模型无关化、混编经济学、多人协作、架构师透镜、前置追问、真相源治理、阶段闸门、元原则、ABDC 决策、沉淀分层、**迭代闭环**、**工作流编排**（确定性扇出防幻觉七件）+ 机制七篇（mechanisms/） |
| `bench/` | **Bench** | 翻车案例集 23 卡（[bench/cases/](bench/cases/README.md)，Euan 实战 18 + IPONews 起累积中 + 前身仓标本 2 + 消化增补）+ 能力探针 + 能力真相表 + 修正闭环 |
| `ledger/` | **Ledger** | run 台账模板 + Euan 九次 run 实测数据（**散文台账，真实跑过**）· jsonl 落盘规范 + audit-lint 状态机校验脚本（**机件，L 档多 agent 编排时启用，尚未在真实项目验证**）|
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
只显性三次：每台机器 clone 一次仓、每个项目接入时说一句口令（要问你定档和需求，显性是设计）、之后只剩四个日常口令（结账 / 升级 / 整理想法 / 更新仪表盘）。日常开发零操作——接入时种下的 AGENTS.md 会被 Claude Code / Codex / Grok 自动读取，规则自动生效。触发路径见顶部「指令速查」：中文口令与 `/agent-on`、`$agent-on` 三路等价，同一份内核。个别界面不识别斜杠命令（显示在列表里但敲了没反应，或须从补全菜单选中）——直接说中文口令即可。

**Q：换电脑 / 别人的电脑路径不一样怎么办？会自动建文件夹吗？**
不会自动建——它是个 git 仓，`git clone` 到**任意路径**都行，口令里的路径换成你 clone 的位置即可（仓内全部相对引用，位置无关）。路径只锚在三处填空位：全局路由（agent-memory 的共用真相 AGENT.md）、各项目 lock 文件的「本地路径」行、两条 skill symlink（见换机步骤）。

**Q：这台电脑坏了，怎么恢复？**
换机三步：① `git clone` 本仓到任意路径（GitHub 远端就是异地副本）；② 两条 skill symlink 挂回同一份内核——`ln -s <仓路径>/skill ~/.claude/skills/agent-on`（Claude `/agent-on`）+ `ln -s <仓路径>/skill ~/.agents/skills/agent-on`（Codex `$agent-on`）；③ 全局口令路由随 agent-memory 仓（个人配置云同步，含共用真相 AGENT.md）的 `setup.sh` 自带，不用手配。各项目的 lock / loop-notes 在项目自己的仓里，跟项目走，与本仓互不牵连。

**Q：Codex 也能用吗？**
能，且项目侧零适配——AGENTS.md 本来就是两家共同标准，lock / 模板 / 口令全是文件。机器侧接入两件（一台机器一次，详见 [codex/README.md](codex/README.md)）：① 把 `codex/AGENTS-global-snippet.md` 并入 `~/.codex/AGENTS.md`（全局口令路由）② `ln -s <仓路径>/skill ~/.agents/skills/agent-on`——之后 `$agent-on` 与 Claude 的 `/agent-on` 吃**同一份内核**。Claude Code 独有的子代理编排在 Codex 下按 playbook/workflow-orchestration.md §四退化为手工纪律，闭环全部照跑。模型也可换——「保费」旋钮按模型能力调档（playbook/model-playbook.md）。Grok 更简单：全局规则原生认 AGENT.md 文件名，共用真相一条 symlink 即注入（中文口令直接可用）。

**Q：agent-on 自己怎么进化？**
你在项目里说「agent-on 结账」，带证据的教训回流进 intake/，消化会话把它落成 playbook / kit 的具体文件修改并记 CHANGELOG、打 tag——每次消化必须产出至少一处文件改动，只读不改算消化失败（防「文档写了没人执行」的二代翻车）。

**Q：为什么信这套东西？**
它不是设计出来的，是三代试错演化出来的：一代想自建编排引擎（5200+ 行设计物，引擎目录 0 行代码）、二代用锁堵漂移（防漂移框架自己漂进文档洁癖）、三代在真实产品上实战出 Loop Engineering。主线一句话：**从「锁住 AI」到「和 AI 对齐」**。完整家谱与裁决：[snapshot/2026-07-07-fusion-map.md](snapshot/2026-07-07-fusion-map.md)。

## 与其他工具的关系

不冲突，分工明确：GStack / Superpowers 这类强 skill 管**某个环节怎么做**（评审、QA、发布、调试）；agent-on 管**项目怎么启动、怎么推进、怎么不漂、怎么反哺**。相关外部参照：[lipingtababa/agents-zone-skillset](https://github.com/lipingtababa/agents-zone-skillset)。

## 状态与路线

- **v0.2**：五块骨架、三代资产合流、Bench 案例集、迭代闭环六站机制、S/M/L 档位路由、存量项目接入书
- **v0.3 ✅ 达成（2026-07-09）**：Euan 倒仓首次结账 + 首次消化跑通，闭环真转过一圈，已封 `v0.3.0`（冻结令随之解除）
- **v0.4（进行中）**：① 项目仪表盘 + 想法收集箱两个默认件（2026-07-10 已开发，待新项目验证）② 一个新项目 BOOTSTRAP 全流程 dogfood
- **v1.0**：两个以上项目跑完完整闭环（Project A 的坑确实变成了 Project B 的免疫）

## 一句话术语（给非工程师）

- **pin**：项目锁定的 agent-on 版本（tag+commit），像合同注明用哪版图纸——升级永远显式，绝不被动变
- **fixture**：接口两侧共用的冻结样例数据（含排序/空值等语义），并行开发时当裁判用
- **worktree**：git 的「同一个仓开多个工作目录」，多个 AI 并行时各改各的目录，物理上撞不了车
- **L1–L4（候选层）**：教训的沉淀深度——L1 单次复盘 / L2 可复用知识 / L3 流程规则 / L4 用户稳定偏好（全文 playbook/memory-layering.md）
- **结账 / 消化**：结账 = 项目把带证据的教训回流进本仓 intake/；消化 = 本仓会话把它落成正文修改并发版
