```markdown
---
title: dynkin.rs：Cartan 矩阵的 Dynkin 连通分量分类、Bourbaki 置换与折叠 Cartan
source: atlas-rust/dynkin
ingestedAt: 2026-10-05T18:00:00Z
---

# dynkin.rs 知识来源包（草案）

> 状态：**草案**，待维护者逐条核对后收录。
> 依据：仅 `crates/atlas-real-group/src/dynkin.rs` 给出的完整字节。
> 标注约定：【实现】= 代码/注释中可直接读到的事实；【推断】= 阅读代码得出的推断，需维护者确认。本 packet 不做任何数学验收、性能或正确性声明。

## 0. 模块定位（来自模块文档注释）

【实现】模块头注释声明：本文件移植自 upstream `DynkinDiagram`（`structure/dynkin.cpp:28-220`），功能是求有限 Cartan 矩阵的连通分量，并按各分量的 Bourbaki 顶点序分类。分量顺序与每个分量的 `position` 向量"精确复现"upstream `type()`/`perm()`，包括其平局选择（tie choices）：
- 秩二 B/C 由给定顺序决定；
- A 型与 D4 从任一端开始；
- E 型的长臂交换（long-arm swap）。

【推断】上述"精确复现 upstream"是注释中的意图声明，本 packet 不验证该等价性。

## 1. 裸签名清单

### 1.1 pub / pub(crate) 项

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DynkinComponent {
    /// The simple type letter: A, B, C, D, E, F, or G.
    pub(crate) letter: char,
    /// Datum vertex indices of this component.
    pub(crate) support: BTreeSet<usize>,
    /// The component's vertices in Bourbaki order for its type.
    pub(crate) position: Vec<usize>,
}

impl DynkinComponent {
    /// The lowest datum vertex of this component (upstream `offset`).
    pub(crate) fn offset(&self) -> usize;
}

/// Classify a validated Cartan matrix into connected typed components.
pub(crate) fn classify(cartan: &[Vec<i32>]) -> Result<Vec<DynkinComponent>, StructureError>;

/// The permutation from the datum's simple-root order to the Bourbaki
/// order of the classified type (upstream `DynkinDiagram::perm()`,
/// structure/dynkin.cpp:289-295).
pub fn bourbaki_permutation(cartan: &[Vec<i32>]) -> Result<Vec<usize>, StructureError>;

/// The Cartan matrix of the diagram folded by a `delta`-orbit list
/// (upstream `DynkinDiagram::folded`, structure/dynkin.cpp:222-261,
/// here via the `cofold` formulas of structure/rootdata.cpp:1578-1604).
pub(crate) fn folded_cartan(
    cartan: &[Vec<i32>],
    orbits: &[crate::ext_block::ExtGen],
) -> Result<Vec<Vec<i32>>, StructureError>;
```

### 1.2 模块私有项（非 pub，随文件列出以便核对）

```rust
fn first(set: &BTreeSet<usize>) -> usize;

/// Per-component port of `DynkinDiagram::classify(Cartan, comp)`.
fn classify_component(
    cartan: &[Vec<i32>],
    star: &[BTreeSet<usize>],
    down_edges: &[(usize, usize, i32)],
    support: &BTreeSet<usize>,
) -> Result<DynkinComponent, StructureError>;

#[cfg(test)]
mod tests {
    fn letters(comps: &[DynkinComponent]) -> String;        // 拼接各分量 letter
    fn positions(comps: &[DynkinComponent]) -> Vec<usize>;  // 拼接各分量 position
    // 6 个 #[test]，见 §7
}
```

### 1.3 可见性观察

- 【实现】唯一 `pub`（crate 外可见）项是 `bourbaki_permutation`。
- 【实现】`DynkinComponent`、`classify`、`folded_cartan` 均为 `pub(crate)`。
- 【实现】`first`、`classify_component` 为模块私有。
- 【实现】`use std::collections::BTreeSet; use crate::StructureError;`；`StructureError` 的定义不在本文件内。

## 2. 类型与约定

### 2.1 `DynkinComponent`
- `letter: char`：单型字母，取值为 `'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G'`（注释声明）。
- `support: BTreeSet<usize>`：该分量在 datum 中的顶点下标集合（有序、去重由 `BTreeSet` 保证）。
- `position: Vec<usize>`：该分量顶点按其类型的 Bourbaki 顺序排列的 datum 下标。
- 【实现】派生 `Clone, Debug, Eq, PartialEq`。

