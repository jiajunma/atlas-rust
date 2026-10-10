---
title: 公共块的构造与元素编号（PartialBlock）
summary: 部分块消费按 x 排序的 Bruhat 区间并按 (length, x, y) 重编号，完整构造以共享 packet 机制处理全积分与真积分子系统。
sources:
  - partial-common-block.md
kind: concept
createdAt: "2026-10-09T15:03:24.625Z"
updatedAt: "2026-10-10T00:45:20.930Z"
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
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 公共块的构造与元素编号（PartialBlock）
summary: PartialBlock 的部分块构造消费按 x 排序的 Bruhat 区间，最终按 (length, x, y) 编号；完整构造以同一 packet 机制处理 ambient-full 与真积分子系统，空积分子系统产生单元素块。
sources:
  - partial-common-block.md
kind: concept
tags:
  - 公共块
  - 构造算法
  - 元素编号
aliases:
  - 公共块的构造与元素编号partialblock
---

# 公共块的构造与元素编号（PartialBlock）

`PartialBlock` 实现积分子系统上的公共块构造，移植自 Atlas `print_partial_block` wrapper 背后的机制。`build` 根据种子的 Bruhat 区间构造部分公共块，`build_full` 构造完整公共块；部分块构造结束后按 `(length, x, y)` 排序，使元素编号与 oracle 打印行号一致。^[partial-common-block.md:20-38, partial-common-block.md:61-67]

## 构造所需的数据

`StandardReprMod` 是构造使用的标准模参数，由 KGB 元素 `x` 与经过 $(1-\theta)X^*$ 约化并规范化的 `gamma_lambda` 组成；其值相等对应上游哈希表中的相等关系。两条构造路径分别是经 `RepContext::build_srm` 调用的 `build`，以及经 `RepContext::mod_reduce` 调用的 `mod_reduce`；后者用于打印 wrapper 的种子计算。^[partial-common-block.md:23-27]

`IntegralSubsystem` 提供由 `integrality_simples` 构造的单根数据。模块仅移植公共上下文所需的生成元访问器 `parent_nr_simple`、`simple`、`to_simple` 和 `reflection`；Bruhat 生成器不需要完整的子系统根闭包。[[公共上下文的生成元操作（CommonContext）|CommonContext]] 提供标准模参数层面的生成元操作，将 KGB 层面的作用转运到共轭的父单根上。^[partial-common-block.md:28-34, partial-common-block.md:44-55]

## 部分块构造与编号

`bruhat_below` 对应上游 `Rep_table::Bruhat_below`，经 `Bruhat_generator::block_below` 生成种子的 Bruhat 区间标准模参数列表，交给 `PartialBlock::build` 消费。^[partial-common-block.md:35-37]

构造包含两个排序阶段：`build` 接收按 `x` 排序的区间，构造结束后再按 `(length, x, y)` 排序。最终编号依次比较长度、`x` 和 `y`，与 oracle 打印的行号一致；输入顺序不能直接作为最终元素编号。^[partial-common-block.md:65-67]

## 完整公共块构造

`build_full` 对应上游完整公共块构造器。同一套 packet 构造同时处理 ambient-full 与真积分子系统两种情形；当积分子系统为空时，结果为单元素块。^[partial-common-block.md:63-64]

## 元素访问与区间边界

接口提供 `size` 和 `rank`；`element`、`x`、`y`、`length`、`gamma_lambda` 均返回 `Option`。`lookup` 查找标准模参数对应的块元素，区间外返回 `None`，对应上游 `UndefBlock`。^[partial-common-block.md:69-73]

`descent(z, s)` 返回逐生成元的 [[BlockDescent 八值状态体系|BlockDescent]] 状态。`cross(s, z)` 返回 `None` 表示链离开区间或尚未设置。`cayley(s, z)` 返回 Cayley 像对：imaginary ascent 时为前向 Cayley 目标，real descent 时为逆 Cayley 目标；打印层通过 `isWeakDescent` 选择。^[partial-common-block.md:74-78]

打印辅助接口 `highest_x`、`highest_y` 对应上游 `max_x`、`max_y`。`survives` 判定元素 `z` 是否没有任何 singular 生成元作为 descent；singular 标志通过逐子系统生成元检查其余根是否在 `gamma` 的分子上取零得到。^[partial-common-block.md:56-57, partial-common-block.md:79-81]

## 对偶变换中的编号

`dual()` 交换每行的 primal/dual 角色：编号变为 $z'=\mathrm{size}-1-z$，`x` 与 `y` 互换，长度变为 $\mathrm{max\_len}-\ell(z)$，其中 $\mathrm{max\_len}=\ell(\mathrm{size}-1)$。下降状态经 `BlockDescent::dual` 映射，cross/Cayley 链目标同样按反转编号映射；Cayley 第二像仅在第一像有定义时映射。^[partial-common-block.md:83-90]

部分块中离开区间的链在对偶后保持未定义。KL 递归只对链接闭合的源块（完整块）的对偶有效：部分区间的外出 complex ascent 对偶化为 cross 未定义的 complex descent 时，`KlTable` 会拒绝。返回值为 `BareBlock`，因为对偶行不是原 context 的标准模参数，参数池与查找表没有对偶对应物。详见 [[公共块对偶变换及 KL 递归的闭合限制]]。^[partial-common-block.md:92-96]

## 错误处理与证据边界

模块省略属于上游调用方契约的断言，并逐处保留注释；真正的内部不一致通过 [[StructureError 统一错误分类学|StructureError]] 报告，而非 panic。^[partial-common-block.md:40-42, partial-common-block.md:59-59]

来源是对 `partial_block.rs` 的结构性阅读，所读字节来自 dirty 工作区，记录于快照 `2026-10-03-partial-common-block.json`。上游行号转述自源码注释，未经独立重读，可能随版本演进而漂移。^[partial-common-block.md:9-16, partial-common-block.md:100-103]

来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。部分块正确性属于其独立的 HPC 证据链，本页不扩展该证据范围。^[partial-common-block.md:10-11, partial-common-block.md:108-108]

## Sources

- [partial-common-block.md](../../sources/partial-common-block.md)：《部分公共块：Bruhat 区间上的块构造》。
