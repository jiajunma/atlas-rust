---
title: crate 根：60 模块组织、52 条再导出与 A1 原型层（lib.rs）
source: atlas-rust/lib-root
ingestedAt: 2026-10-06T13:30:00Z
---

# crate 根：60 模块组织、52 条再导出与 A1 原型层（lib.rs）

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `crates/atlas-real-group/src/lib.rs`（690 行）：crate 的唯一对外
门面 + crate 私有的 A1 迁移原型层。文档定位："Structural data for real
reductive groups"——刻意只含数学值、不含 Atlas 语法或输出策略，是解释器
未来 domain values 的适配边界；所有构造器都校验 C++ 里隐含的根数据不
变量。本包是结构性阅读，不声称数学验收。

## 模块组织与再导出面

60 条 `mod` 声明：5 个 `pub mod`（`deform`、`ext_block`、`ext_kl`、
`ext_param`、`real_weyl`），2 个带 `#[allow(dead_code)]` 与说明注释
（`global_tits`——消费者在 synthetic-real-form builder；`weyl_size`——
停放的 task #9，ON_DEMAND_PARTITION_DESIGN），其余 53 个私有。对外面
**只能**经根部 52 条 `pub use`（含 `topology::dual_component_group_rank`
单独位于模块声明区的一条位置怪点）。`integer_lattice` 有两条 `pub use`；
`deform` 同时以 `pub mod` 与 `pub use` 双重暴露。**声明却零再导出的 5
个模块**：`matreduc`、`real_projection`、`root_reflection`、`global_tits`、
`weyl_size`。根部错误类型 3 个：`StructureError`（唯一汇聚点）、
`RelationError`、`InnerClassLetterError`。`lattice::pair_coordinates` 被
内部使用但不导出（lattice 只导出 `pair` + 4 个格类型）。

## A1 原型层（全部 pub(crate)，impl 注释自述 pending replacement）

- `LatticeVector(Vec<i32>)`：无校验 newtype；文档自述将被 Weight/Coweight
  的编译期对偶格区分取代。
- `RootDatum`（单根基下）：`new` 的校验顺序是 EmptyRootDatum →
  `BasedRootDatum::standard(...)?`（外部校验器**先于**形状检查传播）→
  NonSquareCartan → InvalidCartanMatrix（对角 ≠2 或非对角 >0）。单根 =
  e_i，单余根 = Cartan 第 j **列**。`from_basis` 再逐个 RankMismatch 后按
  行主序核对 `⟨root_i, coroot_j⟩ == cartan[i][j]`（RootPairingMismatch）。
  `roots()`：±单根播种的 FIFO BFS 反射闭包，i128 中间精度 + i32 收窄
  （溢出 → ArithmeticOverflow），第 4097 个互异向量 →
  RootSystemTooLarge（`LIMIT = 4096`）；输出按坐标字典序排序（确定性）。
- `PrototypeWeylGroup`：以元素对全部单根的作用像为键的 BFS；新元素使
  `elements.len() >= 65_536` → WeylGroupTooLarge（成功阶数至多 65_536）。
  `act_on_root` 按词**逆序**反射。
- `RootType`（CompactImaginary/NoncompactImaginary/Real/Complex）、
  `CartanInvolution`（M²=I + 单根像 ∈ 根系；`compact_imaginary` 标志是
  未校验的调用方断言，**已被 `Grading` 取代**）、`RealReductiveGroup`
  （`simple_real_rank` 刻意窄于 real rank；`classify_simple_root` 按
  像==根/负根/其它分类）。

## 测试锚点与限制

3 个测试：A1 反射取负；固定根按标志判非紧虚根；`i32::MAX` 坐标的配对
**恰报 ArithmeticOverflow 而非回绕**（间接要求 StructureError: PartialEq）。
限制：本文件只覆盖门面与原型层；55+5 个模块的实现不在此（各有来源包或
待补）；`StructureError` 全变体清单见 error-global-tits 包；
`weyl_size`/`global_tits` 的消费者不在本文件。原型 `RootDatum` 与
`root_datum` 模块的 `BasedRootDatum` 名称相近但非同物。

## 来源与限制

精确读取身份见
[`2026-10-06-lib-root.json`](snapshots/2026-10-06-lib-root.json)：
绑定 Git base、文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe
（无工具档案）以完整字节起草（600s 期限，exit 0，590.5s），维护者对照
源码逐条核对改写。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。
