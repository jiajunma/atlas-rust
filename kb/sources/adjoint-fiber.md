---
title: 伴随 Cartan 纤维：构建、投影与 mod-2 商（adjoint_fiber.rs）
source: atlas-rust/adjoint-fiber
ingestedAt: 2026-10-06T12:30:00Z
---

# 伴随 Cartan 纤维：构建、投影与 mod-2 商（adjoint_fiber.rs）

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `adjoint_fiber.rs`（881 行）：给定已验证的 root-datum 对合与建好的
ambient `CartanFiber`，在伴随半单商上构造有限 F₂ Cartan 纤维，并提供经证明
的源→伴随映射。本包是结构性阅读，不声称数学验收；上游引用仅转录自代码
注释。

## 出处绑定模型

`AdjointBasedRootDatum`：只用源 datum 的 Cartan 矩阵构造
`BasedRootDatum::standard`——character 基 = 源单根全基，cocharacter 基 =
对应基本余权基（文档注释）。`AmbientCoweight`/`AdjointCoweight` 各持
`Arc<AdjointProjectionModel>` + 裸 `Coweight`；手写 `PartialEq` 要求
`Arc::ptr_eq(model)` 且坐标相等——裸 `Coweight` 无 datum 出处，绑定记录
调用方断言并**阻止跨投影复用**（两次独立 build 的投影互拒
`DatumMismatch`，有测试锚定）。

## 构建链 `AdjointCartanFiber::build`

① datum 一致性（`DatumMismatch`）；② 源纤维对合一致性
（`CartanFiberInvolutionMismatch`）；③ `validate_adjoint_build_budget`；
④ 构造投影（克隆源单根 + 伴随 datum）；⑤ `root_basis_action`：以根像的
单根坐标列构造半单秩方阵（三处 `Option::None` →
`InvalidRootAutomorphism`）；⑥ 余权作用 = **转置**（注释：对偶作用本是
逆-转置，而 `RootInvolutionData` 已验证根作用是对合，故逆-转置即转置）；
⑦ `LatticeInvolution::new` + `CartanFiber::build_owned`（整数格预算在此
生效）；⑧ `source.validate_induced_map(&fiber, &projection)`——下降证明
钩子（投影以 `ModTwoAmbientMap` 实现者身份传入）。`AdjointCartanFiber`
持有源纤维克隆 + 投影 + 目标纤维；`fiber_map()` 每次调用再克隆三者构造
`FiberToAdjoint`（不缓存稠密 mod-2 矩阵，每次 apply 现算）。

## 投影的两种形态

- `map_coweight`（整数限制 `Y → P∨`）：`ensure_source`（ptr_eq）→ 预算
  检查 → 对每个源单根算 `pair(root, y)` 按单根序产出目标坐标。注释明示：
  可有中心核，不作同构（测试 2：中心方向映到单位元，且 `map(a+b) ==
  map(a)+map(b)`）。
- `apply_mod_two`（私有，经 `ModTwoAmbientMap` 或 `FiberToAdjoint` 可达）：
  按根系数的奇性逐坐标翻转奇偶位；`bit(i) == None` 的分支按非 1 处理。
- `FiberToAdjoint::apply`：`canonical_representative` → mod-2 投影 →
  `element_from_ambient`。

## 预算与错误分支

`AdjointFiberBudget`（`pub const fn new`）：`integer_lattice`（透传 +
`semisimple_rank > max_rank` 拒 `"semisimple rank"`）；
`max_persistent_entries`（需求量 `16·ss² + ss·lr`，`ADJOINT_PERSISTENT_
SQUARES = 16` 份方阵 + 克隆源单根）；`max_projection_operations`（构建期
预检 `lr²·ss·2`；运行期 `check_projection_work` 按 `lr·target·vector_count`
计费——**每次调用独立检查，无跨调用累计**，阅读观察）。所有预算乘加经
`checked_product`/`checked_sum`（溢出 → `ArithmeticOverflow`）。错误面：
`DatumMismatch`、`CartanFiberInvolutionMismatch`、`RankMismatch`、
`AllocationFailed`（一律 try_reserve_exact）、`AdjointFiberResourceLimit`
（三个 resource 字面量）、`InvalidRootAutomorphism`、`InvalidInvolution`
（transpose 非方阵）、`ArithmeticOverflow`。非测试代码无 panic/assert。

## 测试锚点与限制

9 个测试：A1 恒等的生成元投影与 canonical_representative；中心余权入核
（维数 2→1，可加性）；A2 twisted 的伴随 weight/coweight 矩阵字面量
（`[[-1,1],[0,1]]` / `[[-1,0],[1,1]]`）；异源纤维拒
`CartanFiberInvolutionMismatch`；跨投影坐标拒 `DatumMismatch`；A1+A2
非对称作用的矩阵字面量与**逐坐标基交织关系** `map(θ·y) ==
θ_adjoint·map(y)`；rank-33 动态秩（突破打包上限）；两条预算拒绝路径
（persistent entries / projection operations）。未覆盖：
`AllocationFailed`/`ArithmeticOverflow`/`InvalidRootAutomorphism`/
`InvalidInvolution`/`DatumMismatch`（build 入口）各错误分支；
`coordinates`/`same_class`/`basis_representatives`/`identity` 直接调用。
`AdjointCartanFiber` 无 Eq（值相等不能表达出处——用 `ambient_fiber()`
的确切实例）。`dimension()` 从不枚举纤维元素。

## 来源与限制

精确读取身份见
[`2026-10-06-adjoint-fiber.json`](snapshots/2026-10-06-adjoint-fiber.json)：
绑定 Git base、文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe
（无工具档案）以完整字节起草（800s 期限，exit 0，312.4s），维护者对照
源码逐条核对改写。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。
