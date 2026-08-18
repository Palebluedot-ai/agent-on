# 案例 38:全 job 秒死 = 账单层,不是代码

> 层级:L2 | 来源:Dartify 2026-08-16 | 入册:2026-08-17

## 症状
18:52Z 起所有 Actions run 的全部 job 1–4 秒 failure、`steps:[]`、`--log-failed` 报 "log not found";Re-run 同症状复现。org 级瘫痪约 6.5 小时,每条 PR 看起来都在「红」。

## 修法
别按测试红分诊、别反复 Re-run。拉 job annotation 取证——实证文案 "The job was not started because recent account payments have failed or your spending limit needs to be increased"(run 31902347328)。只有账号管理员能修:值守动作 = 取证 + 推通知 + 每轮一次最小探针测恢复。用户修 billing 后同一 run rerun 全绿。

## 可复用规则
CI 全线 job 秒死且 step 零执行 → 先拉 job annotation 查账单/配额层,再想代码;这类红不属于任何 PR,也不该堵任何合流。

## 已固化到哪
kit/babysit/BABYSIT-TEMPLATE.md §4(分诊手册)。
