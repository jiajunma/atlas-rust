```markdown
---
title: real_weyl.rs：实 Weyl 群与块稳定子的构造、对偶 fiber 重放与打印层
source: atlas-rust/real-weyl
ingestedAt: 2026-10-05T18:00:00Z
---

> 草案状态：待维护者逐条核对。
> 标注约定：【已实现】= 可直接从所给字节读出的行为；【注释声明】= 源文件文档注释中的 upstream 对应关系/出处（本草案未独立验证）；【推断】= 阅读推断，需维护者确认。

## 0. 文件定位

- 路径：`crates/atlas-real-group/src/real_weyl.rs`
- 职责【注释声明】：移植 upstream `realweyl::RealWeyl`（realweyl.h:36-181, realweyl.cpp:34-69）、`realweyl::RealWeylGenerators`（realweyl.h:183-236, realweyl.cpp:81-159）与打印层 `realweyl_io::common_print`（realweyl_io.cpp:72-146），并移植解释器包装 `output::printRealWeyl`（output.cpp:445-474）与 `output::printBlockStabilizer`（output.cpp:361-390）的 `x`/`y` 选取逻辑。

## 1. 裸签名清单

### 1.1 pub 类型

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LieTypeComponent {
    pub letter: char,
    pub rank: usize,
}

#[derive(Clone, Debug)]
pub struct RealWeyl {
    // 全部字段私有：
    imaginary: Vec<RootId>,
    imaginary_compact: Vec<RootId>,
    imaginary_orth: Vec<RootId>,
    imaginary_r: Vec<ModTwoVector>,
    real: Vec<RootId>,
    real_compact: Vec<RootId>,
    real_orth: Vec<RootId>,
    real_r: Vec<ModTwoVector>,
    complex: Vec<RootId>,
    complex_type: Vec<LieTypeComponent>,
    imaginary_type: Vec<LieTypeComponent>,
    imaginary_compact_type: Vec<LieTypeComponent>,
    real_type: Vec<LieTypeComponent>,
    real_compact_type: Vec<LieTypeComponent>,
}

#[derive(Clone, Copy, Debug)]
pub struct RealWeylContext<'a> {
    pub inner_class: &'a InnerClass,
    pub classification: &'a CartanClassification,
    pub dual_inner_class: &'a InnerClass,
    pub dual_classification: &'a CartanClassification,
    pub budget: &'a CartanClassificationBudget,
}

#[derive(Clone, Debug)]
pub struct RealWeylGenerators {
    // 全部字段私有：
    imaginary: Vec<WeylElement>,
    imaginary_compact: Vec<WeylElement>,
    imaginary_r: Vec<WeylElement>,
    real: Vec<WeylElement>,
    real_compact: Vec<WeylElement>,
    real_r: Vec<WeylElement>,
    complex: Vec<WeylElement>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RealWeylGeneratorSection {
    pub header: String,
    pub words: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RealWeylPrint {
    pub header: String,
    pub summaries: Vec<String>,
    pub generator_sections: Vec<RealWeylGeneratorSection>,
}
```

### 1.2 impl 块与方法签名

