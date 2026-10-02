# BOOTSTRAP — 新项目冷启动指令（给 AI 读）

> 职责边界：你（Claude Code / Codex）被用户要求「读本文件并初始化项目」时，按本文件从零搭起项目骨架。本文件自包含——初始化过程不需要先读本仓其他文档；需要模板时按文中路径取用。
> 版本：v0.2（2026-07-07）。总目标见 [CHARTER.md](CHARTER.md)。

## 0. 你在做什么

用户要开一个新项目。你的任务：在当前目录搭起 Loop Engineering 骨架，让后续所有会话（换窗口、换模型、换工具都算）零铺垫接续工作。**如果项目已经初始化过、你只是新会话来续跑——不走本文件，走 `boot/session-handshake.md` 的简短对齐续跑。**

一条底层原则贯穿全部：**prompt 易挥发，文件系统持久**——规则、状态、决策、契约，一切关键资产都必须落成文件，不许只活在对话里。

## 1. 先定档，再收需求（一次问完，别挤牙膏）

**第一组 · 三个核对点**（已有文件和指令能确定的直接采用，只问真实缺口）：

1. 有真实用户或真实数据吗？
2. 几天内搞完，还是持续迭代几周以上？
3. 碰钱 / 安全 / 对外服务吗？

判定：装备按协作复杂度选，验证按本次改动风险选。单人或目标明确默认 **S 轻装**；确实需要跨会话状态才加 **M 标准**；确实需要接口两侧并行才加 **L 并行件**。真实用户、支付或外部服务会提高相关行为的验证要求，**不自动要求补齐整套 MRD、仪表盘和台账**。既有用户禁令、发布授权和安全检查照旧；调整装备时记录原因，不静默撤销已批准的约束。

> **测试资源（条件触发）**：项目依赖第三方 testnet / 沙箱 / staging 时，先只读确认能拿到资源。不可用就记录验证缺口，选择可用的验证方式；使用真实环境仍须已有明确授权，不由“测试网不可用”推成主网执行授权，也不自动整包升档。来源：aster-agent 2026-09-14 测试资源过期实证。

| 档 | 播种 | 不播 |
|---|---|---|
| **S 轻装** | 三件套（AGENTS-lite + loop-notes.md + agent-on.lock.md） | 想法箱、phase 卡、progress.yaml、契约、run 台账、dashboard 按需 |
| **M 标准** | S 的三件 + 需要时加状态源与自包含任务卡 | MRD / PRD / dashboard 只按实际需要添加 |
| **L 并行件** | M + 接口契约 / 派工回执 / 共享改动协调 | 未使用的模板与全套流程不播种 |

> `thoughts-and-ideas.md` 按需记录新想法；`dashboard.html` 是可选视图。没有实际读者就不生成第二份状态面。

**第二组 · 需求核对**（复用已确认内容，只问会影响实现的缺口；复杂 / 高风险项目可选深挖版 `boot/new-project-questionnaire.md`）。偏好缺口是幻觉的第五类来源（详见 playbook/elicitation-protocol.md）——先收齐再动手：

