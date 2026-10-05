---
title: 内类范围 KGB 图与 print_X 布局（global_kgb.rs）
source: atlas-rust/global-kgb
ingestedAt: 2026-10-06T04:00:00Z
---

# 内类范围 KGB 图与 print_X 布局（global_kgb.rs）

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `crates/atlas-real-group/src/global_kgb.rs`（1370 行）：上游
`kgb::global_KGB`（kgb.h:213-266，kgb.cpp:190-233、331-479）的移植，加
`kgb_io::print_X` 版式（io/kgb_io.cpp:57-159）。一个 `GlobalKgb` 枚举同一
内类**全部强实形**的 KGB 元素，按扭对合分 tau 包。上游第二构造器（从任意
`GlobalTitsElement` 播种）与 Bruhat/Hasse 层未移植。上游行号仅转录自代码
注释，本包未核对上游字节；不声称数学验收。

## 环面元素：算术历史而非规范形

`GlobalTorusElement`（私有）= `numerator: Vec<i64>` + `denominator: i64`，
表示 `exp(iπ·numerator/denominator)`，坐标按模 `2Z^rank` 理解（上游
`y_values::TorusElement`）。关键纪律：**构造入口约化、反射不再约化**——
`reduce_raw`/`exp_pi`/`exp_2pi` 把分子 `rem_euclid` 进 `[0, 2·denominator)`，
但 `simple_reflect`（对偶侧预根 datum 的 `numer −= ⟨numer,α⟩·α∨`）故意不做
事后约化，所以打印输出里能出现负分子（B2 元素 15 的 `[0,-1]/2` 锚定此行
为）。`add` 是 lcm 合并 + gcd 归一化 + **每坐标至多一次**条件减法（上游只纠
正落在 `[2,4)` 的和，非规范加数原样通过）。`negative_at` 要求配对整值
（否则 `KgbInvariantViolation "integral root evaluation"`），奇为紧。
`imaginary_cross_act`（tits.cpp:167-174）：非紧时按 `coroot·(denominator −
remainder)` 的 `exp_2pi` 加数平移。`log_2pi` 返回分子/两倍分母，仅 gcd 归一。

## 去重指纹与播种数据

- `fingerprint`（x_pack，involutions.cpp:279-295）：`log_2pi` 分子投影到
  `θ+I` 饱和像的**适应基**（`adapted_basis`）各列，对分母取 `rem_euclid`。
  注释论证无损性：同一生饱和像的两基相差幺模（故 mod-d 可逆）左因子；只留
  `diagonal.len()` 个分量。预算把格条目限在 64 位，故 i128 累加不溢出。
- `fundamental_fiber`：`dualPi0(−δ^t)` 子商（tori.cpp:163-175 经
  cartanclass.cpp:209-212），与 `CartanFiber::build_owned` 同公式以使基序
  一致；`+I` 项靠 `ModTwoVector::from_ones` 对重复下标 xor 翻转实现（压入
  对角下标）。
- `square_class_generators`（tits.cpp:318-356）：`−δ_Y` 的 +1 特征空间 mod 2
  后，在 twist 不动生成元上求单根配对奇偶；映射子空间的**非主元位置**（升
  序）当选为平方类生成元。
- `fundamental_coweights`（rootdata.cpp:1015-1018）：对 `[C|I]` 做精确有理
  高斯消元（无 `RationalMatrix` 依赖，就地 `malachite::Rational`），
  `ω∨ᵢ` 坐标 = `Σₖ C⁻¹[k][i]·coroot_k[j]`；lcm 公分母、整性检查失败报
  `KgbInvariantViolation "fundamental coweight fraction"`。

## GlobalKgb::build 六阶段

前置：`table.inner_class() != inner_class` → `DatumMismatch`（对
classification 的归属无显式检查，靠调用方）。上游按 `global_KGB_size` 精确
预留，本移植动态增长（预测仅分配提示）。

- **A** 逐 Cartan `table.add_cartan`。
- **B** 对合生成（长度区间 BFS）：恒等播种下标 0；对每个 (generator,
  parent) 用 `hasTwistedCommutation` 移植（`(change>0) ==
  has_left_descent`），commutes 取 `table.cayley` 否则 `table.cross`；收尾
  校验生成数 = 表计数。
