---
title: 公共块对偶变换及 KL 递归的闭合限制
summary: dual 反转编号、交换坐标并变换长度、状态与链接，返回 BareBlock；部分区间的缺失链接可能在对偶后形成 KL 表拒绝的下降。
sources:
  - partial-common-block.md
kind: concept
createdAt: "2026-10-09T15:03:41.079Z"
updatedAt: "2026-10-09T19:34:06.038Z"
tags:
  - 块对偶
  - KL递归
  - 闭合条件
aliases:
  - 公共块对偶变换及-kl-递归的闭合限制
  - 公K递
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 公共块对偶变换及 KL 递归的闭合限制

公共块的 `dual()` 是纯数据变换，对应上游 `Bare_block::dual`，也是 `BlockGraph::dual` 在公共块上的对应操作。它交换每行的 primal/dual 角色，返回 `BareBlock`；变换后的数据能否用于 KL 递归，受源块的链接闭合性限制。^[partial-common-block.md:83-96]

## 对偶变换规则

设块大小为 \(N\)，原元素编号为 \(z\)，长度为 \(\ell(z)\)，并令 \(\mathrm{max\_len}=\ell(N-1)\)。对偶化将元素编号反转为 \(z'=N-1-z\)，交换 `x`、`y` 坐标，并将长度反射为 \(\mathrm{max\_len}-\ell(z)\)。每个下降状态通过 `BlockDescent::dual` 映射，相关状态体系见 [[BlockDescent 八值状态体系]]。^[partial-common-block.md:85-90]

每条有定义的 cross/Cayley 链，其目标编号 \(c\) 映为 \(N-1-c\)；Cayley 第二像仅在第一像有定义时映射。部分块中原本未定义的链保持未定义，对偶化不会补齐离开区间的链接。^[partial-common-block.md:89-95]

## 部分块与 KL 递归的闭合限制

[[公共块的构造与元素编号（PartialBlock）|部分公共块]] 消费 `bruhat_below` 产生的 Bruhat 区间。其 `cross(s, z)` 返回 `None` 时，对应上游的 `UndefBlock`，表示链接离开区间或尚未设置；`cayley(s, z)` 则提供 imaginary ascent 的前向 Cayley 目标或 real descent 的逆 Cayley 像对。^[partial-common-block.md:65-78]

上游只对完整块进行对偶化，此时 cross 链总有定义。来源明确将对偶后的 KL 递归限定于链接闭合的源块（完整块）：部分区间中指向区间外的 complex ascent，对偶化后成为 cross 未定义的 complex descent，`KlTable` 会拒绝这种情形。因此，部分块可以完成数据层面的对偶变换，但其结果不保证满足 KL 递归的要求。^[partial-common-block.md:92-96]

## 返回类型与参数边界

`dual()` 返回 `BareBlock`，因为对偶块的行不是原块 context 中的标准模参数，参数池与查找表也没有对偶对应物。该返回值表达块数据的对偶，不能视为原上下文中仍具有标准模参数查找能力的新公共块；参数表示可参见 [[标准模参数的约化表示（StandardReprMod）]]。^[partial-common-block.md:95-96]

## 证据范围

本说明依据对 `partial_block.rs` 的结构性阅读，所读字节来自 dirty 工作区快照。上游行号转述自源码注释，未独立重读上游，可能随版本演进漂移。来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；部分块的正确性属于独立的 [[HPC 验收证据链]]。^[partial-common-block.md:9-16, partial-common-block.md:100-108]

## Sources

- [partial-common-block.md](partial-common-block.md) — 部分公共块：Bruhat 区间上的块构造。
