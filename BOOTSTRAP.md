# BOOTSTRAP — 新项目冷启动指令（给 AI 读）

> 职责边界：你（Claude Code / Codex）被用户要求「读本文件并初始化项目」时，按本文件从零搭起项目骨架。本文件自包含——初始化过程不需要先读本仓其他文档；需要模板时按文中路径取用。
> 版本：v0.2（2026-07-07）。总目标见 [CHARTER.md](CHARTER.md)。

## 0. 你在做什么

用户要开一个新项目。你的任务：在当前目录搭起 Loop Engineering 骨架，让后续所有会话（换窗口、换模型、换工具都算）零铺垫接续工作。

一条底层原则贯穿全部：**prompt 易挥发，文件系统持久**——规则、状态、决策、契约，一切关键资产都必须落成文件，不许只活在对话里。

## 1. 先问用户（一次问完，别挤牙膏）

偏好缺口是幻觉的第五类来源（详见 playbook/elicitation-protocol.md）——先收齐再动手：

1. 项目一句话：做什么，给谁用？
2. 有没有参照物（长得像哪个产品 / 网站 / App）？——品味前置，选择比描述便宜十倍
3. 技术栈有倾向吗？还是要我推荐？
4. 这个项目碰不碰「钱、真实用户数据、对外服务」？（决定车道，见 §3）
5. 单人 + AI，还是有其他协作者？
6. 有没有明确不做 / 暂缓的事？（暂停项要写成禁令，不是删掉）

## 2. 搭骨架（六步，照做）

1. 建目录：`docs/{state,phases,snapshots}/`；将来有接口两侧并行的可能就加 `contracts/fixtures/`
2. 拷 `kit/AGENTS-skeleton.md` → 项目根 `AGENTS.md`，用 §1 的答案填空（不留 `[占位]`）；另建一行 `CLAUDE.md`：「规则权威见 AGENTS.md」——AGENTS.md 是 Claude Code 与 Codex 的共同标准，双工具通吃
3. 拷 `kit/progress-template.yaml` → `docs/state/progress.yaml`——**单一状态写者**：只有 orchestrator 主会话能写它
4. 拷 `kit/phase-card-template.md` → `docs/phases/_TEMPLATE.md`
5. 需求三分法：已确认 → AGENTS §硬约束；有方向没定死 → `docs/requirements.md` 待拍板区；缺信息 → 回 §1 追问。**暂停项写成禁令条款**
6. 写第一张 phase 卡 `docs/phases/phase-s0.1-<slug>.md`：自包含（新会话只读这张卡就能干活）、验收 ≤8 条、每条能翻译成测试名或命令输出

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

## 6. 沉淀纪律（新项目对 agent-on 的回馈）

- 项目里建 `loop-notes.md`：手工重复两次以上的环节、踩的坑，当场记一行，不展开
- 每次编排 run 合流时往 run 台账记一行（`ledger/run-ledger-template.md` 的 schema）
- 里程碑时用户说「**agent-on 结账**」→ 把可复用的沉淀（含对本文件的改进意见）搬回本仓

## 7. 初始化完成的验收（对用户交付）

- [ ] AGENTS.md 已填空，无 `[占位]` 残留；CLAUDE.md 指针就位
- [ ] progress.yaml、phases/_TEMPLATE.md、第一张 phase 卡就位
- [ ] 需求三分法讲给用户听过：已确认 / 待拍板 / 暂停禁令三张清单
- [ ] 用户知道口令「agent-on 结账」
- [ ] 以上每条都有实际文件路径或命令输出作证（L2 对你自己同样生效）
