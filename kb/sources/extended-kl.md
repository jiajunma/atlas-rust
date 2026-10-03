---
title: 扩展 KLV 多项式表：primitivisation 符号与逐列存储
source: atlas-rust/extended-kl
ingestedAt: 2026-10-03T10:32:42Z
---

# 扩展 KLV 多项式表：primitivisation 符号与逐列存储

编辑状态：**结构性阅读，草稿经 Kimi probe 起草、维护者逐条对照源码核对后改写**。
本包解释 `ext_kl.rs` 的表结构与 deliberate deviations；扩展 KL 的正确性属于
它自己的 HPC 证据链（unitarity gate 等），本包不重述也不扩展。所读字节见
[`snapshots/2026-10-03-extended-kl.json`](snapshots/2026-10-03-extended-kl.json)
（`ext_kl.rs` SHA-256
`923c34ffcaae013a3ae9e9aad25b1b8daff69457c3d056e1dd1f6fca7543916b`，dirty
工作区）。

## 定位

扩展（twisted）KLV 多项式表，上游 `gkmod/ext_kl.{h,cpp}`；结构上镜像
[KLV 多项式表](kl-polynomial-table.md)，但递归是 extended-block 版本。三个
层次：`DescentTable`（ext_kl.cpp:20-118）预计算 descents/good ascents 与带
primitivisation 符号翻转的 primitive 索引；`ExtKlTable`（ext_kl.cpp:120-841）
按块元素 `y` 各存一列 pool index；模块尾部是 `condense`/`ext_kl_matrix`/
`contributions` 等自由函数。

## 池与符号的分离存储

多项式池复用 `kl_polynomial.rs` 的共享 `KlHashTable`：条目是 `i32` 上的
`KlPol`，恰对应上游 `IntPolEntry = Polynomial<int>`，因此 `KlHashTable`
已经是上游的 `ext_KL_hash_Table = HashTable<IntPolEntry, ext_kl::KLIndex>`
（Atlas.h:478；`KLIndex = unsigned int`，Atlas.h:475）。索引中**不打包符号
位**（上游也不）：primitivisation 符号存于独立的 `prim_flip` bitmap，查询时
另行 consult。`ExtKlTable::kl_pol_index` 返回 `(KLIndex, bool)` 对，
`raw_ext_KL` 包装层将其渲染为 `inx.second ? -inx.first : inx.first`
（atlas-types.w:8713-8714）。

## DescentTable：descent 与 good ascent 的预计算

`DescentTable<'a>`（上游 `ext_kl::descent_table`，ext_kl.h:30-97）持有每元素
的 `descents`/`good_ascents` 两个 `RankFlags`、`prim_index`、`prim_flip` 与
`block: &ExtBlock`。构造（ext_kl.cpp:20-91）对每个 `x`、每个生成元 `s` 调
`block.descent_type(s, x)`：`is_descent()` 置 `descents`；否则若
`!has_double_image()` 置 `good_ascents`（"at most one upward neighbour"）。
`rank > MAX_FOLDED_RANK` 时报 `StructureError::ResourceLimitExceeded`。

`prim_index[mask][x]` 是 `x` 对 descent set `mask` primitivise 后的位置
（ext_kl.h:39）；`prim_flip[x]` 是使 `x` 的 primitivisation 拾取符号的那些
descent sets（ext_kl.h:40；ext_kl.cpp:79-80）。构造对每个 mask 做 `x` 的递减
循环：取 `x` 在 mask 内的第一个 good ascent；遇到 like-nonparity 或跨越
partial-block 边（`some_scent` 为 `None`）记 `DEAD_END`；否则沿 cross 链接
继承下标，并按 `epsilon` 与既有 flip 位的差异置位。收尾把索引反转为递增存储
（ext_kl.cpp:84-87）；整行无 primitive 时保持 `DEAD_END`。

## 查询与回溯接口

- `very_easy_set(x,y) = good_ascents(x) ∩ descents(y)`（ext_kl.h:55-56）；
  `easy_set(x,y) = descents(y) ∖ descents(x)`（ext_kl.h:58-59）。