```rust
impl RealWeylContext<'_> {
    pub fn real_weyl(
        &self,
        form: WeakRealFormId,
        cartan: CartanId,
        dual_form: Option<WeakRealFormId>,
    ) -> Result<RealWeyl, StructureError>;

    pub fn real_weyl_print(
        &self,
        form: usize,
        cartan: usize,
    ) -> Result<RealWeylPrint, StructureError>;

    pub fn block_stabilizer_print(
        &self,
        form: usize,
        cartan: usize,
        dual_form: usize,
    ) -> Result<RealWeylPrint, StructureError>;

    // 私有：
    fn dual_side(&self, cartan_class: &CartanClass) -> Result<DualSide, StructureError>;
}

impl RealWeyl {
    pub fn imaginary(&self) -> &[RootId];
    pub fn imaginary_compact(&self) -> &[RootId];
    pub fn imaginary_orth(&self) -> &[RootId];
    pub fn imaginary_r(&self) -> &[ModTwoVector];
    pub fn real(&self) -> &[RootId];
    pub fn real_compact(&self) -> &[RootId];
    pub fn real_orth(&self) -> &[RootId];
    pub fn real_r(&self) -> &[ModTwoVector];
    pub fn complex(&self) -> &[RootId];
    pub fn complex_type(&self) -> &[LieTypeComponent];
    pub fn imaginary_type(&self) -> &[LieTypeComponent];
    pub fn imaginary_compact_type(&self) -> &[LieTypeComponent];
    pub fn real_type(&self) -> &[LieTypeComponent];
    pub fn real_compact_type(&self) -> &[LieTypeComponent];
}

impl RealWeylGenerators {
    pub fn build(
        real_weyl: &RealWeyl,
        root_system: &RootSystem,
        involution: &RootInvolutionData,
    ) -> Result<Self, StructureError>;

    pub fn imaginary(&self) -> &[WeylElement];
    pub fn imaginary_compact(&self) -> &[WeylElement];
    pub fn imaginary_r(&self) -> &[WeylElement];
    pub fn real(&self) -> &[WeylElement];
    pub fn real_compact(&self) -> &[WeylElement];
    pub fn real_r(&self) -> &[WeylElement];
    pub fn complex(&self) -> &[WeylElement];
}

impl RealWeylPrint {
    pub fn render(&self) -> String;
}
```

### 1.3 pub(crate) 项

```rust
pub(crate) fn twisted_orbit_size(
    root_system: &RootSystem,
    involution: &RootInvolutionData,
    full_order: &malachite::Integer,
) -> Result<usize, StructureError>;
```

### 1.4 文件私有项（参考，不在公开 API 内）

- `struct DualSide { twisted: TwistedInvolution, grading: CartanGradingData, partition: WeakRealFormPartition, labels: RealFormLabels }`
- `struct FiberSide { compact_basis: Vec<RootId>, orth: Vec<RootId>, r_vectors: Vec<ModTwoVector> }`
- `enum PrintKind { RealWeyl, BlockStabilizer }`（`#[derive(Clone, Copy, Debug, Eq, PartialEq)]`；注释声明 upstream `which_group` 的 `dual_real_W` 未移植）
- 私有函数：`common_print`、`weyl_type`、`two_type`、`format_word`、`fiber_side`、`simple_basis`、`simple_complex`、`subsystem_cartan`、`lie_type`、`r_group_elements`、`primal_roots_of_dual`、`reflect_weight`、`accumulate_weight`、`parity_dot`、`sort_by_upstream_key`、`checked_cartan`
- 本文件未定义任何 trait。

## 2. RealWeyl / RealWeylGenerators 的表示与不变量

### 2.1 RealWeyl 载荷

- 【已实现】14 个字段全部私有，仅经 14 个 getter 暴露切片；构造只发生在 `RealWeylContext::real_weyl` 内。
- 【注释声明】根列表保存 **primal** `RootId`，顺序为 upstream `RootNbr` 升序（`imaginary`/`real` 经 `sort_by_upstream_key`；`complex` 例外，为 `makeSimpleComplex` 的输出顺序）；对偶侧列表 `real_compact`、`real_orth` 经 coroot 向量映射回 primal。
- 【注释声明】R-群位向量（`imaginary_r`/`real_r`）每个 `orth` 条目占一个坐标，顺序为 upstream `BinaryMap::kernel`（bitvector.cpp:268-277）的核生成元顺序。
- 【已实现】类型载荷 `complex_type`/`imaginary_type`/`imaginary_compact_type`/`real_type`/`real_compact_type` 均为 `Vec<LieTypeComponent>`，由 `lie_type(&subsystem_cartan(...)?)` 产生，分量顺序来自 `crate::dynkin::classify`。

### 2.2 关键不变量：对偶侧转置配对

- 【已实现】`real_type` 与 `real_compact_type` 用 `subsystem_cartan(root_system, …, /*transposed=*/ true)` 计算；其余三个类型用 `transposed=false`。
- 【注释声明】这复现 upstream `drd.subsystem_type`：对偶根向量即 primal coroot 向量，对偶子系统 Cartan 矩阵为 primal 的转置——B/C 互换即在此进入（oracle：`Sp(4,R)` Cartan #3 在 C2 datum 上打印 `W^R is a Weyl group of type B2`）。
- 【已实现】测试 `sp4_real_weyl_and_block_stabilizer_match_oracle` 中 `real_weyl_output(&fx, 2, 3)` 确实断言输出 `"W^R is a Weyl group of type B2"`。