### 2.2 `offset()`
- 【实现】返回 `support` 的最小元素（`*self.support.iter().next().expect("nonempty Dynkin support")`）；空集合会 panic。注释标明对应 upstream `offset`。
- 【实现】本文件内未见 `offset()` 的调用点。【推断】供 crate 内其他模块使用。

### 2.3 Cartan 约定
- 【实现】`folded_cartan` 文档注释明确 crate 约定：`cartan[i][j]` 是 `<alpha_i, alpha_j^v>`。
- 【实现】`classify` 文档注释：输入"必须满足 based-datum Cartan 不变量（对角线为 2、非正对角外邻接对称）"；违规以 layout invariant 错误报告，"因为调用方从已检查的数据构造"。

## 3. `classify`：输入契约与分量顺序

### 3.1 输入契约（按检查出现顺序）

【实现】对 `cartan: &[Vec<i32>]` 依次强制：
1. 方形：任一行 `row.len() != rank`（`rank = cartan.len()`）→ `Err(StructureError::NonSquareCartan)`。
2. 对角线：`cartan[i][i] != 2` → `LayoutInvariantViolation { invariant: "Cartan diagonal" }`。
3. 非对角线取值：entry 不在 `-3..=0` → `LayoutInvariantViolation { invariant: "Cartan off-diagonal" }`。
4. 零模式对称：`cartan[i][j] != 0` 而 `cartan[j][i] == 0` → `LayoutInvariantViolation { invariant: "Cartan adjacency symmetry" }`。

注意：第 4 条只检查零/非零模式对称，不检查数值配对（例如两个方向都为 `-2` 在此处不报错，见 §6）。
【实现】空输入（`rank == 0`）全部检查空真通过，返回 `Ok(vec![])`（测试 `bourbaki_permutation_is_trivial_on_canonical_rank_two` 锚定空排列结果）。

### 3.2 邻接与多重边的构建

【实现】（注释称"mirroring the upstream ctor"）：
- `star: Vec<BTreeSet<usize>>`：按行主序扫描，若 `cartan[i][j] != 0`（`i != j`）则 `star[j].insert(i)`。即 `star[v]` 是 `v` 的全部邻接点（依赖第 4 条对称性）。
- `down_edges: Vec<(usize, usize, i32)>`：若 `entry < -1`（即 `entry ∈ {-2, -3}`），压入 `(i, j, -entry)`。第三个分量是边标号（重数），取值为 `2` 或 `3`。
- 【实现】`down_edges` 按行主序收集**有序对**；若同一对顶点两个方向都 `< -1`，会压入两条记录（后续在 `classify_component` 中触发 `"multiple labelled edges"`，秩二时先被乘积检查拦截）。

### 3.3 分量划分与顺序保证

【实现】注释原话："Components, in upstream first-fresh-vertex order with first-match merging." 算法：
- 按 `i = 0..rank` 升序处理；若 `star[i]` 的所有邻居都 `> i`（空集空真成立，孤立顶点成为单点分量），新建分量 `{i}`。
- 否则找**第一个**与 `star[i]` 相交的已有分量 `first_match`，把 `i` 插入其中；再从 `first_match + 1` 起扫描，把所有与 `star[i]` 相交的后续分量移除（`comps.remove`）并合并进 `first_match`。合并未命中时返回 `LayoutInvariantViolation { invariant: "component merge" }`。
- 合并判据是"与 `star[i]` 相交"，而不是"与合并后的分量相交"。
- 【推断】分量在 `comps` 中的先后次序等于各分量最小顶点的升序（顶点按升序处理、合并保留 `first_match` 槽位）。注释声称这与 upstream 的分量顺序一致。
- 【推断】`"component merge"` 分支在通过 §3.1 第 4 条对称性检查的输入下不可达（较小邻居 `j < i` 必已落入某分量且与 `star[i]` 相交），属防御性返回；需维护者确认。

### 3.4 输出

【实现】`comps.iter().map(classify_component(...)).collect()`：对每个分量调用 `classify_component`，任一失败则整体 `Err`。返回值顺序即 §3.3 的 `comps` 顺序。

