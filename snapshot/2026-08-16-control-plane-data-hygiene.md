# 2026-08-16 控制面数据卫生审计（octal-escaped owns + 归属重叠）

轨道:`agent-on-data-hygiene`(worktree `agent-on-data-hygiene-0d76ed`)。
范围:只动 agent-on 控制面元数据(`.git/agent-on/lanes/*.json`)与本审计快照;项目侧仓库零改动。

## 结论(TL;DR)

- **问题 1(octal-escaped owns)已不存在**:全控制面(Agent-On + Dartify 两个 lane registry,机器上仅此两个)扫描 `\nnn` 序列为零。d35 组与 `legacy-crm-agent-nlp` 的 `owns` 中 CJK 路径(`design/incoming/客迹-inventory.md` 等)均为真 UTF-8,且与磁盘真实文件逐一对得上。
- **问题 2(babysit-state-sync × ag1-nav 重叠)已无未决冲突**:babysit 工作区**净**(`git status --porcelain` 空),不存在"脏文件";其 changed 2(`dashboard.html`、`docs/state/progress.yaml`)是已经 squash 并入 main 的 #158 内容在三点 diff 里的残影,且完全落在它自己的 owns 内。check 无 OVERLAP、无 OUT-OF-BOUNDS。
- **双仓 `agent-on worktree check` 均 PASS(exit 0)**。Agent-On 仓此前唯一 FAIL 因是本 worktree 未登记,已按 placeholder 指示重划 claim 修复。

## 根因与来龙去脉

1. **octal 来源**:git 在非 `-z` 输出(`status`/`diff --name-only` 默认 `core.quotepath=true`)会把非 ASCII 文件名转成 `"\345\256\242..."` 八进制转义。早期登记 parked 轨时若把这种输出直接粘进 `--owns`,就永远匹配不上真实文件。
2. **已有的系统性防御(读取侧)**:CLI 的 `changed_files`/`nul_paths`(cli/src/worktree.rs)全部走 `git ... --name-only -z`,NUL 分隔、无引号无转义 —— 检查侧不可能再产生 octal 路径。本次实测 d35 三条 CJK changed 路径以真 UTF-8 返回并命中 owns(out_of_bounds 全空)。
3. **数据侧修复发生于 2026-08-15T23:30Z 的重登记**:现存 parked 轨(d35-grouping-codex、d35-pr117-legacy、legacy-crm-agent-nlp、babysit-state-sync、t47-branch-protection)created_at 均为该时刻,owns 已是解码后的规范 UTF-8。本次审计为核验 + 收尾,未再需要改写任何 owns。

## before / after

| 轨 | before(goal 描述的缺陷态) | after(2026-08-16 实测) |
|---|---|---|
| d35-grouping-codex / d35-pr117-legacy / legacy-crm-agent-nlp | owns 含 `\345\256\242...` 八进制串,永不匹配 | owns 为真 UTF-8(`design/demo-v9/客迹 v9.dc.html`、`design/incoming/客迹-inventory.md`、`design/incoming/客迹-客户屏-状态清单.md`),磁盘文件存在,changed(97/89/89)全部命中,out_of_bounds=[] ,状态保持 parked |
| babysit-state-sync | `dashboard.html`+`docs/state/progress.yaml` 脏文件与 ag1-nav owns 相交 | 工作区净;两文件为 #158 squash 已并入内容;owns 覆盖自身 changed;parked 语义下 ownership 不活,check 无冲突报告;状态保持 parked |
| ag1-nav | 与 babysit 冲突方 | active,changed 3,out_of_bounds=[];对状态面四件的 owns 为唯一活跃声明 |
| (Agent-On)本 worktree | UNREGISTERED → 全仓 FAIL | 按 `data-hygiene-placeholder` 记录内指示移除占位、以真实边界 claim 为 `agent-on-data-hygiene`(base main,owns 仅本快照);check PASS |

## 重叠的语义裁定(为什么不删 babysit 的 owns)

- 系统语义:overlap 只在 `ownership_live`(active/blocked/ready)轨之间判定;parked 轨的 owns 是"复活时重划"的占位声明(其 goal 字段明示),对 claim 与 check 均不构成阻塞。
- 若强行从 babysit 删掉那两条 owns,其 changed 2 立刻变 OUT-OF-BOUNDS,check 反而转 FAIL —— parked 轨的 owns 必须继续覆盖自身对 base 的 changed 集。现状即是数据真实态:活跃归属唯一(ag1-nav),parked 占位自洽。

## 残余风险与移交

- **摄入口未设防**:`normalize_owns`(claim `--owns` 入口)不做 octal 反转义 —— 未来若再有人把带引号/八进制的 git 输出粘进 `--owns`,同类缺陷可复发。通用解法:在 `normalize_owns` 里加 git-octal-unescape(剥外层引号 + `\nnn` 序列按字节解码为 UTF-8,配拒绝非法序列的测试)。
- `cli` 归 active 轨 `landing-control-plane-v1` 所有,本轨不越界改代码;此项已作为跟进任务移交(见会话任务卡)。
