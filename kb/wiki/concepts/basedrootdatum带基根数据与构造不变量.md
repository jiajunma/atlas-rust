---
title: BasedRootDatum：带基根数据与构造不变量
summary: 区分格秩与半单秩，按固定顺序校验 Cartan 矩阵、根与余根数量、维度及配对，支持中心环面与纯环面且不设全局秩上限。
sources:
  - root-datum-dual.md
kind: concept
createdAt: "2026-10-09T15:09:56.541Z"
updatedAt: "2026-10-09T21:08:52.045Z"
tags:
  - 根数据
  - Rust设计
  - 构造校验
aliases:
  - basedrootdatum带基根数据与构造不变量
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: BasedRootDatum：带基根数据与构造不变量
summary: 区分格秩与半单秩，按固定顺序校验 Cartan 矩阵、根与余根的数量、维度及配对；支持中心环面和纯环面，不设全局秩上限。
sources:
  - root-datum-dual.md
kind: concept
tags:
  - 根数据
  - Rust设计
  - 构造校验
---

# BasedRootDatum：带基根数据与构造不变量

`BasedRootDatum` 表示在 character/cocharacter 格中经过校验的带基根数据。它区分整个约化环面的格秩 `lattice_rank` 与简单根个数 `semisimple_rank`；后者也是 Cartan 矩阵的阶数，二者仅在无中心环面时相等。类型有意不设置全局秩上限，源材料以 A33 测试作为设计锚点。^[root-datum-dual.md:18-21]

## 数据表示与接口

该类型具有四个私有字段：`lattice_rank: usize`、`cartan: Vec<Vec<i32>>`、`simple_roots: Vec<Weight>` 和 `simple_coroots: Vec<Coweight>`，并派生 `Clone`、`Debug`、`Eq`、`Hash`、`PartialEq`。两个构造入口是 `standard` 与 `from_simple_data`；五个不可失败访问器分别提供格秩、半单秩、Cartan 矩阵、简单根与简单余根。其他操作包括 radical/coradical 基计算、简单反射，以及 crate 内部的可失败复制。^[root-datum-dual.md:29-37]

## 构造不变量与检查顺序

`from_simple_data` 按固定顺序检查输入：首先验证 Cartan 矩阵并确定 `semisimple_rank`；然后要求 `lattice_rank >= semisimple_rank`；再检查简单根与简单余根的个数均等于 `semisimple_rank`；随后检查每个根、余根的 `rank()` 均等于 `lattice_rank`。上述数量或秩条件不满足时返回 `StructureError::RankMismatch`。^[root-datum-dual.md:39-44]

最后逐 `(row, column)` 验证配对关系 \(\langle\alpha_{\mathrm{row}},\alpha^\vee_{\mathrm{column}}\rangle=\mathrm{cartan[row][column]}\)。若不一致，返回 `StructureError::RootPairingMismatch { row, column, expected, actual }`，保留错误位置、期望值与实际值；相关错误体系见 [[StructureError 统一错误分类学]]。^[root-datum-dual.md:45-46]

`standard(cartan)` 取单位坐标向量为简单根，取 Cartan 矩阵的**列**为简单余根，因此总有 `lattice_rank == semisimple_rank`。一般构造允许格秩大于半单秩，以容纳中心环面；也允许空 Cartan 矩阵配合正格秩，表示无根的纯环面。^[root-datum-dual.md:48-51]

## Cartan 矩阵校验

`validate_cartan` 依次拒绝非方阵、对角元不为 2 或非对角元为正、零模式不对称，以及非有限型矩阵。非方阵返回 `NonSquareCartan`，其余上述失败返回 `InvalidCartanMatrix`。零模式对称要求 \(C_{ij}=0\) 当且仅当 \(C_{ji}=0\)。空矩阵通过全部检查，与纯环面的构造边界一致。^[root-datum-dual.md:53-58]