## 4. `classify_component`：单分量的类型判定与 Bourbaki 序

### 4.1 秩 ≤ 2 的特判

【实现】
- 秩 1：`letter = 'A'`，`position = vec![first(support)]`（`first` 取 `BTreeSet` 最小元）。
- 秩 2：按 `support` 升序取 `(i, j)`（`i < j`），`position` 初始 `[i, j]`。按 `cartan[i][j] * cartan[j][i]` 分派：
  - `1` → `'A'`（顺序不变）。
  - `2` → 若 `cartan[i][j] == -1` 则 `'C'`，否则 `'B'`。**顺序不变**。注释："Exceptionally the given order decides the type (dynkin.cpp:113)." —— 即 B2/C2 的歧义由 datum 给定顺序消解：较小下标顶点那一行的非对角元为 `-2` 时判 `'B'`，为 `-1` 时判 `'C'`。
  - `3` → 若 `cartan[i][j] != -1`（即为 `-3`）则 `position.swap(0, 1)`，判 `'G'`。测试注释："G2 swaps to put the short root first."
  - 其他乘积（`0, 4, 6, 9` 等）→ `LayoutInvariantViolation { invariant: "rank-two Cartan product" }`。

### 4.2 秩 > 2：度分析

【实现】对 `support` 中每个 `i`，按 `star[i].len()`（整个图的度）分派：
- `0 | 1` → 记入 `extremities`（端点集）。
- `2` → 通过。
- `3` → 记入 `fork`；若已有 `fork` → `"multiple fork nodes"`。
- `≥ 4` → `"node degree above three"`。

随后：若 `extremities.len() < 2` → `"diagram loop"`。

### 4.3 多重边检查

【实现】遍历 `down_edges`，跳过 `!support.contains(&i)` 的边（注意：只检查第一个端点 `i` 是否在分量内）：
- `label == 3` → `"oversized type G diagram"`（即 G2 型边只允许出现在秩 ≤ 2 分量）。
- 第一条合法边记 `upper = Some(i)`、`lower = Some(j)`；第二条 → `"multiple labelled edges"`。

### 4.4 字母判定（秩 > 2）

【实现】
- 若存在多重边（`upper.is_some()`）：
  - 同时有 `fork` → `"fork and labelled edge"`。
  - `lower ∈ extremities` → `'B'`；
  - 否则 `upper ∈ extremities` → `'C'`；
  - 否则 → `'F'`。
- 若无多重边：
  - 无 `fork` → `'A'`；
  - 有 `fork`：若 `star[fork] ∩ extremities` 计数 `== 1` → `'E'`，否则 → `'D'`。（【实现】即 D/E 的区分只看"与 fork 相邻的端点数是否为 1"；计数为 0、2、3 都判 `'D'`。）

### 4.5 起点（`start`）选择与各型特判

【实现】`remain = support.clone()`，`position` 预分配 `comp_rank`。按字母：
- `'A'`：`start = first(&extremities)`（最小下标端点；对应模块注释的 "any-end starts for A"）。
- `'B'`：从 `extremities` 移除 `lower.expect("type B edge")`，取剩余最小端点。
- `'C'`：对称地移除 `upper.expect("type C edge")`。
- `'D'`：
  - `comp_rank == 4`：`first(&extremities)`（"any-end start for D4"）。
  - 否则：从 `extremities` 移除 `star[fork.expect("type D fork")]` 的全部元素；若剩余 `!= 1` → `"fork node without adjacent extremities"`；`start` 为唯一剩余端点（长臂末端）。
- `'E'`：
  - `comp_rank > 8` → `"oversized type E diagram"`。
  - `short_arm = extremities ∩ star[fork]`；大小 `!= 1` → `"type E short arm"`。
  - `arm = extremities \ short_arm` 的最小元（`expect("a longer arm exists")`）。
  - 若 `star[arm]` 与 `star[fork]` 不相交（长臂端点离 fork 超过 2 步）：视为最长臂，执行**长臂交换**——从 `extremities` 移除 `arm`，改取剩余最小元（注释："swap for the orthogonal long arm"）。
  - 交换后仍不相交 → `"fork node with two too long arms"`。
  - `position` 先压入 `[arm, first(&short_arm)]` 并从 `remain` 移除；`start = star[arm] ∩ star[fork]` 的（唯一）公共邻居（`expect("type E common neighbour")`）。即 E 型 `position` 前两项是"正交长臂端点、短臂端点"，随后从长臂上距 fork 两步的顶点继续遍历。
