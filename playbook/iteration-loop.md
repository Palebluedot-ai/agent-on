# 迭代闭环:项目怎么反哺 agent-on

> 职责边界:本篇管「agent-on 如何通过一个个真实项目自我迭代」——六站闭环、谁在哪个仓写什么、版本怎么走;不管单个项目内部怎么开发(那是 BOOTSTRAP 和 sop 的事)。
> 源流:2026-07-07 用户拍板需求;三镜头提案(最小闭环 / 证据链 / 多项目产品化)+ 对抗评审合成,2026-07-08 落盘。本篇的产生过程本身走了一遍 [abdc-decision.md](abdc-decision.md)(自举)。

## 〇、一句话

**每个项目都是一次 dogfood run:播种 → 采集 → 结账回流 → 消化成具体文件修改 → tag 发布 → 下游升级。** Project A 的坑,变成 Project B 的免疫。

## 一、六站闭环(谁执行 · 读写什么)

| 站 | 谁执行 | 动作 | 读 / 写 |
|---|---|---|---|
| ①种 | 新项目首会话(BOOTSTRAP) | 搭骨架 + 实例化 `agent-on.lock.md`(pin) | 读 kit 模板;写项目文件 |
| ②采 | 项目 orchestrator | 六类触发当场留痕 | 写项目 loop-notes.md / run jsonl 的 memory_card |
| ③结 | 项目 orchestrator(口令「agent-on 结账」) | 打包 Promotion Card,跨仓写承接层 | 写 agent-on `intake/<日期>-<项目>.md`;**绝不碰 canonical** |
| ④消化 | **agent-on 仓自己的会话**(单写者) | 三态分诊 → 落成具体文件修改 | 写 playbook/kit/bench 正文;intake 卡原地标去向 |
| ⑤发布 | 同上,消化收尾 | CHANGELOG 条目 + annotated git tag | 写 CHANGELOG.md,打 tag |
| ⑥升级 | 项目 orchestrator(独立口令「agent-on 升级」) | 读 CHANGELOG 区间,bump pin | 写项目 lock;仅 major 才动实例化文件(用户批准) |

## 二、项目侧:回流账本只新增一个文件

(采集用的 loop-notes.md 是站点不是账本,S 档的 AGENTS-lite 是规则不是账本——闭环在项目侧唯一新增的**账本**是 lock。)

`agent-on.lock.md`([../kit/agent-on-lock-template.md](../kit/agent-on-lock-template.md) 实例化;项目 AGENTS.md 首节放一行指针),三段:

1. **pin 行**:`agent-on @ vX.Y.Z (<commit>)`——tag 人读,commit 不可变
2. **last_settlement 回执**:上次结账日期 + intake 文件名——增量结账的锚点(与 memory_card 的 `sync_status` 互为校验,防重复回流)
3. **local_deviations 登记簿**:项目哪里偏离了 pin 版规则、为什么——「脚手架不合身」是 agent-on 自我迭代最值钱的信号,记在这,随下次结账自动回流

从 kit 实例化的每个文件头部自带一行 `<!-- instantiated-from kit/<文件> @ vX.Y.Z -->`,升级 diff 的基准随文件走,不设集中映射表(集中表没人更新,必然失真)。**实例化文件是项目自己的 canonical:上游新版对它只是承接层材料,永不自动覆盖**——truth-hierarchy「历史材料不得自动覆盖当前」的镜像应用。

## 三、采集:六类触发,当场一行

**返工**(完成声明被推翻)/ **撞车**(worktree、状态冲突)/ **用户纠正** / **Error Signal 中高严重度** / **手工重复第 2 次** / **脚手架不合身**(方法论自身的锅 → 记 local_deviations)。

格式钉死单行五字段:`日期|触发类型|一句现象|证据指针(commit/输出/run_id)|候选层(L1-L4)`。

纪律挂门禁不挂自觉:合流 checklist 有「触发扫尾」行,漏记=合流不过;**无证据指针的条目,结账时不予出仓**(防日记);L1 一行零门槛(防空账本——产量为零比日记更危险)。

## 四、结账与消化(执行书见 [../boot/settlement.md](../boot/settlement.md))

**负担预算(硬验收,超了就砍门禁)**:结账 = 一句口令 + 零确认;消化 = 一场会话 + 一组选择题;升级 = 一次 diff 批准。

