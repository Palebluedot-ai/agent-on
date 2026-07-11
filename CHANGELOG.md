# CHANGELOG — agent-on 发布账本

> 职责边界:人读的版本账本;版本真相 = git annotated tag(不设 VERSION 文件)。semver 判据:**major = 不动手会坏 / minor = 不动手不坏 / patch = 不用知道**;major 条目必附迁移注记,否则不许打 tag。L3 规则改动必须成对列出 playbook + kit 双落点。

## [未发布]

### 第二次消化（2026-07-11，7 张卡：外部记忆系统输入 5 + 用户功能两问 2）

去向：6 landed / 1 半落半缓。定级 minor；按用户拍板**攒批至新项目 dogfood 后随 v0.4.0 一并封版**。

- **no-database-stance（入宪）**：CHARTER 边界节「记忆不建数据库」（markdown 即记忆，DB 永远可选旁挂、坏退 grep、不进必需件）+ 拍板链登记
- **auto-snapshot-triggers（L3 双落点；设计经对抗评审击穿四处后定稿）**：playbook/sop.md Phase 0「拍板即快照」（D 表搭车）+ Phase 7「快照三写点」（决策边界/险段前/交接收口，时钟回合永不触发）——playbook 侧；boot/session-handshake.md staleness 标红 + 消费记录、kit/claude-hooks-template.md 机械地板（PreCompact agent 型 hook，Claude 专属可选）——boot/kit 侧
- **conversation-idea-capture（L3 双落点）**：BOOTSTRAP §6 + kit/AGENTS-skeleton §9 想法类捷径（boot/playbook 侧）+ kit/thoughts-and-ideas-template 头部「AI 也会代笔」（kit 侧）——三保险：只进速记区/保守偏置/升级需求归用户
- **distill-merge-abstraction**：settlement 下半场「同类多条先合并抽象成一条 L2」
- **memory-health-visibility（半落）**：settlement 收尾「顺口报成长」播报（数字从文件数出禁手编）；元仪表盘维持 deferred（触发=服务 2-3 项目）
- **deferred 两张入 Backlog**：cases-retirement-tiering、semantic-retrieval-adapter（见下）

### 新增（v0.4 功能）
- **Codex 适配层**（2026-07-11 用户提——兑现宪章承诺 3「工具无关」还停在纸面的机器侧半边）：`codex/` 三件——AGENTS-global-snippet（并入 `~/.codex/AGENTS.md` 的口令路由，Codex 不读 CLAUDE.md）+ prompts/agent-on.md（Codex 自定义 prompt 约定的斜杠命令，与 skill/SKILL.md 互为镜像、arg 无关设计）+ README（接入两行/谁写谁读/卸载/诚实边界：中文口令永远主路）。项目侧零适配（AGENTS.md 双工具原生）。README FAQ「Codex 能用吗」与换机步骤已更

### 新增（v0.4 功能，2026-07-10 用户提；待新项目 dogfood 验证后随 v0.4 发版）
- **项目仪表盘**（源自 Euan `docs/dashboard.html` 已验证原型）：`kit/dashboard-template.html`——纯静态单文件零依赖，五块（当前阶段 / To-do 分「🫵等你·🤖等我」/ 执行计划 / 决策台账 / 里程碑），内容集中在 DATA 对象、从真相源重绘。**接线**：BOOTSTRAP §2 第 8 步播种（M/L）+ merge-checklist 第 7 步合流必更 + 口令「更新仪表盘」。四铁律：从真相源读了重绘 / 禁手填 / 单写者 / 忠于真相不定义真相
- **想法收集箱**（源自 Euan `docs/idea.md`）：`kit/thoughts-and-ideas-template.md`——两区（📥速记区你随手写 + 🗂已整理 AI 维护带日期），整理时归类成文标去向（待评估 / 升级需求→requirements / 结账→agent-on / 搁置 / 弃）。**接线**：BOOTSTRAP 播种（全档）+ session-handshake 握手提醒速记区非空 + 口令「整理想法」
- 两个项目内口令：BOOTSTRAP §6 + 全局 CLAUDE.md 路由——「整理想法」（全档）、「更新仪表盘」（M/L）

## [v0.3.0] - 2026-07-09 · 首次真实闭环转完一圈

> **里程碑**：CHARTER 定义的 v0.3 门槛达成——Euan 仓内 `agent-on/` 倒仓首次结账（@345bad6）+ 首次消化（本次会话）跑通。semver：minor（新增内容/模板行/案例/机件，无 breaking，存量项目可不动手）。这是「项目 → 结账 → 消化 → 发布」闭环的**第一次真实运转**（v0.2 那次是自建过程内部验证，非下游结账口令回流）。

