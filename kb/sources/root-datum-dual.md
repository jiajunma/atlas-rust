---
title: BasedRootDatum 与对偶内类构造（root_datum.rs / dual.rs）
source: atlas-rust/root-datum-and-dual
ingestedAt: 2026-10-05T18:20:00Z
---

# BasedRootDatum 与对偶内类构造（root_datum.rs / dual.rs）

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `crates/atlas-real-group/src/` 的 `root_datum.rs`（543 行）与
`dual.rs`（517 行）两个文件。它是结构性阅读，不声称根数据层或对偶构造的数学
验收；所有上游引用（rootdata.cpp:858/859、rootdata.cpp:1357-1365、
innerclass.cpp:435-441、complexredgp.cpp `InnerClass::numDualRealForms`）仅
转录自代码文档注释，本包未核对上游字节。

## 概览

- **root_datum.rs** 定义 `BasedRootDatum`：在一对 character/cocharacter 格中
  经过校验的简单根数据。它区分 `lattice_rank`（整个约化环面的秩）与
  `semisimple_rank`（简单根个数，即 Cartan 矩阵阶数），二者仅在无中心环面时
  相等；类型文档明言**故意不设全局秩上限**（测试以 A33 印证）。
- **dual.rs** 不定义任何类型，只有自由函数：对偶根数据 `dual_datum`、最长
  Weyl 元的下坡行走 `longest_action`、对偶内类 `dual_inner_class`、跨对偶的
  Cartan 类对应 `dual_cartan_correspondence`、对偶实形式计数
  `dual_real_form_count`，以及 crate 内部的 `dual_twisted_representative`。

## root_datum.rs：BasedRootDatum

裸签名清单（完整）：`BasedRootDatum`（四个私有字段 `lattice_rank: usize`、
`cartan: Vec<Vec<i32>>`、`simple_roots: Vec<Weight>`、
`simple_coroots: Vec<Coweight>`；派生 `Clone, Debug, Eq, Hash, PartialEq`）；
`standard`、`from_simple_data`、五个不可失败访问器（`lattice_rank`、
`semisimple_rank`、`cartan_matrix`、`simple_roots`、`simple_coroots`）、
`coradical_basis`、`radical_basis`、`reflect_weight`、`reflect_coweight`、
`pub(crate) try_clone`；私有辅助 `annihilator_matrix`、`coroot_rows`、
`root_rows`、`budget`、`reflect_coordinates`、`ensure_lattice_rank`、
`validate_cartan`、`is_finite_type`。

### 构造门控（`from_simple_data`，检查顺序固定）

1. `validate_cartan` 得 `semisimple_rank`（见下）；
2. `lattice_rank < semisimple_rank` → `StructureError::RankMismatch`；
3. 简单根/余根个数不等于 `semisimple_rank` → `RankMismatch`；
4. 每个根/余根的 `rank()` 必须等于 `lattice_rank`（`ensure_lattice_rank`）；
5. 逐 `(row, column)` 校验配对 `pair(root, coroot)` 等于 `cartan[row][column]`，
   否则 `StructureError::RootPairingMismatch { row, column, expected, actual }`。

`standard(cartan)` 以标准坐标构造：简单根取单位坐标向量，简单余根取 Cartan
矩阵的**列**，因此恒有 `lattice_rank == semisimple_rank`。合法边界包括
`lattice_rank > semisimple_rank`（中心环面）与空 Cartan 加正 `lattice_rank`
（纯环面，无根）。

### `validate_cartan` 与 `is_finite_type`

`validate_cartan` 依次拒绝：非方阵（`NonSquareCartan`）；对角元不为 2 或
非对角元为正（`InvalidCartanMatrix`）；零模式不对称（`(entry == 0) !=
(transpose == 0)` → `InvalidCartanMatrix`）；非有限型（同上）。空矩阵通过
全部检查（循环空转），与纯环面测试一致。

`is_finite_type` 用 `malachite::Rational` 做两遍精确有理计算：第一遍按连通
分量播种 scale 1 并沿非零 Cartan 边传播
`expected = scale[row] * C[row][col] / C[col][row]`，冲突即 false；第二遍对
scale 加权矩阵做 LDLᵀ 式分解，任一 pivot ≤ 0 即 false。「该检查等价于有限
型」是代码意图，本包不作数学验收。两处 `.expect(...)` 是实现内部不变量
（pending 顶点必有 scale）。

### radical/coradical 基

