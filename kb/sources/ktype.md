---
title: K 型值与 RepContext 谓词/变形（ktype.rs）
source: atlas-rust/ktype
ingestedAt: 2026-10-06T09:20:00Z
---

# K 型值与 RepContext 谓词/变形（ktype.rs）

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `ktype.rs`（981 行）：上游 `gkmod/K_repr.h/cpp` 的 K_type 层。
`KType { x: KgbId, lam_rho: Weight, height: u32 }` = 标准表示参数去掉 ν 的
K-限制；存储的 `lam_rho` 恒为其 `(1−θ_x)X*`-陪集的 `lambda_unique` 当选
代表（规范化只在 `sr_k` 发生一次），`height` 构造时预计算。本包是结构性
阅读，不声称数学验收；上游行号仅转录自注释。

## 构造与谓词链

- `new`（pub(crate)）是原始构造器，不校验不变量——crate 内可装入任意
  height，纪律靠调用方；crate 外只能经 `sr_k`。
- `sr_k(rc, x, λ−ρ)`：`lambda_unique` 规范化 + 预存 `(1+θ)λ` 的 height，
  其中 `(1+θ)λ = λ_ρ + θ·λ_ρ + (1+θ)ρ`（theta_plus_1_lambda 同式）。
- `theta_plus_1_eval(α)`（私有核）：`⟨λ_ρ, α∨⟩ + colevel(α) +
  ⟨λ_ρ, (θα)∨⟩ + colevel(θα)`，谓词集只用其符号/零测试。
- 谓词：`is_standard`（单虚余根上 eval ≥ 0）；`is_dominant`（所有单根
  theta_plus_1_eval ≥ 0）；`is_nonzero`（无奇异紧单虚根；**假设
  is_standard 成立但不检查**）；`is_semifinal`（测试权重
  `2λ_ρ + 2ρ − 2ρ_R` 在实单根上配对 ≡ 0 mod 4）；`is_normal`（无奇异
  复下降；上游断言四联前提，本移植因计算是 total 而不检查——注释声明）；
  `is_final`（eval<0 拒绝；eval==0 时按 KGB 状态分派：ic 拒绝、Real 奇
  配对拒绝、Complex 下降拒绝、inc 放行）。
- `equivalent`：先同 Cartan 类，再各自 `to_canonical_fiber` 后严格相等。

## 变形与展开

- `made_dominant`：非标准输入报 `"standard K-type in make_dominant"`；
  预算 = `weight_defect((1+θ)λ)`，超限报 `"dominance termination"`；对负
  求值复单根 cross + 反射，每轮末尾 lambda_unique 重规范化。height 在
  Weyl 共轭移动下不变（注释断言），原样携带。
- `made_theta_stable`：耗尽复下降；图大小为「慷慨」终止界
  （`"theta-stable termination"`）。
- `to_canonical_fiber`：沿 `InnerClass::canonicalize` 的词 cross；每个
  生成元必须是复单根（`"canonical fiber cross"`）。
- `normalised`：典范化后耗尽奇异复下降与负复求值；预算 =
  `weight_defect + 图大小 + 1`（`"normal form termination"`）。
- `finals_for`（K_repr.cpp:290-396）：带号重数表（**无序**，合并在语言
  层）。工作栈驱动，无显式终止计数。分支：ic 且 eval<0 → 双反射 +
  coef 取负；ic 且 eval==0 → 丢弃（奇异紧因子）；inc 且 eval<0 → 推入
  Cayley 像项，若 cross 不动（type-2）再推 `λ_ρ+α` 移位项，然后 cross
  过去并取负；Complex 且（eval<0 或下降）→ cross + 反射；Real 且
  `⟨λ_ρ, α∨⟩` 奇 → 投影到墙（`shift=(eval+1)/2`）并按逆 Cayley 分裂
  （无双值则只推 first；`None` 报 `"parity real inverse Cayley"`）。
  **height 来源不对称**（阅读观察）：todo 新项经 `sr_k` 重算 height，
  而结果项用 `KType::new(x, normalized, 沿用待处理项的 height)`。
  `simple_reflect` 第三参数：`im_wt` 用 0、`lr` 用 1（全文件一致；
  语义在 RepContext 侧）。
- `kgp_set`（K_repr.cpp:398-464）：先 `made_theta_stable`；Levi 生成元 =
  theta-stable 元的实单根（映射不回生成元下标者**静默跳过**）；BFS 由
  `present` 位图限界（每 KGB 元至多入队一次）。Real 分支的
  `shift = eval/2` **不查奇偶**（final/semifinal 前提由调用方负责——注释
  称 wrapper 先检查）；注释「first 更可能已插入，故后试」：先处理
  second。Complex 分支反射用第三参数 0。

## 测试锚点与限制

9 个测试：split A1 的冻结契约锚点（x=2 的 [0] K 型、六谓词全真、三个
变形均不动、模 2X* 相等性）；finals_for 三形态（final 自身、奇异保留、
负参数下降为两项 x=0[coef −1]+x=2[coef +1]）；reducibility_points 两空；
StandardRepr 往返（sr ↔ sr_k_of_standard/sr_of_ktype）；su(2,1) 的非
final 锚点（x=4/5 的 elected 代表，注释记录 lambda_unique 在 release
build 不记录主元取负、x=4 当选基为 (2,−1),[1,0]）。**测试 8/9
（su21_deform_*、su21_finals_for_singular_gamma_zero）只有 eprintln!
无断言**——观察型，不构成机械锚点。未覆盖：kgp_set 整体、全部终止预算
错误、equivalent 的异 Cartan 分支、to_canonical_fiber 的错误分支、各
溢出/分配分支。

## 来源与限制

本包在**未变的字节**上取代 2026-10-03 的初读包（同一 SHA-256，旧快照
[`2026-10-03-ktype.json`](snapshots/2026-10-03-ktype.json) 保留）：初读覆盖
表示不变量、谓词链与规范化链；本次补充 finals_for 的分支结构、kgp_set、
测试锚点普查与错误分支普查。

精确读取身份见
[`2026-10-06-ktype.json`](snapshots/2026-10-06-ktype.json)：
绑定 Git base、文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe
（无工具档案）以完整字节起草（1100s 期限，exit 0，349.5s），维护者对照
源码逐条核对改写。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。
