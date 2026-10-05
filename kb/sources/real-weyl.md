---
title: 实 Weyl 群与块稳定子：real_weyl.rs 的构造、对偶 fiber 重放与打印层
source: atlas-rust/real-weyl
ingestedAt: 2026-10-05T21:00:00Z
---

# 实 Weyl 群与块稳定子：real_weyl.rs 的构造、对偶 fiber 重放与打印层

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `crates/atlas-real-group/src/real_weyl.rs`（1895 行）。它是结构性
阅读，不声称实 Weyl 层的数学验收；上游引用（realweyl.{h,cpp}、
realweyl_io.cpp、output.cpp、cartanclass.cpp、bitvector.cpp 等行号）仅转录
自代码文档注释，本包未核对上游字节。

## 概览

本文件移植 upstream `realweyl::RealWeyl`（realweyl.h:36-181,
realweyl.cpp:34-69：对 Cartan 类 `cc`、实形代表 `x`、对偶实形代表 `y`，收集
简单虚/实/复根及其子系统类型、紧根基与两个 R-群）、
`realweyl::RealWeylGenerators`（realweyl.h:183-236, realweyl.cpp:81-159）与
打印层 `realweyl_io::common_print`（realweyl_io.cpp:72-146），并移植
`output::printRealWeyl`/`printBlockStabilizer` 的 `x`/`y` 选取
（output.cpp:445-474/361-390：`x = G_C.representative(rf,cn)`，`y` 分别为
对偶伴随 fiber 的零元/对偶形式代表）。上游 gkmod 的 `blockstabilizer` 子系统
不需要：包装只转发 `(rf, cn, drf)`。

## 裸签名清单（完整）

```rust
pub struct LieTypeComponent { pub letter: char, pub rank: usize }  // Copy
pub struct RealWeyl { ..14 个私有字段.. }        // 仅 14 个 getter 暴露切片
pub struct RealWeylContext<'a> { pub inner_class, pub classification,
    pub dual_inner_class, pub dual_classification, pub budget }    // Copy
pub struct RealWeylGenerators { ..7 个私有字段.. } // 仅 getter
pub struct RealWeylGeneratorSection { pub header: String, pub words: Vec<String> }
pub struct RealWeylPrint { pub header, pub summaries, pub generator_sections }
    // render(&self) -> String 给出字节级契约
impl RealWeylContext<'_> {
    pub fn real_weyl(&self, form: WeakRealFormId, cartan: CartanId,
        dual_form: Option<WeakRealFormId>) -> Result<RealWeyl, _>
    pub fn real_weyl_print(&self, form: usize, cartan: usize) -> Result<_, _>
    pub fn block_stabilizer_print(&self, form: usize, cartan: usize,
        dual_form: usize) -> Result<_, _>
}
pub(crate) fn twisted_orbit_size(&RootSystem, &RootInvolutionData,
    &malachite::Integer) -> Result<usize, _>
```

文件私有：`DualSide`、`FiberSide`、`enum PrintKind { RealWeyl,
BlockStabilizer }`（upstream `which_group` 的 `dual_real_W` 未移植）、辅助
`common_print`/`weyl_type`/`two_type`/`format_word`/`fiber_side`/
`simple_basis`/`simple_complex`/`subsystem_cartan`/`lie_type`/
`r_group_elements`/`primal_roots_of_dual`/`reflect_weight`/`accumulate_weight`/
`parity_dot`/`sort_by_upstream_key`/`checked_cartan`。

## 表示与关键不变量

`RealWeyl` 的根列表保存 **primal** `RootId`：`imaginary`/`real` 经
`sort_by_upstream_key` 按 upstream `RootNbr` 序（(height, 反向字典序简单坐标)，
见 `cartan_classification::upstream_positive_key`）；`complex` 例外，为
`makeSimpleComplex` 的输出顺序；对偶侧列表 `real_compact`/`real_orth` 经
coroot 向量映射回 primal（对偶根向量即 primal 余根向量）。
R-群位向量每个 `orth` 条目占一个坐标，顺序为 upstream `BinaryMap::kernel`
（bitvector.cpp:268-277）的核生成元序。

**对偶侧转置配对**：`real_type`/`real_compact_type` 用
`subsystem_cartan(..., transposed=true)`（复现 upstream `drd.subsystem_type`：
对偶子系统 Cartan 矩阵是 primal 的转置）——B/C 互换在此进入（oracle：
`Sp(4,R)` Cartan #3 在 C2 datum 上打印 `W^R is a Weyl group of type B2`，
测试 `(2,3)` 锚定）。其余三个类型用 `transposed=false`。

