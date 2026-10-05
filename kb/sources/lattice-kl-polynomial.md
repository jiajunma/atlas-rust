---
title: 格值类型与 KLV 最小多项式引擎（lattice.rs / kl_polynomial.rs）
source: atlas-rust/lattice-kl-polynomial
ingestedAt: 2026-10-06T05:40:00Z
---

# 格值类型与 KLV 最小多项式引擎（lattice.rs / kl_polynomial.rs）

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `lattice.rs`（434 行）与 `kl_polynomial.rs`（266 行）。两文件源码级
无直接耦合，共享 `StructureError`；错误风格形成显式分工（lattice 全检查，
kl_polynomial 不检查算术）。本包是结构性阅读，不声称数学验收；上游引用
（ratvec.cpp、kl.cpp、repr.cpp 等）仅转录自代码注释。

## lattice.rs：X^* / X_* 与精确有理值

- `Weight(Vec<i32>)` / `Coweight(Vec<i32>)`：字符格与余字符格，刻意是两个
  不同的类型（完美配对但不可互换）；坐标停在 checked 定宽 `i32`，解释器层
  在域边界转换。
- `pair` = 秩检查 + `pair_coordinates`（i128 checked 累加 + `i32::try_from`
  收窄）。`checked_add/sub_weights` 经 `combine_weights(±1)`；`i32::MIN * -1`
  一类情形被 `checked_mul(sign)` 拦截。
- `RationalWeight`（上游 `RatWeight`）：单公分母 `Vec<i64> + i64` 布局，因为
  显示层打印 `[ 5, 0 ]/2`；构造即 gcd 归一、分母恒正（`denominator <= 0` →
  `RepInvariantViolation "rational weight denominator"`）。`combine` 交叉相乘
  公分母后再归一；`apply_matrix` 保持分母、只作用分子；`halve` 只翻倍分母
  **刻意不归一**（归一时机交给调用方，与上游一致）；`normalized` 重跑构造器；
  `integral_coordinates` 是上游 `assert(entry%denominator==0)` 的可检查化
  （`"rational weight integrality"`）；`scale`/`dot_coroot`（`dot_Q`）同风格。
- `RationalCoweight(Vec<Rational>)`：第三方 malachite 类型不进公开 API 的
  包装；只有构造/视图，无算术、无 `Hash`。
- 预算纪律：凡按输入长度新建 Vec 一律 `try_reserve_exact` 并映射
  `AllocationFailed`；例外是 `halve`/`normalized`/`to_rationals` 的裸 clone。
- 复核备注（草案标记、维护者确认属实）：
  - `dot_coroot` 的 `RankMismatch` 字段顺序是 `expected: coroot.rank(),
    actual: self.rank()`，与 `pair`/`combine` 的惯例相反。
  - `apply_matrix` 行长失配也报 `actual: matrix.len()`（同 compose_matrices
    的填报怪癖一族）。
  - `new` 里 `i64::try_from(gcd)`/`checked_div` 两个溢出口在当前不变量下
    不可达（防御性写法）。
  - 维护者补充：本文件 `gcd_u64` 返回 `left.max(1)`（故 `gcd(0,0)=1`），与
    `global_kgb.rs` 同名私有副本返回 `left`（`gcd(0,0)=0`）不同——两处语义
    在各自调用点都安全（分母恒正），但重复实现已开始漂移，值得记录。

## kl_polynomial.rs：KLV 多项式与哈希池

- `KlPol(Vec<i32>)`：ℤ[q]，低次在前；零 = 空向量，非零则最高次非零（`trim`
  维护，这也是能作 HashMap 键的前提）。注意「系数非负、首项系数 1」是数学
  对象的性质，**不是表示不变量**（`from_coefficients` 接受任意 i32）。
- 操作集恰好覆盖 KLV 递归与 μ 修正：`add`/`sub`、`shift`（×(1+q)，
  kl.cpp:409）、`add_shifted`（复下降递归项，kl.cpp:416）、`sub_shifted`
  （μ 修正 safeSubtract，kl.cpp:504-512）、`add_shifted_scaled`（μ 求和，
  kl.cpp:834-836）、`scaled`、`evaluate_at_minus_one`（repr.cpp:1953-1955
  的交替和）、`divide_by_2`（kl.cpp:702，奇系数 →
  `RepInvariantViolation "KL polynomial parity"`，Rust % 保号故负奇也拦截）、
  `quotient_by_1_plus_q`（kl.cpp:711 综合除法 + 截断；**函数体恒 Ok**，Result
  仅为签名形态）。
- 算术全部是不检查的 `i32` 普通运算，无 `ArithmeticOverflow` 通道——与
  lattice.rs 的全检查风格对立；是否可接受取决于系数上界，本包不断言。
- `KlHashTable`（上游 `KL_hash_Table`）：`new()` 种子 0=零、1=一
  （`KLStore{Zero, One}`）；**derive 的 `Default` 给出空池**，与 `new()`
  语义不同——若存在 `default()` 调用点会破坏池索引约定（复核备注）。
- 复核备注：`coefficient` 文档写「panics if out of range」但实现越界返回 0
  （`add`/`sub` 依赖此行为）——文档过期，以实现为准。

## 测试锚点与限制

lattice 2 个测试（i128 收窄出口；Weight/Coweight 类型区分）；kl_polynomial
4 个（池种子序号、shift 展开、q=−1 交替和、sub_shifted 单项）。未测面广：
`RationalWeight` 全部方法、`RationalCoweight` 全部方法、`divide_by_2` 错误
分支、`quotient_by_1_plus_q`、`match_pol` 去重路径等。

## 来源与限制

精确读取身份见
[`2026-10-06-lattice-kl-polynomial.json`](snapshots/2026-10-06-lattice-kl-polynomial.json)：
绑定 Git base、两文件字节 SHA-256 与 Kimi 调用记录（含一次超时重试）。
草案由 Kimi probe（无工具档案）以两文件完整字节起草（r2：600s 期限，
exit 0，326.1s；r1 在 360s 期限超时——25.5KB 提示的实测需求超过 13s/KB
估计，超时规则已上修为 ~24s/KB），维护者对照源码逐条核对改写。本次知识
维护未执行 Atlas、Cargo、测试或 benchmark。
