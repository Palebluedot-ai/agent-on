# intake:onboard-bot-lark 2026-08-15 结账（首次·Lark Worker 联调批，2 卡）

> 背景一句话：把「发开户包」从 inbox-radar 拆成独立 Cloudflare Worker 后，多轮「权限已开 / 审批已过 / 发件邮件已通」但 Lark 私聊仍无反应；定案靠 wrangler tail 无任何来自飞书的 POST。两卡都是 **平台接入与验收顺序** 的 AI 协作教训，不含 Graph 业务域规则。

### platform-permission-is-not-event-subscription（权限勾选 ≠ 事件已订阅：Webhook 验收看流量不看勾选）
- source:onboard-bot-lark @ 8b5e6c4 | pin v0.9.1
- evidence:Worker 已 deploy 至 `https://onboard-bot.mintoken-ai.workers.dev`（版本含 `8b5e6c4` 绑定 `cli_aaf70dd680619eed`）；Graph 烟测 HTTP 202 用户已收信；开放平台侧已批 `im:message` 等权限。同期 `wrangler tail` 仅见本机 curl 的 `url_verification` probe，**零条** Feishu/Lark User-Agent 的 POST。用户报告「发了 发开户包 没有任何反应」。界面事件库常不显示字面 `im.message.receive_v1`，需在 **Messenger / 消息与群组** 下订 **Receive messages**，且请求地址校验成功——权限页与事件页是两套清单
- confidence:medium（单项目多轮复现；机理对任何「开放平台权限 + 事件回调」双配置的集成普适）
- claim:接 IM/Webhook 类开放平台时，**权限开通与事件订阅分开验收**：权限只解决「能不能调 API / 能不能看见事件名」；**没有把「接收消息」订到你的公网 URL，平台就不会推事件**。用户报「bot 没反应」时，第一步固定 `wrangler tail` / 服务端 access log 看有无入站事件 POST——**禁止用「权限页已勾 / 审批已过」推断回调已接通**。UI 事件名往往是「Receive messages / 接收消息 v2.0」，不要只搜 API 字面量
- suggested_landing:playbook 外部依赖/联调验收一节；bench 案例；kit 交付或联调 checklist 一行「tail 见入站事件」
- rollback:revert 落地 commit
- trace:loop-notes 接入后首批联调；会话多轮「权限设完仍无反应」；commits 8b5e6c4 / 5aa6d845 deploy 链
- 状态:landed@同批（C 附3 + sop 13 + merge 5b + bench 33）

### one-bot-one-webhook-url（多用途 bot 共用事件 URL 会静默吞回调：专用 bot 或专用 URL）
- source:onboard-bot-lark @ d484e9d | pin v0.9.1
- evidence:首版曾复用 inbox-radar / Hermes 同系 Lark app（`cli_a943…` 等）；用户确认 bot「连了 Hermes」后发开户包无卡片。计划与 README 已写「共享 callback 会与 multi-purpose Worker 碰撞」。后改为专用 app `cli_aaf70dd680619eed` + 单一 Worker URL；在事件未订满前 tail 仍为空，进一步证明 **URL 指向与订阅缺一不可**
- confidence:medium（单产品线，但与「一个 webhook 入口一个消费者」同构，常见于 Slack/Telegram/飞书）
- claim:Lark/飞书 **一个应用事件订阅通常只有一个请求地址**；把开户 bot 挂在已接 Hermes/编排网关的 app 上，事件会被另一消费者收走或根本不到你的 Worker。新产品面优先 **专用 bot + 专用 Worker URL**；若必须共用，要在同一入口显式路由，禁止假设「审批通过就会到我这边」
- suggested_landing:playbook 外部依赖 / 多消费者路由；bench 短案例；与上卡并列「接入检查单」
- rollback:revert 落地 commit
- trace:会话用户原话「这个 bot 连了 hermes」；plan Risks；commits d484e9d→8b5e6c4 换 bot 序列
- 状态:landed@同批（C 附3 + sop 13 + merge 5b + bench 33）
