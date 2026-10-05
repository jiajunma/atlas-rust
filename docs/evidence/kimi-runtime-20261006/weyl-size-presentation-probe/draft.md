```markdown
---
title: Weyl 群阶计算与实形展示层（weyl_size.rs / presentation.rs）知识来源包（草案）
source: atlas-rust/weyl-size-presentation
ingestedAt: 2026-10-06T00:00:00Z
---

# 范围与标记约定

本包仅依据以下两文件的完整字节起草，不引用任何未给出的代码、文档或历史：

- `crates/atlas-real-group/src/weyl_size.rs`
- `crates/atlas-real-group/src/presentation.rs`

标记约定：
- 【实现】= 字节中可直接读到的行为（代码路径、字面量、错误分支）。
- 【文档】= 文件内注释/模块文档中的声明（含上游文件行号引用），本包不核验其真实性。
- 【推断】= 阅读代码得出的推断，未经运行验证；需维护者复核。

本包不做任何数学验收、性能或正确性声明；所有阶数值仅作为「代码中出现的字面量/断言值」记录。

---

## 一、裸签名清单

### 1.1 `weyl_size.rs`

pub / pub(crate) 项（本文件全部 pub 级项，含无注释项）：

| 可见性 | 签名 |
|---|---|
| `pub(crate)` | `fn weyl_order_of_cartan(matrix: &[Vec<i32>]) -> Result<Integer, StructureError>` |

本文件**无** pub/pub(crate) 的 struct / enum / trait / impl 块。

私有辅助项（非 pub，为完整起见单独列出）：

| 可见性 | 签名 |
|---|---|
| 私有 | `fn component_order(matrix: &[Vec<i32>], nodes: &[usize]) -> Result<Integer, StructureError>` |
| 私有 | `fn branch_lengths(matrix: &[Vec<i32>], nodes: &[usize], degrees: &[usize]) -> Result<Vec<usize>, StructureError>` |
| 私有 | `fn factorial(n: usize) -> Result<Integer, StructureError>` |
| 私有 | `fn two_power_factorial(rank: usize) -> Integer` |

导入：`malachite::base::num::basic::traits::One`、`malachite::Integer`、`crate::grading::try_capacity`、`crate::StructureError`。

测试模块 `#[cfg(test)] mod tests`：

| 项 | 签名 |
|---|---|
| 辅助 | `fn chain(rank: usize, tail: (i32, i32)) -> Vec<Vec<i32>>` |
| 测试 | `fn recognizes_the_classical_series_orders()` |
| 测试 | `fn recognizes_the_exceptional_orders_and_torus_factors()` |

### 1.2 `presentation.rs`

pub / pub(crate) 项（本文件全部 pub 级项，含无注释项）：

| 可见性 | 签名 |
|---|---|
| `pub struct` | `RealFormPresentation`（`#[derive(Clone, Debug, Eq, PartialEq)]`） |
| `pub fn` | `build_presentations(inner_class: &InnerClass, classification: &CartanClassification, order: &ExternalFormOrder, layout: &InnerClassLayout, budget: &IntegerLatticeBudget) -> Result<Vec<RealFormPresentation>, StructureError>` |

`RealFormPresentation` 字段（全部 `pub`，文档注释注明「external (interface) numbering」）：

| 字段 | 类型 | 文档注 |
|---|---|---|
| `name` | `String` | `printType` Lie-algebra name |
| `compact` | `bool` | `IsCompact`：most split Cartan involution 为恒等 |
| `connected` | `bool` | `IsConnected`：对偶分量群平凡 |
| `split` | `bool` | `IsSplit`：most split Cartan involution 为负恒等 |
| `quasisplit` | `bool` | `IsQuasisplit`：本 inner class 的 quasisplit 形 |

本文件**无** trait / enum / impl 块；`build_presentations` 内的 `invariant` 闭包（`|reason: &'static str| StructureError::LayoutInvariantViolation { invariant: reason }`）为局部项。

