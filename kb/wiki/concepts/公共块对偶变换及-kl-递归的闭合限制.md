---
title: 公共块对偶变换及 KL 递归的闭合限制
summary: dual 反转元素顺序、交换 x/y、反射长度并映射下降状态与链接，返回 BareBlock；部分块的未定义链接保持缺失，可能使对偶不满足 KL 递归的链接闭合要求。
sources:
  - partial-common-block.md
kind: concept
createdAt: "2026-10-09T15:03:41.079Z"
updatedAt: "2026-10-09T15:03:41.079Z"
tags:
  - 块对偶
  - KL递归
  - 适用范围
aliases:
  - 公共块对偶变换及-kl-递归的闭合限制
  - 公K递
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 公共块对偶变换及 KL 递归的闭合限制

公共块的 `dual()` 是纯数据变换，对应上游 `Bare_block::dual`，也是 `BlockGraph::dual` 在公共块上的对应操作。它交换每行的 primal/dual 角色；能否对变换后的块进行 KL 递归，则取决于源块的链接闭合性。^[partial-common-block.md:83-96]

## 对偶变换规则

设块大小为 \(N\)，原元素编号为 \(z\)，长度为 \(\ell(z)\)，并令 \(\mathrm{max\_len}=\ell(N-1)\)。对偶化将编号反转为 \(z'=N-1-z\)，交换 `x`、`y` 坐标，并将长度改为 \(\mathrm{max\_len}-\ell(z)\)。每个下降状态通过 `BlockDescent::dual` 映射，参见 [[BlockDescent 八值状态体系]]。^[partial-common-block.md:85-90]

每条有定义的 cross/Cayley 链，其目标编号 \(c\) 映为 \(N-1-c\)；Cayley 第二像仅在第一像有定义时映射。部分块中原本未定义的链保持未定义，因此该变换不会补齐区间外的链接。^[partial-common-block.md:89-95]

## 部分块与链接闭合限制

[[公共块的构造与元素编号（PartialBlock）]] 中，部分块由 `bruhat_below` 产生的 Bruhat 区间构造。其 `cross(s, z)` 返回 `None` 时，对应上游的 `UndefBlock`，表示链接离开区间或尚未设置。`cayley(s, z)` 则保存 imaginary ascent 的前向 Cayley 目标或 real descent 的逆 Cayley 像对。^[partial-common-block.md:65-78]

上游只对完整块进行对偶化，此时 cross 链总有定义。来源明确将对偶后的 KL 递归限制在链接闭合的源块，即完整块：部分区间中指向区间外的 complex ascent，经对偶化成为 cross 未定义的 complex descent，`KlTable` 会拒绝这种情形。因此，部分块能够进行数据层面的对偶变换，并不保证其结果满足 KL 递归的要求。^[partial-common-block.md:92-96]

## 返回类型与参数边界

`dual()` 返回 `BareBlock`。对偶块的行不再是原块 context 中的标准模参数，原参数池与查找表也没有对偶对应物。因此，该接口提供的是块数据的对偶，而非原上下文中带有标准模参数查找能力的新公共块。相关上下文操作见 [[公共上下文的生成元操作（CommonContext）]]。^[partial-common-block.md:95-96]

## 证据范围

本页依据对 `partial_block.rs` 的结构性阅读；来源记录的字节来自 dirty 工作区快照。上游位置转述自源码注释，未独立重读上游，行号可能随版本漂移。本来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；partial block 的正确性属于其独立的 [[HPC 验收证据链]]。^[partial-common-block.md:9-16, partial-common-block.md:100-108]

## Sources

- [partial-common-block.md](partial-common-block.md) — 部分公共块：Bruhat 区间上的块构造。
