# 案例 29:外部副作用不明时先 read-back——禁止盲重放

> 层级:L2 教训 | 来源:inbox-radar 2026-07-30/31 lark-cli 投递 | 入册:2026-07-31 第十七次消化

## 症状
`lark-cli` 实际返回 `{ok:true,data.message_id}` 并送达 7 次,旧解析器误报 `transport_failed`。若按失败自动重试会重复骚扰收件人。recovery journal 正确保持 `delivery_attempting`、拒绝重发。

## 根因
把「解析失败」等同「传输失败」;未用只读 API 对账外部实况。

## 修法
ambiguous 结果 → 冻结 replay → 按时间窗/收件人/幂等 id 做只读 read-back → 再决定对账或重试。修复加 34 项相关测试。

## 可复用规则
可能已产生外部副作用时,禁止盲重放;先权威 read-back。

## 已固化到哪
playbook/sop.md Phase 6½ 第 8 条;merge-checklist 7d 远端权威 API。
