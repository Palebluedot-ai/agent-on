# 案例 33:配置面绿 ≠ 运行面成功

> 层级:L2 | 来源:SalesDashboard + onboard-bot-lark 2026-08-15 | 入册:2026-08-16

## 症状
1. launchd 已装、`runs=1`,到点了——但 `last exit code=1`,stderr 是 Graph DNS 失败,xlsx 没更新。
2. 飞书权限已批、审批已过、发件邮件已通——`wrangler tail` 零条平台 POST,bot「没反应」。
3. 开户 bot 挂在已接 Hermes 的同一 Lark app 上,事件被另一消费者收走或根本不到 Worker。

## 修法
定时验收固定三件:调度器 runs+exit、任务日志、下游产物。Webhook 第一步看入站流量,不是看权限页。新产品面用专用 bot + 专用 URL。

## 可复用规则
配置勾选 / 日历开火 / 权限已批,都只证明控制面动过。完成条件是运行面可观察:exit=0 且产物在,或 tail 见平台 POST。

## 已固化到哪
anti-hallucination C 附3;sop 集成清单 12–13;kit/merge-checklist 5b;phase-card「运行面」。