- **C** 每包派生数据：length、Cartan 类号、打印字（
  `canonical_involution_expr` 经 `format_involution_word`——`n≥0` 打印
  `'1'+n` 加 `^`，`!n` 加 `x`，尾补 `e`，上游字符怪癖原样复现）。
- **D** 基本纤维播种：平方类子集（`2^generators`，checked_shl 预算）给出
  基本余权位移 `rcw`，纤维群 `2^fiber_rank` 个 lift 经 `add_torus_part`
  叠入；全部落入包 0；以 `(identity_id, fingerprint)` 去重，冲突报
  `"fundamental fiber distinctness"`。
- **E** cross/Cayley 闭包（包区间 BFS）：长度差须为偶（`"cross length
  parity"`），`d = Δ/2 ≠ 0` → `simple_reflect` 且状态 Complex；虚根
  （`new_number == index && !has_descent`）→ `imaginary_cross_act`，紧/非紧
  由 `negative_at` 判定；实根要求 cross 像 = 自身（`"real cross image"`）。
  新指纹只允许落在正开启的新包（`"cross image inside closed packet"`），且
  cross 两端 Cartan 类号一致（`"cross Cartan class"`）。Cayley 链接仅对
  ImaginaryNoncompact 元素：torus 部分**原样克隆**，`inverse_cayley` 槽首写
  者居 `.0`、次写者居 `.1`。
- **F** 打印头偏移：正根余根坐标累加得 `dual_two_rho`，
  `exp_2pi(dual_two_rho, 4).log_2pi()`（kgb_io.cpp:152-156）。

收尾扫描保证每个 (element, generator) 状态被写（`"element status"`）；由此
推断 cross 槽不再残留构造期哨兵 `usize::MAX`（cross 写入先于同迭代的状态
写入）。

## 查询与打印层

存储扁平化 `x * semisimple_rank + generator`（cross/cayley 等表）；访问器全
部 `.get` 越界返回 `None`。注意 **`status(element, generator)` 与
`cross(generator, element)` 参数顺序相反**（后者对齐上游
`KGB_base::cross(s, x)`）。`torus_label()` 把 `log_2pi` 错误吞为 `None`，
而 `print_layout` 传播同一错误——两路径错误语义不一致（字节事实）。
`GlobalKgb` 只 derive `Clone, Debug`（无 Eq），快照比较只能借助
`GlobalKgbPrint`。`render` 复现每个 `setw` 填充：元素号宽
`digits(size−1)`、cartan/length 宽取**末行**位数、标签宽
`3·lattice_rank+3`、Cayley 缺失打 `*`。

## 测试锚点与限制

4 个测试：SC A1（5 行，头 `[1]/4`）、adjoint A1（3 行，`[1]/2`）、SC B2
（17 行，`[0,3]/4`，含负分子 `[0,-1]/2`）的**逐字节** print 匹配（对照
`tests/reference/domain/print_x.events.json` 的三个 print_X 块），加 B2 结
构不变量（17 元素、包大小 `[8,2,2,2,2,1]`、包字
`["e","1^e","2^e","1x2^e","2x1^e","1^2x1^e"]`、cross 对合性、Cayley 配对）。
无错误分支测试。文件末尾 NOTE：半单秩 0（平凡群、一维环面）有意未测——
共享内类机制在空生成元集合上 panic（weyl_transducer.rs:485），修复超出本
模块范围。其他隐式前置（维度匹配、`denominator != 0`、直接下标索引）违反
即 panic；`reduce_raw`/`evaluate_at` 等的 `2 * denominator` 为普通乘法
（debug 溢出 panic / release 回绕）。

## 来源与限制

精确读取身份见
[`2026-10-06-global-kgb.json`](snapshots/2026-10-06-global-kgb.json)：
绑定 Git base、文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe
（无工具档案）以完整字节起草（exit 0，520.9s），维护者对照源码逐条核对
改写。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。
