# Commit 分层约定

> 职责边界:本文件规定「一个 commit 怎么切、带什么前缀」,让 git log 一眼可读、每个 commit 可独立回滚(bisectable)。只管提交粒度与前缀,不管分支策略与合流流程(那在 merge-checklist.md)。
> 源流:一代 agent-orchestration-playbook/05_playbooks/commit-layering-convention-v1.md,2026-07-07 批二移植改成通用项目适用。

## 一条铁律:一个 commit 只做一件事

一次提交只承载一个可独立描述、可独立回滚的变更。跨层的改动拆成多个 commit(如先 `decision:` 再 `feat:`),不塞进一个。

**commit 前读全暂存区**:`git status --short`(或等价)确认 staged 集合 = 本 commit 意图;勿假定 `git add <路径>` 已限定范围——它只增不减。见 anti-hallucination 第六型第 4 条。

## 分层前缀(必带)

| 前缀 | 装什么 |
|---|---|
| `decision:` | 拍板、ADR、方向变更(思想层) |
| `docs:` | 文档、说明、注释(不改行为) |
| `feat:` | 新功能/新能力(生产代码) |
| `fix:` | 修 bug(生产代码) |
| `refactor:` | 重构,行为不变 |
| `chore:` | 杂务:依赖、配置、目录调整 |
| `test:` | 加/改测试,不动生产代码 |

## 环节收口 = 一个 commit(规划链与口令动作)

规划链每个环节(MRD / 澄清包 / PRD / plan / 审查改写 / phase 卡)与口令动作(整理想法 / 更新仪表盘 / 结账)收口即 commit,不许攒:规划文档用 `decision:`(拍板类)或 `docs:`(记录类)前缀,中文语义化 message。**commit 时间线就是用户的回退时间线**——不懂 git 的用户靠语义化 message 找回任何一步(BOOTSTRAP §4 L8)。

## 核心原则:思想变更与自动化变更绝不混一个 commit

「决定要做什么」(decision/docs)和「把它实现出来」(feat/fix/refactor/test)分属两次提交。原因:回滚一个错误决策时,不该被迫连带回滚一堆无辜的自动化代码;反之亦然。跨层顺序通常是 `decision:` → `feat:`/`test:` → `docs:`。

## bisectable:每个 commit 可独立回滚

- 每个 commit 落地后仓库应处于一致状态(能构建、测试不比上一个 commit 更红)。
- 不留「半截活」commit(改了调用方没改被调方),否则 `git bisect` 会在中间态误判。
- 这样出问题时能用 `git bisect` 二分定位到**那一个**引入问题的 commit。

### 边界:并行卡在共享文件交织时合一笔

「原子提交保 bisect」的前提是**每笔自洽通过**(编译/测试绿)。并行卡的改动若在共享文件里交织(同一屏既挂玻璃头又加手势触觉),硬拆成 N 笔会留下中间态红——此时**合为一笔** bisectable commit,message 里列卡内明细。做不到自洽的拆分不如一笔诚实。(Euan 实证 3189167:玻璃/微交互/触觉三卡交织,合笔+message 明细;文件域互斥的卡仍逐卡原子提交。)

## 示例

```
decision: 采用契约先行,接口两侧并行前先冻 fixtures
feat: 实现 task-card 校验器
test: 补 task-card 非法枚举的失败用例
docs: 记录 commit 分层约定
chore: 升级 lint 依赖到 v2
```
