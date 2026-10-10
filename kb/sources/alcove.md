---
title: Alcove 几何：alcove_center 与 root_vertex_of_alcove
source: atlas-rust/alcove
ingestedAt: 2026-10-05T18:45:00Z
---

# Alcove 几何：alcove_center 与 root_vertex_of_alcove

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `crates/atlas-real-group/src/alcove.rs`（885 行）。它是结构性阅读，
不声称 alcove 计算的数学验收；上游引用（alcoves.cpp:277-341/347-412/414-428/
29-32/395-408、matrix.cpp:471-503、innerclass.cpp:1123）仅转录自代码文档
注释，本包未核对上游字节。

## 概览

模块拥有 `weyl::alcove_center`（alcoves.cpp:277-341）的领域级形式，服务于
deformation 预处理与整值 datum 规范化：保留标准参数的 KGB 坐标与
`lambda_rho`，仅替换其 infinitesimal character。`root_vertex_of_alcove`
（alcoves.cpp:414-428）服务于 locator 切片的基本 alcove 约化（上游
`InnerClass::int_item` 的步骤 (a)）。对外 `pub` 仅 `alcove_center` 与
`denominator_exceeds_alcove_bound`；`RootNumbering`、`wall_set`、
`checked_dot`、`root_components`、`root_vertex_of_alcove` 为 `pub(crate)`；
其余均为文件内私有。无 trait/enum 定义，唯一 impl 块属于 `RootNumbering`。

## 裸签名清单（完整）

```rust
pub fn alcove_center(rc: &RepContext<'_>, z: &StandardRepr)
    -> Result<StandardRepr, StructureError>
pub fn denominator_exceeds_alcove_bound(rank: usize, denominator: i64) -> bool

pub(crate) struct RootNumbering { npos: usize, by_nbr: Vec<RootId> }  // 字段私有
impl RootNumbering {
    pub(crate) fn new(root_system: &RootSystem) -> Self
    pub(crate) fn id(&self, nbr: usize) -> RootId
    fn is_negative(&self, nbr: usize) -> bool
}
pub(crate) fn wall_set(&RootSystem, &RootNumbering, &RationalWeight)
    -> Result<(BTreeSet<usize>, BTreeSet<usize>), StructureError>
pub(crate) fn checked_dot(left: &[i64], right: &[i32]) -> Result<i64, StructureError>
pub(crate) fn root_components(&RootSystem, &RootNumbering, &BTreeSet<usize>)
    -> Vec<Vec<usize>>
pub(crate) fn root_vertex_of_alcove(&RootSystem, &RationalWeight)
    -> Result<Weight, StructureError>

fn frac_eval_value(..) -> Result<(i64, i64), StructureError>
fn floor_eval_nbr(..) -> Result<i64, StructureError>
fn barycentre_eq(..) -> Result<Vec<(i64, i64)>, StructureError>
fn labels_for_component(..) -> Result<Vec<i64>, StructureError>
fn solve_rational_system(rows: &[(Vec<i64>, i64)], columns: usize)
    -> Option<Vec<Rational>>
fn gcd(mut left: i64, mut right: i64) -> i64
fn checked_lcm(left: i64, right: i64) -> Result<i64, StructureError>
fn root_vertex_simple(..) -> Result<Weight, StructureError>
fn matrix_vector_product(..) -> Result<Vec<i64>, StructureError>
fn rational_inverse(&[Vec<i64>]) -> Result<Option<(Vec<Vec<i64>>, i64)>, StructureError>
```

`root_components` 内部嵌套带路径压缩的并查集 `find`。测试模块仅两个用例
（分母界边界、不自洽超定方程组）。

## alcove_center：契约与流程

返回包含 `z.gamma()` 的 alcove 的重心；KGB 坐标与 `lambda_rho` 保留；返回
参数经 `RepContext::sr_gamma` 重建，打包扭数据与派生高度保持 canonical。

流程（顺序固定）：

1. `RootNumbering::new` + `wall_set` 得 `(walls, integrals)`，
   `barycentre_eq` 得逐墙分数 `fracs: Vec<(i64, i64)>`。
2. 组装线性方程组：每墙一行——系数为 coroot 各坐标 `checked_mul(scale)`，
   右端为 `floor_eval_nbr * scale + fracs.0`（全程 checked，溢出
   `ArithmeticOverflow`）；再对 `datum.radical_basis()?` 的每个元素加一行
   （系数乘 `gamma.denominator()`，右端为与 `gamma.numerator()` 的 checked
   点积）。
