# BOOTSTRAP — 新项目冷启动指令（给 AI 读）

> 职责边界：你（Claude Code / Codex）被用户要求「读本文件并初始化项目」时，按本文件从零搭起项目骨架。本文件自包含——初始化过程不需要先读本仓其他文档；需要模板时按文中路径取用。
> 版本：v0.2（2026-07-07）。总目标见 [CHARTER.md](CHARTER.md)。

## 0. 你在做什么

用户要开一个新项目。你的任务：在当前目录搭起 Loop Engineering 骨架，让后续所有会话（换窗口、换模型、换工具都算）零铺垫接续工作。**如果项目已经初始化过、你只是新会话来续跑——不走本文件，走 `boot/session-handshake.md` 的三步握手。**

一条底层原则贯穿全部：**prompt 易挥发，文件系统持久**——规则、状态、决策、契约，一切关键资产都必须落成文件，不许只活在对话里。

## 1. 先定档，再收需求（一次问完，别挤牙膏）

**第一组 · 定档三问**（决定装备重量——防高射炮打蚊子）：

1. 有真实用户或真实数据吗？
2. 几天内搞完，还是持续迭代几周以上？
3. 碰钱 / 安全 / 对外服务吗？

判定：问 3 任一为是 → **L 全装**；有真实用户或持续迭代 → **M 标准**；都不是 → **S 轻装**。拿不准取低档——**允许升档、不许静默降档**（升档协议见 boot/adopt.md §二）；S 档项目开始碰真实数据的那天，就是升档日。

| 档 | 播种 | 不播 |
|---|---|---|
| **S 轻装** | 三件套（AGENTS-lite + loop-notes.md + agent-on.lock.md）+ thoughts-and-ideas.md | phase 卡、progress.yaml、契约、run 台账、dashboard 全免 |
| **M 标准** | 完整 AGENTS 骨架 + progress.yaml + phase 卡 + dashboard.html + thoughts-and-ideas.md + S 的三件 | contracts/ 与并行装备（用到再加） |
| **L 全装** | 全套（§2 七步一步不少） | — |

> **两个默认件**（v0.4）：`thoughts-and-ideas.md`（全档，你随手写想法、AI 整理）+ `dashboard.html`（M/L，人机共读的项目全貌，合流必更）。维护协议见两个模板各自的头部。

**第二组 · 需求六问**（S 档只问 1 / 2 / 6 三题；复杂 / 高风险项目用深挖版 `boot/new-project-questionnaire.md` 替代）。偏好缺口是幻觉的第五类来源（详见 playbook/elicitation-protocol.md）——先收齐再动手：