### 2.3 RealWeylGenerators 载荷

- 【已实现】7 个列表，与 `RealWeyl` 的根列表一一对应（`imaginary_orth`/`real_orth` 无对应生成元列表，只作为 R-群乘积的因子池）。
- 【注释声明】每个列出的根一个 Weyl 元素；R-群按核位向量每个一个乘积；复根为 `s_rn . s_theta(rn)`（realweyl.cpp:148-156）。
- 【已实现】复根生成元的乘法顺序为 `reflect(root)?.multiply(root_system, &reflect(image)?)?`，即先 `s_root` 再右乘 `s_image`；`image` 来自 `involution.image(root)`，缺失时报 `StructureError::RootSystemInvariantViolation { invariant: "involution image" }`。
- 【注释声明】生成元 WORD 按构造即 canonical：crate 直接由根反射构建元素并打印 `WeylElement::canonical_word`，与 upstream 经 `WeylGroup::word`（weyl.cpp:944-957）得到的 canonical word 相同，因此 `reflection_word`/`to_dominant` 机制未移植。

## 3. 构造路径：从 Cartan 数据到实 Weyl 群

### 3.1 `RealWeylContext::real_weyl`（核心构造）

步骤与错误分支【已实现】：

1. `root_system = self.inner_class.root_system()`；`classification.cartan_class(cartan)` 失败 → `StructureError::IndexOutOfRange { index: cartan.0, upper_bound: classification.cartan_classes().len() }`。
2. 在 `cartan_class.labels().labels()` 中定位 `form`；未找到 → `StructureError::RealFormNotDefinedOnCartan`（【注释声明】对应包装层 "Cartan class not defined for real form"，atlas-types.w:8842-8846）。
3. `x = cartan_class.partition().class_representative(WeakRealFormId(local))`；失败 → `StructureError::CartanClassificationInvariantViolation { invariant: "real-form representative" }`。
4. `dual = self.dual_side(cartan_class)?`（见 §4）。
5. `y` 的两种情形：
   - `dual_form == None`：`dual.grading.adjoint_fiber().identity()?`（零元素；【注释声明】即 upstream 硬编码的对偶 quasisplit 代表元）。
   - `Some(dual_form)`：在 `dual.labels.labels()` 定位，未找到 → `RealFormNotDefinedOnCartan`；代表元缺失 → `CartanClassificationInvariantViolation { invariant: "dual real-form representative" }`；取 `.clone()`。
6. `involution = cartan_class.representative().root_involution()`。
7. `imaginary`/`real`：`involution.imaginary_simple_roots()` / `real_simple_roots()` 经 `sort_by_upstream_key`；`complex = simple_complex(root_system, involution)?`。
8. `primal_side = fiber_side(root_system, involution, cartan_class.grading(), x)?`；`dual_side_fs = fiber_side(dual_system, dual.twisted.root_involution(), &dual.grading, &y)?`。
9. `real_compact`/`real_orth` = `primal_roots_of_dual(root_system, dual_system, &dual_side_fs.compact_basis / .orth)?`（经 coroot 向量回映；未命中 → `CartanClassificationInvariantViolation { invariant: "dual root correspondence" }`）。
10. 五个类型计算（见 §2.2）后组装 `RealWeyl`。

注意【已实现】：`real_r` 字段填的是 **对偶侧** 的 `r_vectors`（`real_r: dual_side.r_vectors`），`imaginary_r` 填 primal 侧的。

### 3.2 两个打印包装

- `real_weyl_print(form: usize, cartan: usize)`【已实现】：
  - `ExternalFormOrder::build(self.inner_class, self.classification)?`；`order.internal(form)` 为 `None` → `IndexOutOfRange { index: form, upper_bound: order.form_count() }`。
  - `checked_cartan`：越界 → `IndexOutOfRange { index: cartan, upper_bound: …len() }`。
  - 以 `dual_form = None` 调 `real_weyl`。
  - 重新取 involution 处使用 `.expect("checked cartan id")`——**panic 路径**（按构造不可达；属潜在 panic 点，需维护者知悉）。
  - `RealWeylGenerators::build(...)` 后 `common_print(PrintKind::RealWeyl, …)`。
