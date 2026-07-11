# 结账与消化执行书(口令「agent-on 结账」/「agent-on 升级」)

> 职责边界:本篇是迭代闭环第③④⑥站的执行书——上半场「结账」由项目会话执行(回流),下半场「消化」由 agent-on 仓会话执行(落地),附「升级」独立口令。为什么这么设计见 [../playbook/iteration-loop.md](../playbook/iteration-loop.md)。
> 负担预算(硬约束):结账 = 一句口令 + 零确认;消化 = 一场会话 + 一组选择题;升级 = 一次 diff 批准。执行中发现超预算的步骤,记 loop-notes 并当场简化。

## 上半场:结账(在项目仓,听到「agent-on 结账」即执行)

0. **对表播报(不升级)**:读项目 `agent-on.lock.md` 的 pin 与 agent-on 的 `CHANGELOG.md`,一句话播报「当前 pin vX.Y.Z,落后 N 版,含 / 不含 major」。升级是另一个口令,此处只播报。
1. **收集出仓候选**(增量,以 lock 的 last_settlement 为锚;**锚为空 = 首次结账**——lock 该行留空、或倒仓/无 lock 的历史项目,都不做增量:全量扫描,收所有未标 `sync_status=synced` 的条目):
   - **主路径(S/M 档,绝大多数项目走这条)**:`loop-notes.md` 里上次回执之后新增的可复用散文条目——六类触发当场记的那些行。散文条目直接装配成 Promotion Card,不经 jsonl。
   - `agent-on.lock.md` 的 local_deviations 新增行(脚手架不合身信号)
   - **L 档旁路(仅多 agent 编排且已启用 run 台账时)**:项目 `ledger/runs/*.jsonl` 里 `suggested_location=agent_on` 且 `sync_status=pending` 的 memory_card。**注意:jsonl 采集链 + audit-lint 尚未在真实项目跑通过——S/M 档没有 run 台账,目录不存在是正常的,别困惑,走主路径即可。**
2. **证据硬门**:没有证据指针(commit / 命令输出 / run_id)的条目不予出仓——留在项目里,不算数。
3. **装配 Promotion Card**(模板 [../kit/promotion-card-template.md](../kit/promotion-card-template.md)):六项缺一拒收,另带 pattern slug + 本项目 pin 版本。
4. **落盘承接层**:全部卡写进 agent-on 仓 `intake/<YYYY-MM-DD>-<项目名>.md` **一个新文件**,在 agent-on 仓单独 commit(只含这一个文件,绝不碰其他)。
5. **回执**:项目侧 memory_card 标 `sync_status=synced`;lock 追加 last_settlement 行;项目仓 commit。
6. **积压播报 + 消化提示(默认动作,不止一句问句)**:数 `intake/` 未标去向的文件数并播报。然后**两个动作一起做**,别只问一句就完:
   - **生成可粘贴的消化开场**:直接吐一行让用户复制到新会话即用的命令,形如 `cd ~/Projects/Agent-On && 说「消化会话」`(或 `/agent-on digest`),并把当前积压数写进去,如「intake 积压 3 份待消化」。用户当场不消化也没关系——命令已备好,下次粘贴即走。
   - **写进项目待办位**:把「agent-on 待消化 N 张」写进项目 `loop-notes.md` 顶部固定待办位(没有就建一行)。session-handshake 的读取表已把它列为必读项——下次这个项目任意握手,都会把「有 N 张卡等着回 agent-on 消化」带出来,不靠人记得。
   - **顺口报成长**:一句「本次回流 N 条;agent-on 现累计 bench 案例 X 张、playbook Y 篇」——数字从 `ls | wc -l` 数出来,禁止手编。「我进步了」可见,是结账习惯活下去的燃料。

> **中断即安全**:结账幂等——intake 文件已落盘的卡就算数;重跑口令时已标 `synced` 的条目自动跳过,不产重复卡。从哪步断的,重念口令从头走一遍即可,已完成的步骤是空操作。

## 下半场:消化(必须换 agent-on 仓会话)

为什么必须换:会话上下文 = 装载的规则集,没读 agent-on 的 AGENTS.md 的会话,不许动它的 canonical(单写者不变量)。

1. **开场频次扫描**:grep `intake/` 全部未收口卡的 pattern slug,同 slug ≥2 个项目 → 置顶,强制升 L3。**同类多条散文条目 → 先合并抽象成一条 L2 再落地,不逐条搬运**(memory-layering:L1 现象与可复用 loop 分开拎)。
2. **三态分诊**([../bench/correction-loop.md](../bench/correction-loop.md)):
   - 低风险(措辞 / 错链 / bench 案例追加):AI 直落 canonical
   - 中风险(模板行 / checklist 行 / playbook 段落)与高风险(schema 必填 / BOOTSTRAP 语义 / 不变量):**打包成一组选择题**(每题 = 卡摘要 + 建议落点 + 采纳/拒绝/缓议),用户一次拍完
   - pin 旧版、新版已修复的摩擦:直接 `rejected(升级 pin 即解)`——这也是版本漂移探测器
3. **落地**:每张采纳卡 = 具体文件修改;L3 规则强制双落点(playbook 正文 + kit 模板或 checklist 行,别停在 playbook);卡在 intake 文件里**原地**标 `landed@<commit>` / `rejected(原因)` / `deferred`。
4. **收尾三件**(缺一 = 消化失败):
   - 至少一处具体文件改动(meta-principles 第三条:反思必须产出协议升级)
   - CHANGELOG 条目:动了什么、来自哪份 intake、L3 改动成对列双落点、semver 档位(用户确认:major=不动手会坏 / minor=不动手不坏 / patch=不用知道)
   - 该打 tag 就打 annotated tag;major 无迁移注记不许打

## 升级(独立口令「agent-on 升级」,在项目仓)

1. 读 CHANGELOG 自 pin 以来的区间
2. **patch / minor**:改 lock 的 pin 行即完成——存量实例化文件不动,新工件自然用新模板
3. **major**:按迁移注记对已实例化文件(看头部 `instantiated-from` 行定基准)出 diff 提案,用户逐条批准后执行
4. **永不从 kit 重拷覆盖实例化文件**——它们是项目自己的 canonical。