`coradical_basis()`：以简单**余根**坐标为行构造矩阵，求
`integer_lattice::saturated_kernel`（预算固定为
`IntegerLatticeBudget::new(64, 100_000, 100_000, 128)`），逐列经
`i32::try_from`（失败 → `ArithmeticOverflow`）包成 `Weight` 返回；文档注释
对应 rootdata.cpp:858 的 `lattice::perp`，即与所有简单余根正交的权。
`radical_basis()` 对称：以简单**根**为行求核，包成 `Coweight`。
`annihilator_matrix` 在行为空时显式构造 `IntegerMatrix::zero(0, lattice_rank,
budget)`——空方程组的核是整个 ambient 格，从空行推断列数会丢秩；两个无根
回归测试（rank 0/1/2/4）固定 coradical 与 radical 都返回 ambient 格的单位
坐标基（torus 修复的回归守卫，见 `docs/slices/torus_radical_2026-09-29.md`
的来源链）。

### 简单反射

`reflect_weight(generator, weight)`：先校验 `weight.rank() == lattice_rank`
（`RankMismatch`），再用 `.get(generator)` 检查下标（越界 →
`IndexOutOfRange { index, upper_bound: semisimple_rank }`）；公式
$x \mapsto x - \langle x, \alpha^\vee_g\rangle\,\alpha_g$，系数经
`pair(weight, &simple_coroots[generator])` 计算。`reflect_coweight` 是对偶
形式 $y \mapsto y - \langle\alpha_g, y\rangle\,\alpha^\vee_g$。底层
`reflect_coordinates` 全程 `i128` 中间值：`try_reserve_exact`（失败 →
`AllocationFailed`）、`checked_mul`、`checked_sub`、`i32::try_from`（失败均
→ `ArithmeticOverflow`）。注意 `.get()` 检查的是兄弟数组而随后直接索引另一个
等长数组，安全性依赖构造门控（两数组长度均为 `semisimple_rank`）。

### `try_clone`（`pub(crate)`）

逐字段可失败复制（`try_reserve_exact` + `try_copy_coordinates`），直接构造
`Self`，既不暴露不可失败的 `Clone` 路径疑虑，也不重跑 `from_simple_data`
校验。在这两个文件内**无调用点**（dual.rs 用派生 `Clone`）；外部调用点不在
本包范围。

## dual.rs：对偶构造族

裸签名清单（完整，本文件无类型定义）：`dual_datum`、`longest_action`、
`dual_inner_class`、`dual_cartan_correspondence`、`dual_real_form_count`（均
`pub`），私有 `two_rho`、`dual_involution`，`pub(crate)
dual_twisted_representative`。

### `dual_datum`

转置 Cartan、互换简单根/余根（上游 `RootDatum(rd, tags::DualTag)`），收尾于
`BasedRootDatum::from_simple_data`——**对偶构造复用全部构造门控**。已校验
datum 的转置理论上应通过重校验，但签名保持可失败，本包不声称其永真。

### `longest_action`（最长元，不下枚举）

返回把 $2\rho$ 送到 $-2\rho$ 的 `WeylAction`（上游
`rd.to_dominant(-rd.twoRho())`）。`two_rho` 用 `i64` 累加器（长度 =
`lattice_rank`）只计入 `is_positive(id) == Some(true)` 的根（`None` 同样
跳过），`checked_add` 后逐分量 `i32::try_from`。行走循环按生成元升序取第一
个与当前权有正余根配对的反射，左合成并作用于当前权；每轮
`steps += 1`，若 `steps > weyl_budget || !advanced` 则
`StructureError::LayoutInvariantViolation { invariant: "longest Weyl element" }`
——预算允许恰好 `weyl_budget` 步，未推进也报同一不变量。文档注释称该下坡
行走步数恰等于最长元约化长度（上游 `WeylGroup::longest` 为 transducer
O(rank)，此为等价 O(length) 行走）——复杂度声明为注释内容，本包不验收。
事实记录：内层配对用未检查的 `i64` 累加（与 `two_rho` 的 `checked_add`
风格不同），是否有意未在本包确认。

### `dual_involution` 与 `dual_inner_class`

`dual_involution` 返回乘积矩阵 `M = q * W0`（`compose_matrices(
distinguished.weight_matrix(), longest.matrix())`），本身**不做取负/转置**；
注释说明对偶 involution 是 `-M^t`，余权作用 `-M` 因 M 是 involution 而在
构造处推导。`dual_inner_class` 逐元素 `checked_neg` 后填
`dual_weight[column][row] = -product[row][column]`（权作用 `-(q·W0)^t`）与
`dual_coweight[row][column] = -product[row][column]`（余权作用 `-(q·W0)`），
与模块文档的 `negative_transposed` 对应，再经 `LatticeInvolution::new` 与
`InnerClass::new(dual, involution, root_budget)` 收尾。预算分工：
`weyl_budget` 只管最长元定位，`root_budget` 约束对偶根系闭包。

### `dual_cartan_correspondence`

