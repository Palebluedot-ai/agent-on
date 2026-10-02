# agent-on Kit — 新项目模板层

> 职责边界：本页是可选模板目录与选用指南，不要求项目全装；初始化规则以 BOOTSTRAP 的最小骨架为准。
> 从 Euan-Flutter 七次编排 run(零目录冲突、后四次零返工)原样抽取。**每个模板都被真实使用过,没有一个是想象出来的。**
> 用法:新项目开工时照 §启动步骤 拷贝改名;规则背景见 ../playbook/sop.md 与 ../playbook/model-playbook.md。

## 内容物

| 文件 | 是什么 | 抽取自 |
|---|---|---|
| [AGENTS-skeleton.md](AGENTS-skeleton.md) | 项目宪法骨架(硬约束/动态需求协议/编排并行协议) | Euan AGENTS.md |
| [phase-card-template.md](phase-card-template.md) | 自包含 phase 卡模板(setpoint/disturbance/机械验收) | 14 张实战卡 |
| [track-prompt-template.md](track-prompt-template.md) | 轨道 agent 派工 prompt，按实际任务表现选辅助 | Run #2-#7 派工词 |
| [review-prompt-template.md](review-prompt-template.md) | 独立对抗审查 prompt(failed→respond→passed) | S2 审查(抓到 Critical 的那次) |
| [deep-research-prompt-template.md](deep-research-prompt-template.md) | 深度调研派工 prompt(v1 骨架 + v2 四纪律:仓内审计先行/授权推翻前提/数字纪律/对抗自核验) | Dartify PR #180(158 断言对抗核验) |
| [merge-checklist.md](merge-checklist.md) | 合流七步 checklist | sop.md Phase 5 + Run #3 教训 |
| [ledger-ratchet-pattern.md](ledger-ratchet-pattern.md) | merge 记账 CI 棘轮模式 | 依从率断档时(Dartify) |
| [worktree-gc-pattern.md](worktree-gc-pattern.md) | worktree 回收执行体 + 孤本保护 | 多 worktree / 日历死线 |
| [worktree-control-plane.md](worktree-control-plane.md) | 多会话轨道合同 + 文件边界/依赖/合流/回收控制面 | 多 Claude/Codex worktree 长期并发 |
| [output-contract.md](output-contract.md) | 按场景汇报：普通进展简短，交付/失败/需决策时补证据与责任人——派工词与值守文档都引用它,不各自抄 | 2026-08-17 用户实测反馈(多 worktree 并行读不动) |
| [babysit/](babysit/README.md) | 值守合并调度:合并权中央化的值班手册(模板 §0–§7 + 四步接入 + 治理条款范本) | Dartify 值守夜班 9 连合 + 三单实战 |
| [babysit/MERGE-POLICY.md](babysit/MERGE-POLICY.md) | **合入授权与时延的唯一真相**:自动合入是默认(fail-open) / 硬停五类 + 需播报两类 / 审计跑不通就退回先问 / 门铃即起跑 / 时延目标 X = CI 中位 + 5 分钟 | 同上 + 2026-08-17 用户「值守太慢」实测 + 2026-08-20 用户全权授权 |
| [../tools/merge-audit/](../tools/merge-audit/README.md) | **值守自动合并的独立审计员**(Python 标准库,零依赖):合前 `precheck` 判档 / 合后 `record` 记账 / 事后 `scan`+`report` 从 GitHub 真相独立重判,点名 VIOLATION·MERGED_RED·UNRECORDED·MISMATCH;规则的可执行真相在它的 `policy.json` | 2026-08-20 用户「记录并监控它有没有越界行为」 |
| [babysit/ROUTING.md](babysit/ROUTING.md) | **「谁执行」的唯一真相**:合并权/对外通信权/跨窗口中转权三条唯一归值守 · 发错窗口的指令按【转投】模板转投不执行 · `agent-on oncall` 在班登记与 PreToolUse 路由闸(无人在班 fail-open) | 2026-08-19 用户拍板「只由一个值守负责，发错窗口的直接转过去」 |
| [progress-template.yaml](progress-template.yaml) | 单写者状态文件骨架 | docs/state/progress.yaml |
| [dashboard-template.html](dashboard-template.html) + [dashboard-check.mjs](dashboard-check.mjs) | 可选项目仪表盘+ DATA 求值闸:漏逗号 / 空洞 / 混型集合,重绘后跑一次再报完成 | IPONews / Dartify #289·#292 |
| [run-ledger-template.md](../ledger/run-ledger-template.md) | Run 台账 schema(含成本列 = Ledger 层) | run-log.md + 混编经济学 |
| [capability-probe.md](../bench/capability-probe.md) | 实际能力缺口的可选诊断 | model-playbook |

## 按需选用

默认轻装只需规则映射、lock 与 loop-notes；复用项目已有目标、待办和状态。持续任务缺少续接记录时选状态或任务卡，接口两侧实际并行时补契约，共享改动由入口调和。规划、仪表盘、能力诊断、独立审查与 run 卡分别按实际缺口选择，不把 M/L 标签变成整包安装义务。

用户已明确目标就沿授权开工；不要求先登记 claim、换模型考试、补完整 MRD/PRD 链或每轮套六段报告。完成说明相关验证的实际结果；高风险任务保留相应测试和审批边界。共享状态只指定一个写者，任务作者维护自己的记录。

普通回复说明进展与下一步；交付、失败或需决策时按 [output-contract.md](output-contract.md) 补齐相应证据与责任人。