- `'F'`：`comp_rank > 4` → `"oversized type F diagram"`；从 `extremities` 移除 `star[lower.expect("type F edge")]` 的全部元素，取剩余最小元。
- 其他字母 → `"component letter"`（【推断】不可达的防御分支：`letter` 只可能由 §4.1/§4.4 赋为 A–G）。

### 4.6 遍历与其余检查

【实现】注释："Traverse the remainder of the diagram starting from |start|." 循环：
1. `position.push(start)`，`remain.remove(&start)`；`remain` 空则退出。
2. `candidate = star[start] ∩ remain`；非空则 `start = first(&candidate)`（最小下标邻居）。
3. 若空且 `letter == 'D'`：要求 `remain ⊆ star[fork]`，否则 `"type D traversal"`；`start = first(&remain)`（即 fork 短臂按升序逐个追加）。
4. 若空且非 `'D'` → `"component traversal"`。

循环结束后：`position.len() != comp_rank` → `"component position count"`。成功返回 `DynkinComponent { letter, support: support.clone(), position }`。

## 5. `bourbaki_permutation` 与 `folded_cartan`

### 5.1 `bourbaki_permutation`（pub）

- 【实现】`Ok(classify(cartan)?.iter().flat_map(|c| c.position.iter().copied()).collect())`：各分量 `position` 的顺序拼接。
- 【实现】文档注释：这是"从 datum 单根序到所分类型的 Bourbaki 序"的置换（upstream `DynkinDiagram::perm()`，`structure/dynkin.cpp:289-295`），`result[i]` 是占据 Bourbaki 位置 `i` 的 datum 顶点；语言层需要它来按 Bourbaki 编号打印 grading。
- 【实现】错误完全来自 `classify`（`?` 透传）。

### 5.2 `folded_cartan`（pub(crate)）

- 【实现】文档注释：对应 upstream `DynkinDiagram::folded`（`structure/dynkin.cpp:222-261`），但此处经由 `structure/rootdata.cpp:1578-1604` 的 `cofold` Cartan 公式计算，注释声称两者给出相同的边重数。折叠后的单根是轨道成员之和；折叠单余根：成员可交换的轨道（长度 2）取第一个成员的余根，不可交换的（长度 3）取两个余根之和。折叠项 `C(i,j)` 是把 `cartan[a][b]` 对"轨道 `j` 的折叠根 `a` × 轨道 `i` 的折叠余根 `b`"求和。
- 【实现】签名：`orbits: &[crate::ext_block::ExtGen]`；用到 `ExtGen` 的字段 `s0`、`s1` 与 `kind: crate::ext_block::ExtGenKind`。本文件中只出现 `ExtGenKind::One` 与 `ExtGenKind::Three` 两个名字；"长度 2"情形由 `kind != One` / `kind == Three` 的分支隐含覆盖（`ExtGenKind::Two` 字样未在本文件出现）。
- 【实现】实现细节：
  - 输出 `orbits.len() × orbits.len()` 的零初始化矩阵。
  - 轨道 `i` 的余根支撑：`[s0]`，若 `kind == Three` 追加 `s1`。
  - 轨道 `j` 的根支撑：`[s0]`，若 `kind != One` 追加 `s1`。
  - 双重循环求和 `entry += cartan[a][b]`。
- 【实现】边界检查：`a >= rank || b >= rank`（`rank = cartan.len()`）→ `Err(StructureError::IndexOutOfRange { index: a.max(b), upper_bound: rank })`（`index` 取较大的越界下标）。
- 【实现】不检查 `orbits` 是否两两不交、是否覆盖全部顶点；不检查结果是否为合法 Cartan 矩阵；空 `orbits` 返回空的 `Ok(vec![])`。【推断】假定 `cartan` 为方形（只按 `cartan.len()` 查行数界；行长短于 `rank` 的非方形输入在 `cartan[a][b]` 处会 panic，本函数无防护）。

## 6. 错误路径汇总

### 6.1 `StructureError` 变体（定义不在本文件；字段从构造语法读得）

