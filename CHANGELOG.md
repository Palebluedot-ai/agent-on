# CHANGELOG — agent-on 发布账本

> 职责边界:人读的版本账本;版本真相 = git annotated tag(不设 VERSION 文件)。semver 判据:**major = 不动手会坏 / minor = 不动手不坏 / patch = 不用知道**;major 条目必附迁移注记,否则不许打 tag。L3 规则改动必须成对列出 playbook + kit 双落点。

## [未发布]

### 新增
- **迭代闭环六站机制**(2026-07-08,三镜头提案 + 对抗评审合成,ABDC 自举):playbook/iteration-loop.md + boot/settlement.md(结账/消化/升级三口令执行书)+ intake/ 承接层 + kit/promotion-card-template.md + kit/agent-on-lock-template.md
- **Bench 案例集**(批三):bench/cases/ 17 张翻车案例卡(Euan 实战 15 + 一代二代标本 2)
- 前身三仓归档标头(批四):agent-orchestration-playbook / communication-governance-playbook / non-drift-communication-protocol 各自 README 指回本仓

### 变更(L3 双落点成对列出)
- 六类触发采集纪律:BOOTSTRAP §6(playbook 侧)+ kit/merge-checklist.md 第 7 步扫尾行(kit 侧)
- lock 播种步骤:BOOTSTRAP §2 第 7 步(playbook 侧)+ kit/AGENTS-skeleton.md §0 指针(kit 侧)
- CHARTER 新增承诺 4「越用越强」(2026-07-07 用户拍板)
- **外部服务集成清单**:sop.md 新增六条实证清单(playbook 侧)+ BOOTSTRAP L7 精简铁律与 phase-card 集成探针既有(boot/kit 侧)——由批三案例审查倒逼产出:5 张卡引用一个从未存在的清单,消化时真的把它建出来。**这是「案例 → 协议升级」回路的第一次真实运转**(2026-07-08)
- **S/M/L 档位路由 + 存量项目接入**(2026-07-08,用户「高射炮打蚊子?已开工的项目怎么用?」两问触发——「脚手架不合身」信号的即时消化):BOOTSTRAP §1 定档三问 + §2 S 档三件套捷径(boot 侧)+ kit/AGENTS-lite.md 轻装宪法(kit 侧)+ boot/adopt.md 存量接入书(考古→定档→增量补件,含升档协议);README 全面重写为「三入口两口令」产品自述

## [v0.2.0] - 2026-07-07

- 批一:五块骨架(Boot/Kit/Playbook/Bench/Ledger)+ 三代资产迁入 + CHARTER / README / AGENTS / BOOTSTRAP 四门面
- 批二:前身仓精选移植 21 文件——五篇方法论(真相源治理/阶段闸门/元原则/ABDC/沉淀分层)、四张卡 JSON Schema、audit-lint、ABDC 四模板、DoD 门禁、会话握手、深挖问卷、能力真相表、修正闭环
- 融合裁决全记录:snapshot/2026-07-07-fusion-map.md

## [v0.1.0] - 2026-07-01

- 工具定义 snapshot(agent-on 边界拍板)+ Loop Engineering 机制七篇导入(project-kickoff-os 时期)

## Backlog(deferred,等 dogfood 数据说话)

- intake-lint:Promotion 六项机器校验(audit-lint 思路扩展)——首次真实结账验收协议后再定
- changelog-lint:「major 无迁移注记不许打 tag」的机器门
- 架构雷达机制移植(Euan `docs/architecture-radar.md` → playbook):非技术 owner 的 unknown-unknowns 解法,信号→动作→量级
- 多协作者结账对接(multi-contributor-protocol 条款)
- **v0.3 门槛**:Euan 仓内 agent-on/ 倒仓首结账跑通 + 一个新项目 BOOTSTRAP dogfood 全流程