**`real_r` 填的是对偶侧的 R-群向量**（`real_r: dual_side.r_vectors`），
`imaginary_r` 填 primal 侧——R-群归属是交错的，阅读时注意。

`RealWeylGenerators`：每个列出根一个 Weyl 元素（经
`WeylAction::root_reflection` → `WeylElement::from_action`）；R-群按核位
向量每个一个乘积（置位按 `orth` 升序右乘）；复根为
`s_rn · s_θ(rn)`（先 `reflect(root)` 再右乘 `reflect(image)`，realweyl.cpp:
148-156）。生成元词按构造即 canonical：打印 `WeylElement::canonical_word`，
与 upstream 经 `WeylGroup::word` 所得的 canonical word 相同，故
`reflection_word`/`to_dominant` 机制未移植。

## `real_weyl` 构造流程（顺序固定）

1. `cartan_class(cartan)` 缺失 → `IndexOutOfRange { index: cartan.0, .. }`；
2. `form` 在该类的标签表中无位置 → `RealFormNotDefinedOnCartan`（包装层的
   "Cartan class not defined for real form"）；
3. `x = partition.class_representative(局部位置)`，缺失 →
   `CartanClassificationInvariantViolation("real-form representative")`；
4. `dual_side(cartan_class)`（见下）重建对偶链；
5. `y`：`None` → 对偶伴随 fiber 零元（upstream 硬编码的 quasisplit 代表）；
   `Some` → 对偶标签定位（缺失 → `RealFormNotDefinedOnCartan`）+ 代表元
   （缺失 → `"dual real-form representative"`）；
6. primal 侧：`imaginary`/`real` 简单根排序、`simple_complex`、`fiber_side`；
   对偶侧 `fiber_side` 后经 `primal_roots_of_dual` 把对偶根映回 primal
   （未命中 → `"dual root correspondence"` 不变量错误）；
7. 五个子系统类型 + 组装。

两个打印包装把解释器的外部形式编号经 `ExternalFormOrder` 译为内部
`WeakRealFormId`（越界 → `IndexOutOfRange`），`block_stabilizer_print` 的
`dual_form` 是对偶内类的外部编号（如对偶 quasisplit 形式）。两处的
`.expect("checked cartan id")` 是按构造不可达的潜在 panic 点。

## `dual_side`：对偶 fiber 链的临时重建

upstream 直接读 `cc.dualFiber()`（cartanclass.cpp:121），但 crate 不能复用
对偶分类存储的 fiber：对偶 Cartan 对合 `-θ` 一般只是典范对偶 Cartan 代表
元的**共轭**（`tw * w0`，innerclass.cpp:435-441），故每次调用都临时重建
整条链（无缓存）：`dual_twisted_representative`（以
`longest_action(对偶, weyl_budget)` 右补最长元）→ `CartanFiber::build` →
`AdjointCartanFiber::build` → `CartanGradingData::build` →
`WeakRealFormPartition::build` → `CayleyCrossDecomposition::build` →
`RealFormLabels::build`（对偶 fundamental 类缺失 →
`"dual fundamental class"` 不变量错误）。预算全部取自
`RealWeylContext.budget`（`CartanClassificationBudget`）的相应子预算；
primal 侧读自已预算好的 `classification`。

## `fiber_side`：单侧 fiber 包

输入 `(root_system, involution, grading, element)`，两侧共用：

- 正虚根按 upstream 键排序；非紧判定（`Fiber::noncompactRoots`，
  cartanclass.cpp:706-712 的线性扩张）：基 grading 取为 `<α, ρ∨_im>` 的
  奇偶（`2ρ∨_im` = 正虚余根之和；`Σ_β bracket(α, β)` 必须恒偶，奇则
  `"imaginary-simple coordinates"` 不变量错误），平移项为 ambient 代表与
  根 datum-simple 坐标的 mod-2 点积（`parity_dot`）；紧根累加进
  `two_rho_ic`。
- `compact_basis = simple_basis(compact)`；`orth` = 非紧且与 `two_rho`
  正交者（`orthogonalMAlpha`，realweyl.cpp:234-249；强正交，构成 A_1^n）。
- `r_vectors`（`rGenerators`，realweyl.cpp:264-279）：把每个 orth 根的
  `m_alpha`（fiber 坐标）按行注入 `ModTwoSubspace`，核生成元对每个**自由列
  （升序）**取置位 `[free] + {有 free 位的主元行}`——即 upstream
  `BitMatrix::kernel` 的生成元序。