- `block_stabilizer_print(form, cartan, dual_form)`【已实现】：对偶侧另建 `ExternalFormOrder::build(self.dual_inner_class, self.dual_classification)?`，`dual_form` 越界 → `IndexOutOfRange { index: dual_form, upper_bound: dual_order.form_count() }`；以 `Some(dual_internal)` 调 `real_weyl`；同样的 `expect("checked cartan id")`；`common_print(PrintKind::BlockStabilizer, …)`。
- 【注释声明】`form`/`dual_form` 为解释器外部形式编号（atlas-types.w:8828-8847 / :8920-8932）；gkmod `blockstabilizer` 子系统不需要，包装只转发 `(rf, cn, drf)`。

### 3.3 单侧 fiber 包：`fiber_side`

输入 `(root_system, involution, grading, element)`，输出 `FiberSide { compact_basis, orth, r_vectors }`；primal 与 dual 两侧共用同一代码【已实现】。

- 正虚根集：`involution.roots_of_kind(RootKind::Imaginary)` 过滤 `is_positive == Some(true)`，再 `sort_by_upstream_key`。
- 非紧判定（【注释声明】`Fiber::noncompactRoots` cartanclass.cpp:706-712 的线性扩张）：`ambient = grading.adjoint_fiber().canonical_representative(element)?`；对每个正虚根 `alpha`，`doubled_sum = Σ_β bracket(alpha, beta)`（`i64::checked_add`，溢出 → `StructureError::ArithmeticOverflow`）；`doubled_sum` 为奇 → `RootSystemInvariantViolation { invariant: "imaginary-simple coordinates" }`；`base_noncompact = (doubled_sum/2) % 2 != 0`；平移项为 `parity_dot(&ambient, simple_coordinates(alpha))`（ambient 坐标与根 datum-simple 坐标的 mod-2 点积）。`base_noncompact ^ parity_dot(...)` 为真 → 非紧，否则紧并累加 `two_rho_ic`（`i32::checked_add`，溢出 → `ArithmeticOverflow`）。
- `compact_basis = simple_basis(root_system, &compact)?`（【注释声明】realweyl.cpp:55 的 `rd.simpleBasis(f.compactRoots(x))`）。
- `orth`：非紧根中满足 `pair(&two_rho, coroot(alpha))? == 0` 者（【注释声明】`orthogonalMAlpha` realweyl.cpp:234-249；注释称这些根强正交，构成 A_1^n）。
- `r_vectors`（【注释声明】`rGenerators` realweyl.cpp:264-279 + `BitMatrix::kernel` bitvector.cpp:234-280）：对每个 `orth` 根取 `m_alpha = fiber.element_from_coweight_mod_two(coroot)?`，其坐标按行注入 `ModTwoSubspace::new(count)?`（空行跳过）；核生成元 = 对每个**自由列（升序）**，置位集合为 `[free] + {pivot : 主元行在 free 列有 1}`，经 `ModTwoVector::from_ones(count, ones)?` 构造。

### 3.4 `simple_basis` 的怪癖（边界条件）

- 【已实现】候选集为排序后的 `BTreeSet`；扫描中若 `gamma = s_alpha(beta)` 非正，则**移除 `alpha` 自身并 `break 'outer`**——整个外循环终止，之后的候选不再检查。【注释声明】此为 upstream `RootSystem::simpleBasis`（rootdata.cpp:621-652）的既有怪癖，移植时保留。
- 【注释声明】调用方只传正根（upstream 先清掉输入集的负半）。
- 反射结果查无 id → `RootSystemInvariantViolation { invariant: "simple-basis reflection" }`。

### 3.5 `simple_complex`（复根子系统简单基）

- 【已实现】`tri`/`trr` 分别为正虚/正实根权重之和（`accumulate_weight` 跳过非正根；溢出 → `ArithmeticOverflow`）；取与两者均正交的正根，经 `simple_basis` 得 `rb`；`subsystem_cartan(…, false)` 后 `crate::dynkin::classify` 得分量。
- 对合配对【已实现】：依序保留当前分量顶点；`image = involution.image(rb[components[index].offset()])`（缺失 → `"involution image"`）；向**后**扫描，删除**第一个**含与 `image` 非正交顶点的后续分量即 `break`（每次只删一个）。
- 【注释声明】对应 `CartanClass::makeSimpleComplex`（cartanclass.cpp:1002-1044）：Dynkin 分量在对合成对的配对中只保留其一。

