---
title: GlobalKgb 的分阶段广度优先构造
summary: 构造依次登记 Cartan 类、生成对合、派生包数据、播种基本纤维、计算 cross/Cayley 闭包并生成打印头偏移，最终检查状态槽完整性。
sources:
  - global-kgb.md
kind: concept
createdAt: "2026-10-09T14:49:33.233Z"
updatedAt: "2026-10-10T00:34:00.915Z"
tags:
  - kgb
  - 图算法
  - 不变量
aliases:
  - globalkgb-的分阶段广度优先构造
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: GlobalKgb 的分阶段广度优先构造
summary: GlobalKgb::build 经六阶段枚举同一内类全部强实形的 KGB 元素，其中两轮 BFS 分别按长度区间生成对合、按包区间建立 cross/Cayley 闭包。
sources:
  - global-kgb.md
kind: concept
tags:
  - KGB
  - 广度优先搜索
  - 内类
aliases:
  - globalkgb-的分阶段广度优先构造
provenanceState: extracted
---

# GlobalKgb 的分阶段广度优先构造

`GlobalKgb::build` 枚举同一内类全部强实形的 KGB 元素，并按扭对合划分 tau 包。构造分为六个阶段，其中两轮广度优先搜索（BFS）分别负责对合生成与 cross/Cayley 闭包。当前移植不包含从任意 `GlobalTitsElement` 播种的第二构造器，也不包含 Bruhat/Hasse 层。^[global-kgb.md:10-15, global-kgb.md:49-76]

## 前置条件与分配策略

构造首先检查 `table.inner_class() != inner_class`，不匹配时返回 `DatumMismatch`。对 `classification` 的归属没有显式检查，一致性由调用方保证。上游按 `global_KGB_size` 精确预留空间，Rust 实现采用动态增长，预测大小仅作为分配提示。^[global-kgb.md:51-53]

## 六阶段流程

### A：登记 Cartan 数据

逐个 Cartan 调用 `table.add_cartan`，完成表数据登记。^[global-kgb.md:55-55]

### B：按长度区间生成对合

以恒等元为下标 0 的种子，按长度区间执行 BFS。对每个 `(generator, parent)`，使用移植的 `hasTwistedCommutation` 判定，条件为 `(change > 0) == has_left_descent`：满足 commutes 时调用 `table.cayley`，否则调用 `table.cross`。结束时校验生成的对合数等于表计数。^[global-kgb.md:56-59]

### C：派生 tau 包信息

为每个包计算长度、Cartan 类号和打印字。打印字由 `canonical_involution_expr` 经 `format_involution_word` 得到：`n ≥ 0` 时打印字符 `'1' + n` 并附加 `^`，另一分支使用 `!n` 并附加 `x`，末尾补 `e`。这些字符规则保留上游行为。^[global-kgb.md:60-62]

### D：播种基本纤维

枚举平方类生成元的子集，得到基本余权位移 `rcw`，再将纤维群的 `2^fiber_rank` 个 lift 经 `add_torus_part` 叠入。平方类子集数为 `2^generators`，受 `checked_shl` 预算约束。全部种子进入包 0，以 `(identity_id, fingerprint)` 去重；冲突时报 `"fundamental fiber distinctness"`。相关构造见 [[基本纤维与平方类播种]]。^[global-kgb.md:63-66]

`fingerprint` 将 `log_2pi` 分子投影到 `θ + I` 饱和像的适应基各列，对分母取 `rem_euclid`，仅保留 `diagonal.len()` 个分量。源码注释以同一饱和像的基之间存在幺模换基、因而模分母可逆来论证其无损性，详见 [[基于饱和像适应基的 KGB 去重指纹]]。^[global-kgb.md:33-36]

### E：按包区间建立 cross/Cayley 闭包

第二轮 BFS 按包区间推进。长度差 `Δ` 必须为偶数，否则触发 `"cross length parity"`；当 `d = Δ/2 ≠ 0` 时，调用 `simple_reflect` 并标记状态为 `Complex`。虚根分支满足 `new_number == index && !has_descent`，调用 `imaginary_cross_act`，由 `negative_at` 判定紧或非紧；实根分支要求 cross 像等于自身，否则触发 `"real cross image"`。^[global-kgb.md:67-70]

新指纹只允许进入当前正在开启的新包，否则触发 `"cross image inside closed packet"`；cross 两端的 Cartan 类号必须一致，否则触发 `"cross Cartan class"`。这两项检查约束新元素的插入位置与 cross 的类归属。^[global-kgb.md:71-72]

Cayley 链接仅为 `ImaginaryNoncompact` 元素建立，torus 部分原样克隆。`inverse_cayley` 槽的首个写入者占据 `.0`，第二个写入者占据 `.1`。^[global-kgb.md:72-74]

环面表示保留算术历史：构造入口进行约化，但 `simple_reflect` 故意不做事后约化，因此打印标签可以带负分子。B2 元素 15 的 `[0,-1]/2` 锚定了这一行为，详见 [[全局环面元素的算术历史表示]]。^[global-kgb.md:19-29]

### F：计算打印头偏移

累加正根的余根坐标得到 `dual_two_rho`，再计算 `exp_2pi(dual_two_rho, 4).log_2pi()`，形成打印头偏移。展示层见 [[GlobalKgb 查询接口与 print_X 布局兼容]]。^[global-kgb.md:75-76]

## 收尾不变量

构造结束后扫描所有 `(element, generator)`，保证每个状态都已写入，否则触发 `"element status"`。源材料进一步根据写入顺序推断：cross 槽先于同次迭代的状态写入，因此检查通过后，cross 槽不再残留构造期哨兵 `usize::MAX`。这一点属于源材料明确标注的代码顺序推断。^[global-kgb.md:78-80]

## 测试与证据边界

现有四个测试包括 SC A1、adjoint A1、SC B2 的逐字节打印匹配，以及 B2 结构不变量检查。B2 检查覆盖 17 个元素、包大小 `[8,2,2,2,2,1]`、包字序列 `["e","1^e","2^e","1x2^e","2x1^e","1^2x1^e"]`、cross 对合性和 Cayley 配对，详见 [[GlobalKgb 的回归测试与证据边界]]。^[global-kgb.md:96-100]

错误分支没有测试覆盖。半单秩 0 的平凡群和一维环面有意未测，因为共享内类机制在空生成元集合上会 panic。维度匹配、非零分母和直接下标访问仍有隐式前置条件，违反时会 panic；部分 `2 * denominator` 使用普通乘法，存在 debug 溢出 panic、release 回绕的限制。^[global-kgb.md:101-105]

本页依据结构性源码阅读，不声称数学验收。源材料中的上游行号转录自代码注释，未核对上游字节；读取身份由快照记录绑定 Git base 和文件字节 SHA-256。源材料所述知识维护未执行 Atlas、Cargo、测试或 benchmark，因此上述测试描述不代表该次维护的执行结果。^[global-kgb.md:9-15, global-kgb.md:109-113]

## Sources

- [global-kgb.md](../../sources/global-kgb.md) — 内类范围 KGB 图与 print_X 布局（global_kgb.rs）。