1. 项目一句话：做什么，给谁用？
2. 有没有参照物（长得像哪个产品 / 网站 / App）？——品味前置，选择比描述便宜十倍；参照物落进规格时拆两栏：**学什么**（信息架构/交互/指标语法）/**不复制什么**（资产/商标/付费墙绕过），自研补齐对方付费体验要写清能力对等边界。竞品**公开面挖不到**的私有栈（DB 引擎名等）**不得阻塞**本项目选型——用可验证产品约束 + 可逆分层锁定。
3. 技术栈有倾向吗？还是要我推荐？——另拆一栏：**交付物终局** vs **生产线/采集 runtime**（本机常驻 ≠ 产品只能本机装）。
4. 这个项目碰不碰「钱、真实用户数据、对外服务」？（与定档三问互验，并决定车道，见 §3）
5. 单人 + AI，还是有其他协作者？这台机器上有没有已装的**环节** skill（如 GStack）？——有则审查/发布/调试点名走它；**制度永远在 agent-on**（证据/禁令/结账）。**不**默认叠 Superpowers 全流程（偏重、易抢跑），见 §4 尾注与 AGENTS §skill 路由。
6. 有没有明确不做 / 暂缓的事？（暂停项要写成禁令，不是删掉——后置的渠道/触点（推送/移动端/多租户）也算暂停项，别只留在对话里）

## 1.5 规划工具箱（按需要选用）

最小规划是一份能开工的说明：目标、范围、验收、风险、下一步。需求已明确就直接实现；按缺口选择下表工具，不要求完整链。不要把同一内容重抄成 MRD、PRD、plan 和卡片。用过的外部 skill 产物保存入仓；同一决策批次可以一起提交。

| 环节 | 适用缺口 | 可选路由与产物 |
|---|---|---|
| 调研 | 选型或外部事实不清 | 已安装的调研 skill；保存相关依据 |
| MRD / PRD | 产品范围、用户问题或复杂需求未对齐 | office-hours 或 kit 的对应模板；已有规格直接复用 |
| 技术方案 | 存在架构选择或高风险迁移 | 一份可审查的 plan，记录取舍与验证 |
| 审查 | 当前方案的复杂度或风险需要复核 | 已安装的 autoplan / review，输入本次范围 |
| 任务卡 | 长任务或跨会话需要可续接目标 | 自包含任务卡；已有任务记录直接复用 |
| 单卡精修 | 一个任务有实质歧义 | 已安装的 spec；只澄清该任务 |

**模板问卷化协议**（第 3、4 环节的引导方式，也是无 GStack 机器的全链兜底）：凡实例化 `kit/prd-template.md` / `kit/requirement-pack-template.md`，每个空节 = 一轮「**AI 从上游文档与对话草拟 + 用户勘误**」，不拿空表逼问（选择比描述便宜十倍）；用户答不上的落 `99_待确认与决策记录`，**禁止 AI 编内容填空**。

## 2. 搭骨架

**S 轻装捷径（三件套，一分钟）**：拷 `kit/AGENTS-lite.md` → 项目根 AGENTS.md 填空（暂停项禁令别空着），另建一行 `CLAUDE.md`「规则见 AGENTS.md」；建空 `loop-notes.md`；实例化 `kit/agent-on-lock-template.md` → `agent-on.lock.md`；需要想法箱时再实例化 `kit/thoughts-and-ideas-template.md`；**initial commit**（没仓先 `git init`，骨架全部入 git——落盘未 commit = 初始化未完成）。完——下面七步全部跳过，§4 铁律只守 AGENTS-lite 那三条底线，§6 沉淀纪律照常（**闭环不分档**：小项目的教训一样回流）。

**M / L 按需选下面的组件**（M 档第 1 步的 `contracts/` 与 §5 并行装备可等用到再加）：

1. 建目录：`docs/{state,phases,snapshots}/`；走了 §1.5 规划链就加 `docs/{product,requirements,plans}/`（做了调研另加 `docs/research/`）；将来有接口两侧并行的可能就加 `contracts/fixtures/`
2. 拷 `kit/AGENTS-skeleton.md` → 项目根 `AGENTS.md`，用 §1 的答案填空（不留 `[占位]`）；另建一行 `CLAUDE.md`：「规则权威见 AGENTS.md」——AGENTS.md 是 Claude Code 与 Codex 的共同标准，双工具通吃
3. 拷 `kit/progress-template.yaml` → `docs/state/progress.yaml`——**单一状态写者**：只有 orchestrator 主会话能写它
4. 拷 `kit/phase-card-template.md` → `docs/phases/_TEMPLATE.md`
5. 需求三分法：已确认 → AGENTS §硬约束；有方向没定死 → `docs/requirements.md` 待拍板区；缺信息 → 回 §1 追问。**暂停项写成禁令条款**
6. 写第一张 phase 卡 `docs/phases/phase-s0.1-<slug>.md`：自包含（新会话只读这张卡就能干活）、验收 ≤8 条、每条能翻译成测试名或命令输出
7. 实例化 `kit/agent-on-lock-template.md` → 项目根 `agent-on.lock.md`（pin 当前 agent-on 的 tag+commit）；AGENTS.md 首节加一行「agent-on 映射见 agent-on.lock.md」。此后凡从 kit 实例化文件，头部都加 `<!-- instantiated-from kit/<文件> @ vX.Y.Z -->`
8. 按需添加想法箱；用户确实需要图形全貌时才添加仪表盘。状态只从原有真相源读取，不新增手工维护义务。
9. **initial commit**：骨架文件全部入 git（没仓先 `git init`）——**落盘未 commit = 初始化未完成，禁止向用户报完成**；此后按完整决策/交付批次提交，保留清晰回退点（§4 L8）

## 3. 车道判定（每个任务先过这道门）

- **Explore 车道**：原型 / 视觉 / 概念验证，先用预览、交互与用户反馈验证。
- **Ship 车道**：进入实际使用的改动，按影响范围补齐验证与权限检查。
- **转入 Ship**：检查原型的质量与依赖，补足缺失验证；合格部分直接复用，不为流程名义强制重写。

## 4. Ship 车道铁律（编号化，违反 = 返工）

- **L1 验证按风险**：业务行为、权限、数据与回归缺陷优先用失败测试固定规格；纯样式、文案、文档和探索任务用相关预览/检查，不为满足流程写镜像测试。TDD 是可选执行方法，完成证据仍是硬要求。
- **L2 完成 = 贴命令实际输出**：禁止「应该没问题」「理论上可行」
- **L3 单一状态写者**：已采用的共享状态源由入口写；执行者只干活汇报，不为满足规则额外生成 progress.yaml
- **L4 契约先行**：接口两侧并行前先冻结 fixtures，**连语义一起冻**（排序 / 空值 / 上限）
- **L5 暂停项 = 禁令**：未写明允许即禁止
- **L6 Error Signal 四要素**：报障必须带 What / Where / How / Severity
- **L7 外部服务第一天**：真实载荷形状对账、函数区 = 数据区、部署后 GET 和 POST 都冒烟
- **L8 产物入仓 + 收口 commit**：走外部 skill（GStack 等）的规划/审查环节，产物常落 `~/.gstack/` 等仓外路径——收口 = 转录进项目 `docs/` + 一个 commit（中文语义化 message），否则该环节不算完成；orchestrator 主会话是规划链落盘与 commit 的唯一责任人（与 L3 同构）。口令动作（整理想法/更新仪表盘/结账）收口同样即时 commit——**commit 时间线就是用户的回退时间线**

**相关案例按需读取**：接外部服务、并行协作或出现相似失败时，从 `bench/cases/README.md` 的场景索引选相关案例，不为档位把整套历史装入每轮上下文。

**skill 分工尾注（2026-08：制度优先，Superpowers 退出默认）**：
- **agent-on** = 制度层（定档/骨架/完成=证据/单写者/结账回流/跨仓闸）——主责，不外包。
- **GStack 等环节 skill（若已装）** = 规划/审查/QA/发布/调试怎么做；产物收口进项目 `docs/` + commit（L8）。
- **kit 模板**（`review-prompt` / `merge-checklist` 等）= **无对应 skill 时的唯一 fallback**——禁止同时开两套审查制度。
- **不默认 Superpowers**：实现不走 subagent-driven-development 默认引擎；brainstorming / writing-plans **点名禁用**抢跑 init 与规划。用户口头点名某 skill 才例外。
- **压制写在双工具共读层**（项目 AGENTS.md；机器侧应用时写 AGENT.md 共读层，不只 `~/.claude/CLAUDE.md`）——不点名 = 可被抢跑（实证：AINVESTMENT——superpowers brainstorming 在 Codex 抢跑 init，骨架零落盘零 commit）。

## 5. 多 agent 并行（需要时才启用）

**单 agent 能干完就别上多 agent**（上下文边界优先）。确要并行时走六步协议：冻契约 → 轨道 = 目录 + git worktree 物理隔离 → 各轨 Fake 对方 → 契约测试当裁判 → 单一状态写者 → 先契约后实现合流。

**先合并共享任务，再决定是否并行。** 不同页面不自动对应不同 worktree/PR；同一组件、token、路由或锁文件交给一个任务，其余先等它合入。确实同时写代码时各用一棵树，不必先 claim。可选 `agent-on patrol start` 自动创建独立巡逻窗口，原窗口继续作为统一入口；`dispatch` 可调用 Claude/Codex/Grok，`--worktree` 才申请独立写树。流程与实测边界见 [kit/patrol-control-plane.md](kit/patrol-control-plane.md)。commit/push 现有同文件闸保持兼容；值守仍拥有原有合并权。`worktree gc --dry-run` 仍只报告；单独启用 janitor 才授权回收本功能创建、明确释放且可恢复的 checkout。

模板按实际任务选：`kit/track-prompt-template.md` 派工、`kit/review-prompt-template.md` 复核、`kit/merge-checklist.md` 合流、`kit/worktree-control-plane.md` 边界/依赖/回收、`kit/babysit/` 值守。能力探针只在用户明确比较或任务暴露实际缺口时使用。

## 6. 沉淀纪律（迭代闭环的采集站，机制见 playbook/iteration-loop.md）

- 六类触发**当场**记一行进 `loop-notes.md`（单行五字段 `日期|触发|一句现象|证据指针|候选层`）：返工（完成声明被推翻）/ 撞车 / 用户纠正 / Error Signal 中高严重度 / 手工重复第 2 次 / **脚手架不合身**（这条另记 agent-on.lock.md 的 local_deviations）
- 用户随口冒出的**产品想法/待办**：记录到项目已有待办；启用了想法箱才写 `thoughts-and-ideas.md`。建议与已确认需求分开，记录后继续主线
- 跨项目可复用的升 memory_card（`suggested_location=agent_on`），**evidence 必填**——没证据的心得出不了仓
- 每次编排 run 合流时记 run 台账一行（`ledger/run-card-logging.md` 规范）
- 里程碑时用户说「**agent-on 结账**」→ 按 `boot/settlement.md` 执行（升级另有口令「agent-on 升级」）

**两个项目内口令（v0.4）**：
- 「**整理想法**」→ 读 `thoughts-and-ideas.md` 速记区，归类成文标去向进「已整理」，清空速记区（换会话握手若发现速记区非空，主动提醒一次）
- 「**更新仪表盘**」→ 已启用时从真相源重绘 `dashboard.html`，合流按实际状态变化更新；数据只从真相源读

## 7. 初始化完成的验收（只查本项目选用的组件）

- [ ] AGENTS.md 已填空，无 `[占位]` 残留；CLAUDE.md 指针就位
- [ ] 已选择的状态源与任务卡就位；已有等价文件直接复用
- [ ] `agent-on.lock.md` 就位（pin 已锚定 tag+commit）
- [ ] 想法箱或仪表盘仅选择启用时就位
- [ ] 需求三分法讲给用户听过：已确认 / 待拍板 / 暂停禁令三张清单
- [ ] 用户知道「agent-on 结账」「agent-on 升级」；其他口令只说明已启用的功能
- [ ] 骨架已 commit：`git log --oneline` 非空且含骨架文件（落盘未 commit = 初始化未完成）
- [ ] 以上每条都有实际文件路径或命令输出作证（L2 对你自己同样生效）

档播错了怎么办：**禁止静默降档**。升档补件走 [boot/adopt.md](boot/adopt.md) §二；**降档**（误播高档 / 项目变瘦）走 adopt **§三**——须用户显式批准、只删不用的件、不重播、local_deviations 登记。改 AGENTS 档位标记 alone 不够，要按 §三清单勾完。
