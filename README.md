<div align="center">

# Agent-On

**零业务仓侵入的跨客户端 AI 协作利器：防假绿验证 · 薄对抗复核 · 跨工具 Worktree 雷达 · 合并树预演**

一次安装，同时赋能 **Claude Code · Codex · Antigravity**。<br>
不往业务仓塞锁文件，不挂常驻拦截钩子，只守住大模型单会话无法自我克服的盲区。

[![release](https://img.shields.io/github/v/tag/Palebluedot-ai/agent-on?label=release&sort=semver)](https://github.com/Palebluedot-ai/agent-on/tags)
![hosts](https://img.shields.io/badge/hosts-Claude%20Code%20%7C%20Codex%20%7C%20Antigravity-6f42c1)
![cli](https://img.shields.io/badge/CLI-Rust-dea584)
[![license](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

[为什么重构为 v2](#为什么是这五件套) · [核心模块](#核心模块一览) · [1 分钟装机](#1-分钟装机) · [CLI 速查](#cli-速查) · [从 v0.x 升级](docs/migration-v2.0.md)

</div>

总目标与边界权威：[CHARTER.md](CHARTER.md) · 版本账本：[CHANGELOG.md](CHANGELOG.md) · **当前推荐版本：`v2.0.0`**

---

## 为什么是这五件套

大模型与主流 AI 客户端（Claude Code、Codex、Antigravity）已经原生内化了基础子代理编排、任务拆解与自动创建 Git worktree。早期重型的跨仓版本锁（`agent-on.lock.md`）、手工轨道登记与常驻值守台账已完成历史使命（归档于 [legacy/v0.30-scaffold/](legacy/v0.30-scaffold/README.md)）。

但经过多项目数百次 PR 与 50 个真实翻车事故检验，**有五件事受限于“单会话视角盲区、同上下文圆谎偏见与本地-真实环境断层”，模型越聪明反而越擅长糊弄**：

1. **假绿测试糊弄**：模型写出的测试 100% 通过，但测的是自己刚编的 mock 或测试文件里重抄一遍的函数，线上照样坏。
2. **同窗口自我圆谎 & 多轮复核空转**：让刚写完代码的同一窗口自审很难推翻自己；而传统独立审查每轮从头重跑全仓测试、重读全量代码，80% 时间在空转。
3. **跨客户端 Worktree 孤岛**：`.claude/worktrees`、`.codex/worktrees`、`.gemini/antigravity/worktrees` 各自为政——互不知道对方在改同一文件、发包时误拿落后主线十几提交的旧树装机、磁盘堆积几十棵死目录不敢删。
4. **高并发 PR 互撞**：同时开五六个 Agent 冲刺时，每条分支单看都绿，第一条合入就把其余全撞成冲突。
5. **排障先甩锅环境 & 静默替用户做决定**：改了没生效先猜缓存/设备问题而不核交付链；把该自己查代码的事实问题抛给用户，却把高返工成本的产品偏好静默选边。

Agent-On `v2.0.0` 把这五块高含金量资产提炼为**零侵入的独立 Skills + 超薄 Rust CLI + 一页全局铁律**。

---

## 核心模块一览

| 模块 | 类型 | 触发场景与作用 |
|---|---|---|
| **[prove-it](skills/prove-it/SKILL.md)** | Skill | **交付前验证 / 写测补测 / 修顽固 Bug**：核对真实业务入口、做“故意改坏看红不红”的错误对照、判别式探针、写后独立回读与主线落差自查 |
| **[thin-review](skills/thin-review/SKILL.md)** | Skill | **合并前独立复核**：派发无聊天历史包袱的空白上下文子代理；作者已贴当前 HEAD 测试输出则免重跑全套；第二轮起只看增量 diff，严格分离阻塞缺陷与非阻塞建议 |
| **[wt-doctor](skills/wt-doctor/SKILL.md)** | Skill + CLI | **并行开工前 / 打包发版前 / 磁盘清理**：跨 Claude / Codex / Antigravity 扫描全部 worktree 未提交撞车、开放 PR 重叠、落后主线提交数，安全回收已合入死目录 |
| **[merge-check](skills/merge-check/SKILL.md)** | Skill + CLI | **多 Agent 并发提 PR 冲刺期**：用 `git merge-tree --write-tree` 在内存中零副作用预演多分支/多 PR 两两冲突、识别无重叠免同步分支（`SYNC_FREE`）并生成零冲突合入波次 |
| **[core-guardrails](rules/core-guardrails.md)** | 全局规则 | **一页常驻铁律**：交付链先于环境排障、事实自查 vs 偏好方差×成本 2×2、门禁退出码保真、不确定外部副作用先只读回读 |
| **[bench/cases](bench/cases/README.md)** | 案例集 | **50 个真实项目翻车标本**：覆盖环境变量解析、边缘剥头、限流掩盖真错、编辑器内存真相、跨仓守卫等实战深坑，供按需检索 |

---

## 1 分钟装机

依赖：[Rust 1.89+](https://rustup.rs) 与 `git`。

```bash
git clone https://github.com/Palebluedot-ai/agent-on.git ~/.local/share/agent-on
cd ~/.local/share/agent-on
cargo install --path cli --force

# 一键将 4 个 Skills 挂载到 Claude Code、Codex 与 Antigravity，并同步一页核心铁律
agent-on install --with-rules

# 自检三端挂载状态
agent-on doctor
```

### 各客户端挂载位置说明

- **Antigravity**：软链至 `~/.gemini/config/plugins/agent-on`，自动发现根目录 [plugin.json](plugin.json)、`rules/AGENTS.md` 与 `skills/*`。
- **Claude Code**：软链 4 个独立技能至 `~/.claude/skills/{prove-it,thin-review,wt-doctor,merge-check}`（`--with-rules` 会在 `~/.claude/CLAUDE.md` 维护受管铁律段落，完整保留你原有的个人规则）；也支持通过 `.claude-plugin` 安装。
- **Codex**：软链 4 个技能至 `~/.agents/skills/*`（`--with-rules` 同步受管段落至 `~/.codex/AGENTS.md`）；也支持通过 `.codex-plugin` 安装。

卸载同样干净无残留：`agent-on install --uninstall`（只移除指向 Agent-On 的软链与受管注释块，绝不碰用户原有内容）。

---

## CLI 速查

```bash
# 1. 跨客户端 Worktree 全景雷达（毫秒级只读）
agent-on wt

# 连同 GitHub 开放 PR 改动文件一起检查撞车
agent-on wt --with-prs

# 安全清理已合入主线且无未提交改动的死 worktree（绝不强删未合入/脏目录/疑似 Squash 分支，保留分支引用）
agent-on wt --prune

# 2. 高并发冲刺期：内存预演多条候选分支或开放 PR 的合并波次（零工作区改动）
agent-on merge-check --branches feat/a,feat/b,fix/c
agent-on merge-check --prs
agent-on merge-check --pr 12,15,18

# 3. 诊断三端挂载状态与旧版 v0.x 钩子残留
agent-on doctor

# 清理老项目的 v0.x 受管 Git hooks 与旧单路由软链
agent-on install --clean-legacy --repo /path/to/old-project
```

---

## 许可证

[MIT](LICENSE) © 2026 YANG CHAO