`simple_basis` 保留 upstream 怪癖（rootdata.cpp:621-652）：候选移除自身
（反射结果非正）时**整个外层扫描终止**，之后的候选不再检查。调用方只传
正根。

`simple_complex`（`CartanClass::makeSimpleComplex`，cartanclass.cpp:1002-1044）：
取与正虚根和、正实根和都正交的正根，求简单基并做 Dynkin 分类；对合成对的
分量只保留其一——向**后**扫描并删除**第一个**含与当前分量最低根之像非正交
顶点的后续分量（每次只删一个）。

`twisted_orbit_size`（`pub(crate)`）：`CartanClass::orbit_size`
（cartanclass.cpp:1041）——复因子只取每个对合配对中的一个分量而非其乘积；
stabilizer 为虚/实/复三个子系统 Weyl 阶之积，整除性失败是不变量错误
（`"integral twisted orbit size"`）。

## 打印层与字节契约

头行逐字节：RealWeyl 为 `real weyl group is W^C.((A.W_ic) x W^R), where:`，
BlockStabilizer 为 `block stabilizer is W^C.((A_i.W_ic) x (A_r.W_rc)),
where:`。摘要 RealWeyl 4 行、BlockStabilizer 5 行；`W^C` 非平凡时带
`isomorphic to ` 前缀。**冒号不一致被保留**（realweyl_io.cpp:105-145）：
`generators for A`/`A_i`/`A_r` 无冒号，`W^C:`/`W_ic:`/`W^R:`/`W_rc:` 有。
`format_word`：空词打印 `e`，否则 1-based 逗号连接。`render` 的字节契约：
每行终止、摘要与节之间恰好一个空行、末节之后无多余字节。

## 测试锚点与 oracle 溯源

测试注释声明：探针 `/tmp/probe_rw_all.at` 对照固定 upstream 构建
（rev 4d3e9449），输出 2026-08-11 重新生成并**逐字节**复制进断言。七个
fixture（InnerClass、弱实形式数、Cartan 类数）：SL(2,R) (2,2)、SU(2,1)
(2,2)、SL(3,R) (1,2)、Sp(4,R) (3,4)、SL(2,C) (1,1)、SL(4,R) (2,3)、
SL(6,R) (2,4)。逐字节输出锚点含 B/C 互换（`Sp(4,R)` (2,3) → B2）、SL(2,C)
的 `W^C is isomorphic to ... A1` 前缀、SL(6,R) 的 rank-2 A 群自由列升序核
生成元。错误路径锚点：形式在该 Cartan 上无定义 → `RealFormNotDefinedOnCartan`；
越界形式/Cartan → `IndexOutOfRange`。

## 与 upstream 的刻意偏差（模块头声明）

1. 生成元词按构造 canonical，`reflection_word`/`to_dominant` 机器未移植；
2. 打印末尾的 `#ifndef NDEBUG` 尺寸断言（及 weylsize 计算）未移植；
3. `printDualRealWeyl`（realweyl_io.cpp:186-195）未移植：无内建包装使用；
   其所需的 imaginary/real 生成元列表仍在计算，后续补齐仅是打印层改动。

## 限制与未覆盖面

- 不做数学/正确性验收；所有 upstream 行号对应均为注释声明，未核对上游
  字节。
- `RealWeyl`/`RealWeylGenerators` 字段不可外部构造；`DualSide`/`FiberSide`
  与全部辅助为文件私有；`twisted_orbit_size` 仅 `pub(crate)`。
- panic 面：两处 `.expect("checked cartan id")`（按构造不可达）；
  `simple_complex` 中 `components[index].support.clone()` 等索引依赖
  `dynkin::classify` 输出与 `rb` 长度一致（阅读观察）。
- `dual_side` 链在每次 `real_weyl` 调用重建（无缓存）；该取舍是否有意未
  确认——这是已知的性能线索候选，本包不做性能声明。
- 预算耗尽路径、非 quasisplit 对偶形式的行为无测试锚点。

## 来源与限制

精确读取身份见
[`2026-10-06-real-weyl.json`](snapshots/2026-10-06-real-weyl.json)：绑定
Git base、文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe（无工具
档案）以完整文件字节起草，维护者对照源码逐条核对改写。本次知识维护未执行
Atlas、Cargo、测试或 benchmark。
