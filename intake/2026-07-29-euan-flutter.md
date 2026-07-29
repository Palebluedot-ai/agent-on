# intake · Euan-Flutter · 2026-07-29 结账(5 卡)

> 来源会话:API 宿主从 Vercel 迁到 Cloudflare Workers,中途改方案为「换新域 dartify.dev」并重建整条发信链路(Resend 发件域 + GoTrue 模板)。
> 素材路径:五条均已先落项目 `agent-on/loop-notes.md`(2026-07-29 段),本文件按 promotion-card 模板装配。
> 域判据自检:五张均属 AI 协作过程(排障方法 / 工具行为 / 测试纪律 / 人机协作模式),无产品域知识、无业务规则出仓。

---

### observe-dont-interrogate(定位依赖对方系统内部状态时,先索取观测,别用「逐项试」代替看一眼)
- source:Euan-Flutter @ 会话记录 2026-07-29 | pin v0.3.0
- evidence:GoTrue 持续 500「Error sending recovery email」。按可能性排序让用户逐项试五轮——①Supabase password 被清空 ②Resend key 绑死旧域 ③password 又粘错 ④Sender email 那组没改 ⑤端口 465→587——**只有②命中**,其余四轮空转,每轮一次人工操作 + 一次往返,耗时约 40 分钟。用户发来一张 SMTP 设置页截图后,`Username: Resend`(大写 R)一眼可见;实测 `resend` 认证成功、`Resend` 回 `535 Invalid username`,当场终结。
- confidence:high(单次但因果极清晰:五轮猜 vs 一次观测,成本差一个数量级;且该模式在任何「配置存在对方系统里」的排障中都成立)
- claim:当故障定位依赖「对方系统里究竟存了什么」而你看不到时,**第一步就索取直接观测**(截图 / 只读 API / 配置导出),不要用「让对方按可能性逐项试」代替观测。自检判据:**如果你正在按可能性排序列试验清单,说明你缺的是观测而不是假设**——停下来先要观测。
- suggested_landing:playbook/systematic-debugging 或 sop 排障段新增一条「观测优先于假设」;kit/AGENTS-skeleton 排障纪律行补半句「列试验清单前先问:能不能直接看到?」
- rollback:revert 落地 commit(纯方法论条,无机制依赖)
- trace:本会话工具输出——五轮往返记录;终局实证 `smtplib.login('resend'|'Resend')` 双跑对照
- 状态:landed@同批(第十四次消化:sop Phase 6 排障·观测优先 + AGENTS-skeleton 排障纪律半句)

---

