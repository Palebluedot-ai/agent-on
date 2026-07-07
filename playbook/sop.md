# Loop Engineering 开发 SOP v1

> 从 Euan-Flutter 全程实证提炼(2026-07-02 需求 → 2026-07-04 六个切片代码完成)。
> 目的:下个项目直接照此跑。每一步都标了本项目的实证证据,不是理论。
> 配套读物:[anti-hallucination.md](anti-hallucination.md)(为什么这套脚手架能压幻觉)。

## 全景:七个阶段

```
0 需求整理 → 1 澄清文档(MRD/PRD) → 2 多声音评审 → 3 架构与切片
→ 4 细颗粒执行 → 5 双轨并行 → 6 审查与QA → (循环 4-6 直到切片打完) → 7 收口沉淀
```

---

## Phase 0 — 需求整理(原料不加工)

1. **原始想法原样落盘**:`Ideas and thoughts/YYYY-MM-DD_HH-MM-SS.md`,口述/截图/碎片全收,**不让 AI 改写**——改写=第一个幻觉入口。
2. **需求澄清会**(人机对话):AI 只提问不脑补,产出 **D 编号决策表**(requirements.md):每条 = 决策项 + 结论 + 日期。悬而未决的写「待定」,不许 AI 填默认值。
3. **边界三分法**(最关键的一步):
   - **硬约束** → AGENTS.md(如「Thin Client 只走网关」「不直连 Supabase」)
   - **v1 范围内** → requirements/PRD
   - **显式暂停项** → 写成禁令:「产品状态机/加密策略=以后再聊,**未获明确指令前不实现、不假设**」
   - 实证:用户 7-04 问「加密放进计划了吗」,一查便知:暂停项有据可查,AI 没偷跑。

## Phase 1 — 澄清文档(MRD → PRD)

- 市场/UX 研究单独成文(参照物锚点:本项目=Attio/Things/eBay 确认卡),给后续 UI 决策一个「像什么」的锚。
- **PRD 用 F 编号**:每个功能 = 用户故事(一句话讲清为谁省了什么)+ 功能点 + **验收标准 checkbox**。
- 验收标准铁律:**机器可验证,或人眼一步可验**。「滚动流畅」不合格;「列表支持搜索+筛选,跨用户 404」合格。
- 外部依赖同步开清单(external-setup.md):什么时点必须就位、谁的动作。人工项(DNS/审批)有等待期,**第一天就催办**。

## Phase 2 — 多声音评审

- 结构:CEO(战略)/ Design / Eng / DX 四声音分别过一遍计划,可加第二模型对抗。
- 产出分四层:**自动生效决策 / 用户闸门(必须人拍)/ TODO 台账 / 拒绝项**。实证:53 决策 = 38 自动 + 4 闸门 + 11 TODO。
- 评审的真正价值是**撞雷**:三声音独立发现了 Vercel 4.5MB 上传硬墙、extracting 卡死、配额并发绕过——全在写代码之前。

## Phase 3 — 架构与切片

- working-plan 核心是**切片表**:每片 = 一个用户可见的闭环(不是技术分层!)+ 出口标准。本项目:S1 认证 → S2 灵魂闭环(最高风险最先)→ S3 客户 → S4 今日+任务 → S5 账户 → S5b 计费 → S6 迁移上架。
- **灵魂切片优先**:产品的命根子(本项目=截图→AI提取→确认落库)排在功能广度前面,先证明产品成立。
- 契约先行的物理形态:`contracts/fixtures/*.json` 目录,**只许主会话改**。
- 状态单点:`docs/state/progress.yaml`,**只有 orchestrator 写**。

## Phase 4 — 细颗粒执行(phase 卡)

一张卡的标准结构(frontmatter + 两节):

```yaml
---
phase: "S4.1"
self_contained: true          # 卡内信息足够独立执行,不依赖会话记忆
required_context: [...]       # 需要读什么(指针,file:line)
max_feedback_loop_min: 30     # 反馈回路上限:30分钟内必须能跑一次验证
setpoint: "一句话定义『做到什么算到位』"
disturbance: "禁区:不许碰什么/不许发明什么"   # 与 setpoint 同权重!
---
## 1. 验收标准   ← checkbox,每条≈一个测试名
## 2. 内联要点   ← 参考模式指针(『照 xxx 文件的写法』)+ 环境坑
```