### 3.6 `twisted_orbit_size`（pub(crate)）

- 【已实现】`stabilizer = weyl_order_of_cartan(虚单子系统) × weyl_order_of_cartan(实单子系统) × weyl_order_of_cartan(complex)`（均 `transposed=false`）；`full_order` 不被整除 → `CartanClassificationInvariantViolation { invariant: "integral twisted orbit size" }`；商转 `usize` 失败 → `ArithmeticOverflow`。
- 【注释声明】复因子只取每个对合配对中的**一个**分量（`CartanClass::orbit_size` cartanclass.cpp:1041）；复用与实 Weyl 实现完全相同的子系统基。

## 4. 对偶 fiber 的重放（`dual_side`）

- 【注释声明】upstream 直接读 `cc.dualFiber()`（cartanclass.cpp:121），但 crate 不能复用对偶分类存储的 fiber：对偶 Cartan 对合 `-theta` 一般只是典范对偶 Cartan 代表元的**共轭**（`tw * w0`，innerclass.cpp:435-441），故临时（ad hoc）重建整条链。
- 【已实现】重建序列（全部消耗 `self.budget`，见 §6）：
  1. `twisted = crate::dual::dual_twisted_representative(self.inner_class, cartan_class.representative(), self.dual_inner_class, &longest_action(self.dual_inner_class, self.budget.weyl_budget())?)?`
  2. `CartanFiber::build(data.involution(), self.budget.integer_lattice())?`
  3. `AdjointCartanFiber::build(dual_system, data, &source, self.budget.adjoint_fiber())?`
  4. `CartanGradingData::build(dual_system, data, &adjoint)?`
  5. `WeakRealFormPartition::build(&grading, self.budget.max_fiber_elements())?`
  6. `CayleyCrossDecomposition::build(self.dual_inner_class, &twisted, self.budget.max_peeling_steps())?`
  7. `fundamental = self.dual_classification.cartan_class(CartanId(0))`，缺失 → `CartanClassificationInvariantViolation { invariant: "dual fundamental class" }`
  8. `RealFormLabels::build(self.dual_inner_class, fundamental.grading(), fundamental.partition(), &grading, &partition, &decomposition)?`
- 【推断】该链在**每次** `real_weyl` 调用时重建（无缓存）；`real_weyl_print`/`block_stabilizer_print` 亦各建一次 `ExternalFormOrder`。这是否为有意取舍需维护者确认（本草案不做性能声明）。
- 【已实现】`primal_roots_of_dual`：以 primal `coroot.as_slice().to_vec()` 建 `HashMap<Vec<…>, RootId>`，按对偶根向量回查；positivity 注释称被保持。

## 5. 打印层与字节级渲染

### 5.1 `common_print`（`PrintKind` 两变体）

- 头行【已实现】：
  - `RealWeyl`：`"real weyl group is W^C.((A.W_ic) x W^R), where:"`
  - `BlockStabilizer`：`"block stabilizer is W^C.((A_i.W_ic) x (A_r.W_rc)), where:"`
- 摘要行【已实现】：RealWeyl 4 行（`W^C`、`A`、`W_ic`、`W^R`）；BlockStabilizer 5 行（`W^C`、`A_i`、`W_ic`、`A_r`、`W_rc`）。`W^C` 行在 `complex` 非空时带 `"isomorphic to "` 前缀。
  - 【推断】`summaries` 以 `try_capacity(4)?` 创建但 BlockStabilizer 推入 5 行——`Vec` 自动增长，容量仅是预分配提示，非上限。