导入：`crate::cartan_classification::CartanClassification`、`crate::error::StructureError`、`crate::form_name::form_type_name`、`crate::inner_class::InnerClass`、`crate::integer_lattice::IntegerLatticeBudget`、`crate::layout::InnerClassLayout`、`crate::real_form_order::ExternalFormOrder`。

测试模块 `#[cfg(test)] mod tests`：

| 项 | 签名 |
|---|---|
| 辅助 | `fn integer_budget() -> IntegerLatticeBudget` |
| 辅助 | `fn presentations(datum: &BasedRootDatum, involution: LatticeInvolution, roots: usize, weyl: usize) -> Vec<RealFormPresentation>` |
| 测试 | `fn simply_connected_a1_presents_compact_then_split()` |
| 测试 | `fn adjoint_a1_split_form_is_disconnected()` |
| 测试 | `fn simply_connected_b2_presents_three_forms()` |

---

## 二、`weyl_size.rs` 逐项解释：Cartan 类型识别与 Weyl 阶计算

### 2.1 模块定位【文档】

- 模块文档：按需 twisted-conjugacy 划分（task #9）将每个轨道大小与上游阶商公式 `|W| / (|W_im| x |W_re| x |W_cx|)`（引用 `cartanclass.cpp:1046-1064`）核对；该核对只需要「被识别 Cartan 矩阵的 Weyl 群的阶」。
- 模块文档：阶采用精确 `Integer` 算术，理由是分量乘积「在 crate 的动态秩范围内很早就会溢出 `u128`」。
- 注：上游行号引用与溢出幅度均为文档声明，本包未核验。

### 2.2 `weyl_order_of_cartan`（唯一 pub(crate) 入口）

输入语义【文档】：Cartan 矩阵条目为 `<alpha_i, alpha_j_vee>`；全零行/列视为环面因子，贡献阶 1。

行为【实现】：
1. `rank = matrix.len()`；逐行检查 `row.len() != rank`，否则返回 `Err(StructureError::NonSquareCartan)`（先于一切计算）。
2. `seen = vec![false; rank]`，`order = Integer::ONE`。
3. 对每个起点 `start`：若 `seen[start]` 或 `matrix[start][start] == 0` 则跳过（零对角行既不作为分量种子，也因其对角为零而永不满足邻居条件）。
4. 连通分量按「非零非对角链接」以 BFS/cursor 扫描构造：`component = try_capacity(rank)?`；邻居 `other` 被加入当且仅当 `!seen[other] && matrix[other][other] != 0 && matrix[node][other] != 0 && node != other`。注意链接方向以**当前节点的行** `matrix[node][other]` 判定。
5. 每个分量 `order *= component_order(matrix, &component)?`；返回 `Ok(order)`。

边界【实现】：全零矩阵（含 rank 0 情形）→ `Ok(Integer::ONE)`；测试中以 2×2 全零矩阵断言 `Integer::ONE`。

### 2.3 `component_order`：按分支形状识别

【实现】流程：

- `rank == 1` → `Ok(Integer::from(2))`（不检查对角值，任何非零对角的单节点分量都返回 2）。
- 对分量内所有无序对 `(a,b)` 计算边重数 `m = C_ij * C_ji`：
  - `matrix[node_a][node_b].checked_mul(matrix[node_b][node_a]).ok_or(StructureError::ArithmeticOverflow)?`（唯一的受检算术）；
  - 乘积非零则 `degrees[a] += 1; degrees[b] += 1`，并更新 `max_multiplicity`（初值 1）。
- 局部别名 `invalid = StructureError::InvalidCartanMatrix`。按 `max_multiplicity` 分派：

