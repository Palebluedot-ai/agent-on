# 多人协作协议:把「不变量」而非「流程」搬过去

> 2026-07-06,真人 co-contributor 到来。这是 agent-on 的一次关键内化:**Loop Engineering 从「一个人指挥一群 agent」扩展到「多个人 + agent」时,到底该改什么、绝不能改什么。**
> 一句话结论:**流程随人数变,不变量一个都不改**——因为不变量本来就是为「多个执行者并行不撞车」设计的,人只是又一种执行者。

## 一、核心洞察:我们早就在做多人协作了

回头看会发现一个反直觉的事实:**这套方法论从第一天起处理的就是「多执行者协同」问题**——只不过执行者是 agent。7 次编排 run、9 次零目录冲突,每一次都是「多个独立执行者 + 一个协调者 + 共享真相源」的演练。

所以真人 co-contributor 不是新问题,是**同一个问题换了执行者类型**。agent 轨道和人类 contributor 的唯一实质差异:

| | agent 轨道 | 人类 contributor |
|---|---|---|
| 全局视野 | orchestrator 喂 | 自己不一定有 → **issue 补上**(自包含卡) |
| 隔离机制 | git worktree | git branch |
| 交付形态 | 报告文本 | PR 描述 |
| 协调者 | 主会话(orchestrator) | maintainer(人或其会话) |
| 其余 | —— 全部相同 —— | |

**结论:不发明新协议,把 §10 编排并行协议映射到人(AGENTS §11)。**

## 二、四条不变量怎么保全(这是全部要点)

多人协作最容易犯的错,是「为了方便多人」把不变量拆了。逐条钉死:

### 1. 单写者状态 —— 位置不动,写者换名
- **诱惑**:让每人都能改 progress.yaml,「各写各的部分」。
- **为什么是灾难**:progress.yaml 是防状态幻觉的根防线(anti-hallucination 机制 4)。多写者 = merge 冲突 + 状态漂移 = 后续所有决策建立在错前提。
- **正解(最少改动)**:功能 PR **永不碰** progress.yaml;写者从「单会话」变「单一合并权威(maintainer)」——**不变量的定义没变(仍是单写者),只是写者身份从会话变成角色**。contributor 用旁路(`logs/<handle>.md` 追加,不同文件零冲突)报进展,maintainer merge 后单独 commit 同步。
- **可复用模式**:任何「多人 + 单一真相源」都这么办——真相源只读于协作者,由合并权威在合并点单独更新。

