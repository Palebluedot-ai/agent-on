# intake · Euan-Flutter · 2026-07-26 结账(3 卡)

> 来源会话:双人分轨开局的协作坑清理(CODEOWNERS 空转 / 工作区脏文件分诊 / 原子提交被暂存残留污染)。
> 主路径素材说明:本项目 `agent-on/loop-notes.md` 自 2026-07-12 起未追加新条目(07-21/07-23 两次结账的卡直接从会话装配),本批同此路径;三卡均为当场发生并已修复的过程教训,证据全为本会话真实命令输出与 commit。
> 域判据自检:三张均属 AI 协作过程(工具行为 / 纪律机械化 / 防幻觉),无产品域知识出仓。

---

### staged-residue-breaks-atomic-commit(`git add <路径>` 不限定提交范围,暂存残留会被吞进原子提交)
- source:Euan-Flutter @ 1d883a8+772b511 | pin v0.3.0
- evidence:本会话实证——先 `git rm -r --cached supabase/.temp/`(9 个 D 进暂存区),随后 `git add .github/CODEOWNERS && git commit -F m1.txt`,产出的 `d1ee650` 把 9 个删除一并吞入,message 只讲 CODEOWNERS。误导点:`git show --stat -1` 输出 `1 file changed` 看起来正常,是靠独立命令 `git ls-files supabase/.temp/` 返回空才交叉验证出真相。修复:`git reset --mixed HEAD~2` 重做为 `1d883a8`(仅 CODEOWNERS)+ `772b511`(仅 untrack+ignore)。
- confidence:high(git 的确定性行为,非偶发;任何声明「原子提交」的项目都会撞)
- claim:声明原子提交的流程,commit 前必须读**全暂存区**(`git status --short`)而非只看自己刚 `git add` 的路径——`git add <路径>` 只增不减,不构成提交范围的限定;且 `git show --stat` 会因只展示末次 commit 而掩盖前一笔的污染,验证必须配一条独立交叉命令。
- suggested_landing:playbook/anti-hallucination.md「工具输出可疑」一节增补一条「命令的默认作用域 ≠ 你传的参数范围」;kit/AGENTS-skeleton.md 提交纪律行补半句「commit 前 `git status --short` 读全暂存区」
- rollback:revert 落地 commit;若判定过度约束(小项目不跑原子提交),降级为 bench 案例
- trace:本会话工具输出——`git show --stat --format="%h %s" -1` 与 `git ls-files supabase/.temp/` 两次交叉;重做锚 `git reset --mixed HEAD~2`
- 状态:pending

---

### dirty-file-not-garbage-triage-before-clean(长期挂 M 的脏文件里混着「没提交的真内容」,清理前必须逐个判语义)
- source:Euan-Flutter @ c739711+772b511 | pin v0.3.0
- evidence:主 worktree 三处「脏」看起来同质,分诊后是三种性质——(1)`design/euan.pen` 挂了 9 天(mtime 07-17),`git diff` 出来是两条**决策批注**(阶段轴五档→四档、双轴定案),与 07-12 拍板 `6c70b9e` 对齐,属没提交的真内容 → 补提 `c739711`;(2)`supabase/.temp/` 9 个文件是 supabase CLI 每次运行重写的机器态(`cli-latest` 已自行从 v2.109.0 漂到 v2.109.1)→ untrack+ignore `772b511`;(3)未跟踪的 `Claude design demo/` 43M 是导出源,锚点本体已在仓内 `design/demo-v9/` → 仅 ignore。若按「清理脏工作区」批量 `git checkout .`,第(1)类会被静默销毁且无人察觉。
- confidence:medium(单项目一次,但与「批量销毁类命令须先读语义」的既有直觉同源,普适性高)
- claim:清理工作区前必须逐文件读 diff 判语义并三分类(真内容→提交 / 机器生成态→untrack+ignore / 真垃圾→丢),禁止对未读过的改动批量 `checkout`/`clean`;「挂了很久」是需要解释的信号,不是可以丢弃的理由。
- suggested_landing:playbook/anti-hallucination.md 或 sop 补一条「工作区分诊三分类」;kit/guard 的 deny 清单旁注一行(`git clean -fdx`/`git checkout .` 已在硬墙,补「读语义」的人侧动作)
- rollback:revert 落地 commit(纯纪律条,无机制依赖)
- rollback_note:该条只增加一次读 diff 的成本,误判风险低
- trace:本会话 `git status --short` → `git diff design/euan.pen` → `ls -l` mtime 核对 → `git log -1 -- design/euan.pen` 对账 `6c70b9e`
- 状态:pending

---

### guardrail-entity-must-resolve(配置里写实体名的护栏,落地时必须验证实体真的解析得到,否则静默空转)
- source:Euan-Flutter @ 1d883a8 | pin v0.3.0
- evidence:`.github/CODEOWNERS` 八条独占区(`/docs/state/` `/contracts/` `/AGENTS.md` 等)的 owner 全是占位符 `@maintainer`——GitHub 解析不到这个 handle,自动 request review 从建档起**三周全程空转**;而 `CONTRIBUTING.md §七` 白纸黑字写着「三层软护栏,CODEOWNERS 自动 request maintainer review」,读起来像已生效。仓里第二个提交者(`@jsui1998-cpu`,PR #10 已合)进场后风险才真正兑现。修复 `1d883a8`:`gh api user` 核实真 handle 后替换,并在文件头写清生效边界(GitHub 只对**非作者**触发,项目主自己动不触发)。
- confidence:high(与 D21 的 `wiring-anchor-def-not-integration` 同族:定义齐全 ≠ 接线生效,此为其在「外部平台配置」维度的实例)
- claim:凡在配置文件里写实体名的护栏(CODEOWNERS / 审批人 / 告警接收人 / webhook 目标),落地时必须做一次**实体存在性验证**(API 查询或一次真实触发)并留证据指针;同时在文档里写明生效边界,禁止把「已配置」叙述成「已生效」。
- suggested_landing:建议**并入既有 slug `wiring-anchor-def-not-integration`** 作为第二实例(消化会话按语义归并,勿另起新篇);双落点:playbook 该条正文补「外部平台配置」维度 + kit 的完成判据 checklist 补「护栏类改动须附实体验证证据」
- rollback:revert 落地 commit;若判定与 wiring-anchor 重复度过高,直接 rejected(已被覆盖)
- trace:本会话 `cat .github/CODEOWNERS`(8 处 `@maintainer`)→ `gh api user --jq .login`(`Palebluedot-ai`)→ `gh api repos/.../collaborators`(`jsui1998-cpu`)
- 状态:pending
