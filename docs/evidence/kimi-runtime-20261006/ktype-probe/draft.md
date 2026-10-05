---
title: KType：K 型值及其 RepContext 操作（atlas-real-group/src/ktype.rs）知识来源包草案
source: atlas-rust/ktype
ingestedAt: 2026-10-06T00:00:00Z
---

# KType：K 型值及其 `RepContext` 操作（`crates/atlas-real-group/src/ktype.rs`）

> 本草案仅依据所给文件字节撰写。条目按【代码事实】（字节中可直接读出的行为）、【文档声明】（注释/doc 中的断言，未在本文件内验证）、【阅读推断】（从代码形态推出、需维护者确认的结论）三类标注。所有上游引用（`K_repr.h` / `K_repr.cpp` 行号）均来自注释，属【文档声明】。

## 0. 文件定位与导入

- 模块 doc：本文件定义 `K_type` 值及其 `Rep_context` 操作，对应上游 `gkmod/K_repr.h`、`gkmod/K_repr.cpp`。【文档声明】
- 模块 doc 核心主张：【文档声明】
  - `KType` 是标准表示的 K-限制 / K 的不可约表示，即 `crate::StandardRepr` 参数数据去掉 `nu` 部分（K_repr.h:25-30）。
  - 存储的是当选（elected）`lambda-rho` 代表元；模 `(1-theta_x)X^*` 的规范化在 `KType::sr_k` 中发生一次（K_repr.cpp:25-32）。
  - 值相等是上游的严格逐分量相等（K_repr.h:56-60）；`equivalent` 关系通过移到典范对合计算（K_repr.cpp:159-171）。
- 导入【代码事实】：
  ```rust
  use crate::lattice::{checked_add_weights, checked_sub_weights, pair};
  use crate::rep_context::RepContext;
  use crate::{KgbId, KgbStatus, RootId, StructureError, Weight};
  ```

## 1. 裸签名清单（含无注释项）

### 1.1 类型定义

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KType {
    x: KgbId,        // 私有字段
    lam_rho: Weight, // 私有字段
    height: u32,     // 私有字段
}
```
- 无 pub 字段；无手工 `Eq/PartialEq` 实现，相等性来自 derive（逐字段）。【代码事实】
- 本文件内无其它 `pub struct/enum/trait`。【代码事实】

### 1.2 `impl KType`（唯一 impl 块）

| 可见性 | 签名 | 有无 doc |
|---|---|---|
| `pub(crate)` | `fn new(x: KgbId, lam_rho: Weight, height: u32) -> Self` | 有 |
| `pub` | `fn sr_k(rc: &RepContext, x: KgbId, lambda_rho: &Weight) -> Result<Self, StructureError>` | 有 |
| `pub` | `fn x(&self) -> KgbId` | **无** |
| `pub` | `fn lambda_rho(&self) -> &Weight` | 有 |
| `pub` | `fn height(&self) -> u32` | **无** |
| `pub(crate)` | `fn theta_plus_1_lambda(&self, rc: &RepContext) -> Result<Weight, StructureError>` | 有 |
| （私有） | `fn theta_plus_1_eval(&self, rc: &RepContext, alpha: RootId) -> Result<i32, StructureError>` | 有 |
| `pub` | `fn is_standard(&self, rc: &RepContext) -> Result<bool, StructureError>` | 有 |
| `pub` | `fn is_dominant(&self, rc: &RepContext) -> Result<bool, StructureError>` | 有 |
| `pub` | `fn is_nonzero(&self, rc: &RepContext) -> Result<bool, StructureError>` | 有 |
| `pub` | `fn is_semifinal(&self, rc: &RepContext) -> Result<bool, StructureError>` | 有 |
| `pub` | `fn is_normal(&self, rc: &RepContext) -> Result<bool, StructureError>` | 有 |
| `pub` | `fn is_final(&self, rc: &RepContext) -> Result<bool, StructureError>` | 有 |
| `pub` | `fn equivalent(&self, rc: &RepContext, other: &KType) -> Result<bool, StructureError>` | 有 |
| `pub` | `fn made_dominant(&self, rc: &RepContext) -> Result<KType, StructureError>` | 有 |
| `pub` | `fn made_theta_stable(&self, rc: &RepContext) -> Result<KType, StructureError>` | 有 |
| `pub` | `fn to_canonical_fiber(&self, rc: &RepContext) -> Result<KType, StructureError>` | 有 |
| `pub` | `fn normalised(&self, rc: &RepContext) -> Result<KType, StructureError>` | 有 |
| `pub` | `fn finals_for(&self, rc: &RepContext) -> Result<Vec<(KType, i32)>, StructureError>` | 有 |
| `pub` | `fn kgp_set(&self, rc: &RepContext) -> Result<Vec<KType>, StructureError>` | 有 |

### 1.3 自由函数

```rust
fn simple_generator(rc: &RepContext, simple: RootId) -> Result<usize, StructureError> // 私有，有 doc
```

### 1.4 `#[cfg(test)] mod tests`（私有模块）

