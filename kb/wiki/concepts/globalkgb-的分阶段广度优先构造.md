---
title: GlobalKgb 的分阶段广度优先构造
summary: GlobalKgb::build 通过 Cartan 登记、对合生成、包数据派生、基本纤维播种、cross/Cayley 闭包和打印头偏移六阶段枚举内类全部强实形的 KGB 元素。
sources:
  - global-kgb.md
kind: concept
createdAt: "2026-10-09T14:49:33.233Z"
updatedAt: "2026-10-09T20:53:34.875Z"
tags:
  - KGB
  - 广度优先搜索
  - 构造不变量
aliases:
  - globalkgb-的分阶段广度优先构造
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: GlobalKgb 的分阶段广度优先构造
summary: GlobalKgb::build 通过六阶段完成 Cartan 登记、对合枚举、包信息派生、基本纤维播种、cross/Cayley 闭包和打印头偏移计算，以两轮 BFS 及计数、包边界和状态检查维护构造不变量。
sources:
  - global-kgb.md
kind: concept
tags:
  - 图构造
  - 广度优先搜索
  - 算法不变量
aliases:
  - globalkgb-的分阶段广度优先构造
---

# GlobalKgb 的分阶段广度优先构造

`GlobalKgb::build` 枚举同一内类全部强实形的 KGB 元素，并按扭对合划分 tau 包。构造分为六阶段，其中两轮广度优先搜索（BFS）分别按长度区间生成对合、按包区间建立 cross/Cayley 闭包。当前移植不包含从任意 `GlobalTitsElement` 播种的第二构造器，也不包含 Bruhat/Hasse 层。^[global-kgb.md:10-15, global-kgb.md:49-76]

## 前置条件与分配策略

构造检查 `table.inner_class() != inner_class`，不匹配时返回 `DatumMismatch`；对 `classification` 的归属没有显式检查，由调用方保证一致性。上游按 `global_KGB_size` 精确预留空间，Rust 实现则动态增长，预测大小仅作为分配提示。^[global-kgb.md:51-53]

## 六阶段流程

### A：登记 Cartan 数据

逐个 Cartan 调用 `table.add_cartan`，完成表数据登记。^[global-kgb.md:55-55]

### B：按长度区间生成对合

以恒等元为下标 0 的种子，按长度区间执行 BFS。对每个 `(generator, parent)`，使用移植的 `hasTwistedCommutation` 判定，其条件为 `(change > 0) == has_left_descent`：满足 commutes 条件时调用 `table.cayley`，否则调用 `table.cross`。结束时校验生成的对合数等于表计数。^[global-kgb.md:56-59]

### C：派生 tau 包信息

为每个包记录长度、Cartan 类号和打印字。打印字由 `canonical_involution_expr` 经 `format_involution_word` 得到：`n ≥ 0` 时打印字符 `'1' + n` 并附加 `^`，`!n` 编码对应的字母附加 `x`，末尾补 `e`；这些字符规则保留上游行为。^[global-kgb.md:60-62]

### D：播种基本纤维

枚举平方类生成元的全部子集，得到基本余权位移 `rcw`，再把纤维群的 `2^fiber_rank` 个 lift 经 `add_torus_part` 叠入。平方类子集数为 `2^generators`，受 `checked_shl` 预算约束。所有种子进入包 0，并以 `(identity_id, fingerprint)` 去重；冲突时报 `"fundamental fiber distinctness"`。参见 [[基本纤维与平方类播种]]。^[global-kgb.md:63-66]

`fingerprint` 将 `log_2pi` 分子投影到 `θ + I` 饱和像的适应基各列，再对分母取 `rem_euclid`，仅保留 `diagonal.len()` 个分量。源码注释以同一饱和像的基之间存在幺模换基说明其无损性，详见 [[基于饱和像适应基的 KGB 去重指纹]]。^[global-kgb.md:33-36]

### E：按包区间建立 cross/Cayley 闭包

第二轮 BFS 按包区间推进。长度差 `Δ` 必须为偶数，否则触发 `"cross length parity"`；当 `d = Δ/2 ≠ 0` 时，调用 `simple_reflect` 并标记为 `Complex`。虚根分支满足 `new_number == index && !has_descent`，调用 `imaginary_cross_act`，并由 `negative_at` 判定紧或非紧；实根分支要求 cross 像等于自身，否则触发 `"real cross image"`。^[global-kgb.md:67-70]

新指纹只允许进入当前正在开启的新包，否则触发 `"cross image inside closed packet"`；cross 两端的 Cartan 类号必须一致，否则触发 `"cross Cartan class"`。这两项检查约束新元素的插入位置与 cross 的类归属。^[global-kgb.md:71-72]

Cayley 链接仅为 `ImaginaryNoncompact` 元素建立，其 torus 部分原样克隆。写入 `inverse_cayley` 时，首个写入者占据 `.0`，第二个写入者占据 `.1`。相关分支见 [[cross 与 Cayley 闭包的根类型规则]]。^[global-kgb.md:72-74]

环面表示保留算术历史：`simple_reflect` 故意不做事后约化，因此闭包构造产生的打印标签可以带负分子。不能把这些标签理解为每一步都重新约化的规范代表元，参见 [[全局环面元素的算术历史表示]]。^[global-kgb.md:19-29]

### F：计算打印头偏移

累加正根的余根坐标得到 `dual_two_rho`，再计算 `exp_2pi(dual_two_rho, 4).log_2pi()`，形成打印头偏移；展示层参见 [[GlobalKgb 查询接口与 print_X 布局兼容]]。^[global-kgb.md:75-76]

## 收尾不变量

构造结束后扫描所有 `(element, generator)`，保证每个状态均已写入，否则触发 `"element status"`。源材料进一步根据写入顺序推断：cross 槽先于同次迭代的状态写入，因此检查通过后，cross 槽应不再残留构造期哨兵 `usize::MAX`。后一点属于源材料的代码顺序推断。^[global-kgb.md:78-80]

## 测试与证据边界

现有四个测试包括 SC A1、adjoint A1、SC B2 的逐字节打印匹配，以及 B2 结构不变量检查。B2 检查覆盖 17 个元素、包大小 `[8,2,2,2,2,1]`、包字序列 `["e","1^e","2^e","1x2^e","2x1^e","1^2x1^e"]`、cross 对合性和 Cayley 配对，详见 [[GlobalKgb 的回归测试与证据边界]]。^[global-kgb.md:96-100]

错误分支没有测试覆盖。半单秩 0 的平凡群和一维环面有意未测，因为共享内类机制会在空生成元集合上 panic。维度匹配、非零分母和直接下标访问仍有隐式前置条件；部分 `2 * denominator` 使用普通乘法，存在 debug 溢出 panic、release 回绕的限制。^[global-kgb.md:101-105]

本说明依据结构性源码阅读；源材料中的上游行号转录自代码注释，未核对上游字节。本次知识维护未执行 Atlas、Cargo、测试或 benchmark，也不声称数学验收。^[global-kgb.md:9-15, global-kgb.md:109-113]

## Sources

- [global-kgb.md](../../sources/global-kgb.md) — 内类范围 KGB 图与 print_X 布局（global_kgb.rs）。