- `weyl_type`：`count == 0` → `"trivial"`；否则 `"a Weyl group of type {letter}{rank}(.{letter}{rank})*"`。
- `two_type`：`rank == 0` → `"trivial"`；否则 `"an elementary abelian 2-group of rank {rank}"`。
- 生成元节【已实现】：仅非空因子输出；节头顺序 `W^C` → A 群 → `W_ic` →（`W^R` | `A_r`、`W_rc`）。**冒号不一致保留**【注释声明】（realweyl_io.cpp:105-145）：`generators for A` / `generators for A_i` / `generators for A_r` **无冒号**，`generators for W^C:` / `W_ic:` / `W^R:` / `W_rc:` 有冒号。
- 词打印：`WeylInterface::new(root_system.datum().cartan_matrix())?` + `element.canonical_word(root_system, &interface)?`；`format_word`：空词 → `"e"`，否则 1-based 逗号连接。

### 5.2 `RealWeylPrint::render` 的字节契约【已实现】

- 每行以 `\n` 结尾；摘要与节之间**恰好一个空行**；最后一节之后（或无节时空行之后）**无多余字节**。

## 6. 预算语义

- 【已实现】`RealWeylContext.budget: &'a CartanClassificationBudget` 仅在 `dual_side` 中被消耗：`weyl_budget()`（供 `longest_action`）、`integer_lattice()`、`adjoint_fiber()`、`max_fiber_elements()`、`max_peeling_steps()`。primal 侧全部读自已预算好的 `classification`。
- 【已实现】多处 `try_capacity`（`crate::grading`）做 Vec 预分配（容量守护，非逻辑上限）。
- 【已实现】测试侧 `classification_budget(weyl)`：`IntegerLatticeBudget::new(64, 100_000, 100_000, 128)`、`AdjointFiberBudget::new(integer, 50_000, 100_000)`、`max_fiber_elements = 64`、`max_peeling_steps = 64`，`weyl` 参数随 fixture 变化（2/8/16/64/4096）。
- 【推断】查询期预算耗尽的表现（具体错误变体）不在本文件字节内，取决于各 `build` 实现，未覆盖。

## 7. 与 Weyl 群 / 根系的作用关系

- 【已实现】反射构造：`WeylAction::root_reflection(datum, root_system, root)?` → `WeylElement::from_action(...)`；单位元 `WeylElement::identity(root_system)?`。
- 【已实现】乘积：`WeylElement::multiply(root_system, &other)?`；R-群元素从单位元起按 `orth` 位**升序**右乘各因子反射（`r_group_elements`）。
- 【已实现】排序键：`sort_by_upstream_key` 用 `cartan_classification::upstream_positive_key`（【注释声明】(height, reverse-lexicographic simple coordinates)，rootdata.cpp 的 `RootNbr` 序）。
- 【已实现】`subsystem_cartan`：`(i,j)` 条目非转置时为 `bracket(basis[i], coroot(basis[j]))`；`transposed=true` 时交换为 `bracket(basis[j], coroot(basis[i]))`，复现 `drd.subsystem_type`。

## 8. 错误分支汇总（本文件显式构造的）

| 变体 | 触发点 |
|---|---|
| `StructureError::IndexOutOfRange { index, upper_bound }` | `cartan_class(cartan)` 未命中；`order.internal(form)`/`dual_order.internal(dual_form)` 为 `None`；`checked_cartan` 越界 |
| `StructureError::RealFormNotDefinedOnCartan` | primal/dual 形式标签在该 Cartan 类上无对应位置 |
| `StructureError::CartanClassificationInvariantViolation` | invariant ∈ `"real-form representative"`、`"dual real-form representative"`、`"dual fundamental class"`、`"dual root correspondence"`、`"integral twisted orbit size"` |
| `StructureError::RootSystemInvariantViolation` | invariant ∈ `"involution image"`、`"imaginary-simple coordinates"`、`"root lookup"`、`"simple-basis reflection"` |
| `StructureError::ArithmeticOverflow` | `fiber_side`/`accumulate_weight`/`reflect_weight` 的 `checked_add/sub/mul`；`twisted_orbit_size` 的 `usize::try_from` |
| panic（非 Result） | 两个打印包装中 `.expect("checked cartan id")`（按构造不可达） |

其余 `?` 传播的错误来自被调用的 crate 内部构建器，变体不在本文件字节内。

## 9. 测试锚点（`#[cfg(test)] mod tests`）