- 辅助：`fn class_budget(weyl: usize) -> CartanClassificationBudget`；`fn with_split_a1<T>(f: impl FnOnce(&RepContext<'_>, &crate::KgbGraph) -> T) -> T`；`fn with_su21<T>(...) -> T`（同形）。【代码事实】
- 测试函数（均 `#[test]`）：`split_a1_k_type_anchors_match_the_frozen_contract`、`split_a1_finals_for_a_final_parameter_is_itself`、`split_a1_finals_for_a_non_dominant_parameter_reflects`、`split_a1_finals_for_a_negative_parameter_descends`、`split_a1_reducibility_points_are_empty_for_gamma_half`、`split_a1_standard_repr_anchors_match_the_frozen_contract`、`a2_su21_context_builds_all_involutions_and_pins_nonfinal_anchors`、`su21_deform_parameter_shapes_match_the_frozen_contract`、`su21_finals_for_singular_gamma_zero`。【代码事实】

## 2. 数据结构与 K 型/参数层职责

- `KType { x, lam_rho, height }`：三元组。struct doc 称存储的 `lam_rho` 恒为其 `(1-theta_x)X^*`-陪集的 `lambda_unique` 代表元，`height` 在构造时预计算（K_repr.h:36-44）。【文档声明】
- 【代码事实】该不变量仅由构造路径维护：`new` 是原始构造器，**不做**任何规范化或校验（doc 明说“normalization is `sr_k`'s job”）；`sr_k` 做规范化并计算 height；`finals_for` 内部也用 `KType::new(x, normalized, height)` 直接装入沿用的高度。
- 【阅读推断】由于字段私有且唯一 impl 在本文件，crate 外部无法绕过 `sr_k` 构造；但 crate 内部（`pub(crate) fn new`）可装入任意 `height`，不变量靠调用方纪律维持。

## 3. 逐项解释

### 3.1 `new`（pub(crate)）
- 原始构造，逐字段装入。doc 称对应上游 struct 构造器（K_repr.h:39-44）。【文档声明】无错误分支。【代码事实】

### 3.2 `sr_k`（pub）— 规范化构造
- doc：对应 `Rep_context::sr_K(KGBElt, Weight)`（K_repr.cpp:25-32）；将 `lambda_rho` 模 `(1-theta_x)X^*` 规范化，并存储 `(1+theta_x)*lambda` 的 height。【文档声明】
- 算法步骤【代码事实】：
  1. `rc.involution_of(x)?`；
  2. `rc.lambda_unique(involution, lambda_rho)?` 得 `normalized`；
  3. `rc.theta_at(x)?`，`theta.act_on_weight(&normalized)?`；
  4. `th1_lambda = checked_add_weights(checked_add_weights(&normalized, &theta*normalized)?, &rc.theta_plus_one_rho_at(x)?)?`；
  5. `height = rc.height(&th1_lambda)?`；
  6. `Ok(Self::new(x, normalized, height))`。
- 错误分支：全部来自上述 `?` 传播（本函数自身不构造错误值）。【代码事实】

### 3.3 访问器 `x` / `lambda_rho` / `height`（pub）
- 分别返回 `KgbId`（复制）、`&Weight`、`u32`；`x()` 与 `height()` 无 doc 注释。无错误分支。【代码事实】

### 3.4 `theta_plus_1_lambda`（pub(crate)）
- doc：对应 `Rep_context::theta_plus_1_lambda`（K_repr.cpp:16-23），公式 `(1+theta_x)*lambda = lambda_rho + theta*lambda_rho + (1+theta)rho`。【文档声明】
- 实现与 `sr_k` 中 `th1_lambda` 的计算同形，对存储的 `lam_rho` 现算。【代码事实】
- 注意：`sr_k` 的 height 取自构造时的该值；之后 `lam_rho` 被反射改变时 height 不随之重算（见 3.9、3.12）。【代码事实】

