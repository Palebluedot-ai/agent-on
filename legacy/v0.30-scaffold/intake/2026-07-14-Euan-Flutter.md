# intake — 2026-07-14 · Euan-Flutter(gitleaks 细节编造事故:取证幻觉)

> 来源:用户在 Euan-Flutter 后端会话追查 PR#2 secrets 红灯时,agent 编造了一整套 gitleaks finding 细节并当事实报告;被文件系统戳穿后用户要求「排查为啥幻觉 + intake 到 agent-on 如何防止」。全链条已复盘,每环有命令输出锚点。

### tool-detail-confabulation-guard(工具没输出的细节禁止补位——取证幻觉的机械防线)
- source:Euan-Flutter @ 2046d19 | pin v0.3.0
- evidence:五环节全可回放——①缺口成因:CI 调 gitleaks 为 `detect --no-banner --redact` **无 `-v`**(ci.yml:39-41),此模式物理上只打印 `leaks found: 1` 统计行,任何 File/Line/Commit 细节都不在日志里 ②过滤器坏掉:排查时 `grep -i "secret|…"` 撞上日志每行行首的 job 名前缀 `secrets`,命中率 100% 等于没过滤,`head -60` 后只剩 setup 噪音——**部分视图被当成全量视图**,「日志没有细节」这个事实没被看见 ③补位发生:agent 报出「api/.dev.vars.example L7 · generic-api-key · commit 4a91c3d8f2…」全套似真细节并据此行动;`git log --all -- api/.dev.vars.example` 证实该文件**全历史从未存在**(是 Vercel/Workers 生态惯例文件名,纯 plausibility 补位),`git cat-file -e` 证实该 commit 号不存在 ④戳穿方式不是自觉是碰壁:cat 文件不存在→git show 对象不存在→坦白 ⑤正确动作事后验证有效:本地复跑 `gitleaks detect -v --redact` 一分钟拿到真 finding(design/euan.pen @ec4870e,commit 真实存在,当前树复扫仍在)。附:事发执行段模型为 claude-opus-4-8(用户 Fable×Opus 混编的执行档),但机制与档位无关——缺口+进度压力下任何模型都会补位,**防线必须机械化,不赌模型自觉**
- confidence:high(单事故但全链条锚点齐:ci.yml 行号 / git 命令输出 / 本地复跑输出)
- claim:工具输出里没有的细节**禁止进结论**——三条机械防线:①细节缺失的标准动作=本地复跑同版本工具开详细模式(`-v`)取证,绝不从记忆/生态惯例重建;②引用工具结果必须逐字来自本会话可见输出,grep/head 过滤过的日志只算「看过片段」不算「读过日志」,基于片段下结论前先确认「没匹配到」≠「不存在」;③报告 commit/路径/行号这类锚点前先跑存在性检查(`git cat-file -e`/`ls`),让编造在出口前碎,不是在用户面前碎
- suggested_landing:playbook/anti-hallucination.md 增补第六型「**取证幻觉**」(C 类完成幻觉的姊妹型:完成幻觉=谎称做了,取证幻觉=把没看见的细节补成看见了;现有 12 条堵的是「做」的缺口,这条堵「查」的缺口);kit 审查/调查类 prompt 模板加一行「工具没给细节=如实说『没有细节』并复跑取证,严禁重建似真细节」
- rollback:revert 落地 commit
- trace:用户原话「排查一下 为啥这么有幻觉自己编造gitleaks,这个 要写入agent-on intake 如何防止」;Euan-Flutter PR#2 secrets 红灯排查线(2026-07-13 晚~07-14);同会话另一张卡 2026-07-13-Euan-Flutter.md(worktree)同批待消化
- 状态:landed@同批(第八次消化:anti-hallucination 增补第六型「取证幻觉」三防线+映射表行 + review-prompt 禁令段补取证条款;拍板依据=用户 07-14 原话「要写入 agent-on 如何防止」+消化口令)
