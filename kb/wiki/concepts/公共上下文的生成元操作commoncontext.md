---
title: 公共上下文的生成元操作（CommonContext）
summary: 将状态、交叉作用、奇偶判定及双向 Cayley 操作转运到共轭父单根，并通过 gamma 的余根零配对确定奇异生成元。
sources:
  - partial-common-block.md
kind: concept
createdAt: "2026-10-09T15:03:13.909Z"
updatedAt: "2026-10-10T00:45:00.692Z"
tags:
  - 公共块
  - 生成元作用
aliases:
  - 公共上下文的生成元操作commoncontext
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 公共上下文的生成元操作（CommonContext）
summary: 将 KGB 层面的生成元作用转运到共轭父单根，为标准模参数提供状态查询、cross、奇偶判定和双向 Cayley 操作，并计算奇异生成元标志。
sources:
  - partial-common-block.md
kind: concept
tags:
  - 公共块
  - 生成元作用
  - Cayley变换
aliases:
  - 公共上下文的生成元操作commoncontext
provenanceState: extracted
---

# 公共上下文的生成元操作（CommonContext）

`CommonContext` 对应上游 `repr::common_context`，在标准模参数（srm）层面提供五个生成元操作，将 KGB 层面的生成元作用转运到共轭的父单根上。它属于积分子系统上的 Bruhat 区间与[[公共块的构造与元素编号（PartialBlock）|部分公共块]]构造机制。^[partial-common-block.md:20-38, partial-common-block.md:44-55]

## 参数与积分子系统

操作对象 `StandardReprMod` 由 KGB 元素 `x` 与经 $(1-\theta)X^*$ 约化并规范化的 `gamma_lambda` 组成；值相等对应上游哈希表中的相等判定。其两条构造路径为 `build` 与 `mod_reduce`，分别经 `RepContext::build_srm` 与 `RepContext::mod_reduce` 调用；后者用于打印 wrapper 的种子计算。^[partial-common-block.md:23-27]

`IntegralSubsystem` 通过 `integrality_simples` 构造单根数据，提供按生成元编号访问的 `parent_nr_simple`、`simple`、`to_simple` 与 `reflection`。模块仅移植公共上下文需要的访问器，Bruhat 生成器不需要完整的子系统根闭包。^[partial-common-block.md:28-32]

## 五个生成元操作

### `status`：根状态与辅助标志

`status` 返回 `(KgbStatus, bool)`。布尔标志的含义随根类型变化：在实根（real）情形表示 `isDoubleCayleyImage`，在复根（complex）情形表示 `isDescent`，在非紧虚根（noncompact imaginary）情形区分 type-1 与 type-2 cross-move。^[partial-common-block.md:48-50]

### `cross`：交叉作用与参数修正

`cross` 先按反射词对 `x` 做 cross，并以 `pos_to_neg` 实根修正平移 `gamma_lambda`，再按父根反射。这一过程同时处理 KGB 元素与参数的变化。^[partial-common-block.md:51-52]

### `is_parity`、`down_cayley` 与 `up_cayley`

其余三个操作分别为奇偶判定 `is_parity`、向下 Cayley 变换 `down_cayley` 与向上 Cayley 变换 `up_cayley`。当提升后的 `gamma_lambda` 不满足奇偶条件时，`up_cayley` 加上 $\alpha_s/2$ 进行奇偶修正。来源没有进一步展开 `is_parity` 与 `down_cayley` 的算法步骤。^[partial-common-block.md:53-55]

## 奇异生成元标志

除五个生成元操作外，`singular_flags` 对应上游 `common_block::singular`：逐个检查积分子系统生成元的余根是否在 `gamma` 的分子上取零。该判定使用 `gamma`；向上 Cayley 变换中的奇偶修正则作用于 `gamma_lambda`。^[partial-common-block.md:53-57]

部分块的 `survives` 判定要求：没有任何 singular 生成元是元素 `z` 的 descent。这将奇异生成元条件与块元素的下降状态联系起来。^[partial-common-block.md:79-81]

## 调用契约与证据边界

这些操作依赖上游调用方契约，模块不重复检查相关断言。被省略的调用方契约断言逐处留有注释；真正的内部不一致通过 [[StructureError 统一错误分类学|StructureError]] 报告，而非 panic。^[partial-common-block.md:40-42, partial-common-block.md:59-59]

本页依据 `partial_block.rs` 的结构性阅读材料，所读字节来自 dirty 工作区，并由阅读快照记录。来源中的上游行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。材料未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；部分块正确性属于其独立的 HPC 证据链，本页不扩展该范围。^[partial-common-block.md:9-16, partial-common-block.md:100-108]

## Sources

- [partial-common-block.md](../../sources/partial-common-block.md)：部分公共块：Bruhat 区间上的块构造。