### 3.5 `theta_plus_1_eval`（私有）
- doc：对应 `Rep_context::theta_plus_1_eval`（K_repr.cpp:35-43）；计算 `<alpha^v + (theta alpha)^v, lambda>` 的整数值，谓词集只用其符号与零测试。【文档声明】
- 算法【代码事实】：`beta = rc.root_involution_image_at(self.x, alpha)?`；取 `system.coroot(alpha)`、`system.coroot(beta)`（`ok_or` 转 `IndexOutOfRange`）；`first = pair(&self.lam_rho, coroot)? + rc.colevel(alpha)?`（`checked_add`，溢出转 `ArithmeticOverflow`）；`second` 同形（对 `beta`）；返回 `first.checked_add(second)`，溢出转 `ArithmeticOverflow`。
- 错误分支：`IndexOutOfRange { index: alpha.0, upper_bound: system.roots().len() }`、同形 `beta.0`、`ArithmeticOverflow`，及 `root_involution_image_at`/`colevel`/`pair` 的传播错误。【代码事实】

### 3.6 谓词组（均 `pub fn … -> Result<bool, StructureError>`）

- **`is_standard`**（K_repr.cpp:46-57【文档声明】）：遍历 `rc.imaginary_simple_roots_at(self.x)?`；对每个根取 coroot（`ok_or IndexOutOfRange`），`eval = pair(lam_rho, coroot)? + colevel(alpha)?`（`checked_add`→`ArithmeticOverflow`）；`eval < 0` 即返回 `Ok(false)`；否则 `Ok(true)`。doc：lambda 在单虚余根上弱支配。【代码事实+文档声明】
- **`is_dominant`**（K_repr.cpp:59-69【文档声明】）：遍历 `system.simple_root_ids().iter().enumerate()`，**忽略枚举下标**（`let _ = generator;`），对每个单根 `theta_plus_1_eval(rc, simple)? < 0` 即 `Ok(false)`。【代码事实】
- **`is_nonzero`**（K_repr.cpp:71-83【文档声明】）：遍历单虚根，`eval == 0 && !rc.simple_imaginary_grading(self.x, alpha)?` 即 `Ok(false)`。doc 声明假设 `is_standard` 已成立，但【代码事实】函数体内不检查此前提。
- **`is_semifinal`**（K_repr.cpp:85-100【文档声明】）：构造测试权重 `2*lam_rho + (two_rho − two_rho_of(positive_real))`：
  - `doubled` 用 `try_reserve_exact(self.lam_rho.rank())`，失败转 `AllocationFailed { requested: rank }`；逐分量 `checked_mul(2)`，溢出转 `ArithmeticOverflow`；
  - `two_rho_of` 的输入是 `rc.positive_real_roots_at(self.x)?`；减法用 `checked_sub_weights`，加法用 `checked_add_weights`；
  - 遍历 `rc.real_simple_roots_at(self.x)?`，`pair(&test_weight, coroot)? % 4 != 0` 即 `Ok(false)`。
  【代码事实】doc 称语义为“没有 really-simple 根在测试权重上为奇”。【文档声明】
- **`is_normal`**（K_repr.cpp:102-123【文档声明】）：遍历全部单根，`rc.is_complex_descent(self.x, simple_generator(rc, simple)?)? && self.theta_plus_1_eval(rc, simple)? == 0` 即 `Ok(false)`。doc 声明：上游断言 `is_standard && is_dominant && is_nonzero && is_semifinal` 且只在形容词链中调用；本移植因“计算本身是 total 的”而无前提求值。【文档声明；代码事实：函数体确实不做前提检查】
- **`is_final`**（K_repr.cpp:126-157【文档声明】）：`for generator in 0..datum.semisimple_rank()`，`eval = theta_plus_1_eval(simple_root_ids()[generator])?`；`eval < 0` → `Ok(false)`；`eval == 0` 时按 `rc.kgb_status(self.x, generator)?` 分派：
  - `KgbStatus::ImaginaryCompact` → `Ok(false)`；
  - `KgbStatus::Real` → `pair(&self.lam_rho, &datum.simple_coroots()[generator])? % 2 != 0` → `Ok(false)`；
  - `KgbStatus::Complex` → `rc.is_complex_descent(self.x, generator)?` → `Ok(false)`；
  - `KgbStatus::ImaginaryNoncompact` → 不做事（继续）。
  【代码事实】

