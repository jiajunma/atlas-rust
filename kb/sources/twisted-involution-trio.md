---
title: 扭对合、对合分类与环境根反射字（twisted_involution.rs / involution_classification.rs / root_reflection.rs）
source: atlas-rust/twisted-involution-trio
ingestedAt: 2026-10-06T05:00:00Z
---

# 扭对合、对合分类与环境根反射字（twisted_involution.rs / involution_classification.rs / root_reflection.rs）

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖三个小而基础的工具文件：`twisted_involution.rs`（215 行）、
`involution_classification.rs`（242 行）、`root_reflection.rs`（72 行）。
三者互不调用，共享 `StructureError`。本包是结构性阅读，不声称数学验收；
上游引用（rootdata.cpp:1092-1095、tori.cpp:162-173 等）仅转录自代码注释。

## twisted_involution.rs：杰出对合的 Weyl 平移

`TwistedInvolution { weyl_action, root_involution }`（均私有）= 一个使
`w·θ` 仍为对合的 Weyl 平移。类型只确立根论条件 `(wθ)² = 1`；Cayley/cross
分解在 `CayleyCrossDecomposition`（cayley-cross 包），典范化在
`InnerClass::canonicalize`。

`new` 的门槛顺序（精确）：① datum 同一性三连查（root_system/distinguished/
weyl_action 任一不符 → `DatumMismatch`）；② 三个 `RankMismatch` 依次核对
（注意 `WeylAction` 用 `rank()`，其余用 `lattice_rank()`，`actual` 各报实测
值）；③ `compose_matrices` 分别复合 weight/coweight 矩阵（Weyl 矩阵 ×
distinguished 矩阵）；④ `LatticeInvolution::new` → `RootInvolutionData::new`
——对合性拒绝发生在这两个构造器内部（`new` 本体无 `InvalidInvolution`
构造点，测试观测到的是传播结果）。存储 WeylAction 与 RootInvolutionData，
不存中间 LatticeInvolution。

`compose_matrices`（`pub(crate)`，被 global_tits 等复用）：i128 checked
累加 + `i32::try_from` 收窄；**怪癖**：形状不符时 `RankMismatch.actual`
恒报 `right.len()`，无论实际哪行 ragged。

测试 4 个：A1 反射平移（Real 根数 2）；A2 阶 3 元 `s0·s1` 被拒
（`InvalidInvolution`）；同 rank 不同 datum（A2 vs B2）→ `DatumMismatch`；
带中心环面的 `diag(1,−1)` distinguished 复合。

## involution_classification.rs：compact/complex/split 计数与纤维秩

`InvolutionClassification`（`Copy`，字段私有）= 整数对合整分解为恒等/交换
对/取负因子时**唯一确定的三个秩**；分解本身刻意不选取、不存储。

- `classify_involution`：形状检查 → **预算闸门**（先
  `IntegerMatrix::from_i32_rows` 记账并立即 drop，使临时矩阵不抬高后续
  live-entry 记账——代码注释明示）→ `is_involution`（i128 checked 三重
  循环，首个不符即短路）→ 构造 `θ+I`（对角 `checked_add(1)`）→ 尾调
  `classify_plus_identity`。
- `classify_plus_identity`（`pub(crate)`，供中心环面商在 Smith 基坐标下复用，
  调用方自负责对合前提）：`plus_rank = rank − ker(θ+I)`（`saturated_kernel`）；
  `complex = (θ+I) mod 2 的像秩`（奇坐标判定用 `entry % 2 != 0`，负奇元同样
  计入）；`compact = plus_rank − complex`；`split = rank − plus_rank −
  complex`；三处 `checked_sub` 下溢 → `IntegerLatticeInvariantViolation`。
- `fiber_rank`：`dualPi0(−θᵀ)` 的 F₂ 维数 = `dim ker((q+I) mod 2) −
  dim span(plusBasis(q)) mod 2`（q = −θᵀ），即 `Cartan_info` 打印的纤维
  大小指数。**防御策略与上文不一致**（阅读观察）：末值用 `saturating_sub`
  钳零而非报错；`kernel_dim = rank − image.rank()` 是普通减法；取负/±1 是
  普通 i32 算术（极端输入 debug 溢出 panic，无防护）。

测试 6 个：恒等 (2,0,0)；A2 反对 (0,1,0)；奇偶区分复/紧+分裂
（`[[1,1],[0,-1]]`→(0,1,0)，`[[1,2],[0,-1]]`→(1,0,1)）；2I 拒；
ragged 拒（形状先于一切）；rank 上限 1 预算下恒等矩阵报
`IntegerLatticeResourceLimit{resource:"rank",limit:1}`（锚定预算闸门在
对合检验之前）。`fiber_rank` 整体无测试。

## root_reflection.rs：共享的环境根反射字

上游 `RootDatum::reflection_word`（rootdata.cpp:1092-1095）的**唯一移植
点**；模块注释禁止消费方另起私有拷贝（extended parameters 与 common block
packet 生成共用此字约定）。`reflection_word(rc, alpha)` =
`to_dominant(reflection(α, 2ρ))`，返回字按上游**原样反转**一次。贪心循环：
每轮从生成元 0 起扫描，取第一个负配对反射，反射后重启扫描；无迭代上限
（终止性依赖数学性质，字节内无护栏）。`wrapping_dot`/`add_scaled` 全程
wrapping i32 算术（溢出静默回绕、无错误路径），长度一致性仅
`debug_assert_eq!`（release 下按 zip 截断）。无文件内测试。

## 接口关系与限制

三文件互不调用；分别服务于对合传输（global_tits 等经 `compose_matrices`）、
Cartan 分类/拓扑（`classify_involution`/`fiber_rank` 的被调方）与表示层
（`RepContext` 消费方）。算术策略对比是有意分层还是漂移，本包存疑备查：
twisted_involution 全 checked；classification 混用 checked/saturating/普通；
root_reflection 全 wrapping。未测路径：`fiber_rank`、`restricted_roots`、
`compose_matrices` 的直接测试、`reflection_word` 全函数。

## 来源与限制

精确读取身份见
[`2026-10-06-twisted-involution-trio.json`](snapshots/2026-10-06-twisted-involution-trio.json)：
绑定 Git base、三文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe
（无工具档案）以三文件完整字节起草（300s 期限，exit 0，274.4s），维护者
对照源码逐条核对改写。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。