### ratelimit-masks-real-error(限流/降级响应会覆盖真实故障码,导致「已修复」误判)
- source:Euan-Flutter @ 会话记录 2026-07-29 | pin v0.3.0
- evidence:同一个 GoTrue 500,排查中反复重试触发 `over_email_send_rate_limit`,GoTrue 转回 **429**;而项目网关按防枚举设计把 4xx 吞成 202(`auth.ts:acceptShaped` 仅 `status>=500` 才翻 503)。于是外部观察到「503 变 202」,我据此判断「5xx 消失 = 根因已解决」并写进了给用户的汇报。**是误判**——把 Supabase 邮件配额从默认调到 100 后,直调 GoTrue 立刻重现 `500 unexpected_failure`(error_id 019fad4b-…)。
- confidence:high(机制普适:任何带限流/熔断/降级的上游都会用自己的响应码盖住下游真实故障)
- claim:排查期间若错误码「变好」,先问**这个改善与我的修复动作之间有没有因果链**;没有因果链的改善不算证据。特别地,限流(429)、熔断、降级响应会**覆盖**真实故障码——解除限流后必须复测,才能判定是否真的修复。
- suggested_landing:playbook/anti-hallucination「工具输出可疑」一节增补「错误码改善 ≠ 修复:先验因果链」;bench 追加本案例(503→429→被吞成202→放开配额后 500 重现)
- rollback:revert 落地 commit
- trace:本会话——`/auth/forgot` 503 ×3 → 直调 GoTrue 得 429 `over_email_send_rate_limit` → 网关回 202 → 用户调高配额 → 直调 GoTrue 复现 500
- 状态:landed@同批(第十四次消化:anti-hallucination 第六型#5 + bench/cases/26)

---

### text-assertion-cannot-prove-structure(文本锚证明不了结构:配置被解析成子键,断言照样全绿)
- source:Euan-Flutter @ c602853 | pin v0.3.0
- evidence:给 wrangler.toml 加域名绑定,按 TDD 先写两条锚(「routes 已启用」「custom_domain = true」),RED(2 failed|7 passed)→ 写 routes → GREEN。**但那是假绿**:`routes` 被写在 `[assets]` 段之后,TOML 将其解析为 `assets.routes` 子键——wrangler 完全不认,等价于 Worker 没有任何入口——而两条**纯文本**断言(grep 字符串是否在文件里)照样全过。靠 `python3 -m tomllib` 读出 `routes=None` 才发现。修法两面:①移到顶层;②补**位置级**判别锚(routes 必须出现在第一个 `[段]` 之前)。反向验证:挪回段内立刻 1 failed 并报出行号,恢复即 10 passed。
- confidence:high(TOML/YAML/JSON 等结构化配置通用;「字符串在文件里」与「解析器认它」是两回事)
- claim:对结构化配置的测试锚必须打在**语义层**:能走真解析器就走解析器;宿主语言拿不到解析器时(如测试是 TS、配置是 TOML),退而做**位置/结构级判别**,并**反向验证**——把配置改坏,锚必须红。只证明「现在是绿的」不足以说明锚有效。
- suggested_landing:playbook 测试纪律段补「结构化配置锚:语义层 > 文本层」;kit 的完成判据 checklist 补一行「配置类锚须附反向验证证据」。与既有「动效锚要能判别档位」是同一原则的不同介质,建议合并抽象为一条**「锚必须能判别」**的通则
- rollback:revert 落地 commit;若判定与既有「锚要能判别」重复,直接 rejected(已被覆盖)
- trace:本会话——`tomllib` 输出 `routes=None` 且 `assets` 内混入 routes;反向验证两次跑测结果
- 状态:landed@同批(第十四次消化:anti-hallucination C 附2 + phase-card 配置锚行)

---

### cwd-not-flag-decides-source(决定工具读哪份文件的是 cwd,不是你传的 flag——推送前须验证生效)
- source:Euan-Flutter @ 会话记录 2026-07-29 | pin v0.3.0
- evidence:多 worktree 环境下推 Supabase 配置:改动在 worktree 的 `config.toml`,link 状态在主目录。执行 `supabase config push --project-ref X --workdir <worktree>`,**以为 flag 指定了读取源**。实际它读**当前工作目录**(主目录)那份旧配置并推送成功(`"service":"auth","status":"updated"`),把当日全部改动冲回旧值:`site_url` 回旧域、回跳白名单删掉新域两条、`email_sent` 100→2、`admin_email` 回 `noreply@mail.euan.pro`、`sender_name` 回 `Euan`。修法:把 link 状态拷进 worktree、**cd 进去再推**,第二次 diff 方向正确,功能实测恢复(`GoTrue /recover` → 200)。**关键细节:推之前我确实对账过两份文件的差异**,却仍然踩了——因为把「知道两份不同」当成了「工具会用对的那份」。
- confidence:high(与 07-26 的 `staged-residue-breaks-atomic-commit` 同族:命令的默认作用域 ≠ 你传的参数范围;两例分别出现在 git 与 supabase CLI,跨工具复现)
- claim:改变「读取位置/作用域」类的 flag,在**不可逆动作**(推送/部署/发布)之前必须**验证它真的生效**——先跑只读命令看工具报告的实际路径,或让工具先输出将要应用的 diff 并核对方向。「我传了 flag」不等于「工具照做了」。多 worktree / 多 checkout 环境下尤其致命,因为同名文件有多份且内容不同。
- suggested_landing:**建议并入既有 slug `staged-residue-breaks-atomic-commit`** 作为跨工具第二实例(消化会话按语义归并);双落点:playbook 该条正文补「读取位置类 flag」维度 + kit 的提交/部署纪律行补「不可逆动作前验证作用域」
- rollback:revert 落地 commit;若判定与 staged-residue 重复度过高,直接 rejected(已被覆盖)
- trace:本会话——两次 `supabase config push` 的 diff 方向相反(第一次 `+site_url = "https://api.euan.pro"`,第二次 `+admin_email = "noreply@mail.dartify.dev"`);修复后 `/auth/recover` 200
- 状态:landed@同批(第十四次消化:并入 anti-hallucination 第六型#4 读取位置维 + AGENTS-skeleton/merge-checklist 作用域验证)

---

### cdn-verify-needs-cachebuster-and-window(CDN 静态资产验证要带 cache-buster 并留传播窗口,否则把缓存滞后误判成部署失败)
- source:Euan-Flutter @ 95c680f | pin v0.3.0
- evidence:改完三张静态页部署到 Cloudflare(`wrangler deploy` 报 `Uploaded 3 of 3 assets`),立刻验证:两张已更新、`reset.html` 仍是旧版(线上 5832 字节 vs 仓内 5842),且首轮 grep 结果自相矛盾(同一页既命中新词又数出旧词)——看起来像「部署失败了一半」。实为 **CF 边缘缓存滞后**:带 cache-buster 重测并等约 1 分钟后,三页全部与仓内逐字节一致。上传回执本身是准确的。
- confidence:medium(单次,但机制通用于任何 CDN/边缘缓存;成本极低、误导性极高)
- claim:验证 CDN/边缘缓存后面的静态资产,请求必须**带 cache-buster 参数**并**留传播窗口后重测**再下结论——部署工具报告的「上传成功」与「边缘可见」是两个事件,中间有延迟。把缓存滞后误判成部署失败会引向完全错误的排查方向。
- suggested_landing:sop 外部服务集成清单 / 部署验证段新增一条;kit 的部署后验收 checklist 补「静态资产验证带 cache-buster + 二次确认」
- rollback:revert 落地 commit(纯验证方法条)
- trace:本会话——首轮 5832/5842 不一致 + 矛盾 grep 输出;带 cache-buster 轮询第 1 次即 5841(去尾换行)一致
- 状态:landed@同批(第十四次消化:sop 集成清单第 8 条 + merge-checklist §5 静态资产行)
