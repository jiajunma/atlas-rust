---
title: 公共块对偶变换及 KL 递归的闭合限制
summary: dual 反转编号、交换坐标并变换长度、状态和链接，返回 BareBlock；部分区间的未定义外出链接可能成为下降缺链，使 KlTable 拒绝其对偶。
sources:
  - partial-common-block.md
kind: concept
createdAt: "2026-10-09T15:03:41.079Z"
updatedAt: "2026-10-10T00:45:19.641Z"
tags:
  - 公共块
  - 对偶
  - KL递归
aliases:
  - 公共块对偶变换及-kl-递归的闭合限制
  - 公K递
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 公共块对偶变换及 KL 递归的闭合限制
summary: dual 反转元素编号、交换坐标并变换长度、下降状态与链接，返回 BareBlock；部分区间的外出链接可能在对偶后形成下降缺链，使 KlTable 拒绝。
sources:
  - partial-common-block.md
kind: concept
tags:
  - 公共块
  - 对偶
  - KL递归
aliases:
  - 公共块对偶变换及-kl-递归的闭合限制
provenanceState: extracted
---

# 公共块对偶变换及 KL 递归的闭合限制

公共块的 `dual()` 是纯数据变换，对应上游 `Bare_block::dual`，也是 `BlockGraph::dual` 在公共块上的对应操作。它交换每行的 primal/dual 角色，返回 `BareBlock`。对偶结果能否用于 KL 递归，还取决于源块是否满足链接闭合条件。^[partial-common-block.md:83-96]

## 对偶变换规则

设块大小为 \(N\)，原元素编号为 \(z\)，长度为 \(\ell(z)\)，并令 \(\mathrm{max\_len}=\ell(N-1)\)。对偶化后的编号为 \(z'=N-1-z\)，`x`、`y` 坐标互换，长度变为 \(\mathrm{max\_len}-\ell(z)\)。每个下降状态通过 `BlockDescent::dual` 映射，相关状态分类见 [[BlockDescent 八值状态体系]]。^[partial-common-block.md:85-90]

每条有定义的 cross/Cayley 链，其目标编号 \(c\) 映为 \(N-1-c\)；Cayley 第二像仅在第一像有定义时映射。部分块中离开区间的未定义链保持未定义，对偶变换不会补齐这些链接。^[partial-common-block.md:89-95]

## 部分块中的链接语义

[[公共块的构造与元素编号（PartialBlock）|部分公共块]] 的 `build` 消费 `bruhat_below` 产生的、按 `x` 排序的标准模参数（srm）区间，并在构造结束时按 `(length, x, y)` 排序。其 `cross(s, z)` 返回 `None` 时，对应上游 `UndefBlock`，表示链接离开区间或尚未设置。^[partial-common-block.md:65-75]

`cayley(s, z)` 返回 Cayley 像对：对 imaginary ascent，表示前向 Cayley 目标；对 real descent，表示逆 Cayley 像。打印层通过 `isWeakDescent` 选择相应含义，相关背景见 [[Cross、Cayley 与逆 Cayley 链接]]。^[partial-common-block.md:76-78]

## KL 递归的闭合限制

上游只对完整块进行对偶化，此时 cross 链总有定义。来源将对偶后的 KL 递归适用范围限定为链接闭合（link-closed）的源块，并以完整块为此处的适用情形。部分区间中指向区间外的 complex ascent，在对偶化后会成为 cross 未定义的 complex descent，`KlTable` 会拒绝这种情形。因此，完成数据层面的对偶变换，并不保证结果满足 KL 递归的要求。^[partial-common-block.md:92-96]

## 返回类型与参数边界

`dual()` 返回 `BareBlock`，因为对偶块的行不是原块 context 中的标准模参数，原有参数池与查找表没有对偶对应物。返回值承载对偶后的块数据，但不提供原公共块参数池和查找表的对偶版本。^[partial-common-block.md:95-96]

## 证据范围

本页依据对 `partial_block.rs` 的结构性阅读，所读字节来自 dirty 工作区快照 `2026-10-03-partial-common-block.json`。部分块的正确性属于其独立的 HPC 证据链，来源不重述或扩展该证据。^[partial-common-block.md:9-16]

来源中的上游行号转述自源码注释，未独立重读上游，可能随版本演进漂移。本来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。^[partial-common-block.md:100-108]

## Sources

- [partial-common-block.md](../../sources/partial-common-block.md) — 部分公共块：Bruhat 区间上的块构造。
