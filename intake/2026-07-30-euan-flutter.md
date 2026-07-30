# intake · 2026-07-30 · Euan-Flutter(harness v3 重建会话,5 卡)

> 结账会话:攻坚阶段入场审计(9 代理 402 取证五维度 + 对抗复核)→ 宪法 v3 + 真相面重同步 + 机械闸上线(PR #22,merged)。全部为 AI 协作过程教训;项目域知识(CRM 语义/上架门槛等)留项目端未出仓。

### paper-mechanism-rots-silently(纸面机制无闸必空转,第二个高速贡献者到来时同时失守)
- source:Euan-Flutter @ 44b944a | pin v0.3.0
- evidence:docs/reviews/2026-07-30-harness-audit.md——issue 层建制 24 天使用数 0(`gh issue list` 返回 `[]`)、`docs/state/logs/` 建目录 24 天零文件、merge --no-ff 被 7/7 squash、PR #10-#20 GitHub review 记录全空(`gh pr view N --json reviews` 逐个 `[]`);三个互不相干的自觉机制在同一周(07-23~29,双人期开始)同时失守
- confidence:high(三机制同周失守、同根因,且对抗复核逐条 CONFIRMED)
- claim:写协作/状态规则时必须同批交付机械闸(CI 断言/脚本校验);交付不了就明写「靠自觉+接受定期审计」;被审计证实空转两周以上的纸面机制,机械化或删除,不许留着装样子(Euan 宪法 v3 §M 元规则)。
- suggested_landing:playbook 纪律/协作篇立「机制须带闸」原则;kit AGENTS-skeleton 加一行「本条靠什么闸?」自问
- rollback:revert 落地 commit
- trace:本会话 harness 审计 workflow wf_2a0aa710-b24(9 代理,402 tool_uses)
- 状态:pending

### truth-surface-feeding-table(状态面登记「谁喂/何时喂/什么闸」三元组,没人喂的面比没有面更危险)
- source:Euan-Flutter @ 44b944a | pin v0.3.0
- evidence:审计 CONFIRMED——唯一人读面 dashboard.html 停 07-27:生产域仍写 api.euan.pro(实际已切 api.dartify.dev)、「等你区」催办用户已办完的 CF 人工项、语音进度写 0%(PR #18 已上线);任何人照它决策都会做错。修法=AGENTS v3 §4 喂养表(四张面 × 唯一写者/喂养时机/机械闸)@ de218b1
- confidence:high(同型失效在 AGENT-HANDOFF「分工快照」上再现:07-26 建,07-30 已过期)
- claim:每张状态面(人读或机器读)登记三元组:唯一写者/喂养时机/机械闸,缺一不开新面;审计发现无人喂养的面,按「错误信息源」处理——当场重绘或声明存档,不许放着继续误导。
- suggested_landing:playbook truth-hierarchy 补「喂养表」节;kit dashboard/状态页模板头部加三元组注释位
- rollback:revert 落地 commit
- trace:审计维度 truth-page + harness-compliance,verdicts CONFIRMED
- 状态:pending

### two-collision-diseases-two-cures(互踩≠撞题:共用目录用 worktree 隔离治,同活双做用开工声明治)
- source:Euan-Flutter @ 44b944a | pin v0.3.0
- evidence:同仓同日两种事故实证——①互踩:PR #23 记录 07-29 两次「test/analyze 把错报在自己没碰过的文件上」(根因=5 条分支挤一个工作目录);②撞题:87205ce 与 PR #21 同 base、同 3 文件、各自独立做「第九轮底栏」,机器实测冲突(审计对抗复核 `git merge-base` 双证)。政策合成落地 AGENTS v3 §11.2/§10.7(rebase 冲突解决 commit de218b1)
- confidence:high(两型事故各有独立机器证据)
- claim:并行事故先分型再立规:环境互踩→强制每会话独立 git worktree(隔离归机制);工作重复→开工前在唯一状态面声明主题(声明是防撞题的广播,不是独占锁)。禁会话数/禁同目录是误诊,会把并行收益一起砍掉。
- suggested_landing:playbook 编排并行篇「两种撞车」节;bench 案例(带两组机器证据)
- rollback:revert 落地 commit
- trace:PR #23 commit 81e68eb 正文 + 审计 collab-worktree 维度 CONFIRMED
- 状态:pending

### code-first-needs-retroactive-ledger(代码先行禁不住,给它 48h 事后追认通道,重点查外围义务)
- source:Euan-Flutter @ 44b944a | pin v0.3.0
- evidence:requirements.md:76 明文「语音 v1 不做」,PR #18 于 07-28 直接上线语音全链路(无 D 编号/无卡);审计顺藤摸出两笔更大的债——隐私政策对「音频交第三方 STT」零披露、上线前置三件未办导致生产必挂。追认落账 D23 @ e093dba,连带登记 T49/T54
- confidence:medium(单项目单例,但「先行需求漏外围义务」的形态可泛化)
- claim:需求变更协议加「事后追认」条款:代码先行发生时不装看不见,merge 后 48h 内补台账,且追认检查单必含隐私/法务/配额等外围义务(先行需求最容易漏的恰是这些);同一轨道两次先行→收紧该轨 PR 审查。追认是止损通道,不是特权。
- suggested_landing:playbook 需求变更/纪律篇;kit 需求协议模板加追认条款行
- rollback:revert 落地 commit
- trace:审计 harness-compliance 维度 §9 指控(CONFIRMED,复核员补抓隐私缺口)
- 状态:pending

### audit-with-adversarial-verifiers(合规审计用「取证代理+对抗复核代理」双层,复核以推翻为目标)
- source:Euan-Flutter @ 44b944a | pin v0.3.0
- evidence:本会话 workflow:5 维度取证代理产出指控,每条 Critical/High 再派独立复核代理「自己跑命令、试图推翻」——成功 REFUTED 两处归因(「PR #19 单方面改验收锚」实为用户参与拍板;「CODEOWNERS 路径失效导致 3fd13fc」实为直推使 CODEOWNERS 本就无效),两条错误指控被拦在检讨报告之外;其余 CONFIRMED 均附独立复跑输出
- confidence:high(两处 REFUTED 就是这套结构的直接产出)
- claim:对「规则 vs 执行」类审计,指控与定罪分离:每条重指控派独立代理重新取证并以推翻为目标;事实与归因分开判(事实成立、归因可 REFUTED)。单层审计的指控直接进报告=把叙事当证据。
- suggested_landing:kit review/audit prompt 模板(加「复核员以推翻为目标+事实归因分判」两行);bench 案例
- rollback:revert 落地 commit
- trace:workflow wf_2a0aa710-b24 verdicts 字段(CONFIRMED/PARTIAL/REFUTED 三态全出现)
- 状态:pending
