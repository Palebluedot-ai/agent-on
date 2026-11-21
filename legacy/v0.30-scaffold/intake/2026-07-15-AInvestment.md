# intake — 2026-07-15 · AInvestment

> 来源：AInvestment 项目会话（Grok · agent-on init 纠偏 + landscape + Finviz 规格拍板）。  
> 项目端结账落盘；**未在项目端 commit agent-on**（跨仓边界）。  
> pin：agent-on @ v0.3.0-39-g32fcfb2

### delayed-data-as-product-truth（延迟数据是产品真相，不是临时凑合）
- source:AInvestment @ 701ac95 / cae7297（规格 D16 同批） | pin v0.3.0-39-g32fcfb2
- evidence:① 用户 2026-07-15 明示「价格本来就不是实时的」；② 项目 `docs/product/info-dashboard-spec.md` D16 + 「不声称实时」；③ `AGENTS.md` 时效红线；④ 对照此前规格把 Elite「实时」当缺口话术的写法（已删）
- confidence:high（用户拍板 + 多文件落地）
- claim:做行情/盘面类产品时，**默认延迟**应写成产品语义（时间戳、窗口、源、延迟口径），禁止用「暂时非实时 / 以后对标 Elite 实时」当验收缺口；实时仅在用户显式批准且数据源合同允许时另开 capability，不得从免费 map 参照物静默升级为 live 承诺
- suggested_landing:playbook 可加「外部数据时效语义」短段；kit/prd-template 或 requirement 清单加「数据时效：实时/延迟/日频」必填行；bench 可选案例「把延迟当缺陷导致错误对标付费实时」
- rollback:revert 落地 commit
- trace:loop-notes 2026-07-15 行「价格非实时」；AInvestment commits cae7297→规格修订
- 状态:rejected(用户裁决:项目域知识归项目端——agent-on 只收 AI 协作过程教训;过程内核「外部数据第一天对账口径」已被 sop 集成清单第 3 条载荷对账覆盖,再加一条=指令膨胀。域教训已在 AInvestment 自己的 AGENTS/规格落地,不丢)

### freeze-deferred-channels-as-ban（后置渠道写成暂停禁令，不只「以后再说」）
- source:AInvestment @ cae7297 | pin v0.3.0-39-g32fcfb2
- evidence:① 用户拍板 MVP「先不要推送，web优先」；② `AGENTS.md` 暂停项显式「推送通道…未再批准前不实现」；③ `docs/requirements.md` 暂停表含推送
- confidence:medium（单项目一次，但与既有「暂停项=禁令」同构加强）
- claim:MVP 明确后置的**渠道/触点**（推送、移动端、多租户等）必须进 AGENTS 暂停禁令表，禁止只留在对话「以后做」——否则实现会话会当 soft backlog 偷做
- suggested_landing:kit/AGENTS-skeleton.md §1 暂停项示例补「渠道类」一行；BOOTSTRAP 需求三分法「暂停」举例加渠道
- rollback:revert
- trace:requirements D10；commit cae7297
- 状态:landed@同批(第八次消化:AGENTS-skeleton §1 暂停项行补渠道类 + BOOTSTRAP 需求六问第 6 问补半句)

### interaction-reference-not-asset-clone（参照站点学交互，禁图床/商标克隆）
- source:AInvestment @ cae7297 | pin v0.3.0-39-g32fcfb2
- evidence:① 热力主参照 finviz.com/map?t=sec，同时 requirements 禁 Finviz 资源盗用；② landscape 对 touji 学信号卡语法不抄 crypto 宇宙；③ 用户要求自研时间窗补 Elite 体验缺口，而非账号爬取
- confidence:high
- claim:产品规格引用外部站点时必须拆成「学什么（信息架构/交互/指标语法）」与「不复制什么（资产/商标/付费墙绕过）」两栏；自研补齐付费功能体验时写清**能力对等边界**（如多时间窗），禁止把对方付费实时/账号能力写进验收
- suggested_landing:kit/prd-template 或 design 相关 checklist「参照物」节；playbook/elicitation 品味前置旁注
- rollback:revert
- trace:info-dashboard-spec §3；requirements 暂停「Finviz 资源盗用」
- 状态:landed@同批(第八次消化:BOOTSTRAP 六问 Q2 参照物扩注 + elicitation-protocol 参照物回显条补两栏拆法)