| 变体 | 载荷 |
|---|---|
| `NonSquareCartan` | 无 |
| `LayoutInvariantViolation { invariant: &'static str }` | 不变量名 |
| `IndexOutOfRange { index: usize, upper_bound: usize }` | 越界下标与上界（字段类型按用法推断为 `usize`） |

### 6.2 错误位点表

| # | 位点 | 变体 / invariant 字符串 | 触发条件 |
|---|---|---|---|
| 1 | `classify` | `NonSquareCartan` | 某行长度 ≠ `rank` |
| 2 | `classify` | `"Cartan diagonal"` | 对角元 ≠ 2 |
| 3 | `classify` | `"Cartan off-diagonal"` | 非对角元 ∉ `-3..=0` |
| 4 | `classify` | `"Cartan adjacency symmetry"` | `cartan[i][j] != 0` 且 `cartan[j][i] == 0` |
| 5 | `classify` | `"component merge"` | 无已有分量与 `star[i]` 相交（【推断】前置检查下不可达） |
| 6 | `classify_component` | `"rank-two Cartan product"` | 秩二分量的非对角乘积 ∉ {1,2,3} |
| 7 | 同上 | `"multiple fork nodes"` | 第二个度-3 顶点 |
| 8 | 同上 | `"node degree above three"` | 度 ≥ 4 |
| 9 | 同上 | `"diagram loop"` | 端点数 < 2 |
| 10 | 同上 | `"oversized type G diagram"` | 秩 > 2 分量含 label 3 的边 |
| 11 | 同上 | `"multiple labelled edges"` | 分量内第二条多重边 |
| 12 | 同上 | `"fork and labelled edge"` | fork 与多重边并存 |
| 13 | 同上 | `"fork node without adjacent extremities"` | D 型（秩 > 4）移除 fork 邻居后端点数 ≠ 1 |
| 14 | 同上 | `"oversized type E diagram"` | E 型 `comp_rank > 8` |
| 15 | 同上 | `"type E short arm"` | 短臂（端点 ∩ fork 邻居）大小 ≠ 1 |
| 16 | 同上 | `"fork node with two too long arms"` | 长臂交换后仍与 fork 邻域不交 |
| 17 | 同上 | `"oversized type F diagram"` | F 型 `comp_rank > 4` |
| 18 | 同上 | `"component letter"` | 起点分派遇到非 A–G 字母（【推断】不可达） |
| 19 | 同上 | `"type D traversal"` | D 型遍历中断且 `remain ⊄ star[fork]` |
| 20 | 同上 | `"component traversal"` | 非 D 型遍历中断 |
| 21 | 同上 | `"component position count"` | `position.len() != comp_rank` |
| 22 | `folded_cartan` | `IndexOutOfRange { index: a.max(b), upper_bound: rank }` | 轨道下标 `a` 或 `b` ≥ `rank` |

### 6.3 panic / 断言点（全部为 `expect`，【推断】均为"按构造不可达"的防御）

- `first()`：`"Dynkin vertex sets are nonempty by construction"`。
- `offset()`：`"nonempty Dynkin support"`。
- `classify_component` 内：`"rank two"`（×2）、`"labelled edge"`、`"type B edge"`、`"type C edge"`、`"type D fork"`（多处）、`"type E fork"`、`"a longer arm exists"`、`"type E common neighbour"`、`"type F edge"`。

## 7. 测试锚点（`#[cfg(test)] mod tests`）

辅助：`letters` 拼接各分量 `letter` 成 `String`；`positions` 拼接各分量 `position`。

1. **`classifies_single_factors_in_bourbaki_order`** — 锚定：
   - A1 `[[2]]` → `"A"`，`[0]`；
   - A2 → `"A"`，`[0,1]`；
   - B2（`cartan[0][1] = -2`）→ `"B"`，`[0,1]`（注释："Cartan(i,j) == -2 with i < j stays type B in the given order"）；
   - C2（`cartan[0][1] = -1, cartan[1][0] = -2`）→ `"C"`，`[0,1]`；
   - G2（`cartan[0][1] = -3`）→ `"G"`，`[1,0]`（交换，短根在前）；
   - G2 另一指向（`cartan[1][0] = -3`）→ `"G"`，`[0,1]`。
