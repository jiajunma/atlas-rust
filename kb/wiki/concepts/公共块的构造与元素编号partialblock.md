---
title: 公共块的构造与元素编号（PartialBlock）
summary: build_full 构造完整公共块，build 消费按 x 排序的 Bruhat 区间并最终按 (length, x, y) 排序，使编号对应 oracle 打印行号。
sources:
  - partial-common-block.md
kind: concept
createdAt: "2026-10-09T15:03:24.625Z"
updatedAt: "2026-10-09T15:03:24.625Z"
tags:
  - 公共块
  - 块构造
  - 编号规则
aliases:
  - 公共块的构造与元素编号partialblock
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 公共块的构造与元素编号（PartialBlock）

`PartialBlock` 表示积分子系统上的部分公共块，其构造以种子的 Bruhat 区间为输入；同一模块还提供完整公共块构造器 `build_full`。它移植了 Atlas `print_partial_block` wrapper 背后的相关机制。^[partial-common-block.md:18-38, partial-common-block.md:61-67]

## 构造所需的数据

块元素使用 `StandardReprMod` 表示：它包含 KGB 元素 `x`，以及经过 $(1-\theta)X^*$ 约化并规范化的 `gamma_lambda`；该值的相等关系对应上游哈希表中的相等关系。两条构造路径分别是经 `RepContext::build_srm` 调用的 `build`，以及经 `RepContext::mod_reduce` 调用的 `mod_reduce`；后者用于打印 wrapper 的种子计算。^[partial-common-block.md:23-27]

`IntegralSubsystem` 由 `integrality_simples` 构造，提供按生成元编号访问的单根数据。这里仅移植公共上下文需要的 `parent_nr_simple`、`simple`、`to_simple` 和 `reflection` 访问器，因为 Bruhat 生成器不需要完整的子系统根闭包。[[公共上下文的生成元操作（CommonContext）]] 则提供标准模参数层面的生成元操作，将 KGB 层面的作用转运到共轭的父单根上。^[partial-common-block.md:28-34, partial-common-block.md:44-55]

## 构造流程与编号规则

部分块构造分为区间生成和块装配两步。`bruhat_below` 经 `Bruhat_generator::block_below` 生成种子的 Bruhat 区间，产出标准模参数列表，随后交给 `PartialBlock::build` 消费。^[partial-common-block.md:35-37]

`build` 接收按 `x` 排序的区间，并在构造结束时按 `(length, x, y)` 重新排序。因此，最终元素编号以长度、`x`、`y` 为排序键，与 oracle 打印的行号一致；输入的 `x` 排序并不是最终编号顺序。^[partial-common-block.md:65-67]

`build_full` 构造完整公共块：空积分子系统对应单元素块，同一套 packet 构造同时处理 ambient-full 和真积分子系统两种情形。^[partial-common-block.md:63-64]

## 元素访问与区间边界

接口提供 `size` 和 `rank`，而 `element`、`x`、`y`、`length`、`gamma_lambda` 均返回 `Option`。`lookup` 将标准模参数映射到块元素，若参数位于区间外则返回 `None`，对应上游的 `UndefBlock`。^[partial-common-block.md:69-73]

`descent(z, s)` 返回元素 `z` 在生成元 `s` 下的 [[BlockDescent 八值状态体系|BlockDescent]] 状态。`cross(s, z)` 返回 `None` 时，表示链离开区间或尚未设置。`cayley(s, z)` 返回 Cayley 像对：在 imaginary ascent 情形下是前向 Cayley 目标，在 real descent 情形下是逆 Cayley 目标；打印层通过 `isWeakDescent` 选择解释方式。^[partial-common-block.md:74-78]

打印辅助接口 `highest_x`、`highest_y` 对应上游的 `max_x`、`max_y`。`survives` 判断元素是否不存在作为其 descent 的 singular 生成元；这些 singular 标志由子系统生成元的余根是否在 `gamma` 的分子上取零决定。^[partial-common-block.md:56-57, partial-common-block.md:79-81]

## 对偶后的编号与适用范围

`dual()` 是纯数据变换：元素编号反转为 $z'=\mathrm{size}-1-z$，`x` 与 `y` 互换，长度变为 $\mathrm{max\_len}-\ell(z)$，其中 $\mathrm{max\_len}=\ell(\mathrm{size}-1)$。下降状态通过 `BlockDescent::dual` 映射，cross/Cayley 链的目标编号也相应反转；Cayley 第二像仅在第一像有定义时映射。^[partial-common-block.md:83-90]

部分块中离开区间的链在对偶后仍未定义，因此对偶结果并不自动适用于 KL 递归。外出的 complex ascent 可能变成 cross 未定义的 complex descent，进而被 `KlTable` 拒绝。返回类型是 `BareBlock`，因为对偶行不再是原 context 的标准模参数，原参数池和查找表没有对应的对偶结构。详见 [[公共块对偶变换及 KL 递归的闭合限制]]。^[partial-common-block.md:92-96]

## 错误处理与证据边界

模块省略了属于上游调用方契约的断言，并逐处保留注释；真正的内部不一致通过 [[StructureError 统一错误分类学|StructureError]] 报告，而非 panic。^[partial-common-block.md:40-42, partial-common-block.md:59-59]

本页依据的是对 `partial_block.rs` 的结构性阅读，所读字节来自 dirty 工作区快照。来源包没有执行构建、测试或原版运行，也不提供数学验收、性能或并行结论；partial block 的正确性属于其独立的 [[HPC 验收证据链]]。上游行号由源码注释转述，未经独立重读，可能随版本变化。^[partial-common-block.md:9-16, partial-common-block.md:100-108]

## Sources

- [partial-common-block.md](partial-common-block.md)：《部分公共块：Bruhat 区间上的块构造》。
