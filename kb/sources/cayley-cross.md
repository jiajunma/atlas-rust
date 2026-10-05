---
title: Cayley/Cross 分解与整对合分类（cayley_cross.rs / involution_classification.rs）
source: atlas-rust/cayley-cross
ingestedAt: 2026-10-05T22:50:00Z
---

# Cayley/Cross 分解与整对合分类（cayley_cross.rs / involution_classification.rs）

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `crates/atlas-real-group/src/` 的 `cayley_cross.rs`（588 行）与
`involution_classification.rs`（242 行）。两文件在所给字节内互不引用，仅共享
`try_capacity`/`StructureError` 等基础设施；本包是结构性阅读，不声称两条线的
数学验收；上游引用仅转录自代码注释。

## cayley_cross.rs：twisted involution 的 Cayley/cross 因子化

`CayleyCrossDecomposition` 存储四部分：`cayley_roots`（RootId 列表）、
`cross_word`（**生成器下标**而非 RootId）、`cross_action`（重放复合的
WeylAction）、输入 `twisted` 的克隆。构造期不变量（文档声明，构造时验证）：
从 distinguished involution 出发，先按序对 cross word 做 twisted 共轭、再
依次左乘每个 Cayley 根的反射，可复现输入；每个 Cayley 根在其重放步上为
imaginary。**两部分在 ports 间不唯一**（Atlas 按其内部 transducer 顺序
peel），跨实现 diff 比较必须用 replay-invariant 或 label 级结果，不得使用
这些原始部分。

`build` 的顺序：

1. **provenance 门**：`twisted.weyl_action().datum()` 不符 → `DatumMismatch`；
   存储对合必须恰为 `w∘δ`（weight 与 coweight 两侧矩阵都比较，否则
   `DistinguishedInvolutionMismatch`）。该门同时支撑终止性论证
   （迫使 `w^{-1} = δwδ`）。
2. **生成器表与 twist 置换**：`simple_ids` 逐个 `id_of` 简单根；`twist[g]` =
   δ 把第 g 个简单根的像映到哪个生成器下标（缺失 →
   `InvalidBasedAutomorphism`）。
3. **peeling 循环**（lowest external descent first）：按生成器升序取第一个
   descent——判定为 `δ(θ(α_g))` 的简单坐标全 ≤ 0。无 descent 且当前作用
   非单位 → `"peeling termination"` 不变量错误。预算检查在**找到 descent
   之后、步进之前**：`steps == max_peeling_steps` →
   `CayleyCrossResourceLimit { resource: "peeling steps" }`（故零需求输入在
   预算 0 下可通过）。字母由根类决定：Real → Cayley 字母 +
   `s_g ∘ current`；Complex → Cross 字母 + `s_g ∘ current ∘ s_{twist[g]}`
   （一步复合两次反射但预算只计一步）；Imaginary → `"descent kind"`（注释：
   imaginary 根不可能是 descent）。
4. **逆序重放收集**：Cayley 字母把 `simple_ids[g]` 追加进 Cayley 集合；
   Cross 字母追加 `cross_word` 并**反射每个已收集的 Cayley 根**。
5. **Cayley 集合后处理**：`ensure_pairwise_orthogonal`（两个方向的 bracket
   都须为 0）；`long_orthogonalize`：正交短对（其和为根——B2 对）换成长
   和与差（差必须为根，否则 `"B2 pair"`），每次替换严格增加长根数故终止；
   逐根 `positive_form` 取正，最终升序排序。输出保证（访问器文档）：
   强正交、正根、升序。
6. **重放验证**：从 identity 出发按序施加 cross 字母，再对（已排序的）每个
   Cayley 根检查其在该重放步为 imaginary（否则 `"Cayley root imaginary"`）
   并左乘其根反射；终态不等于输入 → `"replay equality"`。