### 2. 轨道 = 目录 —— 从 worktree 平移到 branch
- 9 次 run 证明「目录切分 = 零冲突」。人类版:issue 打 `track:` label = 认领目录;同目录同刻单写者。
- `api/` + `app/` + `website/` 可三人并行;两人都要动 `app/` → 先拆到不同子目录或排队。
- **契约是例外**:`contracts/` 谁都依赖,所以 maintainer 独占 + 先冻结(下条)。
- **何时开 worktree(对 agent 轨道同样成立)**:单轨/单人**不开**(YAGNI——Euan 曾误开并行会话改同一目录,git+文件布局撞乱,整套脚手架返工的实证);仅当 ≥2 条写轨真并行**且**文件域可切不重叠时才开。第二个写者出现前先让主树 clean；并行期主树退为只读控制/合流面，**每条写轨**各占一棵 worktree。**三个共享面必须归口单一轨**:`design/`、`contracts/fixtures/`、`docs/state/progress.yaml`——两轨同碰任何一个 = 撞车前置条件成立。跨轨回流走 append-only 新文件(与 intake 规则 1 同构:不动他人文件,git 层物理不撞)。
- **轨道合同把隔离变成可查询事实**:第二个写 worktree 出现时,每轨开工前用 `agent-on worktree claim` 登记单一 goal / 互斥 owns / depends_on / base;提交与合流前 `worktree check` 机械核实际 diff。运行态落 git common dir,各 worktree 共见但不进 commit,避免多分支抢写 progress。衍生功能另开 lane,禁止长寿 worktree 无限扩 scope;完整生命周期见 `kit/worktree-control-plane.md`。
- **新目标不继承旧 HEAD**:优先宿主原生 worktree 路径；手工路径按项目声明（未声明可 `.worktrees/<lane-id>`），分支 `<type>/<issue-or-lane>-<slug>`。每次先 fetch，从 fresh `origin/<default>` 创建；squash 会换 hash，从上一任务 HEAD 续长会把已 landed diff 再背进新 PR。
- **回收是实时证据，不是手填名单**:`agent-on worktree gc --dry-run` 每日及每次合流后重算三判据（已进入 base / 无未保存孤本 / clean），再叠加 unlocked、非活跃与静默窗口。squash 以 MERGED PR + head coverage 为权威；dirty/locked/unknown/无 PR 孤本只报 `review|rescue`。JSON `candidates` 是派生 known-reclaim list；CLI 没有 apply/delete 模式，删除目录/分支与跨树 commit 必须人工且目标明确授权。(Dartify babysit 2026-08-16 + Agent-On 四树动态实盘)
- **交付前对表(硬门)**:从 worktree 装机/演示/截图给人看之前,必须 `fetch` 并对表 default branch(`rev-list --count HEAD..origin/<default>` 非 0 → 先对齐再构建)。worktree ≠ main 的活别名;播报写 hash + 落差,禁空口「最新」。(Dartify 2026-08-08:落后 17 commit 装真机,用户感觉全是旧版)
- **三种并行事故,三种解药**(先分型再立规,禁会话数/禁同目录是误诊):
  1. **环境互踩**:多会话挤同一工作目录 → test/analyze 把错报在没碰过的文件上。解药=**每会话独立 git worktree**(隔离归机制)。(Euan PR #23:5 分支挤一目录)
  2. **工作撞题**:同 base 上各自独立做同一主题(同文件/同功能)。解药=**开工前在唯一状态面声明主题**(广播防撞,不是独占锁)。(Euan 87205ce 与 PR #21 同 3 文件双做第九轮底栏)
  3. **共享编号撞号**:决策号 D-N / 切片号 / 迁移序号等顺序命名空间被两条会话同时取号。worktree 与开工声明都治不了——前者只隔离文件,后者不广播「占哪个号」。解药=**取号即刻落盘**(最小占位行先 commit/push 到共享真相面再写内容),或取号前强制读最新真相面末行。撞号裁决=先合先得 + 后者改号,并扫全仓自指引用。(Euan 2026-07-30:双会话同定 D24 → rebase 改判 D25)
  4. **主题撞题(分钟级错开全盲)**:开工那一刻 `gh pr list` 查过重,提交前 main/别人又开了同题 PR——**只查一次=查重失效**。解药=**开工前 + 提交前各查一次**,按**主题**查(不靠文件路径);发现撞题 → **handoff 交还 ownership、停本轨改码**,改做可并行收尾(文档/结账),禁止双写同一 PR 枝。(Dartify 2026-08-02:#55 与 #59 各修一遍限流 IP,实现逐字相同,8 文件冲突)

### 3. 契约先冻结 —— 三方合同不能随手改
- fixture 改一下,所有依赖它的实现全受影响。所以:改 fixture 走 `needs-contract` issue → maintainer 批 + 冻结进 main(契约 PR 先合)→ 实现方 rebase 跟进。
- CODEOWNERS 把 `contracts/` 标 maintainer-owned = 机器化的「先冻结」提醒。

### 4. 验证后才完成 —— 从「贴输出」升级到「机器强制」
- 多人下「我觉得对了」的风险更高(没有 orchestrator 盯)。所以加一道机器门:**CI test gate 必过**。636 测试在这里从「质量资产」变成「多人安全网」——**任何人都无法静默弄坏 main**。
- 这是多人相比单人**唯一真正加强**的一环:CI 从「好有」变「必须」。

## 三、maintainer 角色的本质(容易误解,单列)

> **maintainer = 状态所有权 + 合并权 + 契约冻结权,不是「最强程序员」。**

- 产品负责人可能是编码新手,新 contributor 可能更资深——**maintainer 身份与编码水平无关**,它守的是「单写者不变量」这个结构性属性。
- 可委托(团队大了轮值),但**同刻只有一位在任**——两个 maintainer 同时 sync progress.yaml 又变成多写者了。
- maintainer 的 review 用对抗式(kit/review-prompt 人类版):跑命令核对而非读声明、删测审计、边界审计。执行者自评永远系统性偏乐观,这条对人对 agent 一样成立(S2 审查抓 Critical 的教训)。

## 三½、机制须带闸(纸面规则无闸 = 装样子)

> 源流:Euan harness 审计 2026-07-30——issue 建制 24 天 0 使用、logs/ 24 天零文件、merge --no-ff 被 7/7 squash、PR review 记录全空;三机制在双人期同周失守。digest paper-mechanism-rots-silently。

写协作/状态/流程规则时必须**同批交付机械闸**(CI 断言 / pre-receive / 脚本校验 / 必须绿的检查)。交付不了就**明写**「靠自觉 + 接受定期审计」——别假装有护栏。被审计证实**空转两周以上**的纸面机制:机械化或删除,不许留着装样子。第二个高速贡献者到来时,无闸机制会同时失守。

### 三½.1 merge 记账棘轮(Dartify 2026-08-06:78 PR 仅 44 记账)

「merge 后须更新 progress/dashboard」若只写进 checklist = 自觉。并行度一上来必断档。机械化**三层**:

1. **闸(CI)**:job 拉已 merge 的 PR 列表,比对状态文件是否出现对应 PR 号/commit 记账行;超宽限(如 30min)→ 红。只拦「都不动」;「补同笔」另有同会话纪律。
2. **触发器**:merge 与记账定义为**一个动作的两半**,同会话完成——别「先合、改天记」。
3. **执行体**:日历/巡检类见 §三½.2(launchd/cron),别把死线只写在 TODOS 里。

闸必须带**三层逃生门**:repo variable 关闸 / 宽限可调 / 一行合法记账即可解锁。**API 失败措辞与真违规必须区分**(假红的闸比没有闸更糟)。工具模式可复用:纯脚本 + CI,改仓名/状态路径/SINCE(上线日=历史不追)即可;模板见 kit/ledger-ratchet-pattern.md。

**元动作自涵盖**:记账 commit 本身必须进入被记账集合,否则盲区自我繁殖(Dartify 2026-08-15:`chore(state)` 把别人记进来,没人回头记它们,#81/#83/#86/#87/#88 五条自己成欠账)。设计时问:「执行这条规则所产生的产物,受不受这条规则约束?」答否 = 盲区。同族:审计不审计审计者、备份不备份备份配置。补账时**补内容不补号码**——闸认的是「PR 号出现在台账原文」,只塞号码能骗过闸,骗不过下一个读台账的人。

**字面匹配盲区(提及 ≠ 记账,但闸分不清)**:正文 grep PR 号的棘轮会把「提及」当「销账」——台账叙述里出现别人未清偿的编号,就替对方满足了机械闸、拆掉其补账压力。写台账区分「记我的账」与「叙述里提到别人」:指代别人未清偿的工作用无编号称呼,其号由义务人自记(Dartify 2026-08-16:值守写 #158 记账时对四笔并行欠账刻意不写其号,条目留痕「免得提前满足棘轮」,其会话随后自行补账,义务链未断)。

### 三½.2 日历死线须有执行体 + worktree 回收

「定期清理 / 某日裁决」写进 TODOS 却**没有任何进程会读日期** = 不会发生(Dartify T51/T58 双双过期零后果)。正解:配 **launchd/cron 每日报告**当执行体——每次重算候选，日志替人看死线；删除仍是独立人工动作，定时器不得顺接 `worktree remove`。

**worktree 回收判据(全中才进候选)**:PR 状态以 `gh` 为准(防 squash 换 hash 误判) + 无未推/独有提交 + clean + 静默窗口(如 24h) + unlocked / 非活跃。通用层不替项目认“假脏”；dirty 原样报告给人分类。**分支默认保留**。**无 PR 孤本 / detached 归属不明 / locked / unknown → 一律只报告不删**。macOS 细节:StartCalendarInterval(睡眠补跑)、PATH/HOME 显式、原子锁、日志进 `~/Library/Logs/`。模式见 kit/worktree-gc-pattern.md。

本机即时面先跑 `agent-on worktree status` 看合同/边界，再跑 `agent-on worktree gc --dry-run` 聚合远端 PR、三判据、locked、du 与 quiet。JSON `candidates` 是当次派生名单；CLI 没有 apply/delete 模式。

### 三½.3 守卫允许集 × PR 作者闸的协作者出口

「contributor 不得改守卫脚本」+「允许集变更须与代码同笔」会让**协作者 PR 死锁**(改代码要动允许集,动允许集又被作者闸拦)。预先写明出口:**守卫/允许集扩面由 maintainer 对同一分支重开 PR**(或 maintainer 提交允许集补丁后合并)——别让贡献者撞死锁后发明绕法。诊断先分清闸按**提交人**还是**PR 作者**(对策不同)。(Dartify #76→#80、#79)

### 三½.4 孤本抢救再回收

自动回收发现「无 PR 但含唯一副本」:①先 **push 远端消单点** ②再 worktree 回收 ③择期 rebase 落地。跨大跨度 rebase 时契约/棘轮锁测试红 = 设计在履职——**显式更新锁列表**,不是放宽锁。

### 三½.5 闸的三张面(触发 / 读取 / 并发)

> 源流:Dartify 2026-08-15——PR-only 闸红着合入、后续 PR 代付;闸读 `git log` 不读工作区导致提交前假绿;直推 main 把自己的 PR 撞成 DIRTY 且 CI 零 job。digest gate-three-faces。

闸不只「过不过」,还有三张会独立失效的面:

1. **触发面 = 它要保护的分支**。只在 `pull_request` 上跑的检查,红着合入后 default branch 永久静默,每一条后续 PR 都替它买单,且症状伪装成「我的改动弄坏了 CI」。红着合 = 把红转嫁给后来者,禁止。任何「本地绿 / CI 红」先查**被 .gitignore 的文件是否被当成必需输入**(开发机文件系统恒富于干净 checkout);凡把路径写进清单/存证,当场 `git ls-tree <default> --name-only` 验它在不在。
2. **读取面:工作区 vs 提交历史**。断言里出现 `git log` / `git blame` / `merge-base` / `git show <ref>:<path>` 任一,本地验证必须在 `git commit` 之后跑。提交前的绿只证明「工作区没坏」,不证明跨文件/跨提交关系成立。写这类闸时应在工作区对目标文件有未提交改动时打 warning。
3. **并发面:直推 default branch 会脏自己的 PR**。机制允许直落 main(记账 / hotfix / release chore)豁免的是评审,不是并发影响。推之前 `gh pr list --state open` 并对改动文件集;撞上后 `mergeStateStatus=DIRTY` 时,不要去查 CI 配置——GitHub 不为 DIRTY PR 起 checkout,仓内 job 一个都不会跑。与 worktree-gc「squash 换 hash → 新枝从旧 HEAD 长 → 同样 DIRTY + 零 job」是同一副面孔的另一种成因。

### 三½.6 值守合并调度(合并权中央化 + 批准来源 + 调度员边界)

> 源流:Dartify 2026-08-16 值守夜班 9 连合 + 2026-08-17 三单实战(#164/#165/#169)。digest merge-dispatcher-serializes-uptodate-gate / relayed-user-instruction-is-not-approval / dispatcher-returns-defects-to-author。

分支保护开 require-up-to-date 硬门后,「合并」变成全局串行资源:N 条会话各自追平自合 = 每次合并把其余人打回 BEHIND,全场 O(N²) 次 rebase;单一值守窗口串行调度 = O(N)。**合并权中央化不是偏好,是硬门下的最优解**——功能会话开完 PR、首轮 CI 触发、描述写全、交单 = 交付完成,不自己合;追平一律走托管平台服务端 update-branch API,不碰任何本地 worktree。接入件与值守文档模板见 kit/babysit/。三条配套纪律:

1. **批准的来源只有一个**:外向硬门动作(合并/删除/迁移)的用户批准必须来自**本会话内的用户输入**。同行会话转述的用户原话再可信也只是情报——注明转述来源、向本人确认后才执行。结果大概率一致,但这一步是权限模型的地基(#169 实测,作者会话回执「该省的从来不是这步」)。
2. **调度员对真缺陷只做取证 + 打回,不代修**:产出 = 证据指针(run id/日志行)+ 缺陷定位 + 闸给的修复选项 + 打回作者会话,到此为止。修复知识在作者上下文里,代修既慢又破坏 lane 边界与责任链(#169:打回后作者 15 分钟修绿,值守零代修)。
3. **记账随合并权走**:谁合谁记,含值守自身的元动作(§三½.1 元动作自涵盖)。

## 四、诚实的约束(不粉饰)

- **私有免费仓无分支保护**:GitHub 不给 API(实测 403)。当前靠 约定 + CODEOWNERS 软 review + CI 硬门 三层替代,够 2-3 人;≥3 人或出事 → 升 GitHub Team。**软护栏不是真护栏,写清楚比假装安全好。**
- **生成文件冲突**:freezed/g.dart 入仓 + CI 拦 diff。两人各改 model 会撞——解法是 rebase 后重新 `build_runner`,不手动 merge。这是入仓换来 clone 即编译的代价,可接受。

## 五、给下个项目 / Kit 的沉淀

- 多人协作不是「等团队大了再设计的东西」——**从第一天用编排并行协议,团队来了直接映射**。Kit 的 AGENTS 骨架已含 §10/§11 双版本。
- 迁移清单:CONTRIBUTING(人类流程)+ CODEOWNERS(独占区)+ ISSUE/PR 模板(自包含卡 + 报告)+ logs/ 旁路(单写者输入)+ CI test gate(机器强制验证)。全部是「把已有 agent 机制换个壳」,零新概念。
- 一句话带走:**好的单人 AI 协作架构,天然就是好的多人架构——因为两者的敌人都是「多执行者 + 共享状态」的漂移,而解药(单写者/目录轨道/契约冻结/机器验证)是同一副。**
