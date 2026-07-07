# 案例 06:Fake→真翻转打破测试的环境假设

> 层级:L2 教训 | 来源:Euan 实战 2026-07-03(Run #3 撞车修复) | 入册:2026-07-07 批三

## 症状
合流后把 provider 默认实现从 Fake 翻成真的（Dio），全量回归里冒出一批测试失败——都是会渲染到该 provider 的界面测试。

## 根因
测试的绿并不只依赖它显式写的东西，还暗含一条「环境假设」:默认 provider 是 Fake。合流翻转把这个默认掀了。S3 时集中钉过一遍 overrides，但 S1 更早写的「只钉 auth 就进壳」的旧测试没钉全量 Fake，翻转后就裸奔连到真实现。反直觉处:测试当初写对了、也一直绿,是脚下的默认被换掉才塌。

## 修法
grep 出所有 `pumpWidget(EuanApp)`（进壳/进页入口）的测试,逐个确认 overrides 用的是全量 Fake helper 而非只钉 auth;补齐后翻转再跑一遍全量。

## 可复用规则
翻转任何全局默认(provider/开关/环境)前,先 grep 所有「进壳/进页」测试,确认它们钉的是全量 Fake overrides,而不是只钉了一半。

## 已固化到哪
已固化:`kit/merge-checklist.md` 合流七步第 4 步「翻转 Fake→真」——明确要求 grep 所有「进壳/进页」测试确认钉了全量 Fake overrides,并标注为 Run #3 教训「只钉 auth 会裸奔」。
