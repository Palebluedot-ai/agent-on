# 合流检查（按影响选用）

> 职责边界：多轨合流时按实际影响选择检查；单页样式修改不套全表，required CI 和真实集成验证仍须满足。
> 双轨模式的第三环:轨道各自绿 ≠ 合流绿 ≠ 边缘出口绿。Run #3(撞车修复)与 Run #7(生产 canary 抓平台行为)的固化。

- [ ] **0. 开工/提交前查重**(第四型撞题):`gh pr list` **开工前一次 + 提 PR/push 前再一次**,按主题(安全洞/功能名)不靠文件路径;撞题 → handoff 不抢轨。
- [ ] **0b. 共享改动对表**(并行时)：看任务意图与已提交/未提交 diff，共享文件归一个任务；claim/owns 非前置。现有同文件闸命中实际未提交冲突才停。依赖先集成再启动消费者，不为每个页面复制 worktree 和完整验证。
- [ ] **0b-1. 实际同文件冲突先协调**：现行闸只看两棵树中最近碰过的同一未提交文件，补 owns 不会解除它。由对应作者收口共享修改，再推进；不替别的窗口 stash、commit 或清理。旧树只报告；已明确启用的受管回收按 [巡逻执行书](patrol-control-plane.md) 处理。
- [ ] **0c. 值守在班对表**(仓启用值守合并调度时):本清单的 merge / 追平执行权归在班值守窗口([kit/babysit/](babysit/README.md))——功能会话开 PR 即交付,不自己合;追平一律服务端 update-branch。值守不在班回退本清单原规则。
- [ ] **1. 顺序合流**:契约变更(若有)先进 main;再 merge 各轨 worktree 分支(`--no-ff` 留 run 痕)。**合流前文件域对照**:各轨改动清单互不重叠?三共享面(design/ · contracts/ · progress.yaml)归口单一轨了吗?重叠先裁决再合。
- [ ] **1b. 撞上改名浪潮时**(对方 PR 做了大规模机械 rename):①从已合并侧反推映射表 ②对冲突文件算 `diff(base套映射, 现状)` 残差行数分档——≈0=纯改名机械解,残差大=真双改手工并 ③**新增文件不触发冲突**,收尾独立 grep 旧标识符(Euan 2026-07-30:100 文件去命名 + 18 冲突残差分档)
- [ ] **2. 合流验证**：共享契约和跨端行为变动跑受影响的完整回归与类型检查；样式/文案先预览和交互，不给每页重复搭 TDD。仓库 required CI 仍须满足；真实失败先修再继续。
- [ ] **2b. 红灯先分来源**:check 在不在本仓 `.github/workflows/`?仓内硬门必须绿;外部集成(Preview/Bot)取证 summary 原文再判——与本 PR 无关的噪音写进悬点栏,别当缺陷堵合流(Euan PR #28 Supabase Preview 缺密钥红 vs ci.yml 四 job 全绿)
- [ ] **2c. 禁止红着合**:闸的触发面必须覆盖它要保护的分支。PR-only 检查红着合入 = 把红转嫁给后来者(后续每条 PR 代付,main 上零症状)。「本地绿 / CI 红」先查被 gitignore 的文件是否被当成必需输入(`git ls-tree <default> --name-only`)。
- [ ] **2d. 删功能先划标识边界**:移除某产品面时先冻「功能标识」vs「共享领域词」;共享词上的 pipeline/dashboard/override 默认保留,除非证据证明只服务被删面。禁止把口令里的业务词当成整域可删。
- [ ] **2e. 门禁链不接管道**:`set -e` 下写 `pytest -q | tail -3 && <提交>`,生效的是 `tail` 的退出码,红测试照样进库。门禁链一律 `set -o pipefail` + 显式 `|| exit 1`,证据输出在提交**之前**读完——被 `tail` 吃掉的那段正是该看的(anti-hallucination C 附6 第 1 条;CryptoQuant `9a4a967`)。
- [ ] **4. 翻转 Fake→真**:provider/开关切真实现;**grep 所有「进壳/进页」测试确认钉了全量 Fake overrides**(Run #3 教训:只钉 auth 会裸奔)。翻转后再跑一遍全量。
- [ ] **5. 部署 + 生产 canary + 并行轨各跑 LIVE**(有部署面/双轨时):部署后跑边缘出口的 LIVE 探针——平台会剥头/整形/缓存,本地绿不算数(Run #7:Vercel 剥 Server-Timing/304 剥光)。**双轨并行时,合流前让每轨各自跑一发 LIVE 当发现器**(不只验证已知):并行 LIVE 会撞出串行/单测永远遇不到的真分支——权限边界、精度窗、回放态(Euan Run #4 GoTrue global-logout 401 真分支 / #6 微秒窗 / #8 清号回放,项目内 3 次独立复现)。**推送/部署前验证作用域**:多 worktree 时确认 CLI 读的是目标树(cwd 常压过 flag),diff 方向正确再不可逆。**静态资产**:带 cache-buster 请求 + 留传播窗口后二次确认再判部署成败(sop 集成清单第 8 条)。**生产 load**:机器 preflight 全绿(skip 仍红);门禁接在唯一执行入口;若用 runtime checkout 须 clean/持久/专用(非开发 dirty 树)。
- [ ] **5b. 外部运行面**(定时 / Webhook / 开放平台):配置勾选与日历开火都不算完成。定时看 runs+exit+产物;Webhook 看 `tail`/access log 是否有平台入站 POST,不是看权限页。一应用一事件 URL,共用会静默吞回调(anti-hallucination C 附3;sop 集成清单 12–13)。
- [ ] **6. 上机人验**:装真机/模拟器,给用户一句「去点哪里看什么」。**交付前对表**:`git fetch` + `rev-list --count HEAD..origin/<default>`=0 再 build/install;播报写 commit hash 与落差(kit/worktree-gc-pattern「交付前对表」)。
- [ ] **7. 记账与回流**：更新项目已使用的状态面；真实返工、撞车、重复手工或脚手架负担记 loop-notes，有证据再提上游。dashboard 仅已启用时重绘。worktree 巡检报告可回收与保护原因，删除遵实际授权，不为收尾强清旧树。
- [ ] **7a. lane 落地与回收**(启用控制面时):远端权威确认该轨已进入 base → `agent-on worktree set-status landed --id <lane>` → `agent-on worktree status` → `agent-on worktree gc --dry-run`;只有 GC `candidate` 才进入人工拆树裁决，review 查 PR/squash，rescue 先 push/commit 消孤本。CLI 不自动删，分支默认保留；locked/dirty/unknown 不删。
- [ ] **7b. 状态面与代码同批收口**:核对 exact HEAD + 测试输出 + progress/phase/dashboard(及计划 TODO 当前态)一致;禁止代码已前进而状态面仍写已关闭 P0 或未来时态。易变计数(ahead/behind)不写入长期真相——push 前 fresh fetch。**合并多个 PR 后**扫一遍 TODO 触发是否已命中未回表(禁靠「顺手」)。**记账棘轮**:merge 与 progress/dashboard 出现本 PR 号是同动作两半——有 CI 闸更好(kit/ledger-ratchet-pattern.md);无闸=自觉必断档。**元动作自涵盖**:本次记账/chore(state) 自己的 PR 号也要入账,禁止只记别人。补账补内容不补号码;**叙述别人未清偿的工作不写其编号**——字面匹配闸会把「提及」当「销账」,替人解锁。
- [ ] **7c. 证据对准当前状态**：代码、base 或相关输入变化时重新验证受影响的范围，required CI 对准当前 head。内容与环境均没变化的检查不因窗口多而重复跑；未实现证据缓存时不能凭旧绿灯跳过必需检查。门禁中发现真实缺陷先修，再对最终状态验证。
- [ ] **7d. 远端状态权威 API**:PR/CI/部署下一步前 `gh pr view` / `gh run view`(或等价) fresh read-back;页面感觉只做线索。`mergeStateStatus=DIRTY` 时先查 base 是否刚动过 / 分支是否从旧 HEAD 长出,不要先查 CI 配置(GitHub 不为 DIRTY PR 起 checkout)。
- [ ] **7g. 直推 default branch 前置**:记账/hotfix 等豁免评审的直推,推之前 `gh pr list --state open` 并对改动文件集——豁免的是评审,不是并发影响。
- [ ] **7h. 读历史的闸在 commit 之后跑**:闸里有 `git log` / `blame` / `merge-base` / `show <ref>:` 时,提交前跑出的绿只证明工作区没坏。
- [ ] **7e. 同名分派点**(switch case / 路由 path / 事件 type / DI token):两分支都改过同一分派结构时,合并后显式检查是否「同键两份」第二份死代码——**两边测试都绿对合并态零保证**。
- [ ] **7f. docs/状态-only 与生产解耦**:仅 progress/dashboard/docs 的变更**不得**触发生产 load/deploy(paths-filter 或 workflow 条件);生产 CI 红/Cloud 挂起应查**是否误把文档提交绑进 deploy job**,勿用「再 update 一遍脚手架」当修复。

## DoD 门禁(一代移植)

> 源流:一代 agent-orchestration-playbook/05_playbooks/execution-layer-dod-v1.md,2026-07-07 批二移植。
> 上面七步管「怎么合」;这一节管「什么才算合完」——七步全打勾但下面四条不满足,不许标 done。

一次编排的完成标准不是「能跑」,而是四条同时成立:

1. **可编排**:状态机稳定,任务始终停在某个已定义状态,没有悬空。
2. **可审计**:每次状态变更都有 audit_event,事件轨迹从 queued 到 done 完整不断。
3. **可追责**:每个决策能说清是谁、对着哪张卡、基于什么证据做的(result_card 的 decisions + evidence_links)。
4. **可回放**:失败能凭 run_id + audit 轨迹定位到出错那一步,不靠记忆复盘。

### 最小卡集规则

只有项目明确采用 audit-event 卡片流程时，任务与结果必须满足其 schema；有实际可复用教训才补 memory_card。普通工作可用目标、验收证据和结果回执，不要求每次交付生成三张卡。

### 固定状态机

`queued → triaged → planned → running → reviewing → done | failed`

选择 audit-event 流程时，只走 `kit/schemas/audit-event.schema.json` 定义的迁移，`agent-on audit-lint` 验其记录；未选择该流程的任务不套固定状态机。