有限型检查使用 `malachite::Rational` 进行两遍精确有理计算：先在每个连通分量以比例因子 1 播种，沿非零 Cartan 边按 `scale[row] * C[row][col] / C[col][row]` 传播，发现冲突即失败；随后对加权矩阵作 LDLᵀ 式分解，任一主元不大于零即失败。参见 [[Cartan 矩阵的精确有理有限型检查]]。该检查与有限型的等价性属于源材料记录的代码意图，并非数学验收结论。^[root-datum-dual.md:60-65]

## 简单反射与算术边界

`reflect_weight(generator, weight)` 先检查权的坐标秩是否等于 `lattice_rank`，再检查生成元下标。相应失败分别返回 `RankMismatch` 与 `IndexOutOfRange { index, upper_bound: semisimple_rank }`。权上的反射为 \(x\mapsto x-\langle x,\alpha_g^\vee\rangle\alpha_g\)，余权上的对偶形式为 \(y\mapsto y-\langle\alpha_g,y\rangle\alpha_g^\vee\)；相关主题见 [[权与余权的简单反射及算术保护]]。^[root-datum-dual.md:81-88]

底层 `reflect_coordinates` 使用 `i128` 中间值，通过 `checked_mul`、`checked_sub` 和 `i32::try_from` 检查算术及收窄转换。分配失败返回 `AllocationFailed`，算术或转换失败返回 `ArithmeticOverflow`。实现先用 `.get()` 检查一个数组，再直接索引另一个数组；其安全性依赖构造时保证简单根与简单余根数组长度均为 `semisimple_rank`。^[root-datum-dual.md:89-92]

## radical 与 coradical 基

`coradical_basis()` 以简单余根坐标为行，调用 `integer_lattice::saturated_kernel` 求整数饱和核，再将各列转换为 `Weight`，得到与所有简单余根正交的权。`radical_basis()` 对称地以简单根为行求核，返回 `Coweight`。计算使用固定预算 `IntegerLatticeBudget::new(64, 100_000, 100_000, 128)`；坐标转换为 `i32` 失败时返回 `ArithmeticOverflow`。详见 [[根数据的 radical 与 coradical 饱和核基]]。^[root-datum-dual.md:67-74]

无根时，辅助函数显式构造 `0 × lattice_rank` 零矩阵，避免从空行推断列数而丢失环境格的秩。空方程组的核是整个环境格；覆盖秩 0、1、2、4 的无根回归测试固定了 radical 与 coradical 均返回单位坐标基的行为。^[root-datum-dual.md:75-79]

`radical_basis` 的文档首行写作“`lattice::perp` of the coroots”，但紧随的定义与实现均使用简单根矩阵的核，即与所有简单根正交的余权。源材料将这一措辞标为疑似笔误，尚未核对所引用的上游字节。^[root-datum-dual.md:193-197]

## 对偶构造与复制

[[对偶根数据与对偶内类构造]] 中的 `dual_datum` 转置 Cartan 矩阵、互换简单根与简单余根，最后调用 `BasedRootDatum::from_simple_data`，复用全部构造校验。其接口保持可失败；源材料没有将已校验 datum 的对偶重校验认定为永远成功。^[root-datum-dual.md:108-112]

crate 内部的 `try_clone` 通过 `try_reserve_exact` 与 `try_copy_coordinates` 逐字段执行可失败复制，然后直接构造 `Self`，不重新运行 `from_simple_data`。源材料覆盖的两个文件中没有该方法的调用点，`dual.rs` 使用派生的 `Clone`；其他文件中的调用情况不在本包范围内。^[root-datum-dual.md:94-99]

## 证据范围

本页依据对 `root_datum.rs` 与 `dual.rs` 的结构性阅读材料，不构成根数据层或对偶构造的数学验收。材料中的上游 C++ 引用仅转录自代码文档注释，未核对上游字节；本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。因此，文中的测试锚点表示来源记录的测试内容，不代表本次运行验证。^[root-datum-dual.md:9-14, root-datum-dual.md:204-216]

## Sources

- [root-datum-dual.md](../../sources/root-datum-dual.md) — BasedRootDatum 与对偶内类构造（root_datum.rs / dual.rs）