测试锚点（6 个）：两种 distinguished 下 identity 分解为空；A1×A1 的
identity+`s0` → 一个 Cayley 根、swap+`s0∘s1` → 一个 cross 字母；A2 最长元
`s0s1s0` → Cayley 根 [1,1] 加 cross_word [0]；B2 短/长 pinning 两例锚定
长根化的生效与 no-op；A2/B2 全枚举 twisted involution 均分解且重放相等、
Cayley 根两两正交；三条负路径（预算 0、异 datum、异 backing）精确匹配。

## involution_classification.rs：整对合的因子计数与分量群秩

`InvolutionClassification { compact, complex, split }`：整对合的
identity/exchanged-pair/negated 三类整分解的**个数**；分解本身被刻意不选定、
不存储。crate 外只能经 `classify_involution` 获得值。

`classify_involution(matrix, budget)` 的顺序：方阵形状检查
（`InvalidIntegerMatrixShape`，最先）→ **预算门**
（`drop(IntegerMatrix::from_i32_rows(...))` 在立方复杂度的对合检查之前强制
执行 rank/存储/系数预算）→ `is_involution`（i128 全程 checked 计算 M² 与
单位阵逐元比较）→ 构造 `θ + I`（对角 `checked_add(1)`）→
`classify_plus_identity`。

`classify_plus_identity`（`pub(crate)`，供 central-torus 商计算在 Smith 基
坐标下复用；对合前提由调用方负责）：`plus_rank = rank − rank(ker(θ+I))`
（饱和核）；`complex = (θ+I) mod 2 的行空间 F₂ 秩`（奇数条目，含负奇数）；
`compact = plus_rank − complex`；`split = rank − plus_rank − complex`。算术
恒等式 `compact + 2·complex + split == rank` 直接可读。

`fiber_rank(weight_matrix, budget)`：对偶分量群 `dualPi0(-θ^T)` 的 F₂ 维数
（tori.cpp:162-173、subquotient.h:79；`Cartan_info` 打印的 fiber size 指数）：
`dim ker((q+I) mod 2) − dim span(plusBasis(q)) mod 2`，`q = -θ^T`。前半用
mod-2 子空间秩，后半用 `q − I` 的饱和核经 `reduce_basis_mod_two`；结尾是
**`saturating_sub`**（不报错），与 `classify_plus_identity` 的
`checked_sub` 风格相反（阅读观察）。`fiber_rank` 无对合前提检查，且在本
文件中没有任何测试。

测试锚点（6 个）：I₂ → (2,0,0)；A2 反向 `[[0,-1],[-1,0]]` → (0,1,0)；
奇偶区分（`[[1,1],[0,-1]]` → (0,1,0)，`[[1,2],[0,-1]]` → (1,0,1)）；
非对合 2·I₂ → `InvalidInvolution`；ragged 矩阵形状错误先于化简；
预算门先于分类（rank 1 预算下 I₂ 报 `IntegerLatticeResourceLimit`）。

## 限制与未覆盖面

- 不做数学/正确性验收；doc 声明（peeling 每步长度减 1 或 2、长根计数严格
  递增故终止、imaginary 根不可能是 descent、跨 port 不唯一）如实转述未验证。
- `cayley_cross` 的失败分支覆盖不全：六种 `CayleyCrossInvariantViolation`
  不变量串均无专门负测试；A1×A1 swap 用例只固定 `cross_word.len() == 1`
  而未固定内容。
- `fiber_rank` 无任何测试；`classify_plus_identity` 仅经
  `classify_involution` 间接覆盖。
- 两文件互不引用（在所给字节内）；是否存在更深调用链（如 `InnerClass`
  内部使用 `classify_involution`）超出本包范围。

## 来源与限制

精确读取身份见
[`2026-10-06-cayley-cross.json`](snapshots/2026-10-06-cayley-cross.json)：
绑定 Git base、两文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe
（无工具档案）以两文件完整字节起草，维护者对照源码逐条核对改写。本次
知识维护未执行 Atlas、Cargo、测试或 benchmark。
