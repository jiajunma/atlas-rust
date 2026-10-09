---
title: 紧致 grading：simple-imaginary 根的紧致性位向量
source: atlas-rust/grading
ingestedAt: 2026-10-03T10:32:42Z
---

# 紧致 grading：simple-imaginary 根的紧致性位向量

编辑状态：**结构性阅读（两次：2026-10-03 初读、2026-10-06 重读，字节未变，
SHA-256 相同）；两份草案均经 Kimi probe 起草、维护者逐条对照源码核对后改写合并**。
本包解释 `grading.rs`（601 行）的位向量纪律与 grading 表；其正确性属于它
自己的 HPC 证据链（Cartan/seed gate 等），本包不重述也不扩展。上游行号
（gradings.cpp:20-23、cartanclass.cpp:172 等）均转述自源码注释。

## Grading：位向量的类型纪律

`Grading` 是 `ModTwoVector` 的 newtype：bit `i` 属于所属模型的
simple-imaginary 根列表的第 `i` 项（该列表是本 crate 确定性根序下
`RootInvolutionData::imaginary_simple_roots` 的副本）。置位表示 NONCOMPACT
（gradings.cpp:20-23）。

**刻意与 ambient coweight 坐标区分**：`CartanFiber` 与 `AdjointCartanFiber`
的 mod-two 坐标索引全 datum 的格坐标或 simple roots，而 grading 只索引
simple-imaginary 位置。在 A2 配 identity involution 等常见情形三者维数相同，
维数检查无法区分它们——必须由类型区分。派生的 `Ord` 是任意的确定性
map-key 序，不是数学序。`from_noncompact` 经 `ModTwoVector::from_ones`；
越界索引报 `IndexOutOfRange`（经 `toggle`）。

## CartanGradingData：一个 Cartan involution 的已验证 grading 表

字段：`imaginary_simple_roots`、`simple_mod_two`、`m_alphas`（ambient fiber
类）、`adjoint_m_alphas`（adjoint fiber 类，用 fundamental-coweight 坐标）、
`base_grading`、`grading_shifts`、`adjoint`。

规范化约定：基点（adjoint fiber 元素零）按 quasisplit normalization 把每个
simple-imaginary 根 grade 为 noncompact，故 `base_grading` 为全一；其余元素的
grading 是对其 canonical ambient representative 的 affine-linear evaluation
（实现上逐根取 **`!dot`**——全一基点与配对值的 XOR）。

`build(root_system, root_involution, adjoint)` 的两道门：根系统与 involution
数据的 datum 一致（`DatumMismatch`）；adjoint 的 ambient fiber 的 involution
与 involution 数据一致（`CartanFiberInvolutionMismatch`）。**刻意不接收
ambient fiber 参数**：值相等无法表达 fiber 同一性，故 `m_alpha` 对着
`AdjointCartanFiber::ambient_fiber` 构建——即 adjoint descent 被证明时所用
的确切来源。

逐虚根的收集（2026-10-06 重读补充）：`m_alpha` = 余根在 ambient fiber 的
mod-2 像；伴随 `m_alpha` 经投影（`Pi(y)_j = ⟨α_j, y⟩` 正是其 bracket 向量，
故配对只保留投影内的单一实现）；`simple_mod_two` = 单根坐标的奇性位
（`*coordinate % 2 != 0` **含负奇数**，B2 测试锚定 coroot `(2,−1)` 归约为
`(0,1)`）。`grading_shifts[i]` = 伴随基代表与各单根奇性向量的 F₂ 配对。
`ensure_faithful_shifts`：shift 列线性相关（`insert` 返回 `false`）→
`GradingShiftsNotFaithful`——上游是 `cartanclass.cpp:172` 的**断言**，这里
改为无条件拒绝；注释称没有已知公共构造路径能达到相关列，纯防御（有注入
测试直测重复列与零列）。

## grading 与元素的互转

- `grading(element)`：先 `canonical_representative`（外来纤维元素报
  `CartanFiberMismatch`），再逐根配对取反。
- `element_from_grading(target)`：返回具有所请求 grading 的唯一 adjoint
  fiber 元素。实现是**增广消元**：每列携带其 grading 位加一个标记该 shift
  的 marker 位（第 `imaginary_rank + adjoint_basis_index` 位），归约右端时
  同时累计求解组合；右端是 target XOR base（base 全一，故差标记 target 的
  compact 位置）；余数低 `imaginary_rank` 位有置位 →
  `StructureError::ImpossibleGrading`；否则按标记位把伴随基代表
  `xor_assign` 汇总为 ambient 代表。唯一性 = 构造期检查的 faithful 不变量。
- `grading_shift(adjoint_basis_index)`：shift 表访问器，以
  `adjoint_fiber().dimension()` 为上界。

## 测试锚点（2026-10-06 重读补充）

9 个测试：SC A1 的 quasisplit 规范化（`m_alpha` 非平凡、伴随平凡、双向往返）；
A2 恒等的四元素双射（根序 index 0 = α₂，shift 为置换矩阵）；A2 扭转拒全紧
grading（`ImpossibleGrading`，且 `adjoint.dimension() == 0`、`grading_shift(0)`
为 `None`）；A1×A1 交换的 `imaginary_rank == 0`；中心余权坐标区分 `m_alpha`
与其伴随像；B2 负奇余根坐标归约；外来输入三连拒（`DatumMismatch` /
`CartanFiberInvolutionMismatch` / `CartanFiberMismatch`）；注入相关列被拒；
33 个 A1 因子保持动态（突破 32/64 位打包限制，`noncompact_indices().count() == 33`）。
未覆盖：build 的两处 `IndexOutOfRange`、多数溢出/分配分支、
`element_from_grading` 的入口 `RankMismatch`。

## 来源与限制

- 源码：[grading.rs](../../crates/atlas-real-group/src/grading.rs)；阅读
  快照 [`2026-10-03-grading.json`](snapshots/2026-10-03-grading.json)
  （初读）与 [`2026-10-06-grading-mod-two.json`](snapshots/2026-10-06-grading-mod-two.json)
  （重读，同一 SHA-256 `a9fe00ba…`，重读与 mod_two.rs 同包进行）。
- 关联：[强实形式分类](strong-real.md)、[Cartan 分类](cartan-classification.md)、
  [弱实形式划分](weak-real-form.md)、[mod-2 线性代数](mod-two.md)。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 初读起草经由本地 Kimi probe（exit 0，147.9s）；重读同样经 Kimi probe
  （1300s 期限，exit 0，458.1s），其逐根收集流程、faithful 门与 9 个测试
  锚点均精确，已并入正文。调用记录见两份快照的 `kimi_assist`。