### 首次消化（2026-07-09，23 张 Promotion Card 分诊落地）

来源：[intake/2026-07-09-agent-on-self-review.md](intake/2026-07-09-agent-on-self-review.md)（17 张四镜头评审卡）+ [intake/2026-07-09-euan-flutter.md](intake/2026-07-09-euan-flutter.md)（6 张 Euan 倒仓 delta）。去向：**21 landed / 1 deferred / 1 rejected**，零残余 pending。

- **低风险直落 7 卡**（@72185bb）：README 诚实化（删「已转过一圈」幻觉、加术语表 5 行、换机三步 FAQ、斜杠 quirk 说明）、settlement 首结账空锚分支 + 幂等注、BOOTSTRAP 档播错不重播、kit/README 限定 M/L 启动步骤 + 删倒仓前旧句
- **中高风险用户拍板 12 卡**（@d42344a）：
  - `digest-friction-paste`：settlement 收尾从「一句问句」改为默认动作（可粘贴消化开场 + 项目 loop-notes 待办位）+ session-handshake 读取表加「待消化」行
  - `prose-first-settle-path`：settlement step1 主路径显式定为「loop-notes 散文 → Promotion Card」，jsonl/audit-lint 降为 L 档旁路并标「尚未实战验证」（settlement + README 结构表 + ledger/run-card-logging 头部）
  - **`cases-delivery-channel`（L3 双落点）**：BOOTSTRAP §4 扫坑指针（playbook 侧）+ kit/phase-card-template 内联要点行（kit 侧）——bench/cases 对下游的送达通道接通
  - **`skill-routing-slot`（L3 双落点）**：kit/AGENTS-skeleton 新增 §skill 路由槽（kit 侧）+ BOOTSTRAP §1 第 5 问采集本机 skill 体系（playbook 侧）
  - `lock-model-premium-field`：kit/agent-on-lock-template 加 model + 保费档位行 + session-handshake 握手对表核模型档位
  - `intake-lint-timing`（争议→现在就做）：新增 `ledger/intake-lint.py`（Promotion 六项完整性校验器，反例测试验证硬门有效）
  - `parallel-live-as-discovery`（Euan 3x 复现 → 升 L3）：kit/merge-checklist step5 加「并行轨各跑 LIVE 当发现器」
  - `readonly-guardrails` / `destructive-api-protection` / `eval-goldset-honesty`（Euan 单次 → bench 案例）：新增 bench/cases/18、19、20 + 索引
  - `architecture-radar`（冻结令期不新开篇）：并入 playbook/architect-lens.md 附节（信号→动作→量级），修复原悬空链
  - `semver-clash`（争议→本次即打 v0.3.0）：里程碑语义与版本号同指 minor bump，累积 minor 工作骑进 v0.3.0
- **deferred 1 卡**：`probe-from-cases-zero`（冻结令期不加探针题，转化时机顺延，Backlog 保留）
- **rejected 1 卡**：`scaffold-not-design`（已在 bench/cases/02 + freedom-vs-discipline §三，结账对照清单漏扫，无新增量）

### 拍板（2026-07-09，四镜头评审后用户裁决）
- **v0.3 门槛砍半**：v0.3 = Euan 倒仓首次结账+消化跑通（单件）；新项目 BOOTSTRAP dogfood 顺延为 v0.4 —— CHARTER 成功标准 + README 路线已更
- **冻结令入宪**：首次真实结账前不新增 playbook 篇目 / kit 模板，修订必须走 intake 消化 —— CHARTER 边界节
- 对外节奏：先自用磨 2-3 个项目再谈开源；首批 16 张评审卡入 [intake/2026-07-09-agent-on-self-review.md](intake/2026-07-09-agent-on-self-review.md) 等首次消化

### 新增
- **迭代闭环六站机制**(2026-07-08,三镜头提案 + 对抗评审合成,ABDC 自举):playbook/iteration-loop.md + boot/settlement.md(结账/消化/升级三口令执行书)+ intake/ 承接层 + kit/promotion-card-template.md + kit/agent-on-lock-template.md
- **Bench 案例集**(批三):bench/cases/ 17 张翻车案例卡(Euan 实战 15 + 一代二代标本 2)
- 前身三仓归档标头(批四):agent-orchestration-playbook / communication-governance-playbook / non-drift-communication-protocol 各自 README 指回本仓

### 变更(L3 双落点成对列出)
- 六类触发采集纪律:BOOTSTRAP §6(playbook 侧)+ kit/merge-checklist.md 第 7 步扫尾行(kit 侧)
- lock 播种步骤:BOOTSTRAP §2 第 7 步(playbook 侧)+ kit/AGENTS-skeleton.md §0 指针(kit 侧)
- CHARTER 新增承诺 4「越用越强」(2026-07-07 用户拍板)
- **外部服务集成清单**:sop.md 新增六条实证清单(playbook 侧)+ BOOTSTRAP L7 精简铁律与 phase-card 集成探针既有(boot/kit 侧)——由批三案例审查倒逼产出:5 张卡引用一个从未存在的清单,消化时真的把它建出来。**这是「案例 → 协议升级」回路的第一次真实运转**(2026-07-08)
- **S/M/L 档位路由 + 存量项目接入**(2026-07-08,用户「高射炮打蚊子?已开工的项目怎么用?」两问触发——「脚手架不合身」信号的即时消化):BOOTSTRAP §1 定档三问 + §2 S 档三件套捷径(boot 侧)+ kit/AGENTS-lite.md 轻装宪法(kit 侧)+ boot/adopt.md 存量接入书(考古→定档→增量补件,含升档协议);README 全面重写为「三入口两口令」产品自述
- **`/agent-on` 斜杠命令**(2026-07-08,用户「怎么更丝滑调用」触发):skill/SKILL.md 六子命令(init/adopt/handshake/settle/digest/upgrade)→ 按表导到 boot/ 对应执行书;skill 源随仓版本化,symlink 挂 ~/.claude/skills/agent-on(不占 claude-memory 同步,换机器一行 symlink);disable-model-invocation(与中文口令 CLAUDE.md 路由分工:斜杠管确定性,口令管自然语言)
- **工作流编排防幻觉七件**(2026-07-08,agent-on 融合工程自身实战的即时消化——脚本控制流/schema 强制/证据派工/对抗 stage/断点续跑/单一合流权):playbook/workflow-orchestration.md(playbook 侧)+ kit/workflow-orchestration-checklist.md 任务书七要素+编排七件(kit 侧);与六步协议按「会不会撞文件」分工,补齐七层防幻觉栈的第 2 层 schema 件与第 5 层整层

## [v0.2.0] - 2026-07-07

- 批一:五块骨架(Boot/Kit/Playbook/Bench/Ledger)+ 三代资产迁入 + CHARTER / README / AGENTS / BOOTSTRAP 四门面
- 批二:前身仓精选移植 21 文件——五篇方法论(真相源治理/阶段闸门/元原则/ABDC/沉淀分层)、四张卡 JSON Schema、audit-lint、ABDC 四模板、DoD 门禁、会话握手、深挖问卷、能力真相表、修正闭环
- 融合裁决全记录:snapshot/2026-07-07-fusion-map.md

## [v0.1.0] - 2026-07-01

- 工具定义 snapshot(agent-on 边界拍板)+ Loop Engineering 机制七篇导入(project-kickoff-os 时期)

## Backlog(deferred,等 dogfood 数据说话)

- ~~intake-lint:Promotion 六项机器校验~~ **✓ v0.3.0 落地**(ledger/intake-lint.py,首结账消化时人眼核卡繁琐 = 工程镜头胜出的实证)
- ~~架构雷达机制移植~~ **✓ v0.3.0 落地**(并入 playbook/architect-lens.md 附节,冻结令期不新开篇)
- changelog-lint:「major 无迁移注记不许打 tag」的机器门
- probe-from-cases-zero:案例 08/09/10/14 转化为新探针题(deferred,冻结令期不加;转化时机顺延——冻结令已解除,下次消化可捡)
- 多协作者结账对接(multi-contributor-protocol 条款)
- cases-retirement-tiering:案例集「活跃扫坑清单 vs 归档库」两级可见性——升成 L3 门禁的案例退出默认扫描(触发信号:案例超 ~40-50 张,或 loop-notes 首现「扫坑捞不到/扫一堆无关」)
- semantic-retrieval-adapter:语义检索可选旁挂(如 gbrain 索引本仓)——三到顶信号(README 漏登记≥2 次 / 同坑异措辞漏合≥3 次 / 时机表单场景 >10 卡退化全扫)任一触发才动;三设计闸:永远 optional / 坏退 grep / 不进 BOOTSTRAP 必需件
- memory-health 元仪表盘(触发=服务 2-3 个项目;结账播报半句已于 2026-07-11 先行落地)
- **v0.4 门槛**:一个新项目 BOOTSTRAP dogfood 全流程(v0.3 已达,门槛顺延)
