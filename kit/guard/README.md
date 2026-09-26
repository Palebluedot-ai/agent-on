# kit/guard — 跨仓、worktree 与跨窗口边界（机械闸）

> 职责：①项目端会话对 agent-on **工作仓 B** 只写 `intake/`，禁止 add/commit/push；②在 Claude/Codex 发起 `git commit/push` 前判**本树**：本树某个未提交文件，另一棵工作树里也未提交、且那一份 7 天内被碰过，才拦（2026-09-24 起不读 lane 登记；见 [kit/worktree-control-plane.md](../worktree-control-plane.md)「闸只拦真冲突」）；③**跨窗口指令路由**——值守在班时，非值守窗口的合并 / 对外通信 / 横向消息拦下并给出转投模板（协议见 [kit/babysit/ROUTING.md](../babysit/ROUTING.md)，登记命令是 `agent-on oncall`，无人在班则整条闸 fail-open）。
> **实现**：逻辑在 Rust CLI 的 `agent-on guard`；本目录 extensionless 文件是 canonical Bash shim，`.sh` 仅为旧个人 hook 的 Bash/Python 双兼容入口。

## 路径 / doctor

```bash
agent-on doctor
agent-on doctor --cwd /path/to/project
```

B 解析序：`AGENT_ON_ROOT` → `~/.config/agent-on/config.json` → lock「本地路径」→ 默认 `~/.local/share/agent-on`。  
**未登记 B 时 guard fail-open**（不拦）。

## Hook 注册与触发成本

Claude（`hooks/hooks.json`）：

```json
{ "type": "command", "command": "bash \"${CLAUDE_PLUGIN_ROOT}/kit/guard/agent-on-git-guard\"" }
```

Codex plugin manifest 指向**同一份** `hooks/hooks.json`，不另养副本。guard 对非 git、git 读命令立即放行；只在 `commit/push` 前跑本树的边界判定（`gate_for`，与 Git hook 同一把尺子）。值守在班时，值守窗口每一次经 guard 的调用还会顺手续一次在班心跳（见 [kit/babysit/ROUTING.md](../babysit/ROUTING.md) §4）。

先保证：

```bash
cargo build --release --manifest-path cli/Cargo.toml
# 或
cargo install --path cli --force
```

## 最小实测

```bash
# 跨仓写 → 2（需已登记 B = 本仓）
export AGENT_ON_ROOT=/path/to/agent-on
echo '{"tool_input":{"command":"git -C '"$AGENT_ON_ROOT"' commit -m x"},"cwd":"/tmp"}' \
  | CLAUDE_PROJECT_DIR=/tmp agent-on guard; echo "expect 2"

# 自会话 → 0
echo '{"tool_input":{"command":"git commit -m x"},"cwd":"'"$AGENT_ON_ROOT"'"}' \
  | CLAUDE_PROJECT_DIR="$AGENT_ON_ROOT" agent-on guard; echo "expect 0"

# 读操作 → 0
echo '{"tool_input":{"command":"git -C '"$AGENT_ON_ROOT"' status"},"cwd":"/tmp"}' \
  | CLAUDE_PROJECT_DIR=/tmp agent-on guard; echo "expect 0"

# 当前 repo 的 commit/push → 只判本树
echo '{"tool_input":{"command":"git commit -m probe"},"cwd":"'"$PWD"'"}' \
  | CLAUDE_PROJECT_DIR="$PWD" agent-on guard; echo "expect 0, or 2 only if another worktree has the same uncommitted file touched within 7 days"
```

若 stderr 含 `blocked:`，那一行写了路径和另一棵树。把该文件在其中一棵树里提交或还原即可；超过 7 天没人碰的那一份不会拦。`error:` 是这棵树的审计没跑起来，先修检查器，不以跳过 hook 当修复。

```bash
# 跨窗口路由：值守在班 + 功能窗口发合并命令 → 2
agent-on oncall claim --session babysit-window-a --cwd "$ONCALL_WORKTREE"
echo '{"tool_name":"Bash","cwd":"'"$FEATURE_WORKTREE"'","tool_input":{"command":"gh pr merge 17 --merge"}}' \
  | agent-on guard; echo "expect 2 + 转投模板"

# 同一条命令在值守窗口 → 0；`agent-on oncall release` 之后任何窗口 → 0
```

