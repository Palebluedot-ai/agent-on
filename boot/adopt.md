# 存量项目接入书(项目已经开工了怎么用 agent-on)

> 职责边界:本篇管三件事——①一个**已存在/已开工**的项目怎么接入 agent-on(BOOTSTRAP 只管从零开始,本篇补这道缝);②项目长大后怎么**升档**(S→M→L);③误播高档或项目变瘦后怎么**降档**(L→M→S,与升档对等协议)。触发语:用户说「读 adopt.md 接管本项目」/「按 agent-on 接管」/「按 agent-on 降档」。
> 铁律:**不推倒、不重来、不回填历史**。项目现有的文件是它自己的 canonical,agent-on 的材料对它只是承接层。

## 一、接入三步

### 第 1 步:考古(读,不改)

- 读项目现有的 README / 规则文件(AGENTS.md、CLAUDE.md 或等价物)/ 最近 10 条 commit / 目录结构
- 读 `git worktree list`；若不止主 worktree，运行 `agent-on worktree status` 摊开每棵树的 branch / dirty / behind / unique / reclaim。**先审计、不先删**：未登记且 `unique>0` 或 dirty 的一律按孤本 `rescue`，进入下面的并行接管，不得因“很旧”直接回收
- 产出两样交给用户:**一句话现状**(这个项目是什么、走到哪了)+ **一张能力真相表**(bench/truth-table-template.md:已自动化 / 手动 / 未实现 / 依赖环境,每项带证据)——接手他人或过去的自己的代码,先摊开「到底有没有」,别信记忆

### 第 2 步:定档(和 BOOTSTRAP 同一把尺)

三问:①有真实用户或真实数据吗?②几天内搞完还是持续迭代几周以上?③碰钱/安全/对外服务吗?
判定:③任一为是 → **L 全装**;①或②为是 → **M 标准**;都不是 → **S 轻装**。拿不准取低档。

### 第 3 步:增量补件(按档,只补缺的)

**所有档共同的三件(闭环必需件):**
- 实例化 `kit/agent-on-lock-template.md` → 项目根 `agent-on.lock.md`(pin 当前版本;last_settlement 留空——**第一次结账从接入日起算,不回填历史**)。保费档位可暂标「待测·暂按某档」:项目已有大规模绿测试与既有纪律实证时,capability-probe 允许延后至首次结账前补跑,不阻塞接管(IPONews 实证,单项目条款)
- 项目里没有 `loop-notes.md` 就建一个空的
- 实例化 `kit/thoughts-and-ideas-template.md` → 项目根 `thoughts-and-ideas.md`(想法收集箱,全档;口令「整理想法」)

**S 轻装**:再无其他。若项目**已有**自己的规则文件,不要替换——在其顶部加 `## agent-on 映射` 一节(两行:lock 指针 + 两个口令)即完成;没有规则文件才拷 `kit/AGENTS-lite.md` 填空。

**M 标准**:上面之外补——
- 规则文件:已有则**合并**(把 AGENTS-skeleton 里项目缺的节逐节问用户后补入,已有条款一律保留原文);没有才拷 skeleton 填空。**低摩擦变体**(项目真相源已成体系或用户明示时可选):缺节内容从项目真相源派生 + 真正需拍板项列入接入报告——问询预算只花在无据可依的节上(IPONews 实证,单项目条款)
- `docs/state/progress.yaml`(kit 模板):只登记**当前未完成的工作**,历史不补。项目已有**等价真相源**(如自有 STATUS 文件)时不强并不替换——按维度分工(谁管模块/TODO、谁管阶段/瓶颈)并把「真相源链」写进映射节,dashboard 数据源清单登记多处(IPONews 实证,单项目条款)
- `docs/phases/_TEMPLATE.md`:从**下一个**任务开始卡片化,进行中的活干完为止不强行改造
- 实例化 `kit/dashboard-template.html` → 项目根 `dashboard.html`,从现状初绘一次(M/L 默认件;口令「更新仪表盘」+ 合流必更,数据只从真相源读禁手填)

**L 全装**:M 之外补——`contracts/fixtures/`(下次接口两侧并行前冻结)+ run 台账(`ledger/run-card-logging.md` 规范,从下次编排 run 起记)。

**已有多 worktree 的增量接管（不分档，出现即处理）**：按 `kit/worktree-control-plane.md` 做恢复六步。仍要继续的写轨逐棵补 claim（单一 goal + 互斥 owns + depends_on + base）；边界撞车时只留一条 active，其余 blocked/parked；只读会话不 claim。先 push/commit 消除孤本，再按依赖合流；只有 `reclaim=safe` 才建议人工拆树。接管只登记**当前活跃工作**，不为已结束历史伪造 lane。

### 收尾验收(对用户交付)