3. `solve_rational_system(&rows, rank)` 返回 `None` →
   `RepInvariantViolation { invariant: "alcove center equations have no unique
   solution" }`（不自洽或解不唯一同此错误）。
4. 通分：各分量分母经 `checked_lcm` 累乘；`entry * denominator` 用
   `i64::try_from(&scaled)`（对整个有理数的精确转换）转整——代码注释明确：
   单用 `numerator_ref` 会把负的 alcove 中心变正（signed-rational 教训的
   落点之一）。
5. **-θ 不动子空间校验**（alcoves.cpp:317-321：修正可能不落在参数连续坐标
   所在的 -θ 不动子空间）：`theta = rc.theta(z)?`，逐行计算并验证
   **`(I+θ)·num(Δ) = 0`**（Δ = `centered_gamma - gamma`，在分子坐标上）：
   `try_fold` 的初始值是 `difference.numerator()[row]`，再逐项累加
   `θ[row][j]·Δ[j]`，所以总量是 `((I+θ)Δ)[row]` 而非裸的 `(θΔ)[row]`；
   分母为正，故分子条件与 Δ 本身等价。这正是错误文本所命名的条件：
   (I+θ)Δ=0 ⟺ θΔ=−Δ ⟺ Δ 在 θ 的 (−1)-特征空间 = −θ 的不动子空间；
   连续坐标（环面因子侧）恰落在该特征空间，因此 alcove 内的居中修正
   不得离开它。否则 `RepInvariantViolation { invariant: "alcove
   correction lies outside the -theta fixed subspace" }`。
   **更正记录（2026-10-10）**：本包此前把该检查写成「θ·Δ 的乘积为零」，
   漏读了 `try_fold` 的非零初始项；核对源码后确认实现与错误命名一致，
   无缺陷。教训：fold 的 init 项属于语义本体，阅读时不得省略。
6. `rc.sr_gamma(z.x(), &lambda_rho, &centered_gamma)` 收尾。

## denominator_exceeds_alcove_bound

上游 deformation 分母守卫 `denominator > 2^rank`：
`rank < i64::BITS as usize - 1 && denominator > (1_i64 << rank)`。rank ≥ 63
时数学阈值超出正 `i64` 范围，且对有符号 `i64` 左移 63 位会得到
`i64::MIN`，故 rank ≥ 63 一律 `false`。单元测试固定 rank 62 的
`2^62`/`2^62+1` 边界与 rank 63/64 在 `i64::MAX` 下的 `false`。

## RootNumbering：正根排序与负根镜像编号

- 收集 `is_positive(id).unwrap_or(false)` 为真的根为正根，排序键：先按
  level（简单坐标分量之和）升序，同 level 按「从最后一个坐标向前」的逐坐标
  差比较（反向字典序，镜像上游编号）；缺失坐标一律 `unwrap_or(&[])`。
- 正根占编号段 `[npos, total)`（按排序序），负根占 `[0, npos)`：
  负根的简单坐标取负后在 `positive_index: BTreeMap<Vec<i32>, usize>` 中直接
  索引（**缺键即 panic**，构造器返回 `Self` 而非 `Result`），
  `by_nbr[npos - 1 - position] = id`，与对应正根顺序相反。
- 该方案隐含 `total == 2 * npos`；`id(nbr)` 越界同样下标 panic。对
  `is_positive`/`simple_coordinates` 的失败一律 `unwrap_or` 静默兜底，与本
  文件他处的 `ok_or(...)?` 风格不同（阅读观察，是否刻意未确认）。

## wall_set：alcove 墙集与整值墙

1. `coroot_table`：全部根的 coroot 坐标向量 `BTreeSet`（缺失 coroot 的根被
   `filter_map` 跳过）。
2. 每根经 `frac_eval_value` 得 `(remainder, denominator)`：朴素
   `rem_euclid`，但**负根整值时改写为 denominator**（正根整值取 0）——负根
   的「整值」落在开区间约定之外。
3. 外层循环：取最小 level（元组字典序），稳定排序把最小层移到前段；内层
   逐个取出最小层根 α：整值（remainder 0）则同时入 `integrals`（恒有
   `integrals ⊆ walls`），一律入 `walls`；随后 drain 剩余层——若
   `α∨ − β∨`（逐分量差）**是**某个 coroot（在 `coroot_table` 中）则丢弃 β，
   且若 β 同属最小层则 `n_min` 做 `saturating_sub`；否则保留 β。
   即：每层只保留不能被已选墙「减去一个 coroot」到达的根。