| `max_multiplicity` | 分支行为 |
|---|---|
| `3` | `rank == 2` → `Ok(12)`（注释 G2）；否则 `Err(InvalidCartanMatrix)` |
| `2` | 任一 `degree > 2` → `Err(InvalidCartanMatrix)`；`rank == 4` 且存在 `a < b` 使 `degrees[a] == 2 && degrees[b] == 2` 且乘积为 2（「内—内双键」）→ `Ok(1152)`（注释 F4）；否则 `Ok(two_power_factorial(rank))`（注释：B/C 链同为 `2^n n!`） |
| `1` | `forks = #(degree >= 3)`；任一 `degree > 3` 或 `forks > 1` → `Err(InvalidCartanMatrix)`；`forks == 0` → `factorial(rank + 1)`（注释 A_n）；否则取 `branch_lengths(...)?`，`sort_unstable` 后匹配：`[1,1,_]` → `two_power_factorial(rank) / 2`（注释 D_n）；`[1,2,2]` → `Ok(51_840)`（E6）；`[1,2,3]` → `Ok(2_903_040)`（E7）；`[1,2,4]` → `Ok(696_729_600)`（E8）；其余 → `Err(InvalidCartanMatrix)` |
| 其他（含 0 不可能、负值、≥4） | `_ => Err(InvalidCartanMatrix)` |

注【推断】：`max_multiplicity` 初值为 1 且只取 `.max(product)`，故符号不一致的非对角对（乘积为负）会把该边计入度数却把分量当作单连通（multiplicity 1）处理；代码未校验非对角条目的取值集合或符号一致性。

### 2.4 `branch_lengths`

【实现】：`fork = degrees.iter().position(|&d| d == 3)`，无则 `Err(InvalidCartanMatrix)`；`lengths = try_capacity(3)?`。对 fork 的每个邻居（跳过自身、要求 `matrix[nodes[fork]][node] != 0`）从 `length = 1` 起向外走：反复寻找 `candidate != previous && candidate != current && matrix[nodes[current]][c_node] != 0` 的下一个节点，找到则推进并 `length += 1`，找不到则停止。每邻居 push 一个长度；最终 `lengths.len() != 3` → `Err(InvalidCartanMatrix)`。

【推断】行走只排除 `previous`/`current`，无环检测：非树的单连通输入（例如某分支引入环）可能导致该循环不终止而非报错；纯环（所有度数 2、`forks == 0`）会落入 A_n 分支返回 `(rank+1)!`。两者均未被测试覆盖，需复核。

### 2.5 算术辅助

- `factorial(n)`【实现】：`Integer::ONE` 乘上 `2..=n`；签名返回 `Result<_, StructureError>` 但函数体内无任何可失败操作。
- `two_power_factorial(rank)`【实现】：先算 `rank!`（`2..=rank` 连乘），再 `value << rank`（即乘 `2^rank`）；返回 `Integer`，不可失败。D_n 分支用它再整除以 `Integer::from(2)`。

### 2.6 B/C 不敏感说明

- 【文档】模块级注释：B/C 取向无关，因为二者阶同为 `2^n n!`，识别因此「退化为分量拆分 + 分支形状分析」。
- 【实现】代码层面确实不读取 `C_ij`/`C_ji` 中哪一个是 `-2`：双键仅以乘积 `C_ij * C_ji == 2` 进入 `max_multiplicity = 2` 分支，阶统一由 `two_power_factorial(rank)` 给出；rank 4 时仅以「双键是否位于两个度为 2 的内节点之间」区分 F4（1152）与 B4/C4（落入 `2^4·4!`）。
- 【实现】测试锚点：`chain(5, (-2, -1))` 与 `chain(5, (-1, -2))` 均断言为 `Integer::from(3_840)`（注释 B5 / C5）。

### 2.7 错误分支汇总（非测试代码）

