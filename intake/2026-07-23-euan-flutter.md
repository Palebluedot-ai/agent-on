<!-- agent-on 结账落盘·项目端只写此文件,不碰 agent-on git(跨仓边界 2026-07-13) -->
# intake 2026-07-23 · Euan-Flutter

> 来源会话:demo v9 完全对齐(D21)五轮修复 + 交接。增量锚 = 2026-07-21 结账之后。
> 全部为 **AI 协作过程教训**(编排/纪律/防幻觉);项目域知识(CRM 业务/wire 口径)留项目端,未出仓。
> pin v0.3.0;项目 HEAD e6a02e2(PR #9,claude/frontend-backend-integration-plan-7c9f64)。

### wiring-anchor-def-not-integration(「定义齐全 ≠ 接线生效」是完成幻觉的独立子类)
- source:Euan-Flutter @ e6a02e2 | pin v0.3.0
- evidence:轮2 `euanBottomSheetTheme` 定义完整(纯白底/圆角24/把手38×4)且有「常量形状」测试全绿,却从没接进 main.dart 的 ThemeData → 死代码,被判完成;轮3 `ab2c3b5` 补「接线锚」修死,变异实证=拆掉 main.dart 接线该锚即红、md5 复原;轮4 review 按此纪律再抓出 3 处同类(customers 漏传 tier / 等)。
- confidence:high(一处犯、institutionalize 后连抓三处,失败模式稳定)
- claim:凡新建共享组件/主题/常量,交卷前必须有一条「接线锚」测试——pump 真实消费方断言效果生效,并变异一次(拆掉接线该锚必须红);只验「常量/组件自身形状」的测试挡不住「定义齐全但没接进消费方」的死代码,这是与「压根没测」不同的一类完成幻觉。
- suggested_landing:playbook/anti-hallucination.md 增「完成幻觉子类:integration-gap(定义≠接线)」+ verification-before-completion 检查项「新建共享件跑一次接线锚变异」;或 bench 案例。
- rollback:revert 落地 commit(纯文档/纪律,无生产影响)。
- trace:本会话轮2→轮3 收口;commit ab2c3b5。
- 状态:landed@同批(第十三次消化:anti-hallucination C 附 + phase-card 接线锚/护栏实体条;与 guardrail-entity 语义归并)

### upstream-digest-not-spec(流水线里上游产物是导航不是真相,下游实现前回读一手源)
- source:Euan-Flutter @ e6a02e2 | pin v0.3.0
- evidence:多 agent 审计工作流产出的「动效审计摘要」两处错——G7 把 340ms 挂错层(demo 实为卡面 .3s / 预览 .34s 两条通道)、C13 摘要写「scale 固定 0.97」实为「0.97+0.03v 跟随」;两处都是 implementer 读 demo 一手源(客迹 v9.dc.html 4812-4859 / 6572-6576)逐行才发现。commit `029e5ab`(deck G7 勘误 + 审计报告更正)、`9523ddc`(C13 逐行读源实现)。
- confidence:high(同一工作流两处独立命中)
- claim:多 agent 流水线里,下游 agent 消费上游产出的审计/摘要/spec-digest 时,实现前必须回读一手源核对关键数值与结构——编排时给下游卡片显式写「读源不读摘要,摘要错了就地勘误回写」;把中间产物当权威会把上游的错原样实现。
- suggested_landing:playbook 编排篇增「流水线中间产物的下游核验义务」;或 orchestration 卡「fan-out 产出物做下游 spec 时必带 re-read-source 指令」。
- rollback:revert 落地 commit。
- trace:motion-glass-audit 工作流 → 轮3/轮5 实现;commit 029e5ab / 9523ddc。
- 状态:landed@同批(第十三次消化:workflow-orchestration §一.3 半句 + checklist 源料行)

### workflow-resume-on-quota-death(workflow 中途熄火走断点续跑,不整轮重发)
- source:Euan-Flutter @ e6a02e2 | pin v0.3.0
- evidence:轮2/轮3/轮4 各遇一次限额熄火(session limit / Fable 5 limit),三次都用 `Workflow({scriptPath, resumeFromRunId})` 原地续跑——同 (prompt,opts) 的已成功卡走缓存、只重跑死的;熄火代理留下的孤儿测试文件先移 scratchpad 保工作树干净再续;其中一次转 Opus 后续跑,代理继承新模型。run:wf_af662187-73f / wf_c87e14a2-5f1 / wf_e46ee655-187 的 resume。
- confidence:medium(本项目多次,跨项目未验)
- claim:长 workflow 中途熄火(限额/模型额度/会话上限),先看工作树是否干净(熄火代理常留半成品,移走保清白),再 {scriptPath, resumeFromRunId} 续跑——同 (prompt,opts) 自动走缓存,只重跑死的和新增的;别整轮重发(浪费 token + 重做已落地的卡)。
- suggested_landing:playbook 编排篇 workflow 操作段增「熄火续跑」小节(清树→resume)。
- rollback:纯操作经验,无落地 commit;误则删该小节。
- trace:本会话三次 resume;/loop 兜底心跳里也写了「限额熄火按重置时间重发」。
- 状态:pending

### interleaved-shared-file-single-commit(并行卡改动在共享文件交织时合一笔 bisect,卡内明细进 message)
- source:Euan-Flutter @ e6a02e2 | pin v0.3.0
- evidence:轮4 玻璃/微交互/触觉三卡在 today/customers/me/shell 等屏文件里改动交织(同一屏既有玻璃头挂载又有手势触觉),硬拆三笔中间态编译/测试不过;合为一笔 `3189167`,卡内明细写进 commit message。此前轮1-3 文件域互斥的卡都是逐卡原子提交。
- confidence:medium(本项目一次清晰判例)
- claim:并行卡的改动在共享文件里交织时,别硬拆成 N 笔(中间态可能编译/测试红);合为一笔 bisectable commit,message 里列卡内明细。「原子提交保 bisect」的前提是每笔自洽通过——做不到自洽的拆分不如一笔诚实。
- suggested_landing:playbook 编排篇「bisectable 原子提交」纪律补一条边界:自洽通过做不到时合一笔+message 明细。
- rollback:纯纪律,无生产影响。
- trace:本会话轮4 收口;commit 3189167。
- 状态:pending
