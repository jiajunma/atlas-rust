---
title: GlobalKgb 的分阶段广度优先构造
summary: GlobalKgb::build 依次完成 Cartan 登记、按长度生成对合、包数据派生、基本纤维播种、cross/Cayley 闭包和打印头偏移计算，并检查计数与状态完整性。
sources:
  - global-kgb.md
kind: concept
createdAt: "2026-10-09T14:49:33.233Z"
updatedAt: "2026-10-09T14:49:33.233Z"
tags:
  - 图构造
  - 广度优先搜索
  - 算法不变量
aliases:
  - globalkgb-的分阶段广度优先构造
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# GlobalKgb 的分阶段广度优先构造

`GlobalKgb::build` 构造同一内类全部强实形的 KGB 元素，并按扭对合划分 tau 包。构造分为六阶段，其中两轮广度优先搜索分别负责生成扭对合，以及建立元素的 cross/Cayley 闭包。当前移植不包含从任意 `GlobalTitsElement` 播种的第二构造器，也不包含 Bruhat/Hasse 层。^[global-kgb.md:10-15, global-kgb.md:49-76]

## 前置条件与分配策略

构造首先检查 `table.inner_class() != inner_class`，不匹配时返回 `DatumMismatch`。对 `classification` 的归属没有显式检查，一致性由调用方保证。与上游按 `global_KGB_size` 精确预留不同，Rust 实现动态增长，预测大小仅作为分配提示。^[global-kgb.md:51-53]

## 六阶段流程

### A：装入 Cartan 数据

逐个 Cartan 调用 `table.add_cartan`，为后续对合枚举准备表数据。^[global-kgb.md:55-55]

### B：按长度区间生成扭对合

以恒等元为下标 0 的种子，按长度区间执行 BFS。对每个 `(generator, parent)`，使用移植的 `hasTwistedCommutation` 判定，其条件为 `(change > 0) == has_left_descent`；满足 commutes 条件时调用 `table.cayley`，否则调用 `table.cross`。结束时校验生成数等于表计数。^[global-kgb.md:56-59]

### C：派生每个 tau 包的信息

为每个包记录长度、Cartan 类号和打印字。打印字由 `canonical_involution_expr` 经 `format_involution_word` 得到：`n ≥ 0` 时输出字符 `'1' + n` 并附加 `^`，`!n` 形式附加 `x`，末尾补 `e`。这些字符规则保留上游行为，相关约定见 [[对合表达式的打印约定]]。^[global-kgb.md:60-62]

### D：播种基本纤维

枚举平方类生成元的全部子集，得到基本余权位移 `rcw`；再把纤维群的 `2^fiber_rank` 个 lift 经 `add_torus_part` 叠入。平方类子集数为 `2^generators`，其计算受 `checked_shl` 预算约束。所有种子进入包 0，并以 `(identity_id, fingerprint)` 去重；冲突时报 `"fundamental fiber distinctness"`。这一过程对应 [[基本纤维与平方类播种]]。^[global-kgb.md:63-66]

去重所用的 `fingerprint` 将 `log_2pi` 分子投影到 `θ + I` 饱和像的适应基各列，再对分母取 `rem_euclid`，仅保留 `diagonal.len()` 个分量。源码注释以同一饱和像的基之间存在幺模换基说明其无损性，详见 [[基于饱和像适应基的 KGB 去重指纹]]。^[global-kgb.md:33-36]

### E：按包区间建立 cross/Cayley 闭包

第二轮 BFS 按包区间推进，根据长度变化及根类型处理 cross。长度差 `Δ` 必须为偶数，否则触发 `"cross length parity"`；当 `d = Δ/2 ≠ 0` 时，调用 `simple_reflect` 并标记为 `Complex`。虚根分支满足 `new_number == index && !has_descent`，调用 `imaginary_cross_act`，并由 `negative_at` 判定紧或非紧；实根分支则要求 cross 像等于自身，否则触发 `"real cross image"`。^[global-kgb.md:67-70]

包的推进受两项不变量约束：新指纹只能进入当前正在开启的新包，否则触发 `"cross image inside closed packet"`；cross 两端的 Cartan 类号必须一致，否则触发 `"cross Cartan class"`。这些检查限制闭包扩展的插入位置，并验证 cross 的 Cartan 类一致性。^[global-kgb.md:71-72]

Cayley 链接只为 `ImaginaryNoncompact` 元素建立，其 torus 部分原样克隆。逆向链接写入 `inverse_cayley` 时，首个写入者占据 `.0`，第二个写入者占据 `.1`；相关根类型规则见 [[cross 与 Cayley 闭包的根类型规则]]。^[global-kgb.md:72-74]

### F：计算打印头偏移

累加正根对应的余根坐标得到 `dual_two_rho`，再计算 `exp_2pi(dual_two_rho, 4).log_2pi()`，形成打印头偏移，供 [[GlobalKgb 查询接口与 print_X 布局兼容]] 所述的展示层使用。^[global-kgb.md:75-76]

## 收尾不变量

构造结束后扫描所有 `(element, generator)`，保证每个状态均已写入，否则触发 `"element status"`。源材料据写入顺序进一步推断：由于同次迭代中 cross 槽先于状态写入，此检查通过后，cross 槽应不再残留构造期哨兵 `usize::MAX`。后一点是基于代码顺序的推断。^[global-kgb.md:78-80]

## 测试与证据边界

现有测试包含 SC A1、adjoint A1、SC B2 的逐字节打印匹配，以及 B2 结构不变量检查。B2 检查覆盖 17 个元素、包大小 `[8,2,2,2,2,1]`、包字序列 `["e","1^e","2^e","1x2^e","2x1^e","1^2x1^e"]`、cross 对合性和 Cayley 配对，详见 [[GlobalKgb 的回归测试与证据边界]]。^[global-kgb.md:94-100]

这些测试未覆盖错误分支；半单秩 0 的平凡群和一维环面也有意未测，因为共享内类机制会在空生成元集合上 panic。维度匹配、非零分母和直接下标访问仍有隐式前置条件，部分 `2 * denominator` 运算还存在 debug 溢出 panic、release 回绕的限制。源材料记录的是结构性阅读，本次知识维护未执行测试，也不声称数学验收。^[global-kgb.md:9-15, global-kgb.md:101-105, global-kgb.md:109-113]

## Sources

- [global-kgb.md](global-kgb.md) — 内类范围 KGB 图与 print_X 布局（global_kgb.rs）