### 3.7 `equivalent`（pub）
- doc：对应 `Rep_context::equivalent`（K_repr.cpp:159-171）：先同 Cartan 类，再各自移到该类典范对合后严格相等。【文档声明】
- 实现【代码事实】：`rc.graph().cartan_of(self.x)`、`cartan_of(other.x)` 分别 `ok_or(IndexOutOfRange { index: x.index(), upper_bound: rc.graph().size() })`；Cartan 不同 → `Ok(false)`；否则 `self.to_canonical_fiber(rc)? == other.to_canonical_fiber(rc)?`（用 derive 的 `==`）。

### 3.8 `made_dominant`（pub）
- doc：对应 `Rep_context::make_dominant`（K_repr.cpp:174-204）；对 `(1+theta)lambda` 取负求值的复单根做 cross 直至支配；非标准输入报错（上游消息 “Non standard K-type in make_dominant”）；存储的 height 不变并原样携带。【文档声明；height 不变性属注释断言，未验证】
- 实现【代码事实】：
  - 先 `z.is_standard(rc)?`，为假则 `Err(RepInvariantViolation { invariant: "standard K-type in make_dominant" })`；
  - 预算：`remaining_steps = rc.weight_defect(&z.theta_plus_1_lambda(rc)?)?`；
  - 循环：扫描 `0..semisimple_rank()`，首个 `theta_plus_1_eval < 0` 的生成元处执行 `z.x = rc.cross_at(z.x, generator)?`、`rc.simple_reflect(generator, &mut z.lam_rho, 1)?`，`break` 内层；无反射则 `Ok(z)`；
  - 每轮 `remaining_steps -= 1`，`< 0` 则 `Err(RepInvariantViolation { invariant: "dominance termination" })`；
  - 每轮末尾重新 `z.lam_rho = rc.lambda_unique(rc.involution_of(z.x)?, &z.lam_rho)?`。
- 【阅读推断】`remaining_steps` 为带符号整型（与 0 比较、在 `normalised` 中与 `i64` 相加），具体类型由 `weight_defect` 返回类型决定，本文件不可见。

### 3.9 `made_theta_stable`（pub）
- doc：对应 `Rep_context::make_theta_stable`（K_repr.cpp:207-233）；耗尽单复下降；每次 cross 都是下降，故以图大小为“慷慨的”终止上界。【文档声明】
- 实现【代码事实】：外层 `for _ in 0..=rc.graph().size()`（即至多 size+1 轮）；内层首个 `rc.is_complex_descent(z.x, generator)?` 为真的生成元处 cross + `simple_reflect(generator, &mut z.lam_rho, 1)?`；无反射时先 `lambda_unique` 重规范化再 `Ok(z)`；外层耗尽则 `Err(RepInvariantViolation { invariant: "theta-stable termination" })`。
- 注意：本函数不改 `height`（clone 携带）。【代码事实】

### 3.10 `to_canonical_fiber`（pub）
- doc：对应全生成元集的 `Rep_context::to_canonical_involution`（K_repr.cpp:236-256）；沿 `InnerClass::canonicalize` 的词 cross 到 Cartan 类的当选纤维。【文档声明】
- 实现【代码事实】：`rc.involution_of(self.x)?` → `rc.table().record(involution)`（`ok_or IndexOutOfRange { index: involution.0, upper_bound: rc.table().involution_count() }`）→ `.twisted_involution().clone()` → `rc.inner_class().canonicalize(twisted)?` 取 `(_, word)`；对 `word` 中每个生成元：`!rc.is_complex_simple(z.x, generator)?` → `Err(RepInvariantViolation { invariant: "canonical fiber cross" })`，否则 cross + 反射；结尾 `lambda_unique` 重规范化。

### 3.11 `normalised`（pub）
- doc：对应 `Rep_context::normalise`（K_repr.cpp:262-289）：先到典范对合，再耗尽奇异复下降（与负复求值）。【文档声明】
- 实现【代码事实】：先 `to_canonical_fiber`；预算 `remaining_steps = weight_defect(theta_plus_1_lambda)? + i64::try_from(rc.graph().size()).map_err(|_| ArithmeticOverflow)? + 1`（注释称：负求值 cross 降低支配亏数、奇异下降 cross 降低对合长度，两者之和为界【文档声明】）；循环中首个满足 `kgb_status == Complex && (eval < 0 || (eval == 0 && is_complex_descent))` 的生成元处 cross + 反射；无反射则重规范化后 `Ok(z)`；预算耗尽 `Err(RepInvariantViolation { invariant: "normal form termination" })`。

