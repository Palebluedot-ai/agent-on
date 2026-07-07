# 结账与消化执行书(口令「agent-on 结账」/「agent-on 升级」)

> 职责边界:本篇是迭代闭环第③④⑥站的执行书——上半场「结账」由项目会话执行(回流),下半场「消化」由 agent-on 仓会话执行(落地),附「升级」独立口令。为什么这么设计见 [../playbook/iteration-loop.md](../playbook/iteration-loop.md)。
> 负担预算(硬约束):结账 = 一句口令 + 零确认;消化 = 一场会话 + 一组选择题;升级 = 一次 diff 批准。执行中发现超预算的步骤,记 loop-notes 并当场简化。

## 上半场:结账(在项目仓,听到「agent-on 结账」即执行)

0. **对表播报(不升级)**:读项目 `agent-on.lock.md` 的 pin 与 agent-on 的 `CHANGELOG.md`,一句话播报「当前 pin vX.Y.Z,落后 N 版,含 / 不含 major」。升级是另一个口令,此处只播报。
1. **收集出仓候选**(增量,以 lock 的 last_settlement 为锚):
   - 项目 `ledger/runs/*.jsonl` 里 `suggested_location=agent_on` 且 `sync_status=pending` 的 memory_card(**S 轻装档没有 run 台账——跳过本项,不要因目录不存在而困惑**)
   - `loop-notes.md` 里上次回执之后新增的可复用条目
   - `agent-on.lock.md` 的 local_deviations 新增行(脚手架不合身信号)
2. **证据硬门**:没有证据指针(commit / 命令输出 / run_id)的条目不予出仓——留在项目里,不算数。
3. **装配 Promotion Card**(模板 [../kit/promotion-card-template.md](../kit/promotion-card-template.md)):六项缺一拒收,另带 pattern slug + 本项目 pin 版本。
4. **落盘承接层**:全部卡写进 agent-on 仓 `intake/<YYYY-MM-DD>-<项目名>.md` **一个新文件**,在 agent-on 仓单独 commit(只含这一个文件,绝不碰其他)。
5. **回执**:项目侧 memory_card 标 `sync_status=synced`;lock 追加 last_settlement 行;项目仓 commit。
6. **积压播报 + 消化提示**:数 `intake/` 未标去向的文件数并播报;≥3 份或发现跨项目同 pattern → 建议:「现在开一个 agent-on 仓会话消化吗?」——这是全流程唯一的问句。

## 下半场:消化(必须换 agent-on 仓会话)

为什么必须换:会话上下文 = 装载的规则集,没读 agent-on 的 AGENTS.md 的会话,不许动它的 canonical(单写者不变量)。

1. **开场频次扫描**:grep `intake/` 全部未收口卡的 pattern slug,同 slug ≥2 个项目 → 置顶,强制升 L3。
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
