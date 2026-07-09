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
| [merge-checklist.md](merge-checklist.md) | 合流七步 checklist | sop.md Phase 5 + Run #3 教训 |
| [progress-template.yaml](progress-template.yaml) | 单写者状态文件骨架 | docs/state/progress.yaml |
| [run-ledger-template.md](../ledger/run-ledger-template.md) | Run 台账 schema(含成本列 = Ledger 层) | run-log.md + 混编经济学 |
| [capability-probe.md](../bench/capability-probe.md) | 新模型能力探针(定保费档位用) | model-playbook §二 |

## 启动步骤(M/L 档,新项目第一小时)

S 轻装档不走这里——三件套捷径见 BOOTSTRAP §2,一分钟播完。

1. 建目录:`contracts/fixtures/` `docs/{state,phases,snapshots}/`
2. 拷 AGENTS-skeleton.md → 项目根 `AGENTS.md`,填 [占位];拷 progress-template.yaml → `docs/state/progress.yaml`
3. 需求三分法(sop Phase 0):硬约束进 AGENTS §1;**暂停项写成禁令**,不是删掉
4. 品味前置(../playbook/freedom-vs-discipline.md §三):参照物锚点 + Explore 画廊投票,**在第一行业务代码之前**
5. 若用新模型:先跑 capability-probe,定本期脚手架档位
6. 每片循环:phase 卡 → 冻契约(**连语义:排序/空值/上限**)→ track-prompt 派工 → merge-checklist 合流 → review-prompt 收口 → run-ledger 记一行

## 四条不许省的纪律(kit 之魂)

TDD(没有失败测试不写生产代码)/ Error Signal 四要素(What/Where/How/Severity)/ 验证后才说完成(贴命令实际输出)/ 单一状态写者(progress.yaml 只有 orchestrator 写)。
