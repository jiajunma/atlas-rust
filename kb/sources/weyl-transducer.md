---
title: Compact Weyl 群的 transducer 表示
source: atlas-rust/weyl-transducer
ingestedAt: 2026-10-03T10:32:42Z
---

# Compact Weyl 群的 transducer 表示

编辑状态：**结构性阅读，草稿经 Kimi probe 起草、维护者逐条对照源码核对后改写**。
本包解释 `weyl_transducer.rs` 的表示与枚举；其正确性属于它自己的 HPC 证据链
（capacity gate 等），本包不重述也不扩展。所读字节见
[`snapshots/2026-10-03-weyl-transducer.json`](snapshots/2026-10-03-weyl-transducer.json)
（`weyl_transducer.rs` SHA-256
`4bb3e72b293e6fa9f4709c996bdef388a14ec3cb81ddce2b61d85acea2718783`，dirty
工作区）。

## 定位与动机

本模块实现 du Cloux / van Leeuwen 的 transducer（parabolic-subquotient）表示
（上游 `structure/weyl.cpp`）。一个 Weyl 元素是 `[u8; WEYL_MAX_RANK]` 的固定
栈数组：第 `i` 项索引 parabolic subquotient $W_{i-1}\backslash W_i$ 的极小
陪集代表元。乘法经由 per-generator transducers 为 $O(\mathrm{length})$；模块
文档以 E6（51840 个元素）为例说明整群枚举比 [Weyl 群层](weyl-layer.md)的
矩阵表示便宜得多——这是文档转述，本包不作性能结论。

## 类型与常量

- `Generator = usize`、`EltPiece = u16`、`WeylElt = [u8; WEYL_MAX_RANK]`（均
  `pub(crate)`）；`WeylElt` 是 C++ `std::array<unsigned char, RANK_MAX>` 的
  等价物，枚举/twisted scan 中零堆分配。
- `WEYL_MAX_RANK = 32`（上游 `utilities/constants.h` 的 `RANK_MAX`）；注释
  明确：这是**表示上界**，不是元素枚举预算（complex rank 6 用 12 pieces）。
- `UNDEF_PIECE = u16::MAX`、`UNDEF_GEN = u16::MAX` 是 transducer 表的哨兵。

## Coxeter 矩阵查表

`coxeter_entry(letter, i, j)`（weyl.cpp:191-215）按连通分型的类型字母与
Bourbaki 序生成元返回 Coxeter 矩阵项：先交换使 $a \le b$；线性图（非 D/E）
按 $b - a$ 分派（0→1；1→按类型 3/4/6，其中 BC 型 $(0,1)$ 与 F 型 $(1,2)$
为 4，G 型为 6；≥2→2）；D/E 型按分叉规则。

## Transducer 与 CompactWeyl

每个 parabolic subquotient 一个 `Transducer`：`offset`、`limit`、
`lengths`、`rights`、平铺的 `table`（entry `< size` 是 shift，
`>= size` 是 transduction，`out = entry - size`）。`CompactWeyl::new(cartan)`
（weyl.cpp:495-547）分三步：分类 Dynkin 图 → 反转 B/C/D 型得内部序 → 为每个
内部生成元构造一个 transducer。`d_out()` 是 internal→external 编号；
`piece_offset(i)` 把 piece 局部字母翻译为全局内部编号。

## canonical_word

`canonical_word(external_word)`（weyl.cpp:944-958 的 `WeylGroup::word`）：
输入是该元素的**任意**词（外部编号，不必 reduced）；经 `inner_mult` 重建元素，
再把 elected piece words 按 piece 递增序拼接、字母经 `d_out` 映射回外部编号。
结果只依赖元素本身，不依赖输入词的选取。`piece_root_permutations` 为每个
(transducer, piece) 预组合一个根置换（simple-reflection 根置换的复合），元素
根置换是它们的复合——不需要矩阵。

## 来源与限制

- 源码：[weyl_transducer.rs](../../crates/atlas-real-group/src/weyl_transducer.rs)；
  阅读快照
  [`2026-10-03-weyl-transducer.json`](snapshots/2026-10-03-weyl-transducer.json)。
- 上游行号均转述自源码注释（weyl.cpp），未独立重读上游，随版本演进可能漂移。
- 关联：[Weyl 群层](weyl-layer.md)、[KGB 图结构](kgb-graph-structure.md)、
  [Inner class 层](inner-class.md)。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 起草经由本地 Kimi probe（无工具 profile，`kimi-code/k3-256k`；exit 0，
  78.1s，420 秒期限）。草案由维护者对照源码逐条核对改写；其「待源码核对」
  项中涉及 `coxeter_entry` 分派、`canonical_word` 流程、Transducer 表编码的
  内容均已按源码落实，其余骨架内容未采用。调用记录见快照的 `kimi_assist`。
