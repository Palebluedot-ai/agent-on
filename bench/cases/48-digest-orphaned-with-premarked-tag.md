# 案例 48：消化做了一半没提交，卡上先写了版本号 —— 号被别人用掉，账就成了假的

> 层级：L2 | 来源：Dartify 2026-09-26 结账两卡（landed-mark-predicts-unminted-tag / digest-session-must-open-in-main-tree）+ 2026-09-26 消化会话现场核验 | 入册：2026-09-26

## 症状

2026-09-26 开消化时，Agent-On 主树的状态：

- 4 份未跟踪的 intake（dartify 08-19 / inbox-radar 09-05 / CryptoQuant 09-06 / aster-agent 09-14，共 25 卡），23 张卡原地标着 `landed@v0.22.0`。`git log --all -- <这 4 个文件>` 零命中——它们从没进过任何 ref。
- 标注指向的落点：12 个 canonical 文件 +76/−5、一张新案例，**全部未提交**，CHANGELOG 没记。`.workbuddy/memory/2026-09-21.md` 旁证这场消化做在 09-21 前后。
- `v0.22.0` 实际钉在 `d83fd19`（09-24 的 commit 闸改动）上。这个 commit **顺带发出了其中两处落点**（BOOTSTRAP §1 第四问、worktree-control-plane「陈年树三条」），CHANGELOG 的 v0.22.0 节只写了闸改动；其余 21 处继续留在工作区。23 个 `landed@v0.22.0` 里，21 个是假账，2 个碰巧成真但没人知道。
- 同一天，桌面宿主把用户的「agent-on 消化」开进了自建的 linked worktree。这棵树从 origin/main 切出来，未跟踪的 intake 一份都看不见，工作区一片干净——按执行书的三检，结论会是「没有积压」。而主树那 12 个文件两天前被碰过，同文件闸会拦下这棵树的每一次提交。

## 根因

三个缺口叠在一起：

1. **去向标注预写了还不存在的版本号。** 执行书要求「去向标注与落地改动同 commit」，但没禁止先写版本号、后提交。标注一旦脱离 commit，就只是一个对未来的猜测；号被别的提交用掉，猜测就成了假账，而且看起来和真账一模一样。
2. **发版提交没核 diff 的来历。** v0.22.0 那一场在同一个主树里改 BOOTSTRAP 和 worktree-control-plane，把另一场消化留在这两个文件里的改动一起提交了；其余文件不在它的改动范围里，原样留下。「这个 hunk 是不是本批的」没人问。
3. **消化会话开在哪，执行书不查。** 三检查本地 = origin、worktree 列表、工作区干净，唯独不查「本会话是不是主树」。linked worktree 里三检全过，恰恰是最危险的一种通过。

## 修法

- **收编而不是重写**：本场把主树的材料原样拷进 worktree、逐字节比对，请用户在主树 `stash push -u` 清场（可逆），然后按主题分 commit 落地，23 个标注逐卡改成真实落点（两处写 `landed@d83fd19`，其余写本批 commit），CHANGELOG 补记 v0.22.0 漏记的两处。收编时还查出草稿里的四个问题（把没实现的 pre-push 闸写成已存在、映射表声称的 kit 落点不存在、案 46 误诊、一条已过时的实测），一并修掉——**别人的草稿要当草稿核，不当成品搬**。
- **规则**：settlement 下半场第 0 步三检改四检（第四检 = 主树自证，两条出口）；第 3 步禁止预写版本号；第 4 步提交与发版前核 diff 的来历。
- **机械闸**：`agent-on tag-release` 拒绝 intake 里指向未打 tag 的 `landed@vX.Y.Z`。

## 可复用规则

1. **去向标注只能引用已经存在、并且包含落点的 commit 或 tag**。写不出 hash 就写 `landed@同批（落点）`，让落地 commit 一起带走。
2. **提交前问一句：这个 hunk 是不是本批的？** 工作区里有来历不明的未提交改动，先问清归属再动——别挪开它发版再挪回来，也别顺手一起提交。
3. **自检全过时，先问自检是在哪里跑的。** 在一棵看不见队列的树里，「没有积压」是测量位置的问题，不是事实。

## 已固化到哪

- `boot/settlement.md` 下半场第 0 步（四检 + 两条出口）、第 3 步（禁止预写版本号）、第 4 步（核 diff 来历；`--push` 在 worktree 分支上不推 main）、上半场第 6 步（开场命令写明主目录）
- `playbook/iteration-loop.md` §四消化链、`skill/SKILL.md` digest 行、`intake/README.md` 第 3 条、`kit/promotion-card-template.md` 状态注释
- `cli/src/tag_release.rs`：未打 tag 的 `landed@vX.Y.Z` 拒绝发版
- 相关：案 40（出口可达性——第四检只写「停下」就是死锁）、案 46/47（本批收编时顺带改判的根因）
