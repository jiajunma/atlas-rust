---
title: 公共块的构造与元素编号（PartialBlock）
summary: build 消费按 x 排序的 Bruhat 区间并最终按 (length,x,y) 编号；build_full 以同一 packet 机制处理全积分与真积分子系统。
sources:
  - partial-common-block.md
kind: concept
createdAt: "2026-10-09T15:03:24.625Z"
updatedAt: "2026-10-09T19:34:12.270Z"
tags:
  - 公共块
  - 元素编号
aliases:
  - 公共块的构造与元素编号partialblock
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 公共块的构造与元素编号（PartialBlock）

`PartialBlock` 表示积分子系统上的部分公共块，消费种子下方 Bruhat 区间中的标准模参数来完成构造；同一模块也提供完整公共块构造器 `build_full`。这些机制移植自 Atlas `print_partial_block` wrapper 背后的实现。部分块最终按 `(length, x, y)` 排序，使元素编号与 oracle 打印的行号一致。^[partial-common-block.md:20-38, partial-common-block.md:61-67]

## 构造所需的数据

块构造使用 [[标准模参数的约化表示（StandardReprMod）|StandardReprMod]]：它包含 KGB 元素 `x`，以及经过 $(1-\theta)X^*$ 约化并规范化的 `gamma_lambda`；值相等对应上游哈希表中的相等关系。其两条构造路径分别是经 `RepContext::build_srm` 调用的 `build`，以及经 `RepContext::mod_reduce` 调用的 `mod_reduce`，后者用于打印 wrapper 的种子计算。^[partial-common-block.md:23-27]

[[积分子系统（IntegralSubsystem）|IntegralSubsystem]] 由 `integrality_simples` 构造，提供按生成元编号访问的单根数据。这里仅移植公共上下文需要的 `parent_nr_simple`、`simple`、`to_simple` 和 `reflection` 访问器，Bruhat 生成器不需要完整的子系统根闭包。[[公共上下文的生成元操作（CommonContext）|CommonContext]] 则将 KGB 层面的生成元作用转运到共轭的父单根上，提供标准模参数层面的操作。^[partial-common-block.md:28-34, partial-common-block.md:44-55]

## 部分块构造与编号

部分块构造先生成区间，再装配块。`bruhat_below` 对应上游 `Rep_table::Bruhat_below`，经 `Bruhat_generator::block_below` 产出种子的 Bruhat 区间标准模参数列表，随后交给 `PartialBlock::build` 消费。相关过程见 [[种子下方的 Bruhat 区间生成]]。^[partial-common-block.md:35-37]

`build` 接收按 `x` 排序的区间，在构造结束时再按 `(length, x, y)` 排序。因此，最终编号依次以长度、`x` 和 `y` 为排序键，与 oracle 打印行号一致；输入阶段按 `x` 排列的顺序并非最终元素顺序。^[partial-common-block.md:65-67]

## 完整公共块构造

`build_full` 对应上游完整公共块构造器。空积分子系统产生单元素块；同一套 packet 构造同时处理 ambient-full 和真积分子系统两种情形。^[partial-common-block.md:63-64]

## 元素访问与区间边界

接口提供 `size`、`rank`，而 `element`、`x`、`y`、`length`、`gamma_lambda` 均返回 `Option`。`lookup` 将标准模参数映射到块元素，区间外的参数返回 `None`，对应上游的 `UndefBlock`。^[partial-common-block.md:69-73]

`descent(z, s)` 返回元素在生成元下的 [[BlockDescent 八值状态体系|BlockDescent]] 状态。`cross(s, z)` 返回 `None` 表示链离开区间或尚未设置。`cayley(s, z)` 返回 Cayley 像对：imaginary ascent 情形下表示前向 Cayley 目标，real descent 情形下表示逆 Cayley 目标；打印层通过 `isWeakDescent` 选择解释方式。^[partial-common-block.md:74-78]

打印辅助接口 `highest_x`、`highest_y` 对应上游的 `max_x`、`max_y`。`survives` 判断元素是否没有任何 singular 生成元作为其 descent；singular 标志则逐子系统生成元检查其余根是否在 `gamma` 的分子上取零。详见 [[部分公共块的访问器与边界语义]]。^[partial-common-block.md:56-57, partial-common-block.md:79-81]

## 对偶后的编号与限制

`dual()` 进行纯数据变换：元素顺序反转为 $z'=\mathrm{size}-1-z$，`x` 与 `y` 互换，长度变为 $\mathrm{max\_len}-\ell(z)$，其中 $\mathrm{max\_len}=\ell(\mathrm{size}-1)$。下降状态经 `BlockDescent::dual` 映射，cross/Cayley 链的目标编号也相应反转；Cayley 第二像仅在第一像有定义时映射。^[partial-common-block.md:83-90]

部分块中离开区间的链在对偶后保持未定义。KL 递归只对链接闭合的源块（完整块）的对偶有效：部分区间的外出 complex ascent 对偶化后成为 cross 未定义的 complex descent，会被 `KlTable` 拒绝。对偶返回 `BareBlock`，因为其行不再是原 context 的标准模参数，参数池与查找表没有对应的对偶结构。详见 [[公共块对偶变换及 KL 递归的闭合限制]]。^[partial-common-block.md:92-96]

## 错误处理与证据边界

模块省略属于上游调用方契约的断言，并逐处保留注释；真正的内部不一致通过 [[StructureError 统一错误分类学|StructureError]] 报告，而非 panic。^[partial-common-block.md:40-42, partial-common-block.md:59-59]

来源是对 `partial_block.rs` 的结构性阅读，所读字节来自 dirty 工作区快照。该来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；部分块正确性属于其独立的 [[HPC 验收证据链]]。来源中的上游行号由源码注释转述，未经独立重读，可能随版本变化。^[partial-common-block.md:9-16, partial-common-block.md:100-108]

## Sources

- [partial-common-block.md](partial-common-block.md)：《部分公共块：Bruhat 区间上的块构造》。
