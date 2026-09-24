# commit 闸只看同一份未提交文件（2026-09-24）

> 职责边界：本页记这次改了哪条判定、为什么。机制正文在 [kit/worktree-control-plane.md](../kit/worktree-control-plane.md)「闸只拦真冲突」一节。
> 触发：用户原话——lane 经常锁死、不能 commit；希望优化，不要特别多的限制。对照是 Superpowers：模型自己会做的手续，不必再办一遍。
> 拍板：commit / push 不读 lane 登记。

## 判定

本 worktree 的某个未提交文件（staged / unstaged / untracked），在另一棵 worktree 里也是未提交的，而且那一份的 mtime 在 `dormant_after_days`（默认 7）以内。人读一行：`blocked: <路径> is also uncommitted in <另一棵树>`。

没撞上：hook 不说话，`worktree status` 和 `worktree check` 打 `ok`。`check` 非零只来自本树的 `blocked`，或本树审计跑不起来。

删掉的文件没有 mtime，不拿来挡别人。`dormant_after_days: 0` 表示同一路径一律算新鲜。

merge / rebase / cherry-pick 进行中照旧放行。

## 留下的

`claim` / `edit` 仍可用 owns 记账，重叠时仍拒绝。那道门只有有人去跑 `claim` 才碰到，不挡 commit。旧的 lane JSON 留在原地。

## 拿掉的

用 `owns` 预留还没写的目录。那是 Orbit 一条 14 天前的 `active` 占住整个 `src/` 的原因。两个会话真在写同一个文件时，脏文件自己会撞上。
