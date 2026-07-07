# Loop 沉淀笔记(agent-on 素材,随切片追加)

## 2026-07-03 · S1 真机联调:vercel dev 三连坑(可复用排障 loop)
症状:App 报"网络不给力"。排障链:curl health(NOT_FOUND)→ 换官方 Hono 模式(api/index.ts + vercel.json rewrite,弃 [[...route]] 通配)→ 函数挂起 30s(hono/vercel 是 fetch 风格,本地 Node 运行时等 (req,res))→ 换 @hono/node-server@1 的 /vercel 适配器(v2 删了该出口)→ body 空(vercel dev 无视 bodyParser:false)→ 网关双路兜底 readJsonBody。
**可复用 loop**:本地网关不通时,永远先 curl 分层定位(health→带体端点→带鉴权端点),每层只修一个变量;平台本地模拟器(vercel dev)的行为≠生产,兜底写在服务端而不是跟模拟器缠斗。

## 2026-07-03 · 用户设计反馈(S1 真机首见)
「UI 和我想象的完全不一样,不好看」——S0/S1 骨架未经视觉工序,预期内;但暴露流程缺口:S0.3 跳过了「Pencil 稿先行」。纠正:S2 起核心屏先 Pencil(对照 handoff Daylight 基因)再写码;v1 收尾统一 /design-review。
**可复用 loop**:骨架期就该给用户打预防针「这是线框不是设计」,否则第一眼失望是必然。

## 2026-07-03 · 可复用 Loop:契约先行的双轨并行(agent-on 核心模式)
**问题**:单 agent 串行,前后端互相等;多 agent 裸并行,撞文件撞状态(本项目 fork 撞车实证)。
**解法(六步协议)**:①契约先行(fixtures+schema 定稿,~30min,这步绝不并行)→ ②轨道=目录+git worktree 物理隔离 → ③各轨 fake 对方(前端 FakeRepo 对 fixtures 开发,后端照 fixtures 实现,零等待)→ ④契约测试当裁判(双端消费同一 fixture 文件,擅改=两端 CI 同红)→ ⑤单一状态写手(orchestrator 主会话独占 progress.yaml/合并/审查,子代理只干活汇报)→ ⑥合并顺序:先契约后实现。
**通用化条件**:任何"接口两侧"的工作都适用(前后端/服务间/SDK 与消费方);关键投资是第②④步的基建(worktree 纪律+共享 fixtures 机制)——本项目在 S0.4 就埋了 fixtures 机制,并行时零改造直接兑现。
**harness 启示(agent-on)**:orchestrator 的三顶帽子(派活/收货审查/唯一状态写手)必须在同一个脑子里,拆开就回到撞车;子代理的任务书=自包含 phase 卡,天然适配。

## 2026-07-03 · env 行内注释:三种解析器三种行为(经典环境陷阱)
症状:同一请求,curl/vitest 全 200,经 vercel dev 必 503,且用户的另一消费方正常——排除了 key/上游/网络。
定位法:二分(绕开进程直跑=好 → 嫌疑锁定进程)→ 给进程加一行 env 自曝 → 抓到 model="x-ai/grok-4.3          # 与旧 PWA 管线一致…"——.env 的行内注释被 vercel 的 dotenv 解析器并进了值,而 shell source 和 node --env-file 都会剥掉它。
**规范(已执行)**:.env 值行禁止行内注释,注释一律独立成行。
**可复用 loop**:环境型"薛定谔故障"(A 路通 B 路不通)优先怀疑配置解析差异,让故障进程自曝实际 env 是最快的一刀。


## 2026-07-03 · Run #3 的两条模式级教训

1. **Fixture 的隐性契约维度**:JSON fixture 明面锁"形状",暗面还带着"顺序/上限/空值语义"。Run #3 撞车点=orchestrator 冻结 fixture 时手写顺序与 phase 卡排序规则不一致,两轨各信一边。Loop 化对策:冻结前跑一个 fixture-lint(排序字段自查),或干脆规定 fixture 一律按响应端排序规则生成。
2. **测试的"环境假设"会被合流翻转打破**:provider 默认从 Fake 翻 Dio 后,所有会渲染到该 provider 的测试都需要显式钉 Fake。S3 时 helpers 集中钉住了,但"只钉 auth 就进壳"的旧测试(S1 写的)漏网。对策:翻转 checklist 固定一条——grep 全部 pumpWidget(EuanApp) 的测试,确认 overrides 用的是全量 helper。


## 2026-07-04 · 部署日双坑

1. **@vercel/node 的 ESM 陷阱**:`type:module` 下它把 TS 编译成 ESM JS 但**不重写 import specifier**——本地 vercel dev 宽容免后缀,生产 Node 严格要 `.js`。修法=全仓相对 import 机械补 `.js`(84 处,正则一把过,tsc+vitest 双验)。教训:**本地 dev 与生产的运行时差异要在第一次部署时就冒烟**,别攒到上线前。
2. **PostgREST 时间戳微秒窗**(S7.1 轨发现):DB 原生 created_at 带 6 位微秒,`toISOString()` 截到毫秒后 `eq` 永不中、`lt` 留微秒窗跳行。修法=cursor 保留 DB 原生串不归一。这类「精度被序列化悄悄吃掉」的坑,只有 LIVE 实测能抓——单测的 fake 时间戳自己就是毫秒的。


