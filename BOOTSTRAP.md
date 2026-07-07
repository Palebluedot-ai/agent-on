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
| **S 轻装** | 三件套：AGENTS-lite（十几行宪法）+ loop-notes.md + agent-on.lock.md | phase 卡、progress.yaml、契约、run 台账全免 |
| **M 标准** | 完整 AGENTS 骨架 + progress.yaml + phase 卡 + S 的三件 | contracts/ 与并行装备（用到再加） |
| **L 全装** | 全套（§2 七步一步不少） | — |

**第二组 · 需求六问**（S 档只问 1 / 2 / 6 三题；复杂 / 高风险项目用深挖版 `boot/new-project-questionnaire.md` 替代）。偏好缺口是幻觉的第五类来源（详见 playbook/elicitation-protocol.md）——先收齐再动手：

1. 项目一句话：做什么，给谁用？
2. 有没有参照物（长得像哪个产品 / 网站 / App）？——品味前置，选择比描述便宜十倍
3. 技术栈有倾向吗？还是要我推荐？
4. 这个项目碰不碰「钱、真实用户数据、对外服务」？（与定档三问互验，并决定车道，见 §3）
5. 单人 + AI，还是有其他协作者？
6. 有没有明确不做 / 暂缓的事？（暂停项要写成禁令，不是删掉）

## 2. 搭骨架

**S 轻装捷径（三件套，一分钟）**：拷 `kit/AGENTS-lite.md` → 项目根 AGENTS.md 填空（暂停项禁令别空着），另建一行 `CLAUDE.md`「规则见 AGENTS.md」；建空 `loop-notes.md`；实例化 `kit/agent-on-lock-template.md` → `agent-on.lock.md`。完——下面七步全部跳过，§4 铁律只守 AGENTS-lite 那三条底线，§6 沉淀纪律照常（**闭环不分档**：小项目的教训一样回流）。

**M / L 档走七步**（M 档第 1 步的 `contracts/` 与 §5 并行装备可等用到再加）：

1. 建目录：`docs/{state,phases,snapshots}/`；将来有接口两侧并行的可能就加 `contracts/fixtures/`
2. 拷 `kit/AGENTS-skeleton.md` → 项目根 `AGENTS.md`，用 §1 的答案填空（不留 `[占位]`）；另建一行 `CLAUDE.md`：「规则权威见 AGENTS.md」——AGENTS.md 是 Claude Code 与 Codex 的共同标准，双工具通吃
3. 拷 `kit/progress-template.yaml` → `docs/state/progress.yaml`——**单一状态写者**：只有 orchestrator 主会话能写它
4. 拷 `kit/phase-card-template.md` → `docs/phases/_TEMPLATE.md`
5. 需求三分法：已确认 → AGENTS §硬约束；有方向没定死 → `docs/requirements.md` 待拍板区；缺信息 → 回 §1 追问。**暂停项写成禁令条款**
6. 写第一张 phase 卡 `docs/phases/phase-s0.1-<slug>.md`：自包含（新会话只读这张卡就能干活）、验收 ≤8 条、每条能翻译成测试名或命令输出
7. 实例化 `kit/agent-on-lock-template.md` → 项目根 `agent-on.lock.md`（pin 当前 agent-on 的 tag+commit）；AGENTS.md 首节加一行「agent-on 映射见 agent-on.lock.md」。此后凡从 kit 实例化文件，头部都加 `<!-- instantiated-from kit/<文件> @ vX.Y.Z -->`

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

## 5. 多 agent 并行（需要时才启用）

**单 agent 能干完就别上多 agent**（上下文边界优先）。确要并行时走六步协议：冻契约 → 轨道 = 目录 + git worktree 物理隔离 → 各轨 Fake 对方 → 契约测试当裁判 → 单一状态写者 → 先契约后实现合流。
模板：`kit/track-prompt-template.md`（派工，含按模型能力调档的脚手架旋钮）、`kit/review-prompt-template.md`（对抗式独立审查）、`kit/merge-checklist.md`（合流七步）。换新模型先跑 `bench/capability-probe.md` 定档。

## 6. 沉淀纪律（迭代闭环的采集站，机制见 playbook/iteration-loop.md）

- 六类触发**当场**记一行进 `loop-notes.md`（单行五字段 `日期|触发|一句现象|证据指针|候选层`）：返工（完成声明被推翻）/ 撞车 / 用户纠正 / Error Signal 中高严重度 / 手工重复第 2 次 / **脚手架不合身**（这条另记 agent-on.lock.md 的 local_deviations）
- 跨项目可复用的升 memory_card（`suggested_location=agent_on`），**evidence 必填**——没证据的心得出不了仓
- 每次编排 run 合流时记 run 台账一行（`ledger/run-card-logging.md` 规范）
- 里程碑时用户说「**agent-on 结账**」→ 按 `boot/settlement.md` 执行（升级另有口令「agent-on 升级」）

## 7. 初始化完成的验收（对用户交付；S 档只查第 1、3、4、5 条，第 1 条查 AGENTS-lite）

- [ ] AGENTS.md 已填空，无 `[占位]` 残留；CLAUDE.md 指针就位
- [ ] progress.yaml、phases/_TEMPLATE.md、第一张 phase 卡就位
- [ ] `agent-on.lock.md` 就位（pin 已锚定 tag+commit）
- [ ] 需求三分法讲给用户听过：已确认 / 待拍板 / 暂停禁令三张清单
- [ ] 用户知道两个口令：「agent-on 结账」（沉淀回流）、「agent-on 升级」（bump pin）
- [ ] 以上每条都有实际文件路径或命令输出作证（L2 对你自己同样生效）
