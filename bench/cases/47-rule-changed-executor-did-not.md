# 案例 47：规则改了，执行面没换 —— 修好的闸在仓里，拦人的闸在缓存里

> 层级：L2 | 来源：Dartify 2026-09-26 结账（executing-hook-drifts-from-pin）+ 2026-09-26 消化会话干跑对照 | 入册：2026-09-26

## 症状

- Dartify 结账会话一条**只读**盘点命令（对 Agent-On 仓跑 `git log --all` / `git stash list` / `git branch -a`）被 PreToolUse 拦下。拦截文案点名了执行体：`python3 "/Users/chao/.claude/plugins/cache/agent-on/agent-on/0.5.0/kit/guard/agent-on-git-guard.sh"`，注明来自 `agent-on@inline plugin`。
- CryptoQuant（09-06）和 aster-agent（09-14）报的「跨仓闸按命令文本判越界」（案 46），拦截文案点名的也是同一份缓存脚本。
- 同一台机器上，`~/.claude/settings.json` 的 PreToolUse(Bash) 挂着另一条：`bash "$HOME/Projects/Agent-On/kit/guard/agent-on-git-guard"`，转发到仓里编出来的 Rust 闸。**两代闸并排挂着，每条 Bash 各判一遍，谁拦算谁。**
- `agent-on doctor` 只报 read_root / work_root / config / env，对这份还在执行的缓存 hook 只字未提。

## 根因

**hook 挂的是某个路径上的某份副本，不是「这个规则」。** 插件装在 0.5.0（`claude plugin list`：Version 0.5.0，enabled），仓里的 `.claude-plugin/plugin.json` 已是 0.12.1。缓存里那份是 7 月的 Python 实现（5764 字节），会把命令文本里出现的所有 agent-on 路径都算成写入目标；仓里的同名文件早已换成 430 字节的转发壳，判据在 Rust 里按目标仓判。

2026-09-26 干跑对照（cwd 为临时项目仓）：commit message 引了 Agent-On 路径、`cp` 上游模板后在项目仓 commit，老闸 exit 2、Rust 闸 exit 0；真写 Agent-On 的两条对照，两边都是 2。**修复早就在仓里，误拦还在发生**——因为执行面没换，而且没有任何一步会去核它：升级只改 lock 的 pin，doctor 不看 hook，发版不看宿主。

连锁后果：09-21 那场消化读到案 46 的症状，结论写成「闸体修复 deferred，须单独一轨」——把一个已经修好的问题当成待修，差一点又开一条轨去修它。

## 修法

- **本机**：`claude plugin update agent-on@agent-on`（重启生效）。新版插件的 hook 经 `kit/guard/agent-on-git-guard` 转发到 `agent-on guard`。**但「缓存里不再留一份会过期的判据」只说对了脚本那一层**（2026-09-26 `agent-on doctor` 实测改正）：shim 优先跑插件目录里编好的 `cli/target/release/agent-on`，而目录型 marketplace 会把 `cli/target/` 一起拷进缓存——本机缓存里那份编于 09-24，v0.23.0 起的闸修复都不在执行面上，主树里那份也一样旧。所以顺序是先在工作仓重编，再 `plugin update`。
- **流程**：升级节加「升级后核执行面」（boot/settlement.md 升级第 5 步）；kit/guard/README「执行面自检」给出三条核对命令。
- **工具**：`agent-on doctor` 的「hook 执行面」段列出 settings.json 与已启用插件 hooks.json 里的 agent-on 条目，比版本、比 hooks.json 与脚本字节，再追到 shim 最终跑的二进制比编译时间；落后报 `STALE`，找不到二进制报 `GUARD OFF`（`cli/src/doctor.rs`，<!-- src: cli/src/doctor.rs#hook_surface -->）。

## 可复用规则

**规则改了，不等于执行面换了；执行面也不跟着 pin 走。** 三条：

1. **被拦先看执行体**：拦截文案点名的脚本路径是第一线索。路径在缓存目录、版本对不上仓里那份 → 先查执行面，别先改判据，也别教被拦的人改写命令。
2. **发版 / 升级 / 自检都对一遍宿主上实际挂着的 hook**：路径 + 内容哈希，和 READ_ROOT 里的同名文件比；缓存落后或执行面领先 pin，都报出来。
3. **分发不留会过期的逻辑**：hook 只转发到一个随版本走的入口（PATH 上的二进制），缓存里只放转发壳。

与 `playbook/sop.md`「『没生效』先查交付链」（案 32）是一对反面：那条是新东西没送到，这条是**旧规则在本该失效的地方还在生效**。

## 已固化到哪

- `playbook/multi-contributor-protocol.md` §三½.5 第 6 面「执行面」
- `kit/guard/README.md`「执行面自检」
- `boot/settlement.md` 升级第 5 步
- `playbook/anti-hallucination.md` 第六型 #17 末句「被拦先读执行体路径」
- `cli/src/doctor.rs`：`agent-on doctor`「hook 执行面」段（只读；shim 转发顺序由测试钉住，<!-- src: cli/src/doctor.rs#shim_forwarding_order_matches_what_doctor_assumes -->）
- 相关：案 46（判据按文本）、案 40（出口可达性）
