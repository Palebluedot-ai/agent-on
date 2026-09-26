# bench/cases/ — 翻车案例集索引

> 职责边界:结构化翻车案例卡(症状/根因/修法/可复用规则/已固化到哪 五节),给两类读者——新项目里的 AI(动手前扫同类坑)与能力探针出题库。模型会过时,案例集只增值。入册:2026-07-07 批三(17 张,Euan 实战 15 + 前身仓标本 2);审查:全量对抗复核,事实层零编造,指针层问题已修。2026-07-09 首结账消化增补 3 张(18-20,Euan Run #6/#8/#9,confidence low 单次);2026-07-12 第四次消化增补 3 张(21-23,Euan 高置信);2026-07-13 第七次消化增补 1 张(24,IPONews 首笔);2026-07-26 第十三次消化增补 1 张(25,Euan .dc.html);2026-07-29 第十四次消化增补 1 张(26,限流盖故障码);2026-07-30 第十五次消化增补 2 张(27 互踩≠撞题 / 28 审计对抗复核);2026-08-17 值守批消化增补 2 张(37 CI 信号源 / 38 秒死=账单层,Dartify 值守两批);2026-09-26 消化批增补 46–50(46 跨仓守卫按文本判越界,CryptoQuant + aster-agent 两项目四次复现;47 规则改了执行面没换,误拦实出自插件缓存旧闸;48 消化半截 + 预写版本号;49 独立复核共用搜索面;50 旧授权改冻结面,值守拦下的正例),并同批补齐索引里缺失的 41–45 行——那五条此前只出现在「使用时机」里,表里没有,扫表的人会以为案例集停在 40。(46 的草稿写于 09-21 那场未提交的消化,09-26 收编时改判了根因。)

| # | 案例 | 一句话规则 |
|---|---|---|
| [01](01-vercel-dev-local-vs-prod.md) | vercel dev 三连坑 | 本地网关不通先 curl 分层定位,每层只改一个变量;模拟器≠生产,兜底写在服务端 |
| [02](02-wireframe-expectation.md) | 线框期没打预防针 | 交未过视觉工序的产出前,先声明它是什么阶段、这轮看什么 |
| [03](03-parallel-collision-single-writer.md) | 裸并行撞车 | 并行前先冻契约(连语义),轨道=worktree 物理隔离,状态只有一个写者 |
| [04](04-env-inline-comment.md) | .env 行内注释 | 值行禁行内注释;A 路通 B 路不通先怀疑解析差异,让进程自曝 env |
| [05](05-fixture-hidden-contract.md) | fixture 隐性契约 | 冻结 fixture 时把排序/上限/空值语义一起冻死 |
| [06](06-test-env-assumption-flip.md) | Fake→真翻转漏网 | 翻转全局默认前,grep 全部进壳测试确认钉了全量 Fake |
| [07](07-esm-import-suffix.md) | ESM 不重写 specifier | 本地与生产的运行时差异第一次部署就冒烟抓;`type:module` 相对 import 带 `.js` |
| [08](08-postgrest-microsecond-window.md) | 时间戳微秒窗 | 库时间戳当游标一律透传原生串,禁客户端归一;精度坑只有 LIVE 抓得到 |
| [09](09-adapter-trust-post504.md) | 适配器 POST 全挂 | 桥接层够薄就自己写;部署后 GET 和 POST 各打一发 |
| [10](10-basil-silent-field-move.md) | API 版本静默搬家 | 第一次 LIVE 先 dump 真实载荷逐字段对账——别信文档记忆 |
| [11](11-region-mismatch.md) | 函数区≠数据区 | 接托管服务第一天对齐地域,比一切代码优化便宜 |
| [12](12-platform-header-stripping.md) | 平台剥头/304 剥光 | 自定义头/缓存别信本地绿,canary 打真实边缘出口 |
| [13](13-cache-chain-coupling.md) | 缓存链一环不稳 | 上游每次都变,下游缓存全是摆设;不稳的环加复用窗口或别进指纹 |
| [14](14-cli-agent-autoconfirm.md) | echo n 不是护栏 | 会写状态的 CLI 在 agent 环境假定自动 yes;声明式配置全量声明+行为探测 |
| [15](15-website-rework-elicitation.md) | 官网返工·偏好缺口 | 「待拍板」维度只有三条路:可切换/并列变体/先问——静默选边=违纪 |
| [16](16-gen1-engine-vs-discipline.md) | 一代标本·造引擎 | 动手造编排层前先问工具是否已覆盖——能写成约束的别写成系统 |
| [17](17-gen2-governance-drift.md) | 二代标本·治理空转 | 写治理文档前先问这条规则有没有对应的执行或验证 |
| [18](18-readonly-guardrails.md) | 只读三层护栏 | 「绝不能写」的场景用运行时+输入面+静态面三层,别押单点信任 |
| [19](19-destructive-api-protection.md) | 危险端点+终态核查 | 删除类 API 先定「什么绝不能删」(护栏端点 403);验收信终态脚本不信过程 console |
| [20](20-eval-goldset-honesty.md) | 评测集诚实 | 评测瓶颈是数据不是工具;金标 n=1 只立口径不支撑调优,写死在告警里 |
| [21](21-stale-codegen-double-false-negative.md) | 双假阴性 | 测试期望值锚真相源不锚生成物;「重跑生成零 diff」必须真进 CI |
| [22](22-editor-memory-truth-layers.md) | 编辑器三层真相 | 编辑器类 MCP 改的是内存——commit 前查 mtime,验证走数据模型不走渲染 |
| [23](23-sed-charclass-disaster.md) | sed 字符类事故 | 结构化文件一律编辑工具精确锚点;shell 替换只用于自产无特殊字符内容 |
| [24](24-fanout-without-probe.md) | 无探针大扇出 | 大扇出前 1 个探针子代理先验关键工具通路;探针不过 = 不扇出,改主会话定向抓取 |
| [25](25-dc-html-http-and-fiber-drive.md) | .dc.html HTTP+fiber 驱动 | Claude Design 导出禁止 file:// 验收;基线截图走 fiber 无头直切,可一键再生 |
| [26](26-ratelimit-masks-real-error.md) | 限流盖住真故障 | 排查期错误码「变好」须先验因果链;限流/降级解除后复测才可结案 |
| [27](27-two-collision-diseases.md) | 互踩≠撞题≠撞号 | 环境→worktree;撞题→开工声明;撞号→取号即落盘 |
| [28](28-audit-adversarial-verifiers.md) | 审计对抗复核 | 合规审计指控与定罪分离;复核以推翻为目标;事实与归因分判 |
| [29](29-ambiguous-side-effect-readback.md) | 外部副作用 read-back | 解析失败≠传输失败;可能已送达时冻结重放,先只读对账 |
| [30](30-ledger-ratchet-and-author-deadlock.md) | 记账棘轮+作者闸死锁 | merge 记账须 CI 闸;守卫允许集扩面 maintainer 同分支重开 |
| [31](31-worktree-gc-orphan-rescue.md) | 回收执行体+孤本 | 日历死线须进程;无 PR worktree 只报告不删,先 push 再收 |
| [32](32-worktree-stale-ship-and-classifier.md) | 陈旧 worktree 装机 | 交付前对表 default branch;没生效先查交付链;闸拒字面≠禁目标 |
| [33](33-config-green-is-not-runtime.md) | 配置面绿≠运行面 | 定时看 runs+exit+产物;Webhook 看入站 POST;一应用一 URL |
| [34](34-gate-three-faces.md) | 闸的三张面 | 触发面覆盖保护分支;读历史的闸 commit 后跑;直推会脏自己的 PR |
| [35](35-discriminating-probe.md) | 判别式探针 | 证明约束生效靠暗号值,不靠压力测试恰好合规 |
| [36](36-browser-entry-no-node-fs.md) | 浏览器入口打进 node:fs | CSR barrel 不得 re-export Node 写盘;读数与写盘分入口 |
| [37](37-ci-signal-source.md) | 等 CI 看错信号源 | checks --watch 滞后 + 快 workflow 都造假全绿;run id 按 workflowName 过滤后盯 run watch |
| [38](38-instant-job-death-billing.md) | 全 job 秒死=账单层 | 秒死 + 零 step 先拉 job annotation 查账单,别按测试红分诊 |
| [39](39-harness-boilerplate-as-project-rule.md) | 宿主安全句当项目制度 | 答不出文件名的规矩不是制度;过度收紧也是漂移,授权要幂等 |
| [40](40-gate-exit-unreachable.md) | 出口在权限外的闸=死锁 | 每个 FAIL 至少一条出口在被拦者权限内且非破坏;互锁 FAIL 对设计期消除 |
| [41](41-duty-window-double-decision.md) | 值守窗口的双重拍板 | 同一个决定问两遍,第二遍零信息量;内容已拍板的单,合并是机械步骤 |
| [42](42-lesson-misattribution-overtightening.md) | 闸不说出口,人就变成闸 | 事故教训写歪会把唯一正确的动作一起禁掉;新规矩过「哪个动作/正确动作/会不会连坐」三问 |
| [43](43-projection-without-provenance.md) | 没有回指的投影 | 副本不带可验锚点=判定错;投影必须 `<!-- src: 路径#符号 -->` 且机械可验,只报不拦 |
| [44](44-asking-shapes-the-answer.md) | 问法决定答案质量 | 全批落款会吞掉「请复核」;派工必须说清取证面 |
| [45](45-gate-charges-the-wrong-tree.md) | 闸把账算到别人头上 | 有记录 ≠ 有人;红灯只算当事那棵树,记录要有心跳与到期 |
| [46](46-cross-repo-guard-matches-text-not-target.md) | 守卫按文本判越界 | 跨仓守卫判「git 作用于哪个仓」,不判「文本提到哪个仓」;现场出口=拆成两条命令 |
| [47](47-rule-changed-executor-did-not.md) | 规则改了,执行面没换 | 被拦先看执行体路径;发版/升级对一遍宿主上挂着的 hook;缓存里只放转发壳 |
| [48](48-digest-orphaned-with-premarked-tag.md) | 消化半截 + 预写版本号 | 去向只引用已存在的 commit/tag;提交前问 hunk 是不是本批的;自检全过先问在哪跑的 |
| [49](49-independent-review-shares-search-surface.md) | 独立复核共用搜索面 | 独立 = 换搜索面不只换人;否定结论贴正对照与命中数,扫其余树与全部 ref |
| [50](50-stale-and-stretched-approval.md) | 旧授权改冻结面(正例) | 授权只在给出它的那一批、那个问题里有效;转述 / 过期 / 外延三种冒牌都不认 |

## 使用时机

- **接外部服务 / 首次部署前**:案 01/04/07-14/33 + sop.md「外部服务集成清单」(该清单由本批案例审查倒逼产出)
- **上多 agent 并行前**:案 03/05/06/27(互踩 vs 撞题分型)
- **给用户交付产出前**:案 02/15
- **想造框架/写大文档时**:案 16/17
- **设计危险/只读/删除类操作时**:案 18/19
- **给 AI 产品建评测集时**:案 20
- **有代码生成物/批量文本替换/编辑器型 MCP 时**:案 21/23/22
- **shell 工具在 agent 手里的语义陷阱**:案 14/23
- **跑大扇出调研/抓取前**:案 24 + playbook/workflow-orchestration.md §〇 探针闸门
- **跑 Claude Design / .dc.html 交互 demo 前**:案 25(HTTP 服务 + fiber 无头截基线)
- **排障期错误码突然「变好」时**:案 26(限流/熔断是否盖住真因)
- **多周合规/harness 审计时**:案 28(取证+对抗复核双层)
- **外部消息/API 可能已产生副作用时**:案 29(先 read-back 再重试)
- **merge 记账依从率崩 / 多 worktree 堆积时**:案 30/31
- **从 worktree 装机/演示前**:案 32(对表落差 + 交付链先于环境)
- **接定时任务 / IM Webhook / 开放平台时**:案 33(配置勾选≠运行面;一应用一 URL)
- **写或合 CI 闸时**:案 34(触发面/读取面/并发面/出口面——报错即工单)
- **要证明某约束真在生效时**:案 35(判别式探针,不靠压力测试)
- **CSR barrel 同时导出读数与写盘时**:案 36(浏览器入口不得 re-export Node)
- **值守/自动化「等 CI 完成再动作」时**:案 37(信号源显式指定,禁裸 --limit 1)
- **CI 全线 job 秒死零 step 时**:案 38(先查账单层,这红不属于任何 PR)
- **被宿主通用安全句要求「先确认」时**:案 39(答不出文件名的规矩不是制度)
- **设计闸 / 被闸拦住且解法看着互相打架时**:案 40(出口必须在被拦者权限内;互锁 FAIL 对 = 死锁)
- **闸靠一条静态记录判「有人」、或红灯落到不相干的人头上时**:案 45(有记录 ≠ 有人;红只算到当事树,记录要有心跳与到期)
- **跨仓守卫拦下只读引用 / 复合命令 / heredoc 正文时**:案 46(先分段再判目标仓;出口=把上游引用与本项目 git 拆成两条命令)
- **仓里修好了、现场还在被拦 / 拦截文案点名的脚本在缓存目录时**:案 47(先核执行面,别先改判据)
- **开消化 / 发版 / 在工作区看到来历不明的未提交改动时**:案 48(主树自证;标注不预写版本号;不是本批的 hunk 不进本批 commit)
- **要下「X 还没做 / 不存在」的结论、或两遍复核一致时**:案 49(先问两遍看的是不是同一块地方;正对照 + 命中数)
- **要动用户签过字的冻结面、或核一张单的拍板来源时**:案 50(是用户本人在这里说的吗?为这一批说的吗?说的就是这件事吗?)
