# bench/cases/ — 翻车案例集索引

> 职责边界:结构化翻车案例卡(症状/根因/修法/可复用规则/已固化到哪 五节),给两类读者——新项目里的 AI(动手前扫同类坑)与能力探针出题库。模型会过时,案例集只增值。入册:2026-07-07 批三(17 张,Euan 实战 15 + 前身仓标本 2);审查:全量对抗复核,事实层零编造,指针层问题已修。2026-07-09 首结账消化增补 3 张(18-20,Euan Run #6/#8/#9,confidence low 单次);2026-07-12 第四次消化增补 3 张(21-23,Euan 高置信);2026-07-13 第七次消化增补 1 张(24,IPONews 首笔);2026-07-26 第十三次消化增补 1 张(25,Euan .dc.html);2026-07-29 第十四次消化增补 1 张(26,限流盖故障码);2026-07-30 第十五次消化增补 2 张(27 互踩≠撞题 / 28 审计对抗复核)。

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