- **颗粒度标尺**:一卡 ≤ 半天;验收条目能直接翻译成测试名;做不到就拆卡。
- **纪律四件套**(贯穿):TDD(没有失败测试不写生产代码)/ Error Signal 四要素(What/Where/How/Severity,不许静默绕过)/ 验证后才说完成(贴命令实际输出)/ 单会话单写者。
- 参考模式指针是压幻觉利器:「照 customers-repo.ts 的 cursor 写法」比三段文字描述更不会跑偏。

## Phase 5 — 双轨并行(orchestrated-parallel v1)

六步协议(AGENTS.md §10),已 5 次 run 零目录冲突:

1. **契约先冻结**:fixtures 写好并 commit,才允许开轨。**冻结时把语义写死**(排序/上限/空值),不留两轨各自解读的缝(Run #3 教训)。
2. **轨道 = 目录 + git worktree**:后端轨只碰 api/(+migrations),前端轨只碰 app/,双向禁入。
3. **互相 Fake**:每轨用 fixture 种子造对方的假实现,自身闭环可测。
4. **契约测试当裁判**:双端各自直接 import 同一份 fixture 文件断言。
5. **单一状态写者**:轨道不写 progress.yaml、不 push;只交完成报告(=数据,非散文)。
6. **合流顺序**:先契约后实现;悬点集中裁决;翻转 provider(Fake→真);全量回归;上机。

**合流 checklist**(每次照抄):
- [ ] merge 后跑双端全量测试(不是只跑新增)
- [ ] 悬点逐条裁决并记录(谁改:契约跟正 or 实现跟正)
- [ ] 翻转后再跑一遍——**检查所有「进壳」测试钉了全量 Fake**(Run #3 教训)
- [ ] run-log 记一条(时长/冲突/悬点/返工),数据不记就丢
- [ ] 清理已合流 worktree

**报告格式即契约**:给轨道 agent 的 prompt 末尾固定要求「逐条验收 ✅/❌ + 测试输出末行 + 文件清单 + 契约悬点 + commit hash」——悬点栏让 AI 把自己的假设显式交出来,而不是埋在代码里。

## Phase 6 — 审查与 QA

- **切片收口 = 独立审查**:另起一个带对抗心态的 agent 复核验收 checkbox,流程 failed → respond → passed 全记录。实证:S2 审查抓到 T17 重写时误删的并发双确认测试(Critical)。
- **QA 三桶纪律**(跑流程阶段只记账不停下):A 未建功能(切片卡管)/ B 疑似缺陷(统一修)/ C 视觉体验(统一 design-review)。防止「见虫就修」把主线打碎。
- **挂账文化**:外部依赖(Stripe key/DNS)没到位,对应验收标记 ⏸ + 写清事后验证步骤。**严禁伪造证据**。

## Phase 7 — 收口沉淀

每个切片合流后 5 分钟内完成:progress.yaml 状态 + run-log 数据行 + loop-notes 教训(如有)+ commit&push;阶段性再做 snapshot(冷启动可恢复)。**沉淀是热的时候做的,冷了就没了。**

---

## 下个项目的启动清单(照抄即用)

1. 建仓:AGENTS.md(宪法)+ CLAUDE.md(协作层)+ contracts/fixtures/ + docs/{state,phases,snapshots}/ + agent-on/(或指向它)
2. Phase 0-1:原始想法落盘 → D 表 → 三分法 → PRD(F 编号+可验证验收)→ external-setup 催办清单
3. Phase 2-3:多声音评审 → 切片表(灵魂切片最先)→ 硬约束进 AGENTS.md
4. 每片循环:phase 卡 → 冻结 fixtures → 双轨并行 → 合流 checklist → 独立审查 → 三桶记账 → 沉淀
5. 全程四纪律:TDD / Error Signal / 验证后才说完成 / 单一状态写者
