---
title: 弱实形式划分：adjoint fiber 的 W_im 轨道
source: atlas-rust/weak-real-form
ingestedAt: 2026-10-03T10:32:42Z
---

# 弱实形式划分：adjoint fiber 的 W_im 轨道

编辑状态：**结构性阅读（两次：2026-10-03 初读、2026-10-06 重读，字节未变，
SHA-256 相同）；两份草案均经 Kimi probe 起草、维护者逐条对照源码核对后改写合并**。
本包解释 `weak_real_form.rs`（836 行）的划分与归因；其正确性属于它自己的 HPC 证据链
（Cartan/seed gate 等），本包不重述也不扩展。所读字节见
[`snapshots/2026-10-03-weak-real-form.json`](snapshots/2026-10-03-weak-real-form.json)
（初读）与
[`snapshots/2026-10-06-weak-real-form-involution-table.json`](snapshots/2026-10-06-weak-real-form-involution-table.json)
（重读，同一 SHA-256 `fe22088f…`，与 involution_table.rs 同包进行）。

## 定位

`WeakRealFormPartition` 把一个 adjoint Cartan fiber 划分为 $W_{im}$ 轨道；
轨道对应 inner class 在该 Cartan involution 处的弱实形式。划分拥有类表与每类
一个确定性代表元；实形标签住在 `RealFormLabels`，square-class/强实层住在
[强实形式分类](strong-real.md)。

## WeakRealFormId：编号约定

编号按各 $W_{im}$ 轨道在 canonical-coordinate 整数序下的最小元升序指派；
class 0 是 identity 元素的轨道（quasisplit normalization）。stage-(d) 排序审计
（`SEED_X0_DESIGN.md`）**证明**该编号与上游内部 `RealFormNbr` 编号一致（同样的
升序轨道播种、同样的 low-pivot RREF subquotient 基、同样的坐标提取）——这些
id 不在 adapter deferral 之内；只有解释器的外部 `FormNumberMap` 顺序需要
adapter 置换。由 fundamental Cartan 的划分铸造的 id 兼任 crate 的全局实形编号。

`MAX_MASK_BITS = 63`：masks 是 `u64`，adjoint fiber 维数必须使每个
`1 << dimension` 移位保持在范围内。

`walk_mask_orbits` 的实现（2026-10-06 重读补充）：u128 计算的
`1 << dimension` 与 `widened(max_elements)` 比较（**故意不饱和**，防止
`dimension == 64` 借 `usize::MAX` 上限漏过）；按掩码升序播种、LIFO 栈游走，
转移规则 = `FiberAction`：`base[i] XOR parity(mask ∧ alpha_columns[i])` 非紧
则平移 `m_alpha_masks[i]`。类表是 `Vec<u32>`（`CLASS_SENTINEL = u32::MAX`
保留），类号按升序最小掩码赋号，首次出现即代表元；生成元对合性
（`⟨α, α∨⟩ = 2` mod 2）仅 `debug_assert`。`seeded_class` 的守卫在单连通积
（无伴随 `m_alpha`，每掩码自成一类）下原则上可达：类序号放不下 `u32` 或
等于哨兵时报 `limit_error("classes", …)`。

## 构建与查询

`WeakRealFormPartition::build(grading, max_elements)`：对已校验 grading 表背后
的 adjoint fiber 做划分；`max_elements` 是调用方对枚举规模 $2^{dimension}$ 的
上界。访问器：`class_count`/`classes`（升序 `ExactSizeIterator`）、
`class_of`/`class_of_mask`（canonical 坐标 mask 查询，支撑语言级
`fiber_partition`）、`class_representative`（该类在 canonical-coordinate 整数序
下的最小元）、`quasisplit_class`（class 0）、`adjoint_fiber`。

## weak_real_form_at_representative：代表元级归因内核

