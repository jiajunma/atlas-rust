---
title: StructureError 错误分类学与全局 Tits 交叉作用传输层（error.rs / global_tits.rs）
source: atlas-rust/error-global-tits
ingestedAt: 2026-10-06T03:10:00Z
---

# StructureError 错误分类学与全局 Tits 交叉作用传输层（error.rs / global_tits.rs）

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `crates/atlas-real-group/src/` 的 `error.rs`（322 行）与
`global_tits.rs`（551 行，含 `#[cfg(test)]` 模块）。依赖方向单向：
`global_tits.rs` 经 `crate::StructureError`（crate 根重导出）依赖 `error.rs`；
`error.rs` 唯一的 crate 内依赖是 `crate::lattice::Weight`。本包是结构性阅读，
不声称错误覆盖面或 Tits 传输的数学验收。

## error.rs：crate 级错误枚举

`StructureError` 是 `atlas-real-group` 的统一错误类型：`#[derive(Clone, Debug,
Eq, PartialEq)]`，`impl std::error::Error` 为空实现体（`source()` 等用默认）。
文件内**没有**构造函数、`From` 转换、测试模块或任何 `pub fn`——所有构造点
分散在各子系统模块中。

共 **53 个变体**（22 个无字段 + 31 个带字段）。字段形态家族：

- `{ invariant: &'static str }` 是不变量违例家族（14 个变体），命名前缀即
  子系统名：`RootSystem` / `CayleyCross` / `RealFormLabel` /
  `CartanClassification` / `StrongReal` / `WeylElement` / `InvolutionTable` /
  `TitsCoset` / `Seed` / `Kgb` / `Block` / `Rep`（rep-context）/
  `RealFormOrder` / `Layout`（inner-class layout）。Display 统一为
  `<子系统> {invariant} invariant was violated`。
- `{ resource: &'static str, limit: usize }` 是资源预算家族（7 个变体）：
  `RootSystem` / `WeakRealForm` / `CayleyCross` / `StrongReal` /
  `InvolutionTable` / `Seed` / `AdjointFiber` 各自的 `ResourceLimit`。
  例外：`IntegerLatticeResourceLimit` 的 `limit` 是 **`u64`**（全家唯一）。
- 规模上限：`RootSystemTooLarge`（root-system closure exceeded the
  finite-system limit）、`WeylGroupTooLarge`、`ResourceLimitExceeded { limit }`、
  `AllocationFailed { requested }`。
- 输入验证：`EmptyRootDatum`、`NonSquareCartan`、`InvalidCartanMatrix`、
  `RankMismatch { expected, actual }`、`DatumMismatch`（operations require the
  same based root datum）、`IndexOutOfRange { index, upper_bound }`、
  `RootPairingMismatch { row, column, expected: i32, actual: i32 }`（唯一含
  `i32` 字段）。
- 对合/自同构合法性：`InvalidInvolution`、`InvalidBasedAutomorphism`、
  `InvalidRootAutomorphism`、`InvalidRootDatumAutomorphism`、
  `SimpleRootImageNotRoot { simple_root }`、`SimpleCorootImageMismatch
  { simple_root, image_root: Weight }`（唯一含 `Weight`，Display 用
  `{image_root:?}` 即 Debug 格式化）、`DistinguishedInvolutionMismatch`。
- 分级/纤维/强对合：`GradingShiftsNotFaithful`、`ImpossibleGrading`、
  `InvalidStrongTorusFactor`、`ModTwoSubquotientInvariantViolation`（无字段，
  形态与 invariant 家族不同）、`NotInModTwoSubspace`、`CartanFiberMismatch`、
  `CartanFiberInvolutionMismatch`、`CartanFiberMapDoesNotDescend { relation }`、
  `RealFormNotDefinedOnCartan`。
- 算术与移植状态：`InvalidIntegerMatrixShape`、`ArithmeticOverflow`（"root-system
  arithmetic overflow"）、`NotYetImplemented { feature }`——唯一带文档注释的
  变体，语义是「上游 oracle 已定义但本 crate 尚未移植的代码路径，大声报错而
  非在错误近似上继续计算」。

## global_tits.rs：全局 Tits 交叉作用的有理环面传输

模块定位（模块文档）："Exact rational torus transport for Atlas's global Tits
cross action." 与 `crate::TitsCoset` 不同，本载体保留**完整有理余特征**
（含中心坐标）；向纤维 mod-two 商的规约发生在之后。`GlobalTitsElement`
是 `pub(crate)`（本文件没有任何 `pub` 项），两个字段（`torus_factor:
RationalCoweight`、`twisted_involution: TwistedInvolution`）私有，环面坐标
是 `[0, 2)` 中的典范代表元。

### 构造门槛 `new`

按代码顺序：① 秩门槛——`torus_factor.dimension() !=
inner_class.datum().lattice_rank()` 报 `RankMismatch`；②
`validate_context`（datum 与 distinguished 一致性，见下）；③ 环面坐标经
`normalize_torus_factor`（复制 → 逐坐标 mod 2 规范化 →
`RationalCoweight::from_coordinates`）存储，twisted involution 原样存储。

### `crossed_generator`：单生成元交叉 `s * (t,w) * delta(s)`

每次调用**重新** `validate_context`。`generator >= semisimple_rank` 报
`IndexOutOfRange`；`root_involution().kind(simple_root)` 为 `None` 报
`InvalidRootAutomorphism`。然后按 `RootKind` 三分支就地更新环面坐标：

