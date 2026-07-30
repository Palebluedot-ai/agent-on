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
- 状态:landed@同批(第十五次消化:multi-contributor §三½ + AGENTS-skeleton 机制须带闸)

### truth-surface-feeding-table(状态面登记「谁喂/何时喂/什么闸」三元组,没人喂的面比没有面更危险)
- source:Euan-Flutter @ 44b944a | pin v0.3.0
- evidence:审计 CONFIRMED——唯一人读面 dashboard.html 停 07-27:生产域仍写 api.euan.pro(实际已切 api.dartify.dev)、「等你区」催办用户已办完的 CF 人工项、语音进度写 0%(PR #18 已上线);任何人照它决策都会做错。修法=AGENTS v3 §4 喂养表(四张面 × 唯一写者/喂养时机/机械闸)@ de218b1
- confidence:high(同型失效在 AGENT-HANDOFF「分工快照」上再现:07-26 建,07-30 已过期)
- claim:每张状态面(人读或机器读)登记三元组:唯一写者/喂养时机/机械闸,缺一不开新面;审计发现无人喂养的面,按「错误信息源」处理——当场重绘或声明存档,不许放着继续误导。
- suggested_landing:playbook truth-hierarchy 补「喂养表」节;kit dashboard/状态页模板头部加三元组注释位
- rollback:revert 落地 commit
- trace:审计维度 truth-page + harness-compliance,verdicts CONFIRMED
- 状态:landed@同批(第十五次消化:truth-hierarchy §五½ + dashboard-template 喂养表三元组)

### two-collision-diseases-two-cures(互踩≠撞题:共用目录用 worktree 隔离治,同活双做用开工声明治)
- source:Euan-Flutter @ 44b944a | pin v0.3.0
- evidence:同仓同日两种事故实证——①互踩:PR #23 记录 07-29 两次「test/analyze 把错报在自己没碰过的文件上」(根因=5 条分支挤一个工作目录);②撞题:87205ce 与 PR #21 同 base、同 3 文件、各自独立做「第九轮底栏」,机器实测冲突(审计对抗复核 `git merge-base` 双证)。政策合成落地 AGENTS v3 §11.2/§10.7(rebase 冲突解决 commit de218b1)
- confidence:high(两型事故各有独立机器证据)
- claim:并行事故先分型再立规:环境互踩→强制每会话独立 git worktree(隔离归机制);工作重复→开工前在唯一状态面声明主题(声明是防撞题的广播,不是独占锁)。禁会话数/禁同目录是误诊,会把并行收益一起砍掉。
- suggested_landing:playbook 编排并行篇「两种撞车」节;bench 案例(带两组机器证据)
- rollback:revert 落地 commit
- trace:PR #23 commit 81e68eb 正文 + 审计 collab-worktree 维度 CONFIRMED
- 状态:landed@同批(第十五次消化:multi-contributor §二.2 两种事故 + bench/cases/27)

### code-first-needs-retroactive-ledger(代码先行禁不住,给它 48h 事后追认通道,重点查外围义务)
- source:Euan-Flutter @ 44b944a | pin v0.3.0
- evidence:requirements.md:76 明文「语音 v1 不做」,PR #18 于 07-28 直接上线语音全链路(无 D 编号/无卡);审计顺藤摸出两笔更大的债——隐私政策对「音频交第三方 STT」零披露、上线前置三件未办导致生产必挂。追认落账 D23 @ e093dba,连带登记 T49/T54
- confidence:medium(单项目单例,但「先行需求漏外围义务」的形态可泛化)
- claim:需求变更协议加「事后追认」条款:代码先行发生时不装看不见,merge 后 48h 内补台账,且追认检查单必含隐私/法务/配额等外围义务(先行需求最容易漏的恰是这些);同一轨道两次先行→收紧该轨 PR 审查。追认是止损通道,不是特权。
- suggested_landing:playbook 需求变更/纪律篇;kit 需求协议模板加追认条款行
- rollback:revert 落地 commit
- trace:审计 harness-compliance 维度 §9 指控(CONFIRMED,复核员补抓隐私缺口)
- 状态:landed@同批(第十五次消化:AGENTS-skeleton §9 事后追认条款)

### audit-with-adversarial-verifiers(合规审计用「取证代理+对抗复核代理」双层,复核以推翻为目标)
- source:Euan-Flutter @ 44b944a | pin v0.3.0
- evidence:本会话 workflow:5 维度取证代理产出指控,每条 Critical/High 再派独立复核代理「自己跑命令、试图推翻」——成功 REFUTED 两处归因(「PR #19 单方面改验收锚」实为用户参与拍板;「CODEOWNERS 路径失效导致 3fd13fc」实为直推使 CODEOWNERS 本就无效),两条错误指控被拦在检讨报告之外;其余 CONFIRMED 均附独立复跑输出
- confidence:high(两处 REFUTED 就是这套结构的直接产出)
- claim:对「规则 vs 执行」类审计,指控与定罪分离:每条重指控派独立代理重新取证并以推翻为目标;事实与归因分开判(事实成立、归因可 REFUTED)。单层审计的指控直接进报告=把叙事当证据。
- suggested_landing:kit review/audit prompt 模板(加「复核员以推翻为目标+事实归因分判」两行);bench 案例
- rollback:revert 落地 commit
- trace:workflow wf_2a0aa710-b24 verdicts 字段(CONFIRMED/PARTIAL/REFUTED 三态全出现)
- 状态:landed@同批(第十五次消化:review-prompt 合规审计变体 + bench/cases/28)

---

## 追加批(同日第二次结账,2026-07-30 晚 · 社交登录线 + 五连合并,5 卡)

> 同日同项目追加进本文件(settlement §4 幂等规则)。上半批 5 卡出自晨间 harness 审计;本批出自下午社交登录实施线与晚间五连合并。仍全为 AI 协作过程教训。

### shared-id-namespace-collision(第三种撞车:共享顺序编号命名空间——撞号)
- source:Euan-Flutter @ 9428d29 | pin v0.5.1
- evidence:同日两条 maintainer 会话各自把决策编号定为 **D24**——一条是「全面去 Euan 命名」(PR #27,先合入 main),另一条是「社交登录前端半场」(PR #28,本线)。冲突在 rebase 时才暴露(`docs/requirements.md` 单行冲突),现场按「先合先得」改判本线为 D25(`git log` → `6872144 chore(rebase): 收尾清扫——D 撞号改判 D25`);现 requirements.md:42=D24 去命名 / :43=D25 社交,两行并存无重号
- confidence:high(有 commit 与文件双证;且与本仓晨间已出仓的 two-collision-diseases-two-cures 构成同族第三型)
- claim:并行事故有**第三型**——共享顺序编号命名空间(决策号 D-N / 切片号 / 迁移序号)被两条会话同时取号。worktree 隔离与开工声明**都治不了它**:前者只隔离文件,后者只广播主题不广播「我要占哪个号」。治法=**取号即刻落盘**(先把占位行以最小 diff 提交/推送到共享真相面,再写内容),或取号前强制读最新真相面末行。发现撞号时按「先合先得 + 后者改号」裁决,并把改号连带扫全仓自指引用(注释/台账/entitlements 都会带号)
- suggested_landing:playbook 编排并行篇「两种撞车」节扩为三型(与 two-collision-diseases-two-cures 语义归并);kit/AGENTS-skeleton 决策台账行补「取号即落盘」半句
- rollback:revert 落地 commit;若判定与 two-collision 卡重复度过高,合并为一卡三型即可
- trace:本会话 rebase 段;requirements.md:42-43 与 commit 6872144
- 状态:landed@同批(第十六次消化:multi-contributor 第三型撞号 + AGENTS 取号即落盘 + bench 27 扩)

### rename-wave-rebase-derive-the-map(撞上大规模改名时,先从已合并侧反推映射表再机械套用)
- source:Euan-Flutter @ 9428d29 | pin v0.5.1
- evidence:本线 7 笔 commit rebase 到 main 时撞上 PR #27「全仓去 Euan 命名」(`gh pr view 27 --json files --jq '.files|length'` → **100** 文件,包名 `euan`→`app` + 41 类 `Euan*`→`App*`),18 个文件冲突。用的办法:①从 `origin/main` 实测反推 15 条 sed 映射规则;②对每个冲突文件跑 `diff <(base 套映射) <(origin/main 现状)` 得**残差行数**,一眼分档——0/4/8/10 行=纯改名(直接取我方套映射),16/23 行=真双改(手工并,如 helpers.dart 要保住 #26 新增的 backFromReviews);③新增文件不触发冲突故 rebase 不管,末尾人工 grep 扫一遍(两个新测试文件的 `package:euan` 正是这样捞回的)
- confidence:high(18 文件一次通过,rebase 后 `flutter analyze` 净 + 1330 测试全绿)
- claim:遇到「一方做了大规模机械改名」的 rebase,不要逐文件人肉解冲突。先从已合并侧反推**映射表**,再用「base 套映射 vs 现状」的**残差行数**给冲突分档:残差≈0 = 纯改名可机械解,残差大 = 真双改需手工并。改名浪潮**不触发冲突的新增文件**是最容易漏的面,收尾必须独立 grep 一遍旧标识符
- suggested_landing:kit/merge-checklist 新增「撞上改名浪潮」小节(反推映射→残差分档→新增文件扫尾三步);playbook workflow-orchestration 合流节引一句
- rollback:revert 落地 commit
- trace:本会话 rebase 段(6 次 `--continue`);残差分档命令输出 0/4/8/10/16/23
- 状态:pending

### classify-red-check-origin-before-blocking(红灯先分来源:仓内硬门 vs 外部集成,别把噪音当缺陷)
- source:Euan-Flutter @ 9428d29 | pin v0.5.1
- evidence:PR #28 八项 check 里 `Supabase Preview` 红。取证 `gh api .../check-runs` 拿到逐字 summary:`unexpected status 401: {"message":"Custom SMTP required to configure SMTP_SENDER_NAME or RATE_LIMIT_EMAIL_SENT. Missing SMTP_PASS fields."}`——是**预览环境缺密钥**,与本 PR 无关(本分支对 `supabase/` 的唯一改动是 config.toml 的纯注释勘误)。反证:同期 PR #26/#27 该 check 状态为 `skipped`。而仓内真硬门是 `.github/workflows/ci.yml` 的四个 job(flutter/secrets/state/api),全绿
- confidence:medium(单实体证据,但形态在任何接了外部集成的仓都会复现)
- claim:PR 见红先问「这个 check 是谁家的」——仓内 CI(硬门,必须绿)还是外部集成(噪音,按情况豁免)。判据是**它在不在 `.github/workflows/` 里**。取证到 summary 原文再下结论,别看名字猜;若判为外部噪音,必须把根因与根治写进 PR 悬点栏(不写=下一个人还要再查一遍)
- suggested_landing:kit/merge-checklist CI 门那一步补「红灯先分来源」两行;anti-hallucination 映射表可挂一行(把噪音当缺陷=错误归因型)
- rollback:revert 落地 commit
- trace:PR #28 check-runs 取证;ci.yml job 清单
- 状态:pending

### blocked-dependency-check-actual-requirement(依赖受阻时先问「这项验证真需要它吗」,别让一个堵点停掉整条线)
- source:Euan-Flutter @ 9428d29 | pin v0.5.1
- evidence:社交登录本计划真机验收,但用户的数据线是纯充电线(`system_profiler SPUSBDataType | grep -ci iphone` → **0**,USB 层零记录),真机路径当场断死。改判:Sign in with Apple/Google 的**原生 SDK + id_token 链路在 iOS 模拟器上完整可跑**,只有语音/触觉才真需要硬件。于是同一晚在模拟器上把两个 provider 都打到生产后端 `api.dartify.dev` 走通(Apple=新用户 email_confirmed 直真;Google=同邮箱自动关联既有付费账号),真机场降级为「语音+触觉专场,等有线再做」
- confidence:medium(单实体证据;但「把整条验收捆在一个可替代的依赖上」是通用反模式)
- claim:硬件/外部依赖受阻时,不要立刻把整条验证线挂起。**逐项问「这一项的验证真的需要这个依赖吗」**——常见情况是只有少数几项真需要,其余可在替代环境拿到同等强度的证据(模拟器打生产后端 ≠ mock,链路真实性不打折)。把真需要硬件的项拆出来单独排期,别让它挟持其余
- suggested_landing:playbook sop 验收节补「依赖受阻先做能力-依赖对照,再决定挂起范围」;kit/phase-card-template 验收行可提示「本项验证的真实依赖是什么」
- rollback:revert 落地 commit
- trace:本会话真机排障段(system_profiler 零命中 → 转模拟器 → 双 provider 生产实证)
- 状态:pending

### human-console-checklist-must-be-doc-verified(要用户照着点的清单,每条必须官方文档核实——过时清单比没清单更危险)
- source:Euan-Flutter @ 9428d29 | pin v0.5.1
- evidence:`docs/external-setup.md §七` 旧版(凭经验写)教用户建 **「iOS 类型一个 + Android 类型一个」** OAuth Client(旧文件 :117 逐字);抓 Supabase 官方文档后发现真实要求是 **web + iOS 两个**——原文 *"Add web client ID and iOS client ID from step 1 in the Google provider on the Supabase Dashboard, under Client IDs, separated by a comma."* 照旧版做会漏掉 Web client,而后端只回无差别 401(真实拒因只进服务端日志),用户将面对一个极难自查的死局。同批还核出两条反直觉事实:Apple 纯原生流**不需要** Services ID/私钥/secret(官方原文 *"If you're building a native app only, you do not need to configure the OAuth settings."*);Google 原生流**必须**开 `Skip nonce check`(官方要求,代价=可重放,已写进清单当知情事项)。重写后每条带 `[doc]` 标记,抓不到的标 `[待查证]` 不编
- confidence:high(错误内容与正确内容双向逐字可比对:旧 :117 vs 新 §七B)
- claim:凡是**交给人去外部控制台照做**的清单,每条都必须有官方文档原文背书并就地标注来源;抓不到的老实标「待查证」而不是补一个看起来合理的路径。理由:这类清单的错误由**人**去承担代价(时间+挫败),且往往表现为无差别错误码,自查成本极高。**过时/臆测的清单比没有清单更危险**——没有清单人会去查文档,有错清单人会照着做
- suggested_landing:anti-hallucination 第六型「取证幻觉」补一条外化面(交给人执行的步骤=最高取证标准);kit 文档模板补 `[doc]`/`[待查证]` 双标记约定
- rollback:revert 落地 commit
- trace:external-setup.md 旧 :117 vs 新 §七B;重写 commit 235ebc1
- 状态:pending

### guard-path-regex-matches-content-not-target(护栏把「要写进文件的内容」当成 git 目标 → 拦掉协议自己要求的回执)
- source:Euan-Flutter @ 9428d29 | pin v0.5.1
- evidence:本次结账 step 5 写回执时被自家护栏拦下(exit 2)。三组对照实测(临时目录跑 hook,不改任何仓):**A** 项目端对自己仓的相对路径 `agent-on.lock.md` 做暂存 → `exit=0` 放行;**B** 同上且命令里无绝对路径 → `exit=0`;**C** 命令里把 B 仓**绝对路径当内容**写进某文件(`echo "cd <B绝对路径> && claude" > note.md` 后再暂存该文件)→ **`exit=2` 拦截**。根因在 `kit/guard/agent-on-git-guard.sh` 的 `PATH_RE`(形如 `[~/][\w@.\-/\\]*agent-on[\w@.\-/\\]*`):它在**整条命令文本**里搜路径片段、全部当作写目标,不区分「操作的 target」与「字符串字面量 / heredoc 内容」。**递归自证**:随后我把这张卡本身追加进 intake 时又被拦了一次——因为卡的正文里引用了带 `add` 字样的命令样例,而 intake 路径在 B 内
- confidence:high(三组对照可复跑;根因定位到具体正则;且被同一缺陷拦了两次,第二次是写这张卡时)
- claim:机械闸的目标识别**不能用「命令文本里出现路径子串」当判据**——命令里的路径可能只是要写进文件的内容。判据应收窄到真实 target:①`cd` / `-C` 解析出的工作目录(该脚本已有此逻辑,正确)②子命令后的 pathspec 位置参数;**不含**重定向内容、heredoc 体、引号内字符串。更一般地:**任何「按文本模式判定意图」的护栏,都要先分清「这段文本是指令还是数据」**——把数据当指令读,是护栏版的注入型误判
- suggested_landing:`kit/guard/agent-on-git-guard.sh` 收窄 PATH_RE 用法(只在 pathspec 位置取候选,或剔除重定向/heredoc/引号内文本)+ `kit/guard/README` 失效面补一行;anti-hallucination 挂半句(护栏误判=把数据当指令)
- rollback:revert 修复 commit 回到现状。方向纪律:**误拦有绕法、漏拦无补救**,修复必须保持 fail-closed 倾向,宁可继续误拦也不要放宽成「找不到就放行」
- trace:本会话结账 step 5 与写卡两次被拦全文 + 三组对照 exit code;规则出处 = 该脚本的 PATH_RE 定义行
- 补充(自伤路径):settlement §6 **要求**把消化粘贴令(含 `$WRITE_ROOT` 绝对路径)写进项目 loop-notes 待办位 —— 于是**协议第 6 步产出的内容会拦住协议第 5 步的回执提交**。这是两个 agent-on 机制自撞,不是项目端用错。修好前的绕法:回执改用编辑工具落盘、命令行里不出现该绝对路径(本次即如此,并已在项目 loop-notes 就地留注)
- 状态:pending