- `x_index`/`self_index`/`flips`（ext_kl.h:64-68）经 `mask_of` 查表。
- `is_extremal`：`descent_set(x) ⊇ descents(y)`（ext_kl.h:76-77）；
  `is_primitive`：`good_ascent_set(x) ∩ descents(y)` 为空（ext_kl.h:78-79）。
- `length_floor`（ext_kl.h:70-71）与 `col_size`（ext_kl.cpp:94-100）；
  回溯接口 `extr_back_up_mask`/`prim_back_up_mask`（ext_kl.h:82-91）。

## 辅助多项式

`qk_plus_1(k) = 1 + q^k`（ext_kl.cpp:178-184）；`qk_minus_1(k) = q^k - 1`
（:186-192）；`qk_minus_q(k) = q^k - q`（:194-200），均返回 `KlPol`。
`pol_mul_spol` 是带符号 `SPol` 与 `KlPol` 的乘积（ext_kl.cpp:209-212 等）。

## ExtKlTable：列式存储与访问

`ExtKlTable<'a>`（上游 `ext_kl::KL_table`，ext_kl.h:114-190）对每个块元素
`y` 存一列 pool index，按 `x` 对 `y` 的 descent set 的 primitive 位置寻址。
`new`（ext_kl.cpp:120-135）始终使用自有 pool。访问器：`rank`/`size`/
`descent_set`/`descent_type`（返回 `DescValue`）/`l`/`polys`；
`kl_pol_index`（ext_kl.cpp:137-147）返回 `(KLIndex, bool)`；`p(x,y)`
（:149-154）给出 twisted KLV 多项式 $P_{x,y}$（flip 时经 `scaled(-1)`）；
`is_extremal`/`is_primitive`；`nonzero_column`（:156-167）自 `y` 递减列出
非零 `x`；`mu(i,x,y)`（:169-176）取 $q^{(\ell(y/x)-i)/2}$ 的系数
（$i = 1,2,3$）。

## 填充策略与错误处理

`fill_columns(limit)`（ext_kl.cpp:429-448）计算所有 `y < limit` 的列；
`limit == 0` 填满整块。**Deliberate deviation**：出错时清空出错列并传播错误，
而非上游 `catch(...)` 吞掉（ext_kl.cpp:437-444）；列不变式（`column[y]`
要么空要么完整）保持。

## Deliberate deviations 清单

模块文档载明的有意偏离（均不改变可观察结果）：

- `KL_table` 始终拥有自己的 pool；共享池指针模式与 `swallow`
  （ext_kl.cpp:854-894，partial-block 迁移）推迟到需要它们的 common-block
  切片。
- `fill_columns` 传播错误（见上节）。
- `#ifndef NDEBUG` 辅助 `get_M`（ext_kl.cpp:248-347）与 down-set defect 检查
  （:618-637）未移植；`get_Mp` 是生产路径。
- `check_polys`（:917-936）未移植：它需要 `ExtBlock` 不保留的 untwisted
  block。
- `ext_kl_matrix`（:939-1020）在 `ExtBlock` 边界处切分：从 `StandardRepr`
  构建扩展块需要 `StandardReprMod`/`common_block`（后续切片）；此处移植的是
  `eblock` 已存在之后的部分：fill、expand、`condense`、parity 符号翻转与
  pool-index 矩阵。

## 来源与限制

- 源码：[ext_kl.rs](../../../crates/atlas-real-group/src/ext_kl.rs)；阅读快照
  [`2026-10-03-extended-kl.json`](snapshots/2026-10-03-extended-kl.json)。
- 上游行号均转述自源码注释（ext_kl.h/ext_kl.cpp/Atlas.h/atlas-types.w），
  未独立重读上游，随版本演进可能漂移。
- 关联：[扩展块](extended-block.md)、[KLV 多项式](kl-polynomial-table.md)、
  [完整块图](block-graph.md)、[形变驱动](deformation-drivers.md)。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 起草经由本地 Kimi probe（无工具 profile，`kimi-code/k3-256k`；exit 0，
  260.5s，420 秒期限）。草案由维护者对照源码逐条核对改写；其「待源码核对」
  项中涉及池/符号分离、`kl_pol_index` 返回对、`fill_columns` 错误策略的内容
  均已按源码落实，其余骨架内容未采用。调用记录见快照的 `kimi_assist`。