## 2026-07-04 · 适配器信任的边界(生产 POST 全挂)

@hono/node-server/vercel 的 handle() 在本地 vercel dev 一切正常,生产运行时上 **POST 一律 504**(流式 body 永不结束,连 Content-Length:0 都挂)。GET 正常 → 症状极具迷惑性。修法:弃用适配器,自建 20 行 (req,res) 桥——body 全量缓冲(控制面 JSON 都小,平台 4.5MB 兜底),流不存在就不会挂;env 继续传 {incoming} 保住 dev 的 readJsonBody 兜底路径。
教训:**第三方适配器 = 又一层「本地与生产行为分叉」的来源**;桥接层足够薄时,自己写比信任适配器更可控。冒烟清单加一条:部署后 GET 和 POST 都要打一发。


## 2026-07-04 · basil:API 版本迁移的静默字段搬家

Stripe 2025-03-31(basil)把 `current_period_end` 从订阅顶层挪到 `items.data[]` 层。单测全绿(fixture 按旧文档写)、webhook 200、状态同步正常——只有 period_end 静默为 null。生产实测才暴露。修法:双层读取(顶层优先,items 兜底)+ 用真实载荷形状补测试。
教训:**外部 API 的「文档记忆」是幻觉高发区**——模型学的是旧版文档,SaaS 的字段搬家不报错只给 null。防法:集成第一次 LIVE 时,把真实载荷 dump 下来跟代码假设对一遍(「载荷形状对账」进 SOP 集成清单)。


## 2026-07-04 · 性能日三连:平台的三个脾气

1. **地域错配是最大的隐形延迟**:Vercel 函数默认 iad1(美东),DB 在新加坡——每查询横跨太平洋。`"regions":["sin1"]` 一行:/reviews 289→42ms(7×),/today 1591→333ms。教训:**接外部服务第一天就对齐地域**,这比一切代码优化都便宜。进 SOP 集成清单:「函数区=数据区」。
2. **Vercel 剥函数的 Server-Timing 头**(平台保留),双发 `x-server-timing` 穿透;**304 响应更是剥光所有自定义头**——304 的价值在省 body,头断言只在应用层测试成立。教训:平台对响应的「整形」也是本地/生产分叉源,canary 要测真实边缘出口。
3. **B5×B6 咬合**(网关轨发现):签名 URL 每次重签会让 ETag 永不相同——**缓存链上任何一环的不稳定都会废掉下游所有缓存**。复用窗口(10min 签/5min 复用)让 memo 与 304 互相成立。


## 2026-07-04 · 「echo n 不是护栏」:CLI 的 agent 模式自动确认事故

复盘:用 `echo n | supabase config push` 想只看 diff 不应用——但 CLI 检测到非交互环境(--agent auto)**直接自动确认**,把最小 config.toml 里未声明的键全推成了 CLI 默认值:邮箱验证被关(安全闸门失效)、MFA TOTP 被关、storage.vector 被开。靠行为探测(注册后立即登录)发现验证已失效,10 分钟内声明式修复并复测回位。
三条铁律入 SOP:
1. **任何会写状态的 CLI,在 agent 环境下都假定它自动 yes**——「管道喂 n」不是护栏,查 --dry-run/--agent 旗标才是;没有 dry-run 就先在无害目标上试行为。
2. **声明式配置必须全量声明**:工具的「最小文件」语义可能是「未声明=重置为默认」,不是「未声明=不动」。在乎的键,等于默认值也要写出来。
3. **配置变更后跑行为探测**,不要只信工具回执——「注册后立即登录应被拒」这种一行探针,比读十页文档可靠。


## 2026-07-04 · 架构雷达:非技术决策者的「不知道自己不知道」解法

用户(PM/业务背景)提出真需求:「我不知道自己不知道什么,需要你主动提醒边界/框架/前沿」。制度化解法=**架构雷达**(docs/architecture-radar.md):
① 坐标系(每维度:我们在哪/主流在哪/**什么信号该动**/动的成本)② 地雷清单(平时不响踩了才响的非代码雷)③ 触发式重构预案表 ④ 扫描节律(里程碑必扫+随时口令「扫雷达」)。
关键设计:**用「信号→动作→量级」代替「建议堆」**——非技术决策者不需要理解方案,只需要认得信号和批预算。这是 agent-on 的可复用模式:任何 AI+非技术 owner 的组合都需要一台雷达。


## 2026-07-05 · 官网 v1 返工复盘 → 前置追问协议

三个返工缺口全在 orchestrator spec(待拍板被静默选边/范围靠推断/单方向违反自家投票原则),执行 agent 无责。同项目对照组一锤定音:Calendar 4 变体零返工 vs 官网 1 方向全返工——**变体的边际成本线性,返工的成本全量**。产出:elicitation-protocol.md(偏好缺口=幻觉第五类+2×2 判据+五协议)+ kit/explore-prompt-template.md(烧进模板,下次自动生效)。