### 3.12 `finals_for`（pub）— final K 型展开
- doc：对应 `Rep_context::finals_for(const K_repr::K_type&)`（K_repr.cpp:290-396）；返回带号重数表 `Vec<(KType, i32)>`，**无序**，系数合并在语言层发生；final 输入恰好返回自身重数 1。【文档声明】
- 实现要点【代码事实】：
  - 工作栈 `todo = vec![(self.clone(), 1_i32)]`，`pop` 取值；`coef: i32` 只会被取负（`coef = -coef`）；
  - 每个待处理项取出时记录 `height = ktype.height()`，并最终以 `KType::new(x, normalized, height)` 装入结果（注释称 height 在 Weyl 共轭移动下不变【文档声明】）；而推入 `todo` 的新项经 `KType::sr_k` 构造（height 由 `sr_k` 重算）——两条路径的 height 来源不同【代码事实】；
  - `im_wt = lr + theta*lr + theta_plus_one_rho`（注释指 K_repr.cpp:306-311）；`'restart` 循环中扫 `0..semisimple_rank()`，`eval = pair(&im_wt, &datum.simple_coroots()[s])?`，`eval > 0` 跳过；
  - `KgbStatus::ImaginaryCompact`：`eval < 0` → `simple_reflect(s, &mut im_wt, 0)?`、`simple_reflect(s, &mut lr, 1)?`、`coef = -coef`、`continue 'restart`；`eval == 0` → `dropped = true; break 'restart`（丢弃该奇异紧致因子项）；
  - `KgbStatus::ImaginaryNoncompact` 且 `eval < 0`：`sx = cross_at(x, s)?`；`cx = rc.graph().cayley(x, s)?.ok_or(RepInvariantViolation { invariant: "noncompact imaginary Cayley" })?`；推入 `(sr_k(cx, &lr)?, coef)`；若 `sx == x`（注释：Type-2 Cayley），再推入 `(sr_k(cx, &shifted)?, coef)`，其中 `shifted = checked_add_weights(&lr, &datum.simple_roots()[s])?`；然后 `x = sx`、双反射、`coef` 取负、`continue 'restart`；
  - `KgbStatus::Complex`：`eval < 0 || is_complex_descent(x, s)?` → cross + 双反射 + `continue 'restart`；
  - `KgbStatus::Real`：`eval_lr = pair(&lr, &datum.simple_coroots()[s])?`；若为奇：`shift = (eval_lr + 1) / 2`；`projected = lr − shift*simple_roots[s]`（`try_reserve_exact`→`AllocationFailed`，逐分量 `checked_sub`/`checked_mul`→`ArithmeticOverflow`）；`rc.graph().inverse_cayley(x, s)?` 为 `None` → `Err(RepInvariantViolation { invariant: "parity real inverse Cayley" })`；有 `second` 则先推 `(sr_k(second, &lr)?, coef)`，再推 `(sr_k(first, &lr)?, coef)`；`dropped = true; break 'restart`；
  - 内层 `for` 自然耗尽 → `break`（未丢弃）：`normalized = rc.lambda_unique(rc.involution_of(x)?, &lr)?`，`result.push((KType::new(x, normalized, height), coef))`。
- 【代码事实】本函数**无显式终止计数器**；doc 的终止性叙述（final 输入立即结束等）属【文档声明】。
- 【代码事实】`simple_reflect` 第三参数字面量：`im_wt` 用 `0`，`lr`/`lam_rho` 用 `1`（全文件一致）。【阅读推断】该参数可能控制是否含 ρ 移位，但其语义定义不在本文件，需到 `RepContext::simple_reflect` 核对。

