---
title: 内类布局与限制根系（layout.rs / restricted_roots.rs）
source: atlas-rust/layout-restricted-roots
ingestedAt: 2026-10-06T01:40:00Z
---

# 内类布局与限制根系（layout.rs / restricted_roots.rs）

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `crates/atlas-real-group/src/` 的 `layout.rs`（443 行）与
`restricted_roots.rs`（178 行）。两文件互不调用；本包是结构性阅读，不声称
这两层的数学验收；上游引用（atlas-types.w:2829-3010、tori.cpp:189-197、
basic_io.cpp 行号）仅转录自代码注释，本包未核对上游字节。

## layout.rs：InnerClassLayout

`InnerClassLayout` 是一个内类的 `lietype::Layout`（上游 `check_involution`
的计算结果，atlas-types.w:2829-3010）：Lie type（半单因子按打印序，Complex
对相邻，随后每个中心环面维一个 `T1`）、每个内类条目一个字母（`'C'` 消耗
两个 type 因子）、datum 单根的 Bourbaki 置换（`perm[k]` = 规范化序下第 k
个单根的 datum 下标，上游 `Layout::d_perm`）。`budget` 只约束中心环面字母
背后的 Smith 基计算，半单数据不用。

`build` 流程：`twist_permutation`（distinguished 对合把每个单根映到哪个单根
——逐根 `id_of`/`image`/回查/去重，任何一步失败都是
`LayoutInvariantViolation`；这即上游的 `weyl::Twist`）→ `dynkin::classify`
→ `inner_class_letters` → 中心环面分支。

`inner_class_letters`（atlas-types.w:2868-2941）：逐 Dynkin 分支判定：
扭转逐点固定 → `'c'`；扭转仍留在分支内 → `'u'`（偶秩 D）否则 `'s'`；
否则 `'C'`：找含 `twist[perm[offset]]` 的后续分支（缺失 →
`"non-matching Complex factor"`），旋转为相邻，并把第二个因子的 Bourbaki
槽位改写为扭转像。**上游移位顺序被逐字复制**：先上移 `type` 切片、再在
已移位切片上求移位宽度——注释指出这仅在「Complex 对跨越多个介于其间且
秩不同的因子」时可观测，而当前没有任何构造夹具覆盖该情形。

`torus_ranks`（`tori::classify`，tori.cpp:189-197）：商对合从根格的 Smith
基读出（`adapted_basis`；`inverse.block × delta × basis.block`，零跳过）；
`tau1 = inv + I` 经 `classify_plus_identity`（见 involution_classification
包）给出 (compact, complex, split)；环面字母按 `'c'`、`'C'`、`'s'` 顺序
追加。

测试锚点（6 个）：A1 紧 (`c`)；twisted A2 为 `'s'`；A1.A1 交换为 `'C'`
（两因子、perm [0,1]）；D4 叉尖交换为 `'u'`；GL(2) 样式的中心环面：商上
-1 作用给出 `'c','s'` 与 "A1.T1"/"cs"；A1.B2.A1 的 Complex 对旋转重排：
`perm == [0, 3, 1, 2]`。

## restricted_roots.rs：X*/ker(1-θ) 与限制根

`RestrictedWeight`（私有 `Vec<i32>`，派生 Ord/Hash 可作 BTreeMap 键）：
`X*/ker(1-θ)` 的不透明元素，坐标经 `(1-θ)(weight)` 编码等价类——是商到
像格的**单射表示**而非环境权坐标：split A1 中 `alpha` 的类被编码为
`2alpha`，但并不与 `2alpha` 的类等同（文档原例）。`restrict` 逐坐标
`checked_sub`；`doubled`（私有，仅供 `is_multipliable`）逐坐标
`checked_mul(2)`。

`RestrictedRootSystem::build`：先验 datum 一致性（`DatumMismatch`）与格秩
（`RankMismatch`）；按根系枚举序遍历，跳过限制为零的根，余者按
`RestrictedWeight` 键聚合成纤维（`BTreeMap`）；`rank` 取自对合的
`anti_invariant_rank`（-1 特征空间秩，非纤维计数）；`roots` 由 BTreeMap
收集，故按键（坐标字典序）升序。`root(weight)` 二分查找；
`is_multipliable(weight)` 判定「二倍类本身也是一个限制根的类」。

测试锚点（3 个）：split A1：rank 1、两个限制根、`alpha` 纤维 multiplicity
1、不可乘；compact A1：无限制根；A2（对合 [[0,-1],[-1,0]]）：rank 1、
lambda 纤维 multiplicity 2、可乘。

## 两文件的接口关系

互不调用。差异：layout 走 `InnerClass::distinguished_involution()` 且收
`IntegerLatticeBudget`（仅环面分支用）；restricted_roots 走
`RootInvolutionData::involution()`（build 内校验 datum/秩一致）且不收预算。
概念上二者处于同一内类数据流水线的不同环节（一个产出 Lie type/内类字母/
Bourbaki 置换，一个产出限制根及纤维），可作为同一 `InnerClass` 的并行消费
者——此关系未在字节中编码（推断）。

## 限制与未覆盖面

- 不做数学/正确性验收；上游引用未核实。
- 两文件的 `Err` 分支均无失败路径测试。
- 中心环面字母仅测试到 `'s'`；`'c'`/`'C'` 环面字母无锚点。
- `inner_class_letters` 的上游移位顺序怪癖无夹具覆盖（见上）。
- `RestrictedRootSystem::rank` 与限制根条数的一致性未断言；
  `is_multipliable` 的数学性质（如与 BC 型非约化根系的关系）未在本文件
  断言，本包不验收。

## 来源与限制

精确读取身份见
[`2026-10-06-layout-restricted-roots.json`](snapshots/2026-10-06-layout-restricted-roots.json)：
绑定 Git base、两文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe
（无工具档案）以两文件完整字节起草（exit 0，182.9s），维护者对照源码逐条
核对改写。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。
