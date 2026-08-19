# agent-on Kit — 新项目模板层

> 从 Euan-Flutter 七次编排 run(零目录冲突、后四次零返工)原样抽取。**每个模板都被真实使用过,没有一个是想象出来的。**
> 用法:新项目开工时照 §启动步骤 拷贝改名;规则背景见 ../playbook/sop.md 与 ../playbook/model-playbook.md。

## 内容物

| 文件 | 是什么 | 抽取自 |
|---|---|---|
| [AGENTS-skeleton.md](AGENTS-skeleton.md) | 项目宪法骨架(硬约束/动态需求协议/编排并行协议) | Euan AGENTS.md |
| [phase-card-template.md](phase-card-template.md) | 自包含 phase 卡模板(setpoint/disturbance/机械验收) | 14 张实战卡 |
| [track-prompt-template.md](track-prompt-template.md) | 轨道 agent 派工 prompt(含 Opus 七脚手架旋钮) | Run #2-#7 派工词 |
| [review-prompt-template.md](review-prompt-template.md) | 独立对抗审查 prompt(failed→respond→passed) | S2 审查(抓到 Critical 的那次) |
| [deep-research-prompt-template.md](deep-research-prompt-template.md) | 深度调研派工 prompt(v1 骨架 + v2 四纪律:仓内审计先行/授权推翻前提/数字纪律/对抗自核验) | Dartify PR #180(158 断言对抗核验) |
| [merge-checklist.md](merge-checklist.md) | 合流七步 checklist | sop.md Phase 5 + Run #3 教训 |
| [ledger-ratchet-pattern.md](ledger-ratchet-pattern.md) | merge 记账 CI 棘轮模式 | 依从率断档时(Dartify) |
| [worktree-gc-pattern.md](worktree-gc-pattern.md) | worktree 回收执行体 + 孤本保护 | 多 worktree / 日历死线 |
| [worktree-control-plane.md](worktree-control-plane.md) | 多会话轨道合同 + 文件边界/依赖/合流/回收控制面 | 多 Claude/Codex worktree 长期并发 |
| [output-contract.md](output-contract.md) | **每轮输出契约(状态面板在前)**:面板四字段 / 机器类别→中文人话 / 拍板必带默认值 / 结论三格 / 撤销两栏 / 具名角色——派工词与值守文档都引用它,不各自抄 | 2026-08-17 用户实测反馈(多 worktree 并行读不动) |
| [babysit/](babysit/README.md) | 值守合并调度:合并权中央化的值班手册(模板 §0–§7 + 三步接入 + 治理条款范本) | Dartify 值守夜班 9 连合 + 三单实战 |
| [babysit/MERGE-POLICY.md](babysit/MERGE-POLICY.md) | **合入授权与时延的唯一真相**:默认合入档 5 类 / 必须先问档 / 门铃即起跑 / 一个口令切高频 / 时延目标 X = CI 中位 + 5 分钟 | 同上 + 2026-08-17 用户「值守太慢」实测 |
| [babysit/ROUTING.md](babysit/ROUTING.md) | **「谁执行」的唯一真相**:合并权/对外通信权/跨窗口中转权三条唯一归值守 · 发错窗口的指令按【转投】模板转投不执行 · `agent-on oncall` 在班登记与 PreToolUse 路由闸(无人在班 fail-open) | 2026-08-19 用户拍板「只由一个值守负责，发错窗口的直接转过去」 |
| [progress-template.yaml](progress-template.yaml) | 单写者状态文件骨架 | docs/state/progress.yaml |
| [run-ledger-template.md](../ledger/run-ledger-template.md) | Run 台账 schema(含成本列 = Ledger 层) | run-log.md + 混编经济学 |
| [capability-probe.md](../bench/capability-probe.md) | 新模型能力探针(定保费档位用) | model-playbook §二 |

## 启动步骤(M/L 档,新项目第一小时)

S 轻装档不走这里——三件套捷径见 BOOTSTRAP §2,一分钟播完。

0. 规划链(BOOTSTRAP §1.5,M/L):MRD→澄清→PRD→plan→审查,产物落 `docs/{product,requirements,plans}/`,每环节收口即 commit
1. 建目录:`contracts/fixtures/` `docs/{state,phases,snapshots}/`
2. 拷 AGENTS-skeleton.md → 项目根 `AGENTS.md`,填 [占位];拷 progress-template.yaml → `docs/state/progress.yaml`
3. 需求三分法(sop Phase 0):硬约束进 AGENTS §1;**暂停项写成禁令**,不是删掉
4. 品味前置(../playbook/freedom-vs-discipline.md §三):参照物锚点 + Explore 画廊投票,**在第一行业务代码之前**
5. 若用新模型:先跑 capability-probe,定本期脚手架档位
6. 每片循环:phase 卡 → 冻契约(**连语义:排序/空值/上限**)→ 多写轨时先 claim 轨道合同 → track-prompt 派工 → merge-checklist 合流 → review-prompt 收口 → run-ledger 记一行

## 四条不许省的纪律(kit 之魂)

TDD(没有失败测试不写生产代码)/ Error Signal 四要素(What/Where/How/Severity)/ 验证后才说完成(贴命令实际输出)/ 单一状态写者(progress.yaml 只有 orchestrator 写)。

**外加一条汇报纪律**:所有会话与子代理的每轮输出走 [output-contract.md](output-contract.md)——状态面板在前,拍板收成一节带默认值,最后一行「球在谁那」。做对了但说不清,用户照样卡住。