若 stderr 含 `跨窗口指令路由拦截`，**别找等价命令偷跑**：按提示三选一（转投 / 让值守下班 / 本窗口接班），后两条都会改在班登记因而留痕。文案里还有一行「值守最近心跳 N 分钟前；M 分钟没心跳自动失效」——值守窗口已经关掉时，等它过期即可，不必 `release --force`。

### v0.6 Codex 旧注册

`python3 .../agent-on-git-guard.sh` 曾因 v0.7 把脚本换成 Bash 而产生 `SyntaxError`。当前 `.sh` 兼容入口已同时支持 `python3` 与 `bash`，但长期建议删除个人重复 hook、使用 plugin；Agent-On 状态检查只提醒，不擅自改 `~/.codex/hooks.json`。

回滚：从 hooks 删掉 PreToolUse 条目即可。

## 执行面自检：在跑的是不是这一份（2026-09-26）

规则在仓里改了，宿主上挂着的 hook 不会跟着换。同一台机器可能**并排挂着两代闸**，每条 Bash 各判一遍，谁拦算谁（实测：插件缓存停在 0.5.0，它的 hook 跑的是老 Python 闸，按命令文本判越界；`~/.claude/settings.json` 里那条转发到仓里的 Rust 闸，早已按目标仓判——误拦全出自前者，bench 案 46 / 47）。

发版、升级 pin、或者被拦得莫名其妙时，对一遍执行面：

```bash
# 1. 一条命令核完（只读，~/.claude 一个字节不写）
agent-on doctor        # 看「hook 执行面」一段：结论行、STALE / GUARD OFF、修法

# 2. 落后就换掉：先重编，再刷插件缓存（重启 Claude 生效；改 ~/.claude 属于用户动作）
cargo build --release --manifest-path <WRITE_ROOT>/cli/Cargo.toml
claude plugin update agent-on@agent-on
```

`doctor` 的「hook 执行面」逐条列 `~/.claude/settings.json` 与已启用插件（`installed_plugins.json` 的 `installPath`）`hooks.json` 里的 agent-on 条目，核四层：①插件版本对 READ_ROOT 的 `.claude-plugin/plugin.json`；②`hooks.json` 对 READ_ROOT 的 `hooks/hooks.json`；③每个被执行的脚本按字节对 READ_ROOT 同名文件；④脚本与仓里一致时，按 shim 自己的转发顺序（插件目录里编好的 `cli/target/release/agent-on` → 脚本所在仓编好的那份 → PATH 上的 `agent-on`）找到真正执行的二进制，比它的编译时间与 READ_ROOT 最近一次 `cli/src` 提交。落后报 `STALE`，shim 找不到二进制（fail-open，闸不生效）报 `GUARD OFF`，名字带 agent-on、却追不到任何 agent-on 树的脚本（多半是早先拷出来的老闸）报 `UNTRACED`；软链先解析到真身再判。

**第④层是 2026-09-26 实测补的**：脚本哈希全一致时只核脚本会报「一致」，可目录型 marketplace 装插件会把 `cli/target/` 一起拷进缓存，shim 又优先跑插件目录里那份——本机两份在跑的二进制都编于 09-24，v0.23.0 起的闸修复一个都不在执行面上。所以要先重编、再 `plugin update`，顺序反了缓存里还是旧的。

判据：插件版本对不上仓里的 `plugin.json`，或者 hook 命令指向一份不经 `agent-on-git-guard` shim 转发的脚本，或者 shim 最终跑的二进制早于仓里最近一次 `cli/src` 提交，就是执行面陈旧。拦截文案里点名的执行体路径是第一线索——指向缓存目录时先查这里，别先改判据，也别教被拦的会话改写命令。

## 分类器/闸拒诊断（命令字面 ≠ 目标）

> 源流:Dartify 2026-08-08——`reset --hard` 拒、同命令间歇拒、带管道拒;换 `ff-only` / 去管道 / 重试即过。

安全闸(PreToolUse / auto classifier)拒绝的是**这条命令字符串**,不是用户目标,且可能间歇误拒。

1. 换**更保守**的等价手段(`reset --hard` → `merge --ff-only`、管道 → 裸跑、整目录 → 显式路径)  
2. **原样重试一次**  
3. 两步不过 → 向用户说明意图并请求授权  

**禁止**:一次拒绝就缩减交付范围或改口「做不到」;也禁止为绕闸升级破坏性。见 anti-hallucination 第六型#17。