### 3.13 `kgp_set`（pub）— KGP 集
- doc：对应 `Rep_context::KGP_set`（K_repr.cpp:398-464）；输入应为 final 或 semifinal；结果按上游 BFS 发现顺序；**semifinal 前提由调用方负责（注释称 wrapper 在调用前检查）**。【文档声明；本文件内不存在该 wrapper 的代码证据】
- 实现要点【代码事实】：
  - 先 `self.made_theta_stable(rc)?`；
  - Levi 生成元：`rc.real_simple_roots_at(theta_stable.x())?` 中能在 `system.simple_root_ids()` 找到位置（`position`）者，收入 `levi_generators`（找不到则静默跳过）；
  - 去重：`present = vec![false; rc.graph().size()]`（按 `KgbId.index()` 置位）；结果首项为 `theta_stable`；队列 `VecDeque<(KgbId, Weight)>`，`pop_front` 取项；
  - `KgbStatus::Real` 分支：`eval = pair(&lam_rho, &datum.simple_coroots()[s])?`，`shift = eval / 2`（注释：final/semifinal 前提保证偶求值【文档声明】；代码**不检查**奇偶【代码事实】）；`new_lr = lam_rho − shift*simple_roots[s]`（`try_reserve_exact`/`checked_*` 同前）；`inverse_cayley(x, s)?` 为 `None` → `Err(RepInvariantViolation { invariant: "KGP real inverse Cayley" })`；按注释“first 更可能已插入，故后试”：先处理 `second`（若存在且未出现：`present` 置位、`push (sr_k(second, &new_lr)?)`、入队），再处理 `first`；
  - `KgbStatus::Complex` 分支：`sx = rc.graph().cross(x, s).ok_or(RepInvariantViolation { invariant: "KGP complex cross" })?`；未出现则 `reflected = lam_rho` 经 `simple_reflect(s, &mut reflected, 0)?`，`push (sr_k(sx, &reflected)?)`、入队；
  - 其余状态（`_ => {}`）忽略；
  - 【代码事实】此处复反射用第三参数 `0`（与 3.12 中 `lr` 用 `1` 不同）。
- 【阅读推断】`present` 位图使每个 KGB 元素至多入队一次，从而 BFS 有界；此界未以显式预算形式写出。

### 3.14 `simple_generator`（私有自由函数）
- 在 `simple_root_ids()` 中 `position` 查找 `RootId` 对应的生成元下标；找不到 → `Err(RepInvariantViolation { invariant: "simple root generator" })`。仅被 `is_normal` 调用。【代码事实】

## 4. 错误分支与预算汇总

### 4.1 本文件直接构造的 `StructureError` 变体【代码事实】

| 变体 | 触发位置（函数） |
|---|---|
| `IndexOutOfRange { index, upper_bound }` | `theta_plus_1_eval`（`alpha.0`/`beta.0` vs `roots().len()`）；`is_standard`、`is_nonzero`、`is_semifinal`（coroot 查找，同上界）；`equivalent`（`x.index()` vs `graph().size()`）；`to_canonical_fiber`（`involution.0` vs `involution_count()`） |
| `ArithmeticOverflow` | `theta_plus_1_eval`、`is_standard`、`is_nonzero`（`checked_add`）；`is_semifinal`、`finals_for`、`kgp_set`（`checked_mul`/`checked_sub`）；`normalised`（`i64::try_from(graph().size())`） |
| `AllocationFailed { requested }` | `is_semifinal`、`finals_for`、`kgp_set` 中 `try_reserve_exact(rank())` 失败（`requested = rank()`） |
| `RepInvariantViolation { invariant }` | 字符串字面量共 10 处：`"standard K-type in make_dominant"`、`"dominance termination"`、`"theta-stable termination"`、`"canonical fiber cross"`、`"normal form termination"`、`"noncompact imaginary Cayley"`、`"parity real inverse Cayley"`、`"KGP real inverse Cayley"`、`"KGP complex cross"`、`"simple root generator"` |

### 4.2 预算与终止机制【代码事实】
- 本文件任何 API **不接收**预算结构体参数；预算仅在测试中为他组件构造（`IntegerLatticeBudget::new(64, 100_000, 100_000, 128)`、`AdjointFiberBudget::new(…, 50_000, 100_000)`、`CartanClassificationBudget::new(…, weyl, 64, 64)`、`InvolutionTableBudget::new(64, …)`、`4_096` 等字面量）。
- 终止计数：`made_dominant` 用 `weight_defect`；`made_theta_stable` 用 `0..=graph().size()`；`normalised` 用 `weight_defect + size + 1`。
- 无显式计数：`finals_for`（栈驱动）、`kgp_set`（`present` 位图驱动，见【阅读推断】3.13）。
- 分配防护不一致：`try_reserve_exact` 仅用于三处权重向量构造；`vec![...]`（`todo`、`present`）、`Vec::new`（`result`）、`VecDeque` 无防护。【代码事实】

