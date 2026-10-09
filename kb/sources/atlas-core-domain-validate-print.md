---
title: 校验与打印（domain_builtins.rs 9730–12263）——46 臂 validate、块打印机与 print_text 面
source: atlas-rust/atlas-core-domain-validate-print
ingestedAt: 2026-10-09T16:30:00Z
---

# 校验与打印（domain_builtins.rs 9730–12263）

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包覆盖
`crates/atlas-core/src/domain_builtins.rs` 的 `validate`（9730–10437，
46 臂）、块打印机（10437–10919）与 `print_text` + 各打印机
（10919–12263）。结构性阅读，不声称数学验收。

## `validate`（46 臂）：无值级校验的逐臂契约

每个领域名一臂，按**其注释引用的上游顺序契约**校验（无值门之前做多少，
逐臂不同）：

- `fundamental_weight`/`fundamental_coweight`：arity + 半单秩索引。
- `Weyl_orbit`/`Weyl_orbit_ws`（3 参）：`weyl_subgroup::validate`
  （BuildAndDrop——只构造不计算，见 [子群包](atlas-core-weyl-subgroup.md)）。
- `integrality_simples`/`integrality_rank`/`is_integrally_dominant`/
  `integrality_datum`：校验先于无值门（atlas-types.w:1766）——
  `check_integrality_dimension`。
- `W_refl`：`int_val` 收窄**先于**索引校验是可观察的。
- `real_form`：按参数个数分发 `(InnerClass,int)` 与
  `(InnerClass,mat,ratvec)`；外部形式号经 `order.internal` 翻译
  （`Illegal real form number: n`）。
- `KGB`：先按 kgb_size 边界查（`Inexistent KGB element: n`）。
- `KGB_elt`：**全部**构造检查在无值门之前跑完再丢弃
  （atlas-types.w:4580-4607）。
- `KL_block`：先 `test_standard` 再无值门（atlas-types.w:6868-6872）。

## 块打印机（10437–10919）

- `common_block_rows`：`print_param_block_wrapper`/`print_c_block_wrapper`
  的共享引擎（atlas-types.w:6653-6695）——**每次调用新建**：上游
  `Rep_table` 池只是记忆化，且 dominant gamma 的块修饰符平凡（恒等变换
  词、零移位），故新建打印一致（partial_block/KL_block 先例）。返回块序
  行与 init 索引——按 `(x, gamma-lambda)` 匹配，因为**单按 x 在 R 包内
  有歧义**。
- `located_common_block_rows`：参与 `KL_column`/`KL_block` 用的**共享
  查找序列**的路径（与上一项刻意不同）。
- `partial_block_rows`：部分块打印机的种子到行路径
  （atlas-types.w:6700-6735）：`StandardReprMod::mod_reduce` →
  `common_context` 作用于 gamma_lambda → `Bruhat_below` → 区间上的部分
  `common_block`，按块的最终 `(length, x, y)` 排序重编号；
  **survives 标志用调用方的 gamma** 的 `block.singular`——即使种子先被
  规范化（print_pc_block_wrapper）也如此。返回种子行号（上游
  `init_index`）。
- 完整块路径（`print_param_block_wrapper`）：`mod_reduce` 但**不**
  make_dominant、无池、无修饰符。
- `involution_expression`：`printInvolution`（prettyprint.cpp:219-232）：
  1 基生成元数字、`^` 表交叉、`x` 表共轭、`e` 收尾。

## `print_text`（10919）与各打印机

`print_text` 按名分发：

- `print_KGB`：单参打全部；选择形式（`print_KGB_selection_wrapper`）要求
  所列元素属于**同一**实形（跨形式即 "Real form mismatch when printing
  KGB element"）。`print_kgb`：`kgbsize` 行与 `Base grading` 头先打印
  （kgb_io.cpp:140-148）。
- `print_strong_real`：`output::printStrongReal`（output.cpp:490-540）
  加 `ioutils::foldLine` 折叠。
- `print_gradings`：虚根子系统行 + 实形部分的 gradings；**打印位 i 是
  位 sigma[i]**（上游 `sigma.pull_back(gr)`）。
- `print_X`：上游**无检查**；每次调用新建 InvolutionTable。
- `print_real_Weyl`：检查在臂内按包装器的措辞与顺序先跑——否则外来
  外部形式号会经 `ExternalFormOrder` 静默翻译。
- `print_blockstabilizer`：上游无检查；块只捐出它的两个实形
  （output.cpp:361-390）。

## 边界与限制

- 46 臂的逐臂内容不在本包；每臂的校验顺序契约以实现方注释就地保留。
- 内类装配见 [构造包](atlas-core-domain-construction.md)，身份见
  [领域值包](atlas-core-domain-values.md)，派发的 call 路径见
  [派发包](atlas-core-domain-dispatch.md)。
- 上游行号引用是**实现方移植陈述**；打印/校验兼容以 HPC 语料门为准。
- 字节数/哈希只标识本快照字节（git base `964f0033`）。