- **结账**(项目会话):对表播报(落后几版、是否含 major,**不强制升级**)→ 收集 `sync_status=pending` 的出仓卡 → 装配 Promotion Card(六项缺一拒收)→ 写 intake 一个新文件、单独 commit → 卡标 synced + lock 记回执 → 播报 intake 积压(≥3 份未消化或跨项目同 pattern → 建议当场开消化,这是全流程唯一的问句)
- **消化**(**必须换 agent-on 仓会话**——会话上下文=装载的规则集,没读本仓 AGENTS.md 的会话不许动本仓 canonical):开场 pattern 频次扫描(同 slug ≥2 项目 → 置顶 + 强制升 L3)→ correction-loop 三态分诊(低风险 + bench 案例追加 = AI 直落;中高风险 = 打包成**一组选择题**一次拍完)→ 每张卡原地标 `landed@commit` / `rejected(原因)` / `deferred` → **至少一处具体文件改动**(meta-principles 第三条,只读不改 = 消化失败)→ CHANGELOG 条目(L3 改动强制成对列 playbook+kit 双落点,堵上游侧双写漂移)→ 用户确认定级,该打 tag 就打

**明确不设**:fail-closed 背压禁令(会锁死口令入口——最弱环节不能当全局断路器)、archive 目录(intake 原地标注即收口,目录即仪表盘,`ls` 一眼见积压)。

## 五、版本:tag 即真相,semver 按「下游动不动手」划线

不设 VERSION 文件(与 tag 双头)。CHANGELOG.md 是人读账本。

- **major = 不动手会坏**(schema 必填字段 / 骨架结构 / 口令与仲裁语义变更)——**不附迁移注记不许打 tag**
- **minor = 不动手不坏**(新增篇目 / 模板 / 可选字段)——存量项目可不动,新工件自然用新版
- **patch = 不用知道**(措辞 / 错链 / bench 案例追加)

定级由用户在消化选择题里确认。升级永远是独立口令、显式动作;major 迁移按中风险走,diff 呈报逐条批准。

## 六、多项目并发(A/B/C/D 同时活跃)

- intake 文件名 = 命名空间(`<日期>-<项目>.md`),append-only——四个项目同天结账写四个不同文件,git 层物理不撞,不造锁
- canonical 只有消化会话一个写者,消化天然串行
- 跨项目信号:Promotion Card 必带 pattern slug,消化开场 grep intake/ 同 slug ≥2 项目 = 高优先级,强制升 L3(playbook 规则 + kit checklist 行)——「高频失败优先沉淀为 anti-pattern」的执行点

## 六½、多人 / 上游贡献(与装机路径正交)

装机:plugin(A)+可选任意路径工作仓(B)+`~/.config/agent-on`——不写死 `Projects`。  
回馈:**不是**人人提 PR 改官方正文。

| 层 | 含义 |
|---|---|
| L0 | 项目私货——不出仓 |
| L1 | intake 卡片——结账产物 |
| L2 | 可选:intake-only PR 或 Issue 把 L1 运到官方 |
| L3 | 仅维护者消化写入 playbook/kit + tag |

默认用脚手架即可;贡献自愿。PR 运卡片、消化改宪法。执行书见 [../boot/settlement.md](../boot/settlement.md)「上游贡献形态」。

## 七、已知裂缝与 deferred(诚实清单)

1. pattern slug 是手写弱匹配,措辞不同会漏报——接受粗糙度,消化会话通读兜底;卡片攒到几十张再谈词表
2. Promotion 六项齐目前靠会话自觉——intake-lint(audit-lint 思路扩展)deferred,首次真实结账验收协议后再定
3. 多协作者:同口令、写自己命名空间的 intake 文件、拍板权单归维护者消化——上游运输见 §六½ 与 settlement「上游贡献形态」(intake-only PR/Issue);更重的 multi-contributor 编排协议仍 deferred
4. L3 双写(playbook+kit)对账只有 CHANGELOG 成对格式一道软门——出现真实漂移案例再加机制

## 八、v0.3 验收测试(第一个真实闭环)

**拿 Euan-Flutter 现有仓内 `agent-on/` 文件夹做首次真实结账**:素材倒仓进 intake、消化落 canonical、Euan 降级为纯采集三件套(lock + loop-notes + ledger),同场清除 playbook 正文副本的双头(它违反「只映射不复制」,是现存头号漂移源)。协议跑不通当场改——**不许先写完美再启用**(二代「14 篇文档零实现」的疫苗)。