1. 项目一句话：做什么，给谁用？
2. 有没有参照物（长得像哪个产品 / 网站 / App）？——品味前置，选择比描述便宜十倍；参照物落进规格时拆两栏：**学什么**（信息架构/交互/指标语法）/**不复制什么**（资产/商标/付费墙绕过），自研补齐对方付费体验要写清能力对等边界
3. 技术栈有倾向吗？还是要我推荐？
4. 这个项目碰不碰「钱、真实用户数据、对外服务」？（与定档三问互验，并决定车道，见 §3）
5. 单人 + AI，还是有其他协作者？这台机器上有没有已装的 skill 体系（GStack / Superpowers 等）？——决定审查 / 发布 / 调试各走谁，见 §4 尾注与 AGENTS §skill 路由。
6. 有没有明确不做 / 暂缓的事？（暂停项要写成禁令，不是删掉——后置的渠道/触点（推送/移动端/多租户）也算暂停项，别只留在对话里）

## 1.5 规划链（定档后、播骨架前；S 档跳过本节）

想法到 phase 卡之间隔着一串文档：调研 → MRD → 需求澄清 → PRD → 技术方案 → 审查 → 拆解。原则：**拆解责任在框架不在用户；有强 skill 就路由调用，不自研提问流**（本机 skill 体系从 §1 第 5 问得知）。产物一律转录进项目 `docs/`（§4 L8），每环节收口一个 commit。

| # | 环节 | 有 GStack 时路由 | 产物落盘 | M | L |
|---|---|---|---|---|---|
| 1 | 调研 | office-hours 自带 landscape 搜索；深调研点名 deep-research；UI 项目加 /design-consultation | `docs/research/landscape.md` | 可选 | 必 |
| 2 | MRD | `/office-hours` **强制 Startup mode**（禁 Builder 路由，忽略其营销收尾） | 转录 → `docs/product/mrd.md` | 必 | 必 |
| 3 | 需求澄清 | 无 skill：`kit/requirement-pack-template.md` 问卷化（见下） | `docs/requirements/` | 可选（多角色/有权限面才做） | 必 |
| 4 | PRD | 无 skill：`kit/prd-template.md`——先从 MRD 机械转录，再补问首发范围/非功能 | `docs/product/prd.md` | 必 | 必 |
| 5 | 技术方案 | plan mode 按选定 approach 写 rough plan（正是 autoplan 的输入假设） | `docs/plans/<里程碑>-plan.md` | 必（可粗） | 必 |
| 6 | 审查 | `/autoplan`（**per-milestone 喂，禁整本 PRD**） | plan 原地改写，test plan 拷回 `docs/plans/` | 可选 | 必 |
| 7 | 拆解 | agent-on 自持：§2 的 phase 卡 | `docs/phases/phase-*.md` | 必 | 必 |
| 7b | 单卡精修 | `/spec`（issue 级——只在这里用，放产品层 = 层级错位） | 精修后的 phase 卡 | 复杂卡可选 | 复杂卡可选 |

**模板问卷化协议**（第 3、4 环节的引导方式，也是无 GStack 机器的全链兜底）：凡实例化 `kit/prd-template.md` / `kit/requirement-pack-template.md`，每个空节 = 一轮「**AI 从上游文档与对话草拟 + 用户勘误**」，不拿空表逼问（选择比描述便宜十倍）；用户答不上的落 `99_待确认与决策记录`，**禁止 AI 编内容填空**。

## 2. 搭骨架

**S 轻装捷径（三件套，一分钟）**：拷 `kit/AGENTS-lite.md` → 项目根 AGENTS.md 填空（暂停项禁令别空着），另建一行 `CLAUDE.md`「规则见 AGENTS.md」；建空 `loop-notes.md`；实例化 `kit/agent-on-lock-template.md` → `agent-on.lock.md`；实例化 `kit/thoughts-and-ideas-template.md` → `thoughts-and-ideas.md`（想法收集箱，全档都建）；**initial commit**（没仓先 `git init`，骨架全部入 git——落盘未 commit = 初始化未完成）。完——下面七步全部跳过，§4 铁律只守 AGENTS-lite 那三条底线，§6 沉淀纪律照常（**闭环不分档**：小项目的教训一样回流）。

**M / L 档走七步**（M 档第 1 步的 `contracts/` 与 §5 并行装备可等用到再加）：

1. 建目录：`docs/{state,phases,snapshots}/`；走了 §1.5 规划链就加 `docs/{product,requirements,plans}/`（做了调研另加 `docs/research/`）；将来有接口两侧并行的可能就加 `contracts/fixtures/`
2. 拷 `kit/AGENTS-skeleton.md` → 项目根 `AGENTS.md`，用 §1 的答案填空（不留 `[占位]`）；另建一行 `CLAUDE.md`：「规则权威见 AGENTS.md」——AGENTS.md 是 Claude Code 与 Codex 的共同标准，双工具通吃
3. 拷 `kit/progress-template.yaml` → `docs/state/progress.yaml`——**单一状态写者**：只有 orchestrator 主会话能写它
4. 拷 `kit/phase-card-template.md` → `docs/phases/_TEMPLATE.md`
5. 需求三分法：已确认 → AGENTS §硬约束；有方向没定死 → `docs/requirements.md` 待拍板区；缺信息 → 回 §1 追问。**暂停项写成禁令条款**
6. 写第一张 phase 卡 `docs/phases/phase-s0.1-<slug>.md`：自包含（新会话只读这张卡就能干活）、验收 ≤8 条、每条能翻译成测试名或命令输出
7. 实例化 `kit/agent-on-lock-template.md` → 项目根 `agent-on.lock.md`（pin 当前 agent-on 的 tag+commit）；AGENTS.md 首节加一行「agent-on 映射见 agent-on.lock.md」。此后凡从 kit 实例化文件，头部都加 `<!-- instantiated-from kit/<文件> @ vX.Y.Z -->`
8. 实例化两个默认件：`kit/thoughts-and-ideas-template.md` → 项目根 `thoughts-and-ideas.md`（全档）；`kit/dashboard-template.html` → 项目根 `dashboard.html`（M/L 档，填项目名后从真相源初绘一次）。维护协议见各自模板头部；告诉用户两个口令：「整理想法」「更新仪表盘」
9. **initial commit**：骨架文件全部入 git（没仓先 `git init`）——**落盘未 commit = 初始化未完成，禁止向用户报完成**；此后规划链每环节与口令动作收口即 commit（§4 L8）

## 3. 车道判定（每个任务先过这道门）

- **Explore 车道**：原型 / 视觉 / 概念验证——错误不算错误的域。放飞，一把梭，产物可丢弃，不进主干。
- **Ship 车道**：碰数据、钱、安全、真实用户——全纪律（§4）。
- **两道不许串**：Explore 产物要进主干，必须走 Ship 流程重做。（原理见 playbook/freedom-vs-discipline.md）

## 4. Ship 车道铁律（编号化，违反 = 返工）

- **L1 TDD**：没有失败的测试，不写生产代码
- **L2 完成 = 贴命令实际输出**：禁止「应该没问题」「理论上可行」
- **L3 单一状态写者**：progress.yaml 只有主会话写；子代理只干活汇报
- **L4 契约先行**：接口两侧并行前先冻结 fixtures，**连语义一起冻**（排序 / 空值 / 上限）
- **L5 暂停项 = 禁令**：未写明允许即禁止
- **L6 Error Signal 四要素**：报障必须带 What / Where / How / Severity
- **L7 外部服务第一天**：真实载荷形状对账、函数区 = 数据区、部署后 GET 和 POST 都冒烟
- **L8 产物入仓 + 收口 commit**：走外部 skill（GStack 等）的规划/审查环节，产物常落 `~/.gstack/` 等仓外路径——收口 = 转录进项目 `docs/` + 一个 commit（中文语义化 message），否则该环节不算完成；orchestrator 主会话是规划链落盘与 commit 的唯一责任人（与 L3 同构）。口令动作（整理想法/更新仪表盘/结账）收口同样即时 commit——**commit 时间线就是用户的回退时间线**

**动手前扫坑**（M/L 必读）：接外部服务 / 上多 agent 并行 / 给用户交付产出前，先扫 agent-on `bench/cases/README.md` 的「使用时机表」——17 张实战翻车案例按场景索引，别人踩过的坑动手前先认一遍。

**skill 分工尾注**：本仓 `kit/review-prompt-template.md` 是 fallback 审查协议。若这台机器已装强 skill 体系（GStack `/review`、Superpowers 等），审查 / 发布 / 调试各走哪套写进项目 AGENTS §skill 路由——agent-on 模板只在没有对应 skill 时兜底，避免双审查制度打架。**路由含压制**：被路由掉的抢跑型 skill（brainstorming / planning 类）要在 AGENTS §skill 路由**点名禁用**——不点名 = 可被抢跑；压制必须写在双工具共读层（项目 AGENTS.md、机器侧 AGENT.md），只写单工具专属文件（如 `~/.claude/CLAUDE.md`）则另一家照样抢跑（实证：AINVESTMENT 事故——superpowers brainstorming 在 Codex 会话抢跑 init，骨架零落盘零 commit）。

## 5. 多 agent 并行（需要时才启用）

**单 agent 能干完就别上多 agent**（上下文边界优先）。确要并行时走六步协议：冻契约 → 轨道 = 目录 + git worktree 物理隔离 → 各轨 Fake 对方 → 契约测试当裁判 → 单一状态写者 → 先契约后实现合流。
模板：`kit/track-prompt-template.md`（派工，含按模型能力调档的脚手架旋钮）、`kit/review-prompt-template.md`（对抗式独立审查）、`kit/merge-checklist.md`（合流七步）。换新模型先跑 `bench/capability-probe.md` 定档。

## 6. 沉淀纪律（迭代闭环的采集站，机制见 playbook/iteration-loop.md）

- 六类触发**当场**记一行进 `loop-notes.md`（单行五字段 `日期|触发|一句现象|证据指针|候选层`）：返工（完成声明被推翻）/ 撞车 / 用户纠正 / Error Signal 中高严重度 / 手工重复第 2 次 / **脚手架不合身**（这条另记 agent-on.lock.md 的 local_deviations）
- 用户随口冒出的**产品想法/待办**（非 debug、非状态询问）：AI 当场代笔进 `thoughts-and-ideas.md` 速记区（日期+「对话捕获」标），确认一句继续主线——只进速记区，升级成需求永远由用户拍板
- 跨项目可复用的升 memory_card（`suggested_location=agent_on`），**evidence 必填**——没证据的心得出不了仓
- 每次编排 run 合流时记 run 台账一行（`ledger/run-card-logging.md` 规范）
- 里程碑时用户说「**agent-on 结账**」→ 按 `boot/settlement.md` 执行（升级另有口令「agent-on 升级」）

**两个项目内口令（v0.4）**：
- 「**整理想法**」→ 读 `thoughts-and-ideas.md` 速记区，归类成文标去向进「已整理」，清空速记区（换会话握手若发现速记区非空，主动提醒一次）
- 「**更新仪表盘**」→ 从真相源（progress.yaml / 决策 / phase 卡）重绘 `dashboard.html`（M/L）；合流时也必更（见 merge-checklist 第 7 步）。**数据只从真相源读，禁手填**

## 7. 初始化完成的验收（对用户交付；S 档只查第 1、3、4、5、7 条，第 1 条查 AGENTS-lite）

- [ ] AGENTS.md 已填空，无 `[占位]` 残留；CLAUDE.md 指针就位
- [ ] progress.yaml、phases/_TEMPLATE.md、第一张 phase 卡就位
- [ ] `agent-on.lock.md` 就位（pin 已锚定 tag+commit）
- [ ] `thoughts-and-ideas.md` 就位（全档）；`dashboard.html` 就位并初绘一次（M/L）
- [ ] 需求三分法讲给用户听过：已确认 / 待拍板 / 暂停禁令三张清单
- [ ] 用户知道四个口令：「agent-on 结账」「agent-on 升级」「整理想法」「更新仪表盘」（后两个 S 档只有「整理想法」）
- [ ] 骨架已 commit：`git log --oneline` 非空且含骨架文件（落盘未 commit = 初始化未完成）
- [ ] 以上每条都有实际文件路径或命令输出作证（L2 对你自己同样生效）

档播错了怎么办：改 AGENTS 里的档位标记 + lock 的 local_deviations 记一行即可，**不重播**——升档补件走 boot/adopt.md §二，降档只删不用的件。
