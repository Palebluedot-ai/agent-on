# kit/guard — 跨仓、worktree 与跨窗口边界（机械闸）

> 职责边界：PreToolUse 核跨仓 Git 写入与值守路由授权；同文件保护由 shared Git hooks 在实际 index / 推送范围确定后执行，见 [控制面](../worktree-control-plane.md)。项目端对 Agent-On B 只写 intake、不执行 Git 写入；值守在班时合并/对外通信/横向消息归在班窗口。
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

Codex plugin manifest 指向同一份 hooks/hooks.json。PreToolUse 不重复扫描脏树，只核跨仓和路由授权；同文件判据在 shared Git hooks 执行。未安装原生 hooks 时不能宣称提交保护已启用。值守调用继续续心跳。

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

# 当前 repo 的 commit/push → 路由/跨仓检查，操作范围交给 Git hooks
echo '{"tool_input":{"command":"git commit -m probe"},"cwd":"'"$PWD"'"}' \
  | CLAUDE_PROJECT_DIR="$PWD" agent-on guard; echo "expect 0 unless cross-repo/oncall authorization blocks"
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

doctor 按插件版本、hook 配置、脚本字节与 shim 的真实二进制转发路径逐层核对。新二进制提供 build-info --json：编入的源码内容标识、commit、版本和可核的 release tag。doctor 比较实际二进制与 READ_ROOT/cli 的内容标识；相同源码可在不同路径/时间编译，版本号相同也不能掩盖源码不同。

旧二进制没有构建身份时标 UNVERIFIED，mtime 只用来提示明显陈旧，不能证明包含某次修复。源内容不同标 STALE；shim 无二进制标 GUARD OFF。doctor 只给出实际结果和修法，不修改全局 hook/插件缓存。

doctor 还读取当前仓的受管 Git hook 安装回执，核对它实际调用的 executor。`worktree hooks status` 健康只说明配置与脚本未漂移；已安装时再跑 `hooks install` 不会替换原 executor。按 doctor 显示的实际路径重建/升级：指向 `~/.cargo/bin/agent-on` 时用 `cargo install --path cli`，指向仓内 release 时用 release 构建；插件缓存还须更新与重启。未更新的执行面在升级回执中单列。

## 分类器/闸拒诊断（命令字面 ≠ 目标）

> 源流:Dartify 2026-08-08——`reset --hard` 拒、同命令间歇拒、带管道拒;换 `ff-only` / 去管道 / 重试即过。

安全闸(PreToolUse / auto classifier)拒绝的是**这条命令字符串**,不是用户目标,且可能间歇误拒。

1. 换**更保守**的等价手段(`reset --hard` → `merge --ff-only`、管道 → 裸跑、整目录 → 显式路径)  
2. **原样重试一次**  
3. 两步不过 → 向用户说明意图并请求授权  

**禁止**:一次拒绝就缩减交付范围或改口「做不到」;也禁止为绕闸升级破坏性。见 anti-hallucination 第六型#17。