| 错误 | 触发点 |
|---|---|
| `StructureError::NonSquareCartan` | 任一行长度 ≠ `matrix.len()` |
| `StructureError::ArithmeticOverflow` | `C_ij.checked_mul(C_ji)` 溢出 `i32` |
| `StructureError::InvalidCartanMatrix` | m=3 且 rank≠2；m=2 且度数>2；m=1 且度数>3 或多叉；无度 3 节点时调用 `branch_lengths`；分支数 ≠ 3；排序后分支长度不匹配 `[1,1,_]/[1,2,2]/[1,2,3]/[1,2,4]`；`max_multiplicity` 其他值 |
| `try_capacity` 的传播错误 | 来自 `crate::grading::try_capacity`，具体变体在本字节中不可见【推断为 `StructureError` 的某个变体】 |

panic/断言【实现】：非测试代码无 `panic!`/`assert!`；索引访问依赖入口处的方阵检查保证不越界。

---

## 三、`presentation.rs` 逐项解释：展示层（Lie 代数名 + 拓扑状态位）

### 3.1 模块定位【文档】

- 模块文档：单个实形的展示数据 = Lie 代数名 + `real_form_value::print` 报告的拓扑状态位（引用 `interpreter/atlas-types.w:3566-3575`）。
- 模块文档：这些位是上游 `RealReductiveGroup::construct` 的状态标志（引用 `realredgp.cpp:68-80`）：`IsCompact`/`IsSplit` 将 most split Cartan involution `ms_tau` 与 `+/-1` 比较；`IsQuasisplit` 将形编号与 quasisplit 形比较；`IsConnected` 是拓扑模块对偶分量群的平凡性。

### 3.2 `RealFormPresentation`

【实现】纯数据结构（derive `Clone, Debug, Eq, PartialEq`），五个 `pub bool/String` 字段，语义见 1.2 表（字段文档即上文模块文档的逐位对应）。

### 3.3 `build_presentations`

【文档】按 external（接口）编号索引，计算一个 inner class 所有实形的展示数据。

【实现】流程：
1. `datum = inner_class.datum()`；`rank = datum.lattice_rank()`。
2. `presentations = Vec::with_capacity(order.form_count())`；按 `external in 0..order.form_count()` 顺序 push，故输出以下标对应 external 形编号（顺序保证）。
3. 每个 `external`：
   - `internal = order.internal(external).ok_or(invariant("external form number"))?`
   - `most_split = classification.most_split(internal).ok_or(invariant("most split Cartan"))?`
   - `cartan = classification.cartan_class(most_split).ok_or(invariant("most split Cartan"))?`（同一 invariant 字符串，两个调用点）
   - `ms_tau = cartan.representative().root_involution().involution().weight_matrix()`
   - 扫描 `ms_tau` 全部条目：`diagonal = i32::from(row == column)`；`compact &= value == diagonal`；`split &= value == -diagonal`（初值均为 `true`）。
   - **之后**才检查 `if ms_tau.len() != rank { return Err(invariant("most split involution rank")); }`（秩检查在 compact/split 扫描之后，仅检查行数，不检查各行长度）。
   - `connected = crate::topology::dual_component_group_trivial(ms_tau, datum, budget)?`（委托拓扑模块，预算由参数 `budget: &IntegerLatticeBudget` 传入；其错误变体在本字节中不可见）。
   - `grading = order.special_grading(external).ok_or(invariant("special grading"))?`
   - push：`name: form_type_name(layout, grading)?`、`compact`、`connected`、`split`、`quasisplit: external == order.quasisplit_external()`。
4. 返回 `Ok(presentations)`。

要点：
- 【实现】`quasisplit` 仅由 external 编号与 `order.quasisplit_external()` 相等判定，不读矩阵。
- 【实现】`compact`/`split` 为纯逐元素比较；空矩阵时循环无操作、两者保持 `true`，随后被秩检查拦截（若 rank ≠ 0）。
- 【实现】错误类型统一为 `StructureError::LayoutInvariantViolation { invariant: reason }`，共四个 reason 字符串：`"external form number"`、`"most split Cartan"`（两处共用）、`"most split involution rank"`、`"special grading"`；另有 `dual_component_group_trivial` 与 `form_type_name` 的 `?` 传播。
- 【实现】非测试代码无 `panic!`/`assert!`。

