---
title: Canonical dual 的转置根数据与 G2 预热见证
summary: Canonical dual 交换根与余根并从转置 Cartan 重新编号；来源预期普通 adjoint(G2,false) 无法预热该目标，真正的预热拒绝见证须显式构造转置内容，尚待 capture 验证。
sources:
  - weyl-context-identity-and-sharing.md
kind: concept
createdAt: "2026-10-09T21:13:30.838Z"
updatedAt: "2026-10-09T21:13:30.838Z"
tags:
  - weyl
  - root-data
  - regression-testing
aliases:
  - canonical-dual-的转置根数据与-g2-预热见证
  - CD的G预
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# Canonical dual 的转置根数据与 G2 预热见证

Canonical dual 的构造不仅交换根与余根，还影响根编号及 Weyl group 的共享历史。对 G2 而言，`adjoint(G2,false)` 并不等于所需的 canonical dual 内容，因此用它预热不能检验“目标 dual 已预热时拒绝跨 owner 运算”的行为；有效见证需要显式构造转置后的根数据。^[weyl-context-identity-and-sharing.md:75-94]

## 转置根数据与编号

在冻结的 original Atlas revision `7e1b958c` 中，`PreRootDatum::dualise` 交换 roots 与 coroots，并翻转 `preference`。随后构造的新 `RootDatum` 从转置 Cartan 矩阵重新编号。这个路径与 `DualTag` 元数据 dual 不同：后者保持原序，上游注释说明它仅供 Fokko 使用。不能把保序的元数据 dual 当作 canonical dual 的编号规则。^[weyl-context-identity-and-sharing.md:75-80]

Original Atlas 按完整 `PreRootDatum` 内容驻留对象，键包括 simple roots、simple coroots 和 `prefer_coroots`。驻留表通过弱引用复用仍存活的对象；语言层 RootDatum 的 `=`／`!=` 比较的是规范化后的指针身份。因此，判断某个预热构造是否命中 canonical dual，需要核对完整内容及 preference，不能仅凭群类型名称。^[weyl-context-identity-and-sharing.md:164-172]

## 预热为何改变兼容性

每个存活的 root datum 都有一个初始为空的 Weyl group 槽位。`dual()` 先取得 canonical target：若目标槽位为空，就确保 source 的 Weyl group 已建立，并将同一共享指针安装到目标；若目标已经预热，则绝不覆盖。由此，两个 datum 是否共享 Weyl group，取决于构造与预热历史。^[weyl-context-identity-and-sharing.md:174-181]

Weyl 元素的 `=`、`!=`、`*` 在产生结果前检查 WeylGroup 地址，不一致便抛出 `Weyl group mismatch`；检查先于 `no_value` gate。乘积保留左操作数的 owner，而元素本身强持有 root datum。因此，结构相等或相同的根置换都不足以替代这项兼容性检查。^[weyl-context-identity-and-sharing.md:81-85, weyl-context-identity-and-sharing.md:183-187]

## G2 见证的预测修正

上游逐行阅读表明，G2 的 canonical dual 带有转置 coroot 矩阵，任何 `adjoint(G2,·)` 都无法构造出该内容。据此，来源在 capture 前登记的预期是：`WG_DUAL_OWNER` 与 `WG_REVERSE_OWNER` 在两个引擎中都应打印 `false`。冻结 contract 原先预测的 `true` 应记为预测失准，而非引擎分歧；这仍属于源码预期，不是已经捕获的结果，参见 [[源码预测与原版捕获的证据分离]]。^[weyl-context-identity-and-sharing.md:86-89]

原预热 fixture 使用 `adjoint(G2,false)`，无法占用 canonical dual 的 cold-share 槽位，因此 dual 侧三元组并未触发所需的预热条件，预期两个引擎都不抛错。真正的 G2 预热拒绝见证需要显式构造 `root_datum`，根矩阵取 \(I_2\)，余根矩阵取 \(\begin{pmatrix}2&-3\\-1&2\end{pmatrix}\)，preference 参数取 `false`，以预热准确的转置内容。^[weyl-context-identity-and-sharing.md:89-93]

B2/C2 提供了更直接的对照：C2 的固定 Cartan 矩阵正是 B2 的转置，因此 `dual(SC(B2,true))` 与 `adjoint(C2,false)` 内容一致。G2 预热构造中的问题不影响这一对照。^[weyl-context-identity-and-sharing.md:93-94]

## Rust 实现与见证范围

已落地的 Rust 修复通过 `Arc<DatumWeylIdentity>` 和按完整 datum 内容加 preference 的弱驻留表表达身份。`dual(RootDatum)` 仅在 canonical target 仍 cold 时共享 source 的 abstract group，绝不覆盖预热目标；二元运算先比较 abstract-group `Arc` 身份，再在左侧坐标系重放右侧 external word。关系运算在 `no_value` 级别也执行身份检查。^[weyl-context-identity-and-sharing.md:249-261]

G2 的非对称 interface-order 见证用于补足 A1 的覆盖缺口：A1 跨 dual 乘法只有 \(s_0s_0=1\)，无法发现生成元重编号错误或直接复合 foreign root permutation 的错误。后续还需分别覆盖 B2/C2、两个乘法操作数顺序、inner-class dual construction、`no_value` relations，以及仅由 WeylElt 维持 datum 生命周期的情形。^[weyl-context-identity-and-sharing.md:313-330]

## 验证状态与限制

来源的 2026-10-09 更新记录：AFTER-v5 job `3900050` 已完成并以生产提交 `690c2b92` 落地，但接受范围仅为 A1 限定语义。cold_dual 完整字节相等，prewarmed_dual 在 stdout、退出码和有序 error summary 下保持一致；这不授予缓存、性能、内存或更高 rank 的验收。相关诊断口径见 [[差分诊断的有序错误摘要契约]]。^[weyl-context-identity-and-sharing.md:59-74]

同一更新中，`g2-v1` 的 65 文件 payload 已冻结并彩排通过，因隧道中断暂缓提交。显式转置内容的 G2 transposed-prewarm 等七个后续 fixture 仍为 provisional、未接线。因此，本页的 G2 行为应保留为预登记预测与见证设计，不能表述为已经通过的 G2 兼容性结论。^[weyl-context-identity-and-sharing.md:95-100]

## Sources

- [weyl-context-identity-and-sharing.md](../../sources/weyl-context-identity-and-sharing.md) — Weyl 对象身份、dual 历史与安全共享边界。
