---
title: B2/C2 的跨类型 canonical dual 见证
summary: 来源登记 dual(SC(B2,true)) 与 adjoint(C2,false) 内容一致，可用于检验 canonical dual 的共享及预热历史；该后续见证尚未完成 gate 验证。
sources:
  - weyl-context-identity-and-sharing.md
kind: concept
createdAt: "2026-10-10T01:45:14.979Z"
updatedAt: "2026-10-10T01:45:14.979Z"
tags:
  - Weyl群
  - 对偶根数据
  - 差分测试
aliases:
  - b2c2-的跨类型-canonical-dual-见证
  - B的CD见
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# B2/C2 的跨类型 canonical dual 见证

B2/C2 的跨类型 canonical dual 见证用于检验：通过不同类型名称构造的根数据，能否命中同一个 canonical dual 内容，以及目标的预热历史是否正确影响 Weyl 群共享。其关键依据是 **C2 的固定 Cartan 矩阵恰为 B2 的转置**，因此 `dual(SC(B2,true))` 与 `adjoint(C2,false)` 内容一致。^[weyl-context-identity-and-sharing.md:86-94]

## 转置构造与跨类型对应

canonical dual 的构造先交换 roots 与 coroots，并翻转 `preference`；新 RootDatum 随后从转置 Cartan 矩阵重新编号。这与保持原编号顺序的 DualTag 元数据对偶不同，参见 [[Canonical dual 与 DualTag 的根编号差异]]。B2/C2 的上述内容对应，因而为通过另一类型构造 canonical dual 目标提供了直接见证。^[weyl-context-identity-and-sharing.md:75-94]

这一点比使用 `adjoint(G2,false)` 的预热方案更明确：来源确认，该 G2 构造并不是 canonical dual，无法占用目标的冷共享槽位；真正的 G2 预热见证需要显式构造转置内容。B2/C2 则可以利用两种类型固定 Cartan 矩阵之间的转置关系。^[weyl-context-identity-and-sharing.md:86-94]

## 预热历史决定共享行为

original Atlas 按完整 `PreRootDatum` 内容驻留根数据，其中包括 simple roots、simple coroots 和 `prefer_coroots`。驻留表弱持有对象；相同内容的对象仍存活时，构造会复用其 canonical 活对象身份。相关机制见 [[RootDatum 弱驻留与规范活对象身份]]。^[weyl-context-identity-and-sharing.md:164-172]

每个活根数据的 Weyl 群指针初始为空。调用 `dual()` 时，如果 canonical target 尚未建立 Weyl 群，就先建立 source 的 Weyl 群，再将同一指针装入 target；如果 target 已经预热，则绝不覆盖。因此，即使 dual 的内容对应正确，两侧后续是否共享 Weyl 群身份仍取决于构造与预热历史，参见 [[dual 预热历史与 Weyl 群兼容性]]。^[weyl-context-identity-and-sharing.md:174-187]

Weyl 元素的 `=`、`!=` 和 `*` 在产生结果前检查 Weyl 群地址；不一致时抛出 `Weyl group mismatch`，而不是仅凭结构内容返回布尔值或完成乘法。该检查也先于 `no_value` gate。B2/C2 见证所针对的兼容性因此包含身份与历史语义。^[weyl-context-identity-and-sharing.md:183-187]

## 验证目标与证据边界

B2/C2 是 A1 之后计划逐步覆盖的见证之一。A1 的跨 dual 乘法只有 $s_0s_0=1$，无法暴露错误的生成元重编号或直接复合外来根排列的问题，详见 [[A1 恒等乘积对生成元编号错误的遮蔽]]。后续验证还包括 G2、两个乘法操作数顺序、inner-class dual 构造及 `no_value` 关系。^[weyl-context-identity-and-sharing.md:313-324]

对于共享抽象群身份但使用不同坐标系的 Weyl 元素，已落地修复先检查 abstract-group `Arc` 身份，再在左侧坐标系中重放右侧 external word；这与跨类型见证需要检查的坐标兼容性相关，参见 [[Weyl 元素兼容性与跨坐标词重放]]。该修复的已接受范围仍限定于 A1。^[weyl-context-identity-and-sharing.md:249-261, weyl-context-identity-and-sharing.md:63-69]

截至来源的 2026-10-09 更新，B2/C2 一对 fixture 已起草为 provisional，但尚未接入验证流程。因此，跨类型内容对应属于已登记的源码依据，不能表述为 B2/C2 已通过运行捕获或语义验收，也不能由此推断缓存、性能或内存收益。^[weyl-context-identity-and-sharing.md:86-100, weyl-context-identity-and-sharing.md:320-330]

## Sources

- [weyl-context-identity-and-sharing.md](../../sources/weyl-context-identity-and-sharing.md) — Weyl 对象身份、dual 历史与安全共享边界。