---

## 四、两文件的接口关系

- 【实现】两份字节互不导入、无直接调用关系；`presentation.rs` 未引用 `weyl_size`，反之亦然。
- 【实现】二者共享错误类型 `StructureError`，但导入路径写法不同：`weyl_size.rs` 用 `crate::StructureError`，`presentation.rs` 用 `crate::error::StructureError`。【推断】同一类型经 crate 根再导出，需核对 `lib.rs`/`error.rs` 确认。
- 【文档】架构层面的关联仅见于模块注释：`weyl_size` 服务于 task #9 的 twisted-conjugacy 划分的阶商校验（`cartanclass.cpp:1046-1064`），`presentation` 服务于 `real_form_value::print` 的展示位（`atlas-types.w:3566-3575`，标志源自 `realredgp.cpp:68-80`）；二者都处于 Cartan 分类流水线周边（`presentation.rs` 直接消费 `CartanClassification`），但 `CartanClassification` 内部是否调用 `weyl_order_of_cartan` 在这两份字节中不可见【不做断言】。

---

## 五、测试锚点

### 5.1 `weyl_size.rs`

辅助 `chain(rank, tail)`：对角全 2，相邻链接 `-1`，末端链接用 `tail = (upper, lower)`。

| 测试 | 断言 | 值（注释标注） |
|---|---|---|
| `recognizes_the_classical_series_orders` | `chain(4, (-1,-1))` | `120`（A4） |
| 同上 | `chain(5, (-2,-1))` | `3_840`（B5） |
| 同上 | `chain(5, (-1,-2))` | `3_840`（C5） |
| 同上 | 手工 D5（边 (0,1),(1,2),(2,3),(2,4) 全 -1，叉在节点 2） | `1_920`（D5） |
| `recognizes_the_exceptional_orders_and_torus_factors` | G2 `[[2,-1],[-3,2]]` | `12` |
| 同上 | 4×4 F4 矩阵（双键在中间两个节点之间） | `1_152` |
| 同上 | 手工 E6，「上游编号（0-2 桥、1-3 桥）」，边 (0,2),(1,3),(2,3),(3,4),(4,5) | `51_840` |
| 同上 | 3×3 `diag(2,0,2)`（A1×A1 夹一行环面） | `4` |
| 同上 | 2×2 全零（空根系） | `Integer::ONE` |

### 5.2 `presentation.rs`

辅助：`integer_budget() = IntegerLatticeBudget::new(64, 100_000, 100_000, 128)`；`presentations(...)` 内部链式构造：`InnerClass::new(datum, involution, roots).unwrap()` → `CartanClassificationBudget::new(integer.clone(), AdjointFiberBudget::new(integer.clone(), 50_000, 100_000), weyl, 64, 64)` → `CartanClassification::build` → `ExternalFormOrder::build` → `InnerClassLayout::build(&inner_class, &integer)` → `build_presentations(..., &integer).unwrap()`。

| 测试 | 关键断言 |
|---|---|
| `simply_connected_a1_presents_compact_then_split` | `BasedRootDatum::from_simple_data(1, [[2]], [Weight([2])], [Coweight([1])])`，恒等 involution，`roots=2, weyl=2`；`forms.len()==2`；`forms[0]`：`"su(2)"`、`compact`、`connected`、`!split`、`!quasisplit`；`forms[1]`：`"sl(2,R)"`、`!compact`、`connected`、`split`、`quasisplit` |
| `adjoint_a1_split_form_is_disconnected` | `BasedRootDatum::standard([[2]])`；`forms.len()==2`；`forms[0]` `"su(2)"` 且 `connected`；`forms[1]` `"sl(2,R)"`、`split`、`!connected` |
| `simply_connected_b2_presents_three_forms` | `from_simple_data(2, [[2,-2],[-1,2]], [Weight([2,-2]),Weight([-1,2])], [Coweight([1,0]),Coweight([0,1])])`，`roots=8, weyl=8`；`forms.len()==3`；`forms[0]` `"so(5)"`、`compact`；`forms[2]` `"so(3,2)"`、`split`、`quasisplit`、`connected`（注释：split 形 Sp(4,R)=so(3,2) 排在最后） |

