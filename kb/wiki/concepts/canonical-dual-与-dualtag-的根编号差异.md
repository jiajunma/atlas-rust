---
title: Canonical dual 与 DualTag 的根编号差异
summary: 原版 canonical dual 从转置 Cartan 矩阵重新编号，而仅供 Fokko 使用的 DualTag 元数据对偶保持原序，二者不能混同。
sources:
  - weyl-context-identity-and-sharing.md
kind: concept
createdAt: "2026-10-10T00:55:52.108Z"
updatedAt: "2026-10-10T00:55:52.108Z"
tags:
  - 对偶
  - 根编号
  - baseline-alignment
aliases:
  - canonical-dual-与-dualtag-的根编号差异
  - CD与D的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

# Canonical dual 与 DualTag 的根编号差异

Canonical dual 与 `DualTag` 的关键区别在于根编号的处理：canonical dual 从转置后的 Cartan 矩阵重新构造并编号 `RootDatum`；`DualTag` 元数据对偶则保持原编号顺序。两条路径不能混为一谈，因为对偶构造的编号规则关系到 Weyl 元素跨坐标运算的兼容性。^[weyl-context-identity-and-sharing.md:75-85, weyl-context-identity-and-sharing.md:279-282]

## 两条对偶构造路径

在冻结的 original Atlas revision `7e1b958c` 中，`PreRootDatum::dualise` 只交换 roots 与 coroots，并翻转 `preference`。随后，新 `RootDatum` 根据**转置 Cartan 矩阵**重新编号；来源将这两步分别定位到 `prerootdata.h:101` 和 `rootdata.cpp:820`。因此，交换根与余根的数据操作，不等于保留已有根编号的完整对偶构造。^[weyl-context-identity-and-sharing.md:75-79]

另一条 `DualTag` 元数据对偶路径保持原序。`rootdata.cpp:867–873` 的注释说明该路径仅供 Fokko 使用。讨论 canonical dual 时，不能将这条专用路径的保序性质套用到从转置 Cartan 矩阵构造新对象的过程。^[weyl-context-identity-and-sharing.md:78-80]

## Canonical 身份与 Weyl 群共享

Canonical dual 还涉及对象身份。原版按完整 `PreRootDatum` 内容进行弱驻留，键包含 simple roots、simple coroots 和 `prefer_coroots`；相同对象仍存活时复用其身份，已释放时则在原槽位重建。语言层 RootDatum 的 `=` 与 `!=` 比较的是这一驻留后的指针身份，相关机制见 [[RootDatum 弱驻留与规范活对象身份]]。^[weyl-context-identity-and-sharing.md:164-172]

`dual()` 通过同一个弱驻留表取得 canonical target。若目标尚未构造 Weyl 群，就安装 source 的 Weyl 群指针；若目标已预热，则不覆盖其指针。因此，根数据的对偶关系与 Weyl 群是否共享身份是两个需要分别判断的问题，后者还取决于构造历史，参见 [[dual 预热历史与 Weyl 群兼容性]]。^[weyl-context-identity-and-sharing.md:174-187]

## G2 与 B2/C2 的区别

来源的 G2 预登记指出，canonical dual 带有转置后的 coroot 矩阵，任何 `adjoint(G2,·)` 都无法构造出相同内容。因此，以 `adjoint(G2,false)` 预热并不能占用真正 canonical dual 的共享槽位；这类 fixture 无法检验预热目标导致的拒绝行为。这是 capture 前的源码预期，不是已捕获结论。^[weyl-context-identity-and-sharing.md:86-91]

真正的 G2 预热见证需要显式构造转置内容，其 Cartan 矩阵为
\(\begin{pmatrix}2&-3\\-1&2\end{pmatrix}\)。
相比之下，C2 的固定 Cartan 矩阵正是 B2 的转置，所以 `dual(SC(B2,true))` 与 `adjoint(C2,false)` 内容一致，适合构造对应的预热见证。^[weyl-context-identity-and-sharing.md:92-94]

## 对跨坐标 Weyl 运算的影响

即使两个值共享抽象 Weyl 群身份，它们的坐标 kernel 也可能不同，不能直接比较或复合内部 root permutation。来源规定的处理方式是：先检查抽象群身份，再把右操作数的 external generator word 放到左操作数的 `RootSystem` 中重放，结果归左 owner。这一规则与根编号差异共同构成 [[Weyl 元素兼容性与跨坐标词重放]] 的背景。^[weyl-context-identity-and-sharing.md:279-284]

Rust 已落地的修复采用 abstract-group `Arc` identity 检查，并在左侧坐标系重放右侧 external word；关系检查在 `no_value` 级别仍然执行。该实现经 AFTER-v5 验收后以提交 `690c2b92` 落地，但其验收范围仍限于 A1 语义。^[weyl-context-identity-and-sharing.md:249-261, weyl-context-identity-and-sharing.md:63-74]

## 验证范围

A1 跨 dual 的乘法只涉及 \(s_0s_0=1\)，无法发现错误的 generator renumbering，也无法排除直接复合 foreign root permutation 的错误。根编号与接口顺序的验证因此需要 G2 非对称见证，以及 B2/C2、两个乘法操作数顺序等后续 gate，参见 [[Weyl 语义回归的递进验证门禁]]。^[weyl-context-identity-and-sharing.md:313-324]

截至来源的 2026-10-09 更新，G2 capture pair 已冻结并完成彩排，但因隧道中断尚未提交；显式转置内容预热等后续 fixture 仍为 provisional、未接线。已完成的 A1 验收不能推广为 G2 编号兼容性、高 rank 正确性或性能收益的证明。^[weyl-context-identity-and-sharing.md:63-69, weyl-context-identity-and-sharing.md:95-100, weyl-context-identity-and-sharing.md:326-330]

## Sources

- [weyl-context-identity-and-sharing.md](../../sources/weyl-context-identity-and-sharing.md) — Weyl 对象身份、dual 历史与安全共享边界。
