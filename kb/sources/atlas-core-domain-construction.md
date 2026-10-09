---
title: 领域构造管线（domain_builtins.rs 1628–2527）——datum/内类/实形的构造顺序与预算
source: atlas-rust/atlas-core-domain-construction
ingestedAt: 2026-10-09T15:30:00Z
---

# 领域构造管线（domain_builtins.rs 1628–2527）

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包覆盖
`crates/atlas-core/src/domain_builtins.rs` 的构造管线区域：`build_datum`/
`build_quotient_datum`/`build_quotient_from_handle`/`build_explicit_datum`/
`build_inner_class_context`/`build_inner_class`/`build_dual_inner_class`/
`build_real_form`/`build_custom_real_form`（1628–2527 行）。这是 Weyl 弧的
中心区（内类构造 = Weyl 上下文的双侧管线）。结构性阅读，不声称数学验收。

## `build_datum`（1628）：由 Lie 类型造根数据

- `block_cartan` 得半单 Cartan；`simply` 分两路：**单连通用权格基**
  （根 = Cartan 行、余根 = 基），**伴随用根格基**（根 = 基、余根 =
  Cartan 的**列**）；坐标零填充到格秩。
- `BasedRootDatum::from_simple_data` 校验；`lattice_rank != semisimple` 时
  isogeny 为 `Other`，否则 `classify_isogeny`。
- `RootDatum::type`（rootdata.cpp:1016）把根部的 T1 因子追加在半单类型
  **之后**——上面构造的简单数据已用此序；**不**保留交错的输入环面、也
  不改变调用方的 LieType 值（环面因子教训）。

## 商与显式 datum

- `build_quotient_datum`（1682）/`build_quotient_from_handle`（1702）：
  中央商数据（中间商，不只是 SC/伴随端点——覆盖指令要求的中间商）。
- `build_explicit_datum`（1846）：显式根/余根矩阵（空维保留矩阵维数，
  见 [派发包](atlas-core-domain-dispatch.md)的 `as_matrix_rows` 0xN
  表示）。

## `build_inner_class_context`（2197）：内类装配顺序

固定顺序 + 预算门：`classification_cached`（分类预算）→
`StrongRealClassification::build`（FIBER_BUDGET）→ `ExternalFormOrder::build`
→ `InnerClassLayout::build`（INTEGER_BUDGET）→ **对偶侧只建一次**
（`dual_inner_class` + 其对偶分类 + `dual_form_count` = 对偶弱实形计数 +
`dual_cartan_correspondence`——上游对偶 InnerClass 构造器，
innerclass.cpp:435）→ `build_presentations` → `canonical_forms`
（每形式一个 `Weak` 槽的 Mutex 向量，规范实形弱缓存的槽位表）。

## 内类与对偶内类

- `build_inner_class`（2254）：coweight 部 = 转置；`LatticeInvolution::new`；
  `InnerClass::from_root_involution`——上游接受**任何**根数据对合再左合成
  为 distinguished（`check_involution`，atlas-types.w:2829）。
- `build_dual_inner_class`（2276）：`dual_inner_class` + 对偶 datum 的
  handle：**余根偏好翻转**（RootSystem DualTag，rootdata.cpp:341）+
  逐字母对偶的 Lie 类型——这就是修复弧设计里的 dual-identity 路径
  （等值内容的对偶 datum 经内容弱 interning 共享 Weyl 身份，见
  [领域值包](atlas-core-domain-values.md)）。

## 实形构造

- `build_real_form`（2404）：`order.internal(external)` 翻译外部形式号
  （非法号 → `Illegal real form number: n`）；`canonical_forms` 弱缓存
  命中则共享一个上下文；自定义构造总是新建属主。
- `build_custom_real_form`（2490）：`fresh_table`（带 FIBER/INTEGER 预算的
  InvolutionTable）→ 加**基本**（第一个）Cartan → `RealFormSeed::custom`
  （内部形式号 + 余特征标 + 环面部）；`RealFormContext` 持有
  `FallibleOnce` 的 kgb/rep 与两个形变缓存（普通/扭曲，
  DeformationCache）。

## 边界与限制

- 形变缓存机器（2657–2933）、`strong_components`、RootNumbering、
  CenterClassifier、RootTable、validate/print_text 各待分包；身份机器见
  [领域值包](atlas-core-domain-values.md)，派发见
  [派发包](atlas-core-domain-dispatch.md)。
- 上游行号引用是**实现方移植陈述**；构造兼容以 HPC 差分门为准。
- 字节数/哈希只标识本快照字节（git base `964f0033`，domain_builtins.rs
  sha256 `e6987e7c…`——与已验收 after-v5 清单同字节）。
