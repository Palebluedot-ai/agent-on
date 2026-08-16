# 实战回报:worktree guard 在 Dartify 首次生效的两个发现(2026-08-16)

> 来源:Dartify AG1 会话(项目端);按跨仓规矩只落 intake 素材,不动 canonical。
> 现场:AG1 五 PR 全部合并交付后,收官 chore(state) commit 被 guard 拦下,当场取证。

## 发现 1(bug):`worktree claim --owns` 不按逗号分列表

- 现象:`--owns "a,b,c"` 整串被存成 lane 记录里**单个** glob(`.git/agent-on/lanes/<id>.json`
  的 owns 数组只有一个含逗号长串元素),于是所有 changed 文件全部判 OUT-OF-BOUNDS——
  包括逐字出现在串里的精确路径。
- 复现:agent-on 0.12.1;claim 输出的 `owns:` 行原样回显长串,看不出没分割。
- 正确姿势(试出来的):重复 `--owns` 传多值;但 claim 不允许对已存在 lane 重跑
  (`lane already exists; change its state instead`),而 set-status 只改生命周期不改 owns
  ⇒ **owns 写错一次就没有 CLI 层的改错路径**,只能手改 lane JSON。
- 建议:①claim 对逗号串报错或自动分割 ②给个 `worktree edit --owns` 或 claim `--force`。

## 发现 2(策略疑问):guard 对「全仓任意未注册 worktree」连坐

- 现象:本 lane 注册齐、边界干净(OUT-OF-BOUNDS 清零)后,guard 仍 RESULT: FAIL——
  因为同仓另外 7 个 worktree(其它会话的:值守轨/Codex D35 轨/若干陈年遗留)未注册。
  任何一条会话想 commit,都要求**全部**会话先注册。
- 后果:存量仓首次启用 guard = 全员死锁;单会话无法自救(替活跃会话编 lane 边界会
  反过来拦死它们;替陈年 worktree 编 lane 属越界)。
- 待拍板:①guard 是否应只强制**当前 worktree** 的注册与边界,未注册的**其它** worktree
  降为 warning?②或给一条显式的「控制轨初始化」流程(一次性把存量 worktree 批量登记/
  park),存量仓开闸前先跑?③「不要用跳过 hook」与「fail-open 当二进制缺失」并存的
  口径下,连坐 FAIL 的逃生门是什么?

## 顺带:接入面

- 拦截时 AG1 主线五 PR 已全部合并(含生产部署),被拦的只是收官 chore(state)——
  merge 后 30 分钟记账纪律因此破例,48h 棘轮未破(合并均在当日)。