对原分类中每个 Cartan 类（按 crate Cartan 顺序）输出 `(对偶 CartanId, 对偶
类的 weak-real-form 计数)`。前置校验两侧 fundamental 存在性与 datum 一致性
（`DatumMismatch`）；主流程经 provenance 检查的
`TwistedConjugacyPartition::class_of` 建 `cartan_of_raw` 反查表。设计要点
（文档注释，未独立验证）：上游按反序配对 `tw` 与 `tw * w0` 的对偶 Cartan
（innerclass.cpp:435-441），因对偶 distinguished involution 为 `-(δ·w0)^t`
且转置是逆步的，配对 involution 化简为 `-(w·δ)|co`；上游随后规范化对偶
twisted involution，存储代表元一般是 `tw*w0` 的**共轭**，矩阵比较不可靠，
故实现改以 lattice map 在对偶根上诱导的根像置换为键。公开的
`_weyl_budget` 参数是遗留未使用参数：最长元行走改以
`dual.root_system().roots().len()`（已枚举根数）为预算。「A class miss is
an invariant violation, never a hole」——查不到类是不变量错误而非空洞。
panic 路径：三处 `.expect("cartan_ids yields in-range ids")`。

### `dual_twisted_representative`（`pub(crate)`）

把原代表元的 Weyl 字（经 `WeylElement::from_action` + `canonical_word`，用
原 datum 的 Cartan 建 `WeylInterface`）在**对偶** datum 上逐生成元重放，
末步左合成 `longest`，再以对偶 distinguished involution 构造
`TwistedInvolution`。文档称其为 RealWeylContext 对偶 fiber 所用的同一原词
重放：与裸根置换查找不同，它保留两种格作用与 distinguished-involution
来源信息。

### `dual_real_form_count`

固定管线（全部 `?` 传播）：`dual_inner_class` → `CartanFiber::build` →
`AdjointCartanFiber::build` → `CartanGradingData::build` →
`WeakRealFormPartition::build` → `class_count()`（上游
`InnerClass::numDualRealForms`）。预算依次分给最长元、对偶根系、Cartan
fiber、伴随 fiber 与 weak-real partition。

### 测试锚点（断言值照录）

sc A1 → 2 个对偶实形式；adjoint A1 → 2；等秩紧 A2 → 1（对偶为拟分裂
adjoint A2，唯一的 PSL(3,R) 形式）；扭 A2 → 2；紧 B2 → 3（`w0 = -1` 使紧/
分裂内类两侧重合）。A2 单位 involution 的 `dual_involution` =
`[[0,-1],[-1,0]]`。sc A1 的 Cartan 对应表为
`[(CartanId(1), 1), (CartanId(0), 2)]`（反序配对，oracle capture 3501500
锚定）。紧 B2：对应表长 4、首条计数 1、全格 `-1` involution 类计数 3、
对偶 CartanId 集合恰为 {0,1,2,3}（双射）。

## 两文件的调用关系与观察点

1. `dual_datum` 只读访问器后收尾于 `from_simple_data`；
2. `longest_action` 用 `semisimple_rank`/`simple_coroots` 与派生 `Clone`，
   反射经 `WeylGroup`/`WeylAction`，**不调用** `reflect_weight`；
3. `dual_cartan_correspondence` 的 datum 一致性依赖派生 `PartialEq`；
4. `try_clone`、`coradical_basis`、`radical_basis`、`reflect_weight`、
   `reflect_coweight` 在这两个文件内无调用点。

待核对观察点（保留给后续审查，不阻塞本包收录）：

- `radical_basis` 文档首行写 “`lattice::perp` of the coroots”，而紧随的定义
  与实现均为「以简单根为行的矩阵的核」（与所有简单根正交的余权）；措辞疑似
  笔误，未核对上游 rootdata.cpp:859 字节；
- `longest_action` 内层配对是未检查的 `i64` 累加（与 `two_rho` 的
  `checked_add` 风格不同）；
- `reflect_*` 与 `dual_cartan_correspondence` 存在「先 `.get()` 检查兄弟
  数组、再直接索引」与 `cartan_of_raw[raw]` 直接索引，安全性依赖构造不变量；
- `two_rho` 对 `is_positive(id) == None` 的条目按非正根跳过。

## 来源与限制

精确读取身份见
[`2026-10-06-root-datum-dual.json`](snapshots/2026-10-06-root-datum-dual.json)：
绑定 Git base、两文件的逐字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe
（无工具档案）起草，维护者对照源码逐条核对改写；`Weight`/`Coweight`/`pair`、
`StructureError` 全变体、`IntegerLatticeBudget` 四参数语义、
`saturated_kernel`/`IntegerMatrix`、`WeylGroup`/`WeylAction`/`WeylElement`/
`WeylInterface`、`InnerClass`、`LatticeInvolution`、`CartanClassification`、
`TwistedConjugacyPartition` 与各 fiber 类型均不在本包定义。`is_finite_type`
与有限型的等价、`two_rho` 与 $2\rho$ 的恒等、`longest_action` 的复杂度声明、
对偶实形式计数的数学正确性均为代码意图，本包不作数学、性能或正确性结论。
本次知识维护未执行 Atlas、Cargo、测试或 benchmark。