上游 `real_form_of` 的代表元级内核（atlas-types.w:3878-3894，
innerclass.cpp:1305-1355）：先施加 dual fixed-point projection
$v \mapsto (v + v\theta)/2$（Atlas 的行向量/右乘约定）；投影值的整数配对使
平方中心化，偶数配对标记 noncompact simple-imaginary roots，grading 由此确定
一个局部 adjoint-fiber 轨道，该 Cartan 的标签把此轨道映到 fundamental
weak-form 编号。

前置条件与门控：`twisted` 必须恰为 `classification` 存储的某个代表元（把一般
twisted involution 移到代表元还需经 table-backed Tits cross actions 一并搬运其
torus factor——本 helper 不会隐式构建或扩展该表）。因为
`CartanClassification` 不保留 `InnerClass` 句柄，本函数先把其第一个代表元
重构为归一化 distinguished involution 作为显式 provenance 门（不匹配报
`DatumMismatch`），再检查 `twisted` 的 datum 与 distinguished-involution
分解（不匹配报 `DistinguishedInvolutionMismatch`）。后续的
`minimal_torus_part` 下降还需要目前仍分离的 inverse-Cayley 操作。

闸门顺序（2026-10-06 重读逐行核对）：① 原始因子长度 = 格秩
（`RankMismatch`）；② provenance 门；③ `twisted` 的 datum 门；
④ `w·δ == θ` 分解门（weight+coweight 双矩阵，经 `compose_matrices`）；
⑤ 代表元查找（`"synthetic real-form Cartan representative"`）；⑥ 投影；
⑦ **整性门先于虚 grading 提取**——对**每个单根**（非仅虚根）要求整值配对
（`InvalidStrongTorusFactor`），使实 Cartan（虚基为空）无法绕过；
⑧ 偶配对（`divisible_by(2)`）标记 noncompact 虚根；⑨ grading →
`element_from_grading` → 局部 `class_of` → `labels().label(local)` 得全局
编号（`"synthetic real-form label"`）。

测试锚点（2026-10-06 重读补充，11 个）：合成 A1 锚点（紧/分裂 Cartan 配 0
→ 类 0；紧配 1/2 → 类 1）；秩与半整投影闸门；外来同秩分类先报
`DatumMismatch`；A2 反射的半整投影（Display 文案钉死）；A2 恒等 2 类、
B2 恒等 3 类（代表元精确钉定）、SC A1 平凡作用 2 个单元类、A2 图扭转
1 类；外来元素/欠预算拒绝；rank-33 由预算拒绝而 rank-64 由 mask-bits 拒绝
（诚实的预算门而非秩上限）；`seeded_class` 哨兵守卫直测。未触分支：
`class_of_mask` 越界、各 `ArithmeticOverflow` 转换分支、
边界 `max_elements == 2^dimension` 的显式断言。

## 来源与限制

- 源码：[weak_real_form.rs](../../../crates/atlas-real-group/src/weak_real_form.rs)；
  阅读快照
  [`2026-10-03-weak-real-form.json`](snapshots/2026-10-03-weak-real-form.json)
  （初读）与
  [`2026-10-06-weak-real-form-involution-table.json`](snapshots/2026-10-06-weak-real-form-involution-table.json)
  （重读，同一 SHA-256 `fe22088f…`）。
- 上游行号均转述自源码注释（atlas-types.w/innerclass.cpp/cartanclass.cpp），
  未独立重读上游，随版本演进可能漂移。
- 关联：[强实形式分类](strong-real.md)、[Cartan 分类](cartan-classification.md)、
  [KGB 种子](real-form-seed.md)；`AdjointCartanFiber`、
  `CartanGradingData`、`RealFormLabels` 的展开属于后续来源包。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 初读起草经由本地 Kimi probe（exit 0，164.6s）；重读同样经 Kimi probe
  （1600s 期限，exit 0，658.1s），其闸门顺序、walk_mask_orbits 细节与 11 个
  测试锚点均精确，已并入正文。调用记录见两份快照的 `kimi_assist`。