### 4.3 panic/断言面
- 生产代码无 `panic!`/`assert!`/`unwrap`。【代码事实】
- 【阅读推断】多处直接索引无防护：`system.simple_root_ids()[generator]`、`datum.simple_coroots()[s]`、`datum.simple_roots()[s]`、`present[x.index()]`；若各组件尺寸不一致将按 Rust 索引语义 panic。`is_final` 等循环上界取 `semisimple_rank()`，与 `simple_root_ids()` 长度的一致性假设未在本文件检查。
- 测试代码大量使用 `unwrap`/`assert_eq!`/`assert!`。【代码事实】

## 5. 与其它模块的接口（仅调用点可见事实）

- **`RepContext`**（生产路径）：`involution_of`、`lambda_unique`、`theta_at`、`theta_plus_one_rho_at`、`height`、`root_involution_image_at`、`colevel`、`imaginary_simple_roots_at`、`real_simple_roots_at`、`positive_real_roots_at`、`two_rho`、`two_rho_of`、`simple_imaginary_grading`、`is_complex_descent`、`is_complex_simple`、`kgb_status`、`cross_at`、`simple_reflect(usize, &mut Weight, 整数字面量)`、`weight_defect`、`inner_class`、`graph`、`table`。【代码事实】
- **`RepContext`**（仅测试路径）：`sr`、`sr_gamma`、`sr_k_of_standard`、`sr_of_ktype`、`finals_for(&StandardRepr)`、`finals_for_standard`、`reducibility_points`、`lambda_rho(&StandardRepr)`、`rho()`。【代码事实】注意：`rc.finals_for(&sr)` 作用在 `StandardRepr` 上，与本文件 `KType::finals_for(&self, rc)` 是不同入口。
- **`KgbGraph`**：`size()`、`cartan_of(KgbId) -> Option<_>`、`cayley(x, s) -> Result<Option<KgbId>, _>`、`inverse_cayley(x, s) -> Result<Option<(KgbId, Option<KgbId>)>, _>`、`cross(x, s) -> Option<KgbId>`（返回形状由 `?`/`ok_or` 用法读出）。【代码事实；返回类型为按用法重建，需对照 `KgbGraph` 定义核对】
- **`InnerClass`**：`root_system()`、`datum()`、`canonicalize(twisted) -> Result<(_, word)>`（`word` 可按值迭代为生成元）。**`InvolutionTable`**：`record(involution) -> Option<_>`，记录具 `.twisted_involution()`；`involution_count()`。**RootSystem**：`coroot(RootId) -> Option<_>`、`roots().len()`、`simple_root_ids()`。**Datum**：`semisimple_rank()`、`simple_coroots()`、`simple_roots()`（元素具 `.as_slice()`）。【代码事实】
- **`lattice`**：`checked_add_weights`、`checked_sub_weights`（`Result<Weight, StructureError>`）；`pair(&Weight, 余根) -> Result<i32, StructureError>`。【按用法重建】
- **类型形态**：`KgbId(u64/usize?)` 具元组构造与 `.index()`；`RootId` 具 pub 字段 `.0`；`involution_of` 的返回具 pub 字段 `.0`；`Weight::new(Vec<_>)`、`.rank()`、`.as_slice()`；`RationalWeight::new(Vec<_>, denom)`、`.numerator()`、`.denominator()`；`KgbStatus` 四变体 `ImaginaryCompact/ImaginaryNoncompact/Complex/Real`。【代码事实（按用法）；底层定义不在本文件】

## 6. 测试锚点

固定装置【代码事实】：split A1（Cartan `[[2]]`，根 `[2]`、余根 `[1]`，恒等对合，`InnerClass::new(…, 4)`，图大小断言为 3）；su(2,1)/拟分裂 A2（Cartan `[[2,-1],[-1,2]]`，根为 Cartan 行、余根为单位基，`InnerClass::new(…, 6)`，图大小断言为 6）。两构造器均以闭包形式把 `&RepContext` 与 `&KgbGraph` 交给测试体。

