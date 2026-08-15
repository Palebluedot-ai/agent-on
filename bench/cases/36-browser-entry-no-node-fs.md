# 案例 36:浏览器入口禁止再导出 Node 写盘

> 层级:L2 | 来源:SalesDashboard 2026-08-15 | 入册:2026-08-16

## 症状
看板报 `Module "node:fs" has been externalized… Cannot access "node:fs.existsSync" in client code`。`src/sales/index.ts` 从同一 barrel 导出读数 hook 与 `import-local`/`store`(node:fs)。

## 修法
barrel 只留读数;写盘 API 改走独立入口。测试锁「client sales barrel does not pull node:fs」。

## 可复用规则
给浏览器的公共入口不得 re-export Node 内置。读数与写盘分成两个入口。禁止赌 tree-shake。

## 已固化到哪
kit/AGENTS-skeleton;kit/review-prompt 第 7 步。不升 playbook 长节。