## root_components 与 barycentre_eq

- `root_components`：并查集按 `bracket(α, β) != 0` 合并墙（`bracket` 失败
  被 `unwrap_or(0)` 静默视为不相连——与 `root_vertex_simple` 中 `bracket`
  的 `?` 传播不同）；分量按根首次出现顺序追加，分量内按编号升序。不返回
  `Result`，无错误分支。
- `barycentre_eq`：结果初值 `(0, 1)`；对每个分量求 `labels_for_component`
  （见下），只改写非整值墙：`result[slot] = (1, n_off * labels[position])`。
  整值墙保持 `(0, 1)` 不被改写。

## labels_for_component：分量的本原 coroot 关系

以分量各墙 coroot 为列组 `Rational` 矩阵，Gauss-Jordan 全消元（含主元行
上方），自由列必须恰好一个（否则
`RootSystemInvariantViolation { invariant: "alcove wall component must have
one coroot relation" }`）；关系向量取自由列 1、主元列 `-matrix[pivot][free]`；
分母 `checked_lcm` 通分、转 `i64`、全体 `gcd` 约化，首元素为负则整体
`checked_neg`（首元素为 0 不取负）。

## root_vertex_of_alcove 与 root_vertex_simple

`root_vertex_of_alcove`：逐分量求顶点后求和，使 `gamma - vertex` 落在基本
alcove 的 Weyl 轨道。**显式忽略** `wall_set` 的 `integrals`；每层取值用
**朴素有理下取整** `dot.div_euclid(denominator)`（注释强调不是
alcoves.cpp:29-32 的负根修正版 `floor_eval`，引 alcoves.cpp:422-425）。

`root_vertex_simple`（单分量）：coroot 关系（本原核向量）系数全正；丢弃
第一面系数为 1 的墙（无此墙 → `"alcove component has no coefficient-1
wall"`）；其余墙构造**转置**子 Cartan 矩阵
（`transposed[row][column] = bracket(id(column), id(row))`，参数互换，
此处 `bracket` 用 `?` 传播），`rational_inverse` 求逆（奇异 →
`"alcove generator Cartan matrix is singular"`），`base = C^{-T} · floors`；
若 base 非整（存在分量不被 denominator 整除），依次把每面后续系数为 1 的
墙的取值加 1 重试（上游 `labels_1` 循环，alcoves.cpp:395-408；代码注释逐句
对应 `labels_1.push_back(i-1)`）。首个整的 attempt 以 `entry / denominator`
为系数对**根**坐标做 checked 线性组合后转 `i32` 返回；全部非整则
`"alcove vertex lies outside the root lattice"`。

`solve_rational_system`：增广 `Rational` 矩阵 Gauss 消元至约化阶梯形；任一
列找不到主元、或消元后存在「系数全 0 而 rhs ≠ 0」的行，都返回 `None`（前
者经 `?` 作用于 `Option` 直接整体返回）。`rational_inverse`：`[A|I]` 上
Gauss-Jordan；非方阵或无主元列返回 `Ok(None)`；契约
`(numerator, d)`、`inverse = numerator / d`、`d > 0`（`d > 0` 依赖
`Rational` 分母为正，代码未显式断言），对应 `matrix::inverse(A, d)`。

## 限制与未覆盖面

- 不做任何数学/正确性验收；上游行号引用来自代码注释，未独立验证。
- 测试覆盖薄：`wall_set`、`root_components`、`barycentre_eq`、
  `labels_for_component`、`root_vertex_simple`、`rational_inverse`、
  `checked_dot`、`alcove_center` 端到端均无单元测试。
- 潜在 panic 路径（阅读推断，文件内无防护）：`RootNumbering::new` 的
  `positive_index[&negated]` 缺键；`id()` 越界；`alcove_center` 的
  `difference.numerator()[row]` 越界；`solve_rational_system` 对短行越界；
  `gcd(i64::MIN, 0)` 的 `abs()`。
- 静默兜底与显式报错并存（`unwrap_or` vs `ok_or?`），是否刻意未确认。

## 来源与限制

精确读取身份见
[`2026-10-06-alcove.json`](snapshots/2026-10-06-alcove.json)：绑定 Git base、
文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe（无工具档案）以完整
文件字节起草，维护者对照源码逐条核对改写。`RepContext`/`StandardRepr`/
`RationalWeight`/`RootSystem` 等外部类型契约不在本包范围。本次知识维护未
执行 Atlas、Cargo、测试或 benchmark。