各测试断言【代码事实】：
1. `split_a1_k_type_anchors_match_the_frozen_contract`：`KType::sr_k(rc, KgbId(2), [0])` 的 `lambda_rho == [0]`、`height == 0`；六个谓词全 `true`；`made_dominant`/`normalised`/`made_theta_stable` 均为不动点；`sr_k(x, [2])` 与之相等且 `equivalent` 为真（注释：模 `(1-theta)X* = 2X*`）。
2. `split_a1_finals_for_a_final_parameter_is_itself`：`x=2`、`λ−ρ=[1]`、`γ=[1]/2`，`rc.finals_for` 返回 1 项、系数 1、`x` 与 `γ` 不变。
3. `split_a1_finals_for_a_non_dominant_parameter_reflects`：`x=1`、`γ=[0]/1`（奇异，eval 0 被保留）→ 1 项、`x=1`。
4. `split_a1_finals_for_a_negative_parameter_descends`：`x=1`、`γ=[-1]/1` → 2 项：`x=0` 系数 −1、`γ=[1]/1`；`x=2` 系数 +1。
5. `split_a1_reducibility_points_are_empty_for_gamma_half`：两组参数 `rc.reducibility_points` 均为空。
6. `split_a1_standard_repr_anchors_match_the_frozen_contract`：`rc.sr(x=2,[0],[0]/1)` 的 `height==0`、诸谓词真；`sr_k_of_standard` 得 `lambda_rho==[0]`；`sr_of_ktype` 与原参数 `equivalent`。
7. `a2_su21_context_builds_all_involutions_and_pins_nonfinal_anchors`：`sr_k(x=4,[1,0])` 保持 `[1,0]` 且非 final；`sr_k(x=5,[1,0])` 保持 `[1,0]` 且非 dominant；`sr_k(x=5,[0,0])` 保持 `[0,0]` 且非 final。（测试注释还陈述了 `lambda_unique` 在 release build 不记录枢轴取负、x=4 当选基为 `(2,-1),[1,0]` 等——【文档声明】，需对照 `rep_context`/`lattice` 核对。）
8. `su21_deform_parameter_shapes_match_the_frozen_contract`：对 `x in 0..6`、`nu=[1,1]/1` 仅 `eprintln!` 打印 `lam_rho/gamma/height`，**无任何断言**。【代码事实】
9. `su21_finals_for_singular_gamma_zero`：`x=5`、`γ=[0,0]/1`，仅 `eprintln!` 打印 `rc.finals_for_standard` 各项，**无任何断言**。【代码事实】

## 7. 限制与未覆盖面

- 测试仅覆盖 split A1 与 su(2,1) 两个内类；测试 8、9 为观察型（无断言），不构成机械锚点。【代码事实】
- 以下路径在本文件测试中无直接覆盖【阅读推断，基于测试体通读】：`kgp_set` 整体；`theta_plus_1_lambda`/`theta_plus_1_eval` 的独立行为；`equivalent` 的 Cartan 不同分支；`to_canonical_fiber` 的 `"canonical fiber cross"` 错误；全部终止预算错误（`"dominance/theta-stable/normal form termination"`）；全部 `AllocationFailed`/`ArithmeticOverflow`/`IndexOutOfRange` 分支；`finals_for` 的 Type-2 Cayley 双推入分支在 su(2,1) 下未被断言（A1 负参数用例覆盖了非紧致虚下降的部分形状）。
- 前提条件靠调用方：`is_nonzero`（假设 `is_standard`）、`is_normal`（上游四联前提，本移植不检查）、`kgp_set`（final/semifinal；Real 分支 `eval/2` 不查奇偶）。【代码事实 + 文档声明】
- `finals_for` 输出无序、不合并同项系数（doc 称合并在语言层）。【文档声明】
- `height` 的“Weyl 共轭移动下不变”仅为注释断言；`KType::new` 不校验 height 与 `(x, lam_rho)` 的一致性。【文档声明 + 代码事实】
- `is_dominant` 忽略枚举下标（`let _ = generator;`）。【代码事实】

## 8. 待维护者核对点清单

1. 全部 `K_repr.h/cpp` 行号与上游函数对应关系（本草案无法验证）。
2. `simple_reflect` 第三参数 `0/1` 的语义（本文件仅有调用点字面量）。
3. `RepContext::weight_defect` 的返回类型（决定 `remaining_steps` 类型与上界语义）。
4. `KgbGraph::{cayley, inverse_cayley, cross, cartan_of}` 与 `InvolutionTable::record`、`InnerClass::canonicalize` 的真实签名（本草案按 `?`/`ok_or` 用法重建）。
5. “wrapper 在调用 `kgp_set` 前检查 semifinal”所指的 wrapper 位置（不在本文件）。
6. 测试注释中关于 `lambda_unique` release-build 枢轴取负行为的陈述。
7. `finals_for`/`kgp_set` 无显式预算是否为本 crate 的既定策略（对比其它文件的预算参数风格）。