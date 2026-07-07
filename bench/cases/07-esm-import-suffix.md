# 案例 07:@vercel/node ESM 不重写 import specifier

> 层级:L2 教训 | 来源:Euan 实战 2026-07-04 部署日 | 入册:2026-07-07 批三

## 症状
本地 vercel dev 一切正常,首次部署到生产就挂——相对 import 找不到模块。

## 根因
`type:module` 下 `@vercel/node` 把 TS 编译成 ESM JS,但**不重写 import specifier**。本地 vercel dev 宽容,免后缀也能跑;生产 Node 是严格 ESM,相对 import 必须带 `.js` 后缀。反直觉处:同一份代码同一个运行时家族,宽容度不一样。

## 修法
全仓相对 import 机械补 `.js`——84 处,正则一把过,tsc + vitest 双验证后再部署。

## 可复用规则
本地 dev 与生产的运行时差异,第一次部署就冒烟抓,别攒到上线前;`type:module` 项目的相对 import 一律带 `.js`。

## 已固化到哪
已固化:sop.md「外部服务集成清单」第 1 条(本地模拟器≠生产,首次部署即分层冒烟;详见案 01,同类同愈)。`type:module` 相对 import 带 `.js` 属项目级细则,项目 loop-notes 记即可,不占通用清单。
