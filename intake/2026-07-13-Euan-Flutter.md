# intake — 2026-07-13 · Euan-Flutter(worktree 何时开 + 防撞车)

> 来源:用户在 Euan-Flutter 后端会话里问「以后什么时候开 worktree,怎么防止 worktree 撞车」。当日活标本:同仓两个 worktree(backend-prep 与 design-ui-integration)并存,两条线都可能动 `design/` 目录。

### worktree-when-and-collision-guard(何时开 worktree + 三条防撞车铁律)
- source:Euan-Flutter @ 2046d19 | pin v0.3.0
- evidence:①memory euan-flutter-scaffold 教训段——曾误开 fork 会话在 `.claude/worktrees/` 与主会话**并行改同一目录**,撞乱 git+文件布局,**整套脚手架返工重建**;修复=写进 AGENTS.md §9 + CLAUDE.md「单会话操作」条 + `.claude/worktrees/` 入 `.gitignore` ②当日 `git worktree list` 实录:`backend-prep-workflow-ee62d7`(本会话,动 design/design-token-brief.md @2046d19)与 `charming-proskuriakova-2d3bec`[claude/design-ui-integration-2611f2] 两 worktree 并存——两者文件域都覆盖 `design/`,是撞车的**前置条件活体**;本次靠「只 append 一个新文件、不碰对方在编文件」规避 ③决策锚(同 memory):单人阶段不用 git worktree(单一 progress.yaml 单写者,YAGNI)
- confidence:medium(单项目,但有实际返工的强证据 + 当日活标本双证)
- claim:**何时开**——单轨/单人不开 worktree(YAGNI);仅当 ≥2 轨真正并行 **且** 文件域可切不重叠时才开,一轨一 worktree。**防撞车三条**——① 文件域不重叠(尤其 `design/`、`contracts/fixtures/`、`docs/state/progress.yaml` 这三个共享面必须归口单一轨,别两轨同碰)② progress.yaml 单写手(功能 PR 永不碰,maintainer 合并后单独 chore(state) 同步)③ 物理隔离兜底:worktree 目录入 `.gitignore` + 跨轨回流走 append-only 新文件(=intake 规则1同构:「不动他人文件,多轨在 git 层物理不撞」)
- suggested_landing:playbook/multi-contributor-protocol.md(已含 worktree 提及,补「开 worktree 的判据 + 三共享面归口清单」);或 kit/merge-checklist.md 加一行「worktree 文件域预检:本轨改动是否触碰 design/ · contracts/ · progress.yaml 之外轨已占用的面」
- rollback:revert 落地 commit
- trace:用户原话「以后什么时候开worktree,怎么防止worktree撞车 这个也intake下 到agent-on」+ 本会话 `git worktree list` / `git branch -vv` 实录 + memory euan-flutter-scaffold 教训段
- 状态:landed@同批(第八次消化:multi-contributor-protocol §二.2 补「何时开+三共享面归口」段 + merge-checklist 第 1 步合流前文件域对照)