- Oracle 溯源【注释声明】：探针 `/tmp/probe_rw_all.at`，对照固定的 upstream 构建（rev 4d3e9449），输出 2026-08-11 重新生成并**逐字节**复制进断言（含无生成元节时的结尾空行）。
- Fixture（内部类 / 弱实形式数 / Cartan 类数，由 `fixture_form_and_cartan_counts_match_the_oracle` 锚定）：
  - `ic2` = `InnerClass:SL(2,R)`（sc A1，δ=id）→ (2, 2)
  - `icu` = `InnerClass:SU(2,1)`（SL(3) datum，δ=id）→ (2, 2)
  - `ics` = `InnerClass:SL(3,R)`（δ=w0）→ (1, 2)
  - `icp` = `InnerClass:Sp(4,R)`（δ=id）→ (3, 4)
  - `icc` = `InnerClass:SL(2,C)`（双倍 sc A1，δ=因子交换）→ (1, 1)
  - `ic4` = `InnerClass:SL(4,R)`（δ=w0）→ (2, 3)
  - `ic6` = `InnerClass:SL(6,R)`（δ=w0）→ (2, 4)
- 逐字节输出测试：`a1_real_weyl_matches_oracle`、`su21_real_weyl_matches_oracle`、`sl3r_real_weyl_and_block_stabilizer_match_oracle`、`sp4_real_weyl_and_block_stabilizer_match_oracle`（含 B2 互换锚点 `(2,3)`）、`sl2c_complex_weyl_matches_oracle`（`W^C is isomorphic to a Weyl group of type A1`，生成元词 `1,2`）、`sl4r_real_weyl_matches_oracle`、`sl6r_multi_generator_r_group_matches_oracle`（rank-2 A 群的自由列升序锚点）。
- 错误路径测试：`undefined_cartan_for_form_is_the_wrapper_error`（`RealFormNotDefinedOnCartan`：SU(3) 只有基本 Cartan；Sp(2) 只有 Cartan #0 等）、`out_of_range_form_and_cartan_are_index_errors`（`IndexOutOfRange { index: 2, upper_bound: 2 }` 两例）。
- 块稳定子测试均用对偶 quasisplit 形式：`ExternalFormOrder::quasisplit_external()`。

## 10. 与 upstream 的刻意偏差（模块头声明，【注释声明】）

1. 生成元词按构造 canonical，`reflection_word`/`to_dominant` 机器未移植。
2. `output::printRealWeyl`/`printBlockStabilizer` 末尾的 `#ifndef NDEBUG` 尺寸断言（及 `weylsize` 计算）未移植。
3. `printDualRealWeyl`（realweyl_io.cpp:186-195）未移植——无内建包装使用；其所需的 `imaginary`/`real` 生成元列表仍在计算，后续补齐仅是打印层改动。

## 11. 限制与未覆盖面

- 【已实现】`PrintKind` 无 `dual_real_W` 变体；`printDualRealWeyl` 路径完全缺失（见 §10.3）。
- 【已实现】`RealWeyl`/`RealWeylGenerators` 字段不可外部构造；`DualSide`/`FiberSide`/全部辅助函数为文件私有；`twisted_orbit_size` 仅 pub(crate)。
- 【已实现】`LieTypeComponent.letter` 为 `char`（打印字母+秩）；不携带 Dynkin 顶点信息。
- 【已实现】panic 面：两处 `.expect("checked cartan id")`；`simple_complex` 中 `components[index].support.clone()` 等索引使用未见越界守护（依赖 `dynkin::classify` 输出与 `rb` 长度一致）。【推断】该一致性前提需维护者确认。
- 【已实现】`fiber_side` 假设 `doubled_sum` 恒偶，否则报错（`"imaginary-simple coordinates"`）；`simple_basis` 假设输入仅正根（注释声明，无运行时校验）。【推断】对非正根输入的行为未定义于本文件。
- 未做事项声明：本草案不含任何数学验收、性能或正确性声明；所有 upstream 文件:行号对应均为源文件注释声明，未独立核对 upstream 源码；`real_weyl` 结果与 Weyl 群阶的一致性（`twisted_orbit_size` 之外）在本文件无断言覆盖。
- 【推断】预算耗尽路径、非 quasisplit 对偶形式（`block_stabilizer_print` 的 `dual_form` 非 quasisplit）的行为无测试锚点，待补。
```