---

## 六、限制与未覆盖面

`weyl_size.rs`：
1. 【实现】不校验非对角条目是否属于合法 Cartan 取值集合、不校验符号一致性；乘积为负的边计入度数但不提升 `max_multiplicity`（初值 1，只取 max）【推断：符号不一致输入会被按单连通处理】。
2. 【实现】单节点分量对任何非零对角值都返回阶 2（不检查对角是否等于 2）。
3. 【实现】`max_multiplicity == 2` 分支只要求度数 ≤ 2（链状）；rank > 4 时双键位置不被检查，一律返回 `2^rank · rank!`；rank 4 的 F4/B4/C4 区分只看双键是否位于两个内节点之间。
4. 【推断】`branch_lengths` 无环检测：单连通但含环（且恰有一个度 3 节点）的输入可能不终止而非报错；无叉纯环（度数全 2）会落入 A_n 分支。均未见防护或测试。
5. 【实现】仅乘积 `C_ij * C_ji` 用 `checked_mul`（`ArithmeticOverflow`）；度数、分支长度用 `usize` 普通加法。
6. 【实现】`factorial` 签名返回 `Result` 但体内无可失败操作。
7. 未测试项【实现】：所有错误分支（`NonSquareCartan`/`ArithmeticOverflow`/`InvalidCartanMatrix`）、E7/E8 字面量分支、B4/C4（rank 4 非 F4）分支、`try_capacity` 失败路径。
8. 【文档】阶数值与上游行号引用（`cartanclass.cpp:1046-1064`）为注释声明，本包不核验其数学正确性。

`presentation.rs`：
1. 【实现】`ms_tau.len() != rank` 的秩检查位于 compact/split 扫描之后，且只检查行数不检查行长；出错时结果整体被丢弃，无可观察副作用，但顺序值得复核者注意。
2. 【实现】`connected` 完全委托 `crate::topology::dual_component_group_trivial(ms_tau, datum, budget)`；其预算耗尽/失败行为在本字节中不可见。
3. 【实现】`quasisplit` 仅按 external 编号相等判定，语义正确性依赖 `ExternalFormOrder::quasisplit_external()`（不可见）。
4. 未测试项【实现】：非恒等 involution（非 split inner class）、quasisplit 但非 split 的形、四个 `LayoutInvariantViolation` 分支、`form_type_name` 的更广泛命名情形、`IntegerLatticeBudget` 耗尽路径。
5. 【文档】与上游 `realredgp.cpp:68-80`、`atlas-types.w:3566-3575` 的一致性为注释声明，本包不核验。

---

## 七、维护者核对清单（建议逐条确认后收录）

- [ ] `StructureError` 的两条导入路径（`crate::StructureError` vs `crate::error::StructureError`）是否确为同一类型。
- [ ] `try_capacity` 的失败变体及其在 `StructureError` 中的位置。
- [ ] `branch_lengths` 对含环输入不终止的【推断】是否成立，是否需要防护。
- [ ] D_n 阶表达式 `two_power_factorial(rank) / Integer::from(2)` 的整除在所有调用点是否恒为精确整除【推断：是，但未运行验证】。
- [ ] `component_order` 对 rank>4 双键链不检查双键位置是否为本阶段有意留白。
- [ ] `build_presentations` 中秩检查落后于 compact/split 扫描是否有意为之。
- [ ] `dual_component_group_trivial` 与 `form_type_name` 的错误变体集合。
- [ ] 文档中三处上游行号引用（cartanclass.cpp:1046-1064、realredgp.cpp:68-80、atlas-types.w:3566-3575）是否与当前上游快照一致。
```