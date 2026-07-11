# intake — 2026-07-11 · 外部输入：Agent 记忆系统三方案 + KC SDK（朋友分享）

> 来源：用户转来朋友的多 Agent 记忆系统方案（方案一 SQLite+vss+Redis / 方案二 Git+Qdrant+NATS+CRDT / 方案三 自演化记忆 / KC SDK 纯标准库 facade）。经两路独立分析（对照审查 14 份 canonical 文件 + 数据库与开箱即用专题）提炼。总裁决：**材料是一面照出隐藏债务的镜子，不是可引进的零件库**——三方案架构几乎整体落在宪章「造引擎」死亡名单（它们解决的问题正是 agent-on 用「单人·单写者·纯文件」前提主动消除掉的）。真增量 1.5 条 + 3 条规模信号触发的 deferred。

### no-database-stance（宪章级立场：记忆不建数据库）
- source:外部输入分析 2026-07-11 | agent-on @ 23bc0d8
- evidence:①CHARTER「零铺垫启动 clone→开工≤1h」——内置 DB 则 clone 后先装 colima/docker/配 key②用户机上实锤:`evolving-memory-system/04_protocols/scripts/gbrain_runtime_recover.sh`——专门写的 DB 自愈脚本,其存在本身证明本地 DB 会宕、是单点故障(colima 不跑即 ECONNREFUSED、PGLite 迁移曾得空库)③用户 4 月已写对的原则原文:`evolving-memory-system/SCHEMA.md`「Git repository is the source of truth…Retrieval/index systems are rebuildable cache layers」
- confidence:high（两路分析独立收敛 + 用户自己的先例）
- claim:agent-on 的记忆就是 git 仓里的 markdown——clone 即得、grep 即查、断网可用;数据库永远只能是「可选、可重建、坏了自动退回 grep、不进 BOOTSTRAP 必需件」的旁挂索引缓存。把这条从实践升成宪章级承诺（开箱即用的产品,记忆系统不能是需要先修 docker 的那一环）
- suggested_landing:CHARTER 边界节一条 + README FAQ 一问「要装数据库吗?」
- rollback:revert
- trace:数据库专题四问全文 / 对照审查禁区#2
- 状态:pending（中风险,CHARTER 动向需用户拍板）

### distill-merge-abstraction（消化补半句：同类多条合并抽象）
- source:同上
- evidence:memory-layering.md 已有洞见「loop-notes 里 L1 现象与末尾抽出的可复用 loop 分开拎」;但 settlement.md 下半场只有「同 slug≥2 置顶」,没有「N 条相似条目合并抽象成一条 L2」的显式步骤——外部蒸馏管线里唯一可借的子动作
- confidence:high
- claim:settlement.md 下半场频次扫描步后加半句「同类多条 → 合并抽象为一条 L2 再落地,不逐条搬运」
- suggested_landing:boot/settlement.md 下半场第 1 步
- rollback:revert
- trace:对照审查真增量 B（唯一「现在值得做」项）
- 状态:pending（低风险,下次消化直落）

### cases-retirement-tiering（案例集「只增值」的规模债 → 分层退役）
- source:同上
- evidence:bench/cases「只增值」承诺在 20 张成立,100+ 张撞 memory-layering 自己说的「全塞一个文件捞不出来」;天然退役判据已隐含——**案例一旦升成 L3 门禁（进了 merge-checklist）,作为 L2 扫坑案例的边际价值归零（门禁已自动生效）**;删除违反举证层「只追溯永不删」,正解是降级可见性:活跃扫坑清单 vs 归档案例库两级
- confidence:medium
- claim:案例集加「活跃/归档」两级可见性规则;已固化进 kit 门禁的案例退出默认扫坑清单,索引全留
- suggested_landing:bench/cases/README.md 或 memory-layering.md 一条规则
- rollback:revert
- trace:对照审查真增量 A
- 状态:pending（**deferred**:触发信号=案例超 ~40-50 张,或 loop-notes 首次出现「扫坑捞不到/扫一堆无关」真实摩擦——进 CHANGELOG Backlog）

### memory-health-visibility（方法论健康度可见化——「越用越强」变可验收）
- source:同上
- evidence:CHARTER 承诺 4「越用越强」目前靠翻 CHANGELOG 感受;健康信号散落(intake 积压靠 ls/待消化靠 loop-notes 待办位/偏离靠结账翻);外部方案三「新鲜度仪表盘」的 solo 版=几个活体数字（案例数/已消化/待消化/最近消化日期,全部 ls|wc 与 grep 数得出,不建自动脚本不手填）
- confidence:medium
- claim:agent-on 仓 README 或 CHANGELOG 顶部加一排「活体数字」;或将来出 agent-on 自己的元仪表盘。警惕两头:自动统计脚本=运行时禁区,手填数字=dashboard 禁手填铁律
- suggested_landing:README 顶部一行（消化时定形态）
- rollback:revert
- trace:对照审查真增量 C
- 状态:pending（**deferred**:触发=服务 2-3 个项目/需对外证明「越用越强」时）

### semantic-retrieval-adapter（语义检索的将来形态：可选旁挂,不自建）
- source:同上
- evidence:iteration-loop 裂缝#1（slug 手写弱匹配）是真;但 solo 规模下三条检索通道（README 索引/使用时机表/grep）远未到顶——三个可观测到顶信号:①README 漏登记≥2 次或索引撑破一屏(~50-80 条)②同坑不同措辞 grep 漏合≥3 次或每次消化必须全量通读兜底③使用时机表单场景挂 >10 卡退化为全扫。届时形态照 evolving-memory 先例:**内容仓=canonical 文件,检索层=可重建索引缓存（如 gbrain 索引本仓）,索引失败不阻断消化、坏了无感退回 grep、不进 BOOTSTRAP 必需件**;simhash（汉明≤3）记为将来去重词表的参考方案
- confidence:high（形态）/ low（时机——远未到）
- claim:三条到顶信号 + 最小适配草图入 Backlog,作为「什么时候才 justified」的预立决策,防止到时候临时拍脑袋自建向量库
- suggested_landing:CHANGELOG Backlog 一条（含三信号+三设计闸:永远 optional/坏退 grep/不进必需件）
- rollback:—
- trace:数据库专题第 2/3 问全文
- 状态:pending（**deferred**:三信号任一触发才动）
