---
title: ext_param/star 层：扩展块的参数层
source: atlas-rust/ext-param
ingestedAt: 2026-10-03T10:32:42Z
---

# ext_param/star 层：扩展块的参数层

编辑状态：**结构性阅读，草稿经 Kimi probe 起草、维护者逐条对照源码核对后改写**。
本包解释 `ext_param.rs` 的参数层结构；其正确性属于它自己的 HPC 证据链
（unitarity gate 等），本包不重述也不扩展。所读字节见
[`snapshots/2026-10-03-ext-param.json`](snapshots/2026-10-03-ext-param.json)
（`ext_param.rs` SHA-256
`0eabb0696791338ceb0d55ed4408eb661e022d4cbc3728c40cc09af80b5c8029`，dirty
工作区）。

## 定位与两条移植约定

本模块移植上游 `gkmod/ext_block.cpp` 的参数层：`ExtRepContext`
（`repr::Ext_rep_context`，repr.h:682-714 与 repr.cpp:2786-2836）、
`ExtParam` 值类型（ext_block.h:293-364 与 ext_block.cpp:2283-2420）、比较与
对齐辅助（`same_standard_reps`/`same_sign`/`z_align`/`level_a`，
ext_block.cpp:910-985）、共轭词 `fixed_conjugate_simple`（:525-552）、复根
cross 作用 `complex_cross`（:858-907）与核心的 `star` 计算（:990-1705）。
其上还有三个参数层 finalisation 驱动：`extended_restrict_to_k`（:2435-2547）、
`extended_finalise`（:2598-2721）、`scaled_extended_finalise`（:2736-2807）。

两条保真约定：(1) 所有 `Weight`/`Coweight`/`int` 算术用二进制补码 wrapping
i32，匹配上游 `int` 算术；有理权分子保持 i64。(2) 上游 `assert` 条件转为
`debug_assert`（或 debug-only 的 `validate`），恰如上游在 `NDEBUG` 下编译
消除；真正的数据相关失败以 `StructureError` 暴露。

## ExtRepContext

由 twisting involution `delta` 扩展的 `RepContext`：`delta` 以根系置换表示，
附不动根集与诱导的单生成元 twist。访问器：`rc()`/`delta()`/`delta_of`/
`is_delta_fixed_root`/`twisted`；高级判定：`to_simple_shift`（repr.h:706-708）、
`is_very_complex`（repr.cpp:2804-2813）、`shift_flip`（:2824-2836）。

## ExtParam

扩展参数值类型（ext_block.h:293-364）：字段含 `tw`（Weyl 元素）、`l`
（Coweight）、`gamma_lambda`（RationalWeight）、`tau`（Weight）、`t`
（Coweight）与翻转位。`at` 是 KGB 元素 `x` 处的默认扩展构造
（ext_block.cpp:2297-2311）。派生计算：`theta(ctx)`、`theta_id(ctx)`、
`x(ctx)`（由 `(tw, l mod 2)` 重建 KGB 元素，:2392-2393）、`restrict_mod`
（:2399-2402）、`restrict`（:2405-2410）。

默认扩展一族：`default_extend`（:2341-2345）、`default_extend_srm`
（:2348-2351，要求 `gamma_lambda` 已在 `x` 处 `real_unique`）、
`shifted_default_extension`（ext_block.h:352-361）、`is_default`
（ext_block.h:363-364）。

## star 与 finalisation 驱动

`star(ctx, e, length, n_alpha)`（ext_block.cpp:990-1705）返回
`(DescValue, Vec<ExtParam>)`：根编号 `n_alpha` 的 delta-轨道类型与邻接扩展
参数。三个 finalisation 驱动的队列循环重放 folded-orbit 反射与 `star` 下降，
并跟踪相对默认扩展的净翻转。`extended_finalise(ctx, sr)` 返回
`Vec<(StandardRepr, bool)>`（前置条件经 debug_assert：standard 且
delta-fixed）；`scaled_extended_finalise(ctx, sr, factor_num, factor_den)`
返回 `(StandardRepr, bool)`，缩放 $\nu$ 而保持 $\lambda$ 固定。

## 两个 StarOracle 实现

`ExtParamOracle` 服务 `ExtBlock::tune_signs`（每个父块元素的默认扩展经
`ext_param::def_ext` 重建）；`PartialBlockOracle` 以 `PartialBlock` 父块为
后端，用于 `ExtBlock::build_partial` 之后的 `tune_signs`。

## 来源与限制

- 源码：[ext_param.rs](../../../crates/atlas-real-group/src/ext_param.rs)；
  阅读快照 [`2026-10-03-ext-param.json`](snapshots/2026-10-03-ext-param.json)。
- 上游行号均转述自源码注释（ext_block.h/ext_block.cpp/repr.h/repr.cpp），
  未独立重读上游，随版本演进可能漂移。
- 关联：[扩展块](extended-block.md)、[扩展 KLV 表](extended-kl.md)、
  [表示参数上下文](rep-context.md)、[形变驱动](deformation-drivers.md)。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 起草经由本地 Kimi probe（无工具 profile，`kimi-code/k3-256k`；exit 0，
  188.4s，420 秒期限）。草案由维护者对照源码逐条核对改写；其「待源码核对」
  项中涉及 `star` 返回对、`extended_finalise`/`scaled_extended_finalise`
  前置条件与返回形态的内容均已按源码落实，其余骨架内容未采用。调用记录见
  快照的 `kimi_assist`。