- `Complex`：`t ← t − ⟨α, t⟩·α∨`（`rational_pair` 精确有理点积 +
  `add_scaled_coroot`）。
- `Imaginary`：**整性门槛**——配对 `≠ floor(配对)` 时报
  `InvalidStrongTorusFactor`；否则以 `1 − pairing` 为缩放加余根。
- `Real`：坐标不变。

随后逐坐标 mod-2 规范化（`coordinate -= floor(coordinate/2) * 2`）。Weyl 侧：
`twisted_generator = distinguished_generator_image(...)`，新作用 =
`s_generator ∘ w ∘ s_{twisted_generator}`，再经 `TwistedInvolution::new` 重建。
方法是 `&self -> Result<Self>`，原值不变。

### `crossed_word`：前向顺序

文档注明匹配上游 `cross_act(GlobalTitsElement&, const WeylWord&)`：先
`validate_context` 一次，再**按切片顺序**逐个折叠 `crossed_generator`
（先 0 后 1，与逆序结果不同——由测试锚定）。word 中非法 generator 不在
入口处预检，折叠到时才报 `IndexOutOfRange`。

### 私有辅助

- `validate_context`：weyl_action 或 root_involution 的 datum ≠
  `inner_class.datum()` → `DatumMismatch`；否则比较矩阵复合
  `w·δ` 的 weight/coweight 矩阵与存储对合矩阵（经 `compose_matrices`），
  任一不等 → `DistinguishedInvolutionMismatch`。
- `distinguished_generator_image`：像缺失或像不在单根集合 →
  `InvalidBasedAutomorphism`。
- `rational_pair` / `add_scaled_coroot`：秩不等 → `RankMismatch`；否则精确
  有理累加/逐坐标累加。
- `copy_rationals`：经 `crate::grading::try_capacity` 预算门槛后克隆。
- `normalize_coordinates`：无错误分支。

`global_tits.rs` 直接构造的 `StructureError` 变体共 7 个：`RankMismatch`、
`IndexOutOfRange`、`InvalidRootAutomorphism`、`InvalidStrongTorusFactor`、
`DatumMismatch`、`DistinguishedInvolutionMismatch`、`InvalidBasedAutomorphism`。
`try_capacity` / `compose_matrices` / `WeylAction::*` / `TwistedInvolution::new`
的错误经 `?` 传播，具体变体不在本包范围。

## 测试锚点

`error.rs` 无测试；`global_tits.rs` 的测试模块（10 个 `#[test]`）是唯一锚点：

- `normalizes_every_coordinate_modulo_two`：(−1/2, 9/2) → (3/2, 1/2)。
- `rank_zero_and_an_empty_word_are_identity_transport`：rank 0 + 空 word。
- `a1_imaginary_cross_distinguishes_compact_and_noncompact_factors`：adjoint
  A1，紧因子 1/2 不变，非紧 0 → 1。
- `a1_imaginary_cross_requires_an_integral_root_pairing`：坐标 1/4 恰报
  `InvalidStrongTorusFactor`（整性门槛的精确错误断言）。
- `a1_real_cross_leaves_both_components_unchanged`：实根交叉两分量不变。
- `a2_complex_cross_reflects_the_rational_coweight`：(1/3, 1/2) 经
  `crossed_generator(1)` → (5/6, 3/2)，Weyl 作用 = `s1∘s0∘s1`。
- `a2_word_execution_is_forward_and_noncommuting`：顺序锚点，
  `assert_ne!(forward, reverse)`。
- `b2_complex_cross_uses_the_coroot_not_the_root_direction`：B2 用余根方向。
- `a1_with_central_torus_preserves_the_central_coordinate`：中心坐标经
  mod-2 保留（(0, 7/3) → (1, 1/3)）。
- `rejects_rank_generator_datum_and_distinguished_mismatches`：四个精确错误
  断言（`RankMismatch` / `IndexOutOfRange` / `DatumMismatch` /
  `DistinguishedInvolutionMismatch`，后者用 A2 交换对合构造）。

未覆盖分支：`InvalidRootAutomorphism`、`InvalidBasedAutomorphism`、
`rational_pair`/`add_scaled_coroot` 内部 `RankMismatch`（调用点已保证同秩）、
`try_capacity` 失败路径、非空 word 的错误传播。

## 限制

- `datum.simple_roots()[generator]` 等直接下标索引的越界保护依赖
  `generator < semisimple_rank`；是否存在「semisimple_rank ≤
  simple_roots().len()」不变量不在本包范围（潜在 panic 面，阅读观察）。
- `error.rs` 无序列化/错误码；Display 文案为英文固定字符串含字段插值。
- `GlobalTitsElement` 的 `Eq` 派生自 `RationalCoweight`/`TwistedInvolution`，
  其相等定义不在本包。
- 测试中 `InnerClass::new(datum, distinguished, root_budget)` 的
  `root_budget`（取值 0/2/6/8）语义见 cartan-fibers / real-form-seed 相关包。
- 其余 46 个变体的构造点分散在其他模块，本包不枚举。

## 来源与限制

精确读取身份见
[`2026-10-06-error-global-tits.json`](snapshots/2026-10-06-error-global-tits.json)：
绑定 Git base、两文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe
（无工具档案）以两文件完整字节起草（exit 0，398.9s），维护者对照源码逐条
核对改写。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。
