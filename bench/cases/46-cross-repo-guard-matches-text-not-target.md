# 案例 46：跨仓守卫按「命令文本里有路径」判越界 = 误拦只读引用

> 层级：L2 | 来源：CryptoQuant 2026-09-06 首次结账（同项目三次复现）+ aster-agent 2026-09-14 首次结账（一次确定性复现）| 入册：2026-09-21 草稿，2026-09-26 消化收编时改判根因

## 症状

项目仓会话被「跨仓 git 边界拦截」拦下，而它**对 agent-on 仓一个 git 动作都没有**：

1. 在项目仓提交，commit message 里引用了 `BOOTSTRAP.md` 的绝对路径 → 被拦，改写 message 去掉路径字面后才落地。
2. 在结账会话里跑**只读**的 `git rev-list --count v0.19.0..HEAD`（cwd 在项目仓）→ 要先自查命令文本里不含写动词，才敢跑。
3. 用 Bash heredoc 往 `intake/` 落盘卡片文件 → 被拦，因为**卡片正文**里出现了「revert 落地 commit」「不 add / 不 commit / 不 push」等字样。改用编辑器工具写文件才过。
4. `cp ~/Projects/Agent-On/kit/dashboard-template.html dashboard.html && python3 … && git add -A && git commit …`（cwd = 项目仓）→ 被拦。对 agent-on 仓的唯一操作是 `cp` 读，git 操作对象全是项目仓；**拆成两条命令（先 cp，再 git）即通过**。

第 4 条是决定性的：它证明判据落在**整条命令的文本**上，不在任何一条 git 命令的目标仓上。

## 根因（两层，第二层是 2026-09-26 才查出来的）

**第一层：判据。** 两个项目的拦截文案都点名了执行体 `~/.claude/plugins/cache/agent-on/agent-on/0.5.0/kit/guard/agent-on-git-guard.sh`——插件缓存里的老 Python 闸。它在解析完 git 写操作之后，又把**命令文本里出现的所有 agent-on 路径**（`PATH_RE.findall(cmd)`）追加成目标：commit message、heredoc 正文、`cp` 的源路径，只要文本里有，就算「对 agent-on 仓写」。判据把「提到」当成了「操作」。

**第二层：执行面。** 仓里现行的闸（`cli/src/guard.rs`，Rust）早就只判 git 写操作真正作用的目录：跟踪 `cd`，认 `-C` / `--git-dir` / `--work-tree`，`-m` 的参数跳过，只把写操作的位置参数算目标。2026-09-26 同一台机器上干跑对照（cwd 为一个临时项目仓）：

| 输入 | 老闸（缓存 0.5.0，Python） | 现行闸（Rust） |
|---|---|---|
| `git commit -m 'see <Agent-On>/BOOTSTRAP.md'` | exit 2（误拦） | exit 0 |
| `cp <Agent-On>/kit/dashboard-template.html . && git add -A && git commit -m x` | exit 2（误拦） | exit 0 |
| 对照：`git -C <Agent-On> commit -m x` | exit 2 | exit 2 |
| 对照：`cd <Agent-On> && git add intake/x.md` | exit 2 | exit 2 |

所以修复早就在仓里，误拦还在发生，是因为宿主上**实际在跑的是插件缓存里那份旧副本**：插件装在 0.5.0（仓里已是 0.12.1），它的 hook 和 `~/.claude/settings.json` 里转发到 Rust 闸的那条并排挂着，每条 Bash 各判一遍，谁拦算谁。执行面这一层单独成案，见[案 47](47-rule-changed-executor-did-not.md)。

09-21 的草稿只看到第一层，把结论写成「闸体修复 deferred（硬停第 1 类，须单独一轨）」——没查拦下来的是哪一份闸，就把一个已经修好的问题当成了待修。

## 修法

1. **判据换到目标仓**（现行 Rust 闸已是这样）：判 `git` 命令实际作用的仓（`git -C <path>`、命令内 `cd` 之后的 cwd、`GIT_DIR`、命令参数里的仓路径），不判整条命令文本；commit message 与 heredoc 正文里出现的路径与动词**不构成越界**。
2. **换掉执行面**：本机 `claude plugin update agent-on@agent-on`（重启生效），让插件的 hook 也转发到同一个 Rust 闸。
3. **出口面（现场可用，已实证）**：在执行面换掉之前，拆成两条命令——先做对上游仓的只读引用，再对项目仓 git；或用编辑器工具写文件代替 heredoc。两条都在被拦者权限内、非破坏，符合案 40 的 4B。

## 可复用规则

**任何跨仓守卫的判据都必须落在「这个命令作用于哪个仓」，不能落在「文本里提到了哪个仓」。** 检查清单三条：

1. **先分段再判**：把复合命令按 `&&` / `;` 切开，只对**含 git 动词的那几段**判目标仓；对上游仓只有 `cp`/`cat`/`sed` 读的段落不算。
2. **排除正文**：`-m` 的 message、heredoc 体、`echo`/`cat`/`printf` 的内容是数据不是动作。
3. **拦截文案自带出口**：写明「把上游引用与本项目 git 拆成两条命令」，别只报「禁止对上游仓写」。

**反向自检一句**：这条命令如果被拦，**它到底会对 agent-on 仓产生什么变更**？答不出具体变更（新增 commit / 改文件 / 推 ref），就是误拦。**再加一句**：被拦先看拦截文案点名的执行体路径——路径指向缓存、版本对不上仓里那份，先查执行面，别先改判据。

## 已固化到哪

- **入册**：本案例（CryptoQuant、aster-agent 两张卡）。
- **判据**：现行 `cli/src/guard.rs` 的 `parse_git_command` 已按目标仓判，无需再改。
- **执行面**：`playbook/multi-contributor-protocol.md` §三½.5 的「执行面」、`kit/guard/README.md` 执行面自检、`boot/settlement.md` 升级节「升级后核执行面」；`agent-on doctor` 报 hook 执行面仍待实现（下一条 CLI 轨）。
- 相关：案 39（宿主安全句当制度）、案 40（出口可达性）、案 45（记录 ≠ 有人）、案 47（规则改了，执行面没换）。
