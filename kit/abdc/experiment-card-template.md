# Experiment Card 模板 — 只给 Top1 做最小可证伪实验

> 何时用:ABDC 第四步(D)。评审选出 Top1 后,不直接开工,先花很小成本证伪它。一张卡只验**一个**假设,动作最小、**≤5 工作日**、成本上限写死,判定三态(Pass/Fail/Inconclusive)。方法论见 [../../playbook/abdc-decision.md](../../playbook/abdc-decision.md)。

标题(Title):最小可证伪实验 - <名称>
日期(Date):<YYYY-MM-DD>
负责人(Owner):<谁>
关联 Decision Brief:<路径>
关联 Track Memo / Review Packet:<路径>

## 1) 被验证的假设(只写一个)
假设:
- 如果 <动作>,那么 <结果>,因为 <理由>。

## 2) 最小实验动作
- 动作(Action):
- 范围(最小):
- 时长(**≤5 工作日**):
- 成本上限:

## 3) 指标与阈值(先写清,别事后凑)
主指标(Primary metric):
- 成功阈值(Pass threshold):
- 失败阈值(Fail threshold):

护栏指标(Guardrail,防止「主指标达标但别处崩了」):
- 护栏 1 + 阈值:
- 护栏 2 + 阈值:

## 4) 通过/失败判定规则(**三态**)
- **Pass**:
- **Fail**:
- **Inconclusive**(不确定时怎么办,必须提前写,如「只延 2 天补证,不许新增轨道」):

## 5) 风险与回滚(**必填,守门条件**)
- 主要风险:
- **熔断条件**(出现即立刻停):
- **回滚步骤**(简要):
- 最大可接受损失:

## 6) 证据采集
- 数据来源:
- 采集频率:
- 记录格式:

## 7) 决策出口(三态各走各的)
- 若 Pass → 下一步:
- 若 Fail → 下一步:
- 若 Inconclusive → 下一步:
