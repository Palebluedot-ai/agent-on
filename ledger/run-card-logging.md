# Run 台账落盘规范:机器可读的那一半

> 职责边界:本篇管「每次编排的运行产物怎么落成机器可读的卡片流」;`run-ledger-template.md` 管人读的那张主表 + 明细段。两者互补——机器台账供审计/统计脚本消费,人读台账供季度复盘。同一次 run,两边各记一次:jsonl 落原始卡片,主表落一行汇总。
> 源流:支线 evolving-memory-system/04_protocols/run-card-logging-v1.md,2026-07-07 批二移植

## 一、为什么要机器可读的一半

人读台账(主表)适合季度翻账、看趋势,但不适合脚本消费——表格里「墙钟 Xmin / 冲突 0」是给人看的,程序难稳定解析。机器台账补这一半:**一次 run 的每一步(派工 / 结果 / 沉淀)各落一张结构化卡片,一行一条 JSON**,让 audit / 统计脚本能直接吃。

关系一句话:**jsonl 是原始颗粒(每步一卡),主表是聚合视图(每 run 一行)——先有卡片,合流时再据卡片汇一行进主表。**

## 二、文件规范

- 路径:`ledger/runs/<run_id>.jsonl`,一个 run 一个文件。
- 编码:UTF-8;格式:**每行一个 JSON 对象(一张 card)**,即 JSON Lines。
- 每张卡的 schema 定义指向 `../kit/schemas/`(卡片 schema 由同批次另一轨在写,这里只引用路径,不复制正文)。

允许的 `card_type` 三种,对应编排三步:
- `task_card` — 派工:orchestrator 交给子代理的任务(scope / 约束 / 验收标准)。
- `result_card` — 结果:子代理交回的完成报告(逐条验收 / 测试输出 / 文件清单 / 契约悬点 / commit)。
- `memory_card` — 沉淀:这步产生的、要进长期记忆的东西(对应 memory-layering 的 L2/L3)。

## 三、四条硬约束

1. **`task → result → memory` 强制顺序**:同一 run 内卡片按这个顺序写。先派工、后有结果、再谈沉淀——顺序错了说明流程本身乱了。
2. **durable 任务必须有 `memory_card`**:凡是产出了跨任务可复用沉淀的任务(memory-layering 判定要升 L2/L3 的),必须落一张 memory_card,不能只在脑子里升层。反过来,一次性任务不强求 memory_card。
3. **`run_id` 与文件名一致**:卡片里记录的 run_id 必须等于 `<run_id>.jsonl` 的文件名部分——不然审计时对不上号。
4. **每张卡有稳定 `task_id`**:同一任务的 task/result/memory 三张卡共享一个 task_id,才能把一条任务的全链路串起来。

## 四、校验与落地

- 校验用 `ledger/audit-lint.py`(与卡片 schema 同批次另一轨在写,这里只引用,不检查是否已存在)。它按 `../kit/schemas/` 校验每张卡的结构、检查上面四条硬约束(顺序 / durable 有 memory / run_id 对齐 / task_id 齐全)。
- 落地时机挂在编排流程上:orchestrator 派工时写 `task_card`,收到子代理完成报告时写 `result_card`,决定沉淀时写 `memory_card`;合流(merge-checklist)时据这批卡汇一行进 `run-ledger-template.md` 主表。
- 校验失败即视为该 run 的落盘不合格——和「验证后才说完成」同源:台账没通过 lint,这次 run 的记录就不算数。

## 五、验收清单(照抄)

- [ ] 产生了 `ledger/runs/<run_id>.jsonl`
- [ ] 至少有 `task_card` + `result_card`
- [ ] durable 任务有 `memory_card`
- [ ] `run_id` 与文件名一致、每卡有 `task_id`
- [ ] `audit-lint.py` 校验通过
- [ ] 主表(run-ledger-template.md)据本 run 汇了一行

一句话带走:**机器台账是原料(每步一卡、可 lint、可统计),人读主表是成品(每 run 一行、供复盘);durable 任务必须留 memory_card,是「沉淀不落盘就等于没沉淀」这条纪律的物理执行点。**
