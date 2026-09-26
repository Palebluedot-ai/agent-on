<div align="center">

# agent-on

**给 AI 写代码配一套「项目制度」。**

一句话开工新项目，半路项目也能无痛接入；每个项目踩过的坑，回流成下一个项目的免疫。<br>
Claude Code · Codex · Grok 通用，项目侧零适配。

[![release](https://img.shields.io/github/v/tag/Palebluedot-ai/agent-on?label=release&sort=semver)](https://github.com/Palebluedot-ai/agent-on/tags)
![tools](https://img.shields.io/badge/works%20with-Claude%20Code%20%7C%20Codex%20%7C%20Grok-6f42c1)
![cli](https://img.shields.io/badge/CLI-Rust-dea584)

[它解决什么](#它解决什么) · [核心能力](#核心能力) · [5 分钟装机](#给朋友的-5-分钟装机claude--codex--grok) · [口令速查](#日常怎么用口令速查) · [为什么信它](#为什么信它) · [FAQ](#常见问题)

</div>

*Agent-on is a ready-to-use project scaffold for AI coding agents (Claude Code / Codex / Grok): bootstrap a new project with one sentence, adopt an in-flight one without rebuilding, and flow every lesson back into the methodology — the more projects use it, the stronger it gets.*

总目标与边界的唯一权威：[CHARTER.md](CHARTER.md)。版本账本：[CHANGELOG.md](CHANGELOG.md)（git tag 即版本）。**当前推荐 pin：`v0.25.1`。**

---

## 它解决什么

让 AI 写出一段能跑的代码，今天已经不难。难的是**围绕代码的那一圈事**——而这一圈，正是 AI 最容易翻车的地方：

| 你大概遇到过 | 背后的真问题 |
|---|---|
| AI 说「已完成、测试通过」，一跑是红的 | 没有「完成 = 贴证据」的硬规矩，AI 可以口头交差 |
| 聊着聊着需求变了味，做出来的东西不是你要的 | 需求没有单一权威文件，每轮对话都在悄悄改题 |
| 开两三个会话并行干活，最后合并时互相覆盖 | 多会话之间没有文件边界，也没有合流顺序 |
| 换个窗口 / 隔天再开，AI 忘了做到哪、为什么这么做 | 状态和决策只活在聊天记录里，没落进仓库 |
| 每开一个新项目，都要把同样的规矩重新教一遍 | 上个项目的教训没有地方沉淀，更没有渠道回流 |

**agent-on 不写你的业务代码，也不替你选框架。它管的是「项目怎么启动、怎么推进、怎么不漂、怎么结账」这一层制度。** 装好之后，它以一份 `AGENTS.md` 和几件模板的形式住进你的项目，Claude Code / Codex / Grok 开会话时自动读取，规则自动生效。

## 核心能力

**🚀 一句话开工**
新项目里说「初始化本项目」。AI 先问三个问题给项目定档，然后播种骨架、规则、状态文件——轻量项目一分钟就绪，完整档一小时内第一张任务卡开工。

**🧭 半路项目也能接**
已经写了一半？说「接管本项目」。它先考古（读你现有的 README、规则、近期提交），再定档，**只补缺的件**：你已有的规则文件合并不替换，历史不回填，不推倒重来。

**⚖️ 装备按档发，不拿高射炮打蚊子**
S 轻装 / M 标准 / L 全装三档。自用小脚本只发三件套；碰钱、碰真实用户数据、多 agent 并行才上全套。拿不准就取低档，允许升档，不许悄悄降档。

**✅ 完成 = 贴证据**
任何「做完了」都必须附上验证命令的实际输出。「应该没问题」不算完成。这一条不分档，S 档也有。

**🔀 多会话并行不撞车**
配套 Rust CLI（`agent-on`）提供 worktree 控制面：登记每条执行轨的文件边界、看全场状态；commit 闸只拦真冲突——同一个文件在另一棵工作树里也没提交、而且最近还有人在改。合流有队列、有波次，合并有值守和审计账本。

**♻️ 越用越强**
项目里说一句「agent-on 结账」，带证据的教训回流进本仓；本仓消化成方法论的具体修改、打 tag 发版；其他项目「agent-on 升级」拿到免疫。**Project A 踩的坑，变成 Project B 的默认检查项。**

**🔌 工具与模型无关**
规则载体是跨工具标准 `AGENTS.md`，模板、口令全是纯文件。Claude Code、Codex、Grok 吃同一份内核；模型换了，按能力调整「保费」档位即可。

## 30 秒看它怎么工作

```
你：初始化本项目

AI：先定档，三个问题——
    ① 有真实用户或真实数据吗？
    ② 几天搞完，还是持续迭代几周以上？
    ③ 碰钱 / 安全 / 对外服务吗？

你：没有真实用户；想长期做；不碰钱。

AI：定 M 标准档。已播种：
    · AGENTS.md（项目宪法：完成贴证据、暂停项写成禁令、外向操作先确认…）
    · progress.yaml（唯一状态真相）
    · phase 卡模板 + 第一张卡
    · loop-notes（教训速记）+ agent-on lock（锁定方法论版本）
    下一步：填第一张 phase 卡的验收标准，就可以开工。
```

之后日常开发**零额外操作**。换会话说「握手后继续」，AI 先对齐目标、当前阶段、你的选择，再动手；项目告一段落说「agent-on 结账」，把这个项目学到的东西送回去。

## 适合谁

- **用 AI 协作开发的个人和小团队**，尤其是**不是全职工程师出身的 builder**——你懂业务、懂要什么，但不想每次都从零教 AI 规矩
- 已经被「AI 谎报完成」「多会话互相覆盖」「上下文丢失」坑过的人
- 手上同时有好几个项目，希望一个项目的教训能自动惠及其他项目的人

**不适合**：想要一个编排运行时 / agent 框架的人。agent-on 刻意不造引擎——它的前身造过一次（5200+ 行设计文档，引擎代码 0 行），结论是：工具能力已经够了的时候，立纪律比造引擎管用。

---

## 给朋友的 5 分钟装机（Claude · Codex · Grok）

### 0. 唯一下载源 + 默认目录

| 用途 | 地址 / 路径 |
|---|---|
| **GitHub（唯一官方源）** | https://github.com/Palebluedot-ai/agent-on |
| **推荐 pin** | **`v0.25.1`** |
| **不是** | npm、Claude 官方总商店、App Store |

| OS | 默认工作仓（setup 会放到这里） |
|---|---|
| macOS / Linux | `~/.local/share/agent-on` |
| Windows | `%LOCALAPPDATA%\agent-on` |

本机配置写在 `~/.config/agent-on/config.json` 的 `work_root`；环境变量 `AGENT_ON_ROOT` 可覆盖。

### 1. 一键 setup（三家共用，推荐先跑）

依赖：[Rust](https://rustup.rs) + git。

```bash
git clone https://github.com/Palebluedot-ai/agent-on.git /tmp/agent-on-src
cd /tmp/agent-on-src
git checkout "$(git tag --sort=-v:refname | head -1)"   # 切到最新发布版
cargo install --path cli --force
agent-on setup --with-plugins --with-symlinks
```

`agent-on setup` 会：把工作仓 clone / 更新到默认目录 → checkout 推荐 pin → 写 config →（可选）装 Claude / Codex plugin 与 skill symlink → 跑 `doctor` 自检。已有工作仓只想登记：`agent-on setup --config-only --work-root <路径>`。细节见 [scripts/README.md](scripts/README.md)。

### 2. 按工具补入口

**Claude Code**（setup 带了 `--with-plugins` 可跳过）

```bash
claude plugin marketplace add Palebluedot-ai/agent-on
claude plugin install agent-on@agent-on
```

新开会话或 `/reload-plugins`，然后 `/agent-on init` 或直接说「初始化本项目」。

**Codex CLI**

```bash
codex plugin marketplace add Palebluedot-ai/agent-on
codex plugin install agent-on@agent-on
```

再把 [codex/AGENTS-global-snippet.md](codex/AGENTS-global-snippet.md) 并入 `~/.codex/AGENTS.md`。开工：`$agent-on init` 或「初始化本项目」。首次运行时在 `/hooks` 里检查并信任 guard hook；agent-on 不会静默改写 `~/.codex/`。详见 [codex/README.md](codex/README.md)。

**Grok**

没有 plugin 商店。跑完 setup 后，让全局规则（`AGENT.md`）含 agent-on 路由：新项目读 `BOOTSTRAP.md`、结账读 `boot/settlement.md`、入口 `skill/SKILL.md`。直接说中文口令即可。诚实边界：Grok 多半没有 PreToolUse guard，但装在项目里的 Git hooks 与宿主无关，照样生效。

### 3. 验一下

```bash
agent-on doctor
```

报出 `read_root` / `work_root` 与 hook 执行面都在，就装好了。卡住了：找推荐人，或开 [GitHub Issue](https://github.com/Palebluedot-ai/agent-on/issues)。

---

## 日常怎么用（口令速查）

三条触发路径**结果等价**：中文口令（三家通用）· `/agent-on <cmd>`（Claude Code）· `$agent-on <cmd>`（Codex），背后是同一份内核 [skill/SKILL.md](skill/SKILL.md)。

| 你想… | 说 | Claude / Codex | 做什么 |
|---|---|---|---|
| **开新项目** | 初始化本项目 | `init` | 定档三问 → 播种骨架，当场开工 |
| **接管半路项目** | 接管本项目 | `adopt` | 考古 → 定档 → 只补缺的件，不重建、不回填历史 |
| **换会话接着干** | 握手后继续 | `handshake` | 三步对齐（目标 → 当前阶段 → 你来选）再动手 |
| **看多会话全场** | 检查 worktree | `worktree` | 各执行轨的边界、依赖、漂移、可回收 |
| **自检路径** | agent-on doctor | `doctor` | 打印本机登记与 hook 执行面 |
| **沉淀回流** | agent-on 结账 | `settle` | 把本项目带证据的教训送进 agent-on 的 `intake/` |
| **升级方法论** | agent-on 升级 | `upgrade` | 显式 bump 项目 pin，从不静默变 |
| **整理想法** | 整理想法 | — | 速记区 → 归类成文、标去向 |
| **更新仪表盘** | 更新仪表盘 | — | 从真相源重绘 `dashboard.html`（M/L 档），数字不许手填 |

**入口怎么选**：全新项目 → `init`；已开工但从没接过 agent-on → `adopt`（不是 handshake）；接过的项目每次换会话 → `handshake`。

多会话并行、worktree 控制面、合流队列的完整用法见 [docs/manual.md](docs/manual.md)。

## 三档装备：先定档，再发装备

| 档 | 适用 | 播种什么 | 免掉什么 |
|---|---|---|---|
| **S 轻装** | 自用小工具、脚本、探索 | 三件套：AGENTS-lite（十几行宪法）+ loop-notes + lock，一分钟播完 | phase 卡、状态文件、契约、run 台账 |
| **M 标准** | 有真实用户，单人持续迭代 | 完整 AGENTS 骨架 + progress.yaml + phase 卡 + 三件套 | 契约 fixtures、并行装备（用到再加） |
| **L 全装** | 碰钱 / 数据 / 安全、多 agent 并行、长周期 | 全套 | — |

三条不分档的底线：**完成要贴命令输出；暂停项写成禁令；外向硬门先确认**（merge、打 tag、发布、部署、对外发言这一类。提交、推自己的分支、开 PR 属于本轨内部动作，不用问）。这是制度，不是流程税。

## 迭代闭环：为什么用的项目越多它越强

```
① 种    BOOTSTRAP / adopt 播骨架
② 采    六类触发当场留痕，带证据
③ 结    口令「agent-on 结账」，只写 intake/ 承接层
④ 消化  agent-on 仓会话分诊，落成具体文件修改
⑤ 发布  CHANGELOG + git tag
⑥ 升级  各项目显式 bump pin，拿到免疫
```

每次消化必须落成至少一处具体文件改动，不许只写「已知悉」。社区贡献只交 `intake/` 卡片，正文由维护者消化后发版——官方仓不会被改乱。机制全文：[playbook/iteration-loop.md](playbook/iteration-loop.md)。

## 为什么信它

**它不是设计出来的，是在真实产品上试错出来的。**

- **实战出身**：方法论来自 Euan CRM 的开发全程——9 次多 agent 并行 run 零合并冲突、600+ 测试、生产在线。**每个模板都被真实用过，没有一个是想象出来的。**
- **三代演化**：一代想自建编排引擎（5200+ 行设计、0 行引擎代码），二代用锁堵漂移（防漂移框架自己漂成了文档洁癖），三代在真实产品上实战出 Loop Engineering。主线一句话：**从「锁住 AI」到「和 AI 对齐」**。家谱见 [snapshot/2026-07-07-fusion-map.md](snapshot/2026-07-07-fusion-map.md)。
- **翻车案例集**：[bench/cases/](bench/cases/README.md) 收了 50 张真实翻车卡——AI 谎报、缓存链耦合、闸拦错树、审批被拉伸……每张都带证据和对应的规则落点。
- **自己吃自己的狗粮**：本仓按自己卖的方法论开发——决策入快照、完成贴证据、每次交付必打 tag、CI 把文档纪律变成可以红的闸。到今天已经 40 多个版本，每一版的证据都写在 [CHANGELOG](CHANGELOG.md) 里。
- **诚实的路线图**：v1.0 的定义是「至少两个项目有外人装机开工 + 至少一次外部结账经消化落地」，**目前还没达到**，我们照实写着。

## 与其他工具的关系

不冲突，分工明确：

- **agent-on = 制度层**：启动、推进、不漂、结账回流、完成贴证据、跨仓边界。
- **GStack 等 = 环节能力**：评审、QA、发布、调试，点名调用，产物收回项目仓。
- **Superpowers 不在默认推荐里**（偏重，容易抢跑初始化和规划）；你点名才用。

## 仓库里有什么

| 目录 | 是什么 |
|---|---|
| [BOOTSTRAP.md](BOOTSTRAP.md) + [boot/](boot/) | 冷启动入口：新项目初始化、存量项目接管、换会话握手、结账 / 升级执行书 |
| [kit/](kit/README.md) | 模板层：AGENTS 骨架与轻装版、phase 卡、派工 / 审查词、合流 checklist、worktree 与合流控制面、值守合并、仪表盘、想法收集箱等 |
| [playbook/](playbook/README.md) | 方法论 15 篇：SOP、防幻觉、真相源治理、阶段闸门、多人协作、迭代闭环、工作流编排…… |
| [bench/](bench/) | 50 张翻车案例 + 能力探针 + 能力真相表 |
| [cli/](cli/) | `agent-on` Rust CLI：doctor / setup / worktree / landing / oncall / guard / drift / tag-release 等 |
| [intake/](intake/) | 承接层：各项目结账回流的落点 |
| `ledger/` `snapshot/` `legacy/` | run 台账 · 带日期的决策快照 · 前身仓考古层 |

## 常见问题

**简单项目也要走全流程吗？**
不用。定档三问会把你路由到 S 轻装：三件套一分钟播完，其余全免。唯一不能省的是「完成贴证据」这类底线——那不是流程，是诚实。

**方法论更新了，我的项目会被动改吗？**
不会。项目 pin 在具体版本上，升级是显式口令。只有 major 版本（不动手会坏）才需要改你项目里的文件，而且以 diff 提案呈报，你逐条批准。

**每次都要念口令吗？**
不用。只在三个时刻显性出现：每台机器装一次、每个项目接入时说一句、之后只剩结账 / 升级 / 整理想法 / 更新仪表盘四个日常口令。平时开发零操作。

**Codex / Grok 真的能用吗？**
能，项目侧零适配——`AGENTS.md` 本来就是跨工具标准。Claude Code 独有的子代理编排在 Codex 下退化为手工纪律，闭环照跑。

**别人用了要给你提 PR 吗？**
不用。默认只用；贡献自愿，而且只交 `intake/` 卡片或 Issue，不直接改 playbook / kit。详见 [boot/settlement.md](boot/settlement.md)「上游贡献形态」。

**换电脑、路径不同、Windows？**
不要求固定目录名，clone 到哪都行，登记 `work_root` 即可。完整说明与恢复步骤见 [docs/manual.md](docs/manual.md)。

## 状态与路线

- **当前**：v0.25.x。启动 / 接管 / 握手 / 结账 / 升级闭环已跑通；Rust CLI 覆盖多会话控制面、合流协调、值守合并与审计。
- **下一站 v1.0**：≥2 个项目有外人装机开工 + ≥1 次外部结账经消化落地。定义见 [snapshot/2026-07-16-v10-and-setup.md](snapshot/2026-07-16-v10-and-setup.md)。
- 按版本的演进摘要见 [docs/manual.md](docs/manual.md)，逐版细节见 [CHANGELOG.md](CHANGELOG.md)。

## 一句话术语（给非工程师）

- **pin**：项目锁定的 agent-on 版本，像合同注明用哪版图纸——升级永远显式
- **worktree**：git 的「同一个仓开多个工作目录」。它只隔离环境，不会自动防止两个会话改同一个文件
- **fixture**：接口两侧共用的冻结样例数据，并行开发时当裁判用
- **结账 / 消化**：结账 = 项目把带证据的教训送回本仓 `intake/`；消化 = 本仓把它落成正文修改并发版
- **L1–L4**：教训的沉淀深度，从单次复盘到用户稳定偏好（[playbook/memory-layering.md](playbook/memory-layering.md)）