2. **`classifies_forked_and_exceptional_diagrams`** — 锚定：
   - D4 标准形 → `"D"`，`[0,1,2,3]`；
   - E6（注释："Bourbaki order: 1-3-4-5-6 chain, 2 attached to 4"，以 0 基边 `(0,2),(2,3),(3,4),(4,5),(1,3)` 构造）→ `"E"`，`[0,1,2,3,4,5]`；
   - F4（注释："double edge between vertices 1 and 2"，`cartan[1][2] = -2`）→ `"F"`，`[0,1,2,3]`。
3. **`straightens_permuted_components`** — B3 标准形经置换 `[2, 0, 1]` 重标号后分类为 `"B"`，且 Bourbaki 置换 `pi` 满足 `relabeled[pi[i]][pi[j]] == canonical[i][j]`（注释："The Bourbaki permutation reconstructs the canonical matrix."）。
4. **`splits_disconnected_components_in_first_vertex_order`** — 块对角 A1+A2 → `"AA"`，`positions == [0,1,2]`。
5. **`bourbaki_permutation_straightens_permuted_diagrams`** — A3 经 `[1, 0, 2]` 重标号（顶点 0 变成链中点）→ 置换 `[1, 0, 2]`；D4 经 `[0, 2, 1, 3]` 重标号（fork 从顶点 1 移到 2；注释："triality makes extremity relabeling invisible"）→ 置换 `[0, 2, 1, 3]`。
6. **`bourbaki_permutation_is_trivial_on_canonical_rank_two`** — 标准 B2、C2 → `[0, 1]`（注释："no visible diagram automorphism"）；空矩阵 → 空置换。

【推断】测试中没有任何针对 §6 错误路径的用例；所有测试均为成功路径。

## 8. 限制与未覆盖面

- 【实现】`classify` 文档自述：输入必须是"已验证的" based-datum Cartan；本函数只做结构性检查（方形、对角 2、非对角 ∈ `-3..=0`、零模式对称），其余畸形以各 `LayoutInvariantViolation` 字符串报告，并非独立校验器。
- 【实现】对称性检查不校验多重边两个方向的数值配对；双向都 `< -1` 的输入在秩二落入 `"rank-two Cartan product"`，在秩 > 2 落入 `"multiple labelled edges"`。
- 【实现】规模上界检查仅：E `comp_rank > 8`、F `comp_rank > 4`、G（label 3 边）秩 > 2 拒绝；A、B、C、D 无显式 rank 上界；E 无显式 rank 下界检查。
- 【实现】所有"任选端点/最小邻居"均通过 `BTreeSet` 最小元（`first`）解析，因此结果依赖于 datum 顶点编号；模块注释声称这些 tie 选择与 upstream 一致（本 packet 不验证）。
- 【实现】`folded_cartan` 不校验轨道的完整性/不交性/覆盖性，不校验 `kind` 的全部取值（只区分 `One` / `Three` / 其他），不校验输出是合法 Cartan；对 `cartan` 仅做下标上界检查。
- 【实现】`offset()` 在本文件内无调用点。
- 【实现】`DynkinComponent`、`classify` 为 `pub(crate)`，crate 外唯一入口是 `bourbaki_permutation`。
- 【推断】`"component merge"`、`"component letter"` 及各 `expect` 分支在通过前置检查的输入下不可达；建议维护者确认是否按"防御性分支"收录。
- 【推断】错误路径无测试覆盖；E6/F4/D4 之外的例外型（E7、E8）与秩 > 4 的 D 在测试中出现与否：未出现，建议维护者评估是否需要补充锚点。

## 9. upstream 对应锚点（仅转述文件内注释）

- 整体移植：`structure/dynkin.cpp:28-220`（`DynkinDiagram`）。
- 秩二 B/C 由给定顺序决定：`dynkin.cpp:113`。
- `offset()`：对应 upstream `offset`。
- `bourbaki_permutation`：对应 `DynkinDiagram::perm()`，`structure/dynkin.cpp:289-295`。
- `folded_cartan`：对应 `DynkinDiagram::folded`，`structure/dynkin.cpp:222-261`；计算改用 `structure/rootdata.cpp:1578-1604` 的 `cofold` 公式（注释声称边重数相同，未验证）。
```