- [ ] 一句话现状 + 能力真相表讲给用户听过
- [ ] 档位报给用户并说明为什么
- [ ] lock + loop-notes + thoughts-and-ideas 就位(M/L 另有 dashboard 初绘);规则文件按「已有则合并,没有才新建」处理完
- [ ] 多 worktree 若存在：全场 status 已展示；每条仍写代码的执行轨已 claim 或明确 blocked/parked；所有 rescue 都有“先保存再回收”的下一步
- [ ] 用户知道四个口令:「agent-on 结账」「agent-on 升级」「整理想法」「更新仪表盘」(S 档无仪表盘)

## 二、升档(项目长大了)

**信号 → 动作,不许静默降档:**

| 升档信号 | 动作 |
|---|---|
| S 档项目开始碰真实数据/真实用户 | S→M:按上面 M 档清单补件;AGENTS-lite 换 skeleton 时**保留已填的暂停项禁令原文** |
| 要碰钱/安全/对外服务,或首次多 agent 并行 | M→L:补 contracts/fixtures(并行前冻契约)+ run 台账;TDD 从「建议」变「铁律」 |

升档记一行进 `agent-on.lock.md` 的 local_deviations(日期 + S→M/M→L + 触发信号)——档位变化本身就是回流信号,下次结账带给 agent-on(哪类项目常升档,说明定档三问该修)。

## 三、降档(误播高档 / 项目变瘦了)

> research / MRD 缺口:「不许静默降档」已写,但**用户批准后的可执行降档步骤**曾未与升档对等细化。本节补齐。

### 3.1 硬规矩(先读再动)

1. **禁止静默降档**:AI **不得**因「感觉太重」或省 token 自行拆掉 M/L 件。降档必须有用户**显式批准**(口令示例:「按 agent-on 降到 S」「这个项目降档到 M」)+ 目标档位点名。
2. **只删不用的件、不重播**:降档 = 从现有骨架**减法**;禁止 `git` 清仓重跑 BOOTSTRAP、禁止从 kit **整包重拷覆盖**已填 AGENTS/lock/loop-notes。
3. **闭环三件永不删**:`agent-on.lock.md`、`loop-notes.md`、`thoughts-and-ideas.md`(及 AGENTS 映射/lite 底线)——S 也要结账回流。
4. **三不变量不降档可删**:完成=贴证据、暂停项禁令、外向先确认(及 M/L 上的单一状态写者/契约纪律)——降档减的是**装备重量**,不是诚实。
5. **登记 lock**:每次降档在 `local_deviations` 追加一行:`日期 | L→M 或 M→S | 用户批准摘要 | 删了哪些路径 | 状态=done`。

### 3.2 信号 → 动作(须用户批准后执行)

| 降档 | 典型信号(须用户确认属实) | 动作(只删/停用,不重播) |
|---|---|---|
| **L→M** | 不再多 agent 并行、不再碰钱/对外服务且用户确认 | ① 停用 run 台账义务(可保留历史 `ledger/` 或 `docs` 下已有 run 记录,不再新建 jsonl);② `contracts/fixtures/` **可不删**(留作档案),但新接口并行前不再强制走 L 全套——用户若要清目录须再批一次;③ AGENTS 档位标记改 M;④ TDD 保持铁律(M 仍 Ship) |
| **M→S** | 无真实用户/数据、短生命周期玩具/脚本、用户明确「太重」 | ① **停用** phase 卡流水线:可不删 `docs/phases/` 历史卡(档案),新活不再强制开卡;② **停用** `docs/state/progress.yaml` 单写者义务——文件可留档或用户批后删除;③ **可删** `dashboard.html`(或移到 `docs/archive/`);④ 规则文件:**优先**把 skeleton 收成 AGENTS-lite 三条底线+映射(保留暂停项原文),**禁止**无备份整文件覆盖;⑤ 契约目录同 L→M 默认留档 |

**L→S 一步到位**:拆成 L→M 再 M→S 两步登记(两次 local_deviations),避免一次砍太多不可审计。

### 3.3 执行清单(AI 照做,勾完再报完成)

- [ ] 用户已点名目标档位且口头/书面批准(证据进对话或 lock 行)
- [ ] 列出拟删/拟停用路径清单,用户点头(或用户说「按 adopt §三默认表做」)
- [ ] 执行减法;git 上可回滚(至少一次 commit 专记降档,message 含档位)
- [ ] 更新 AGENTS 内档位标记(若有)+ `agent-on.lock.md` local_deviations 一行
- [ ] 复述:闭环三件仍在;完成=证据仍生效;未重播 BOOTSTRAP

### 3.4 与升档对称表

| | 升档 §二 | 降档 §三 |
|---|---|---|
| 触发 | 信号表 + 可自动建议 | 信号可建议,**必须用户显式批准** |
| 动作 | 按清单**补件** | 按清单**只删不用的件** |
| 重播 | 不重播 | **不重播** |
| lock | local_deviations 记升 | local_deviations 记降 |
| 静默 | 不许静默降 | **不许静默降**(再次强调) |
