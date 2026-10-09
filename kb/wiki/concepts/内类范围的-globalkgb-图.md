---
title: 内类范围的 GlobalKgb 图
summary: GlobalKgb 枚举同一内类全部强实形的 KGB 元素，并按扭对合组织 tau 包；当前移植未包含任意 GlobalTitsElement 播种构造器及 Bruhat/Hasse 层。
sources:
  - global-kgb.md
kind: concept
createdAt: "2026-10-09T14:49:10.247Z"
updatedAt: "2026-10-09T14:49:10.247Z"
tags:
  - KGB图
  - 强实形
  - Rust移植
aliases:
  - 内类范围的-globalkgb-图
  - 内G图
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 内类范围的 GlobalKgb 图

`GlobalKgb` 枚举同一内类中全部强实形的 KGB 元素，并按扭曲对合划分为 tau 包。它是上游 `kgb::global_KGB` 的 Rust 移植，位于 `crates/atlas-real-group/src/global_kgb.rs`，同时实现 `print_X` 的打印布局。当前范围不包括从任意 `GlobalTitsElement` 播种的第二构造器，也不包括 Bruhat/Hasse 层。^[global-kgb.md:10-15]

## 元素表示与去重

图构造使用私有的 `GlobalTorusElement` 保存环面部分，其分子为 `Vec<i64>`，分母为 `i64`，表示 `exp(iπ·numerator/denominator)`，坐标按模 `2Z^rank` 理解。该表示保留算术历史：构造入口执行约化，简单反射后则故意不再约化，因此打印标签可以含负分子。相关细节见 [[全局环面元素的算术历史表示]]。^[global-kgb.md:19-29]

去重指纹将 `log_2pi` 的分子投影到 `θ+I` 饱和像的适应基各列，再对分母取非负余数，只保留 `diagonal.len()` 个分量。源码注释以生成同一饱和像的基之间存在幺模变换说明这种指纹的无损性；格条目预算限制为 64 位，以保障 `i128` 累加不溢出。相关概念见 [[基于饱和像适应基的 KGB 去重指纹]]。^[global-kgb.md:33-36]

## 构造流程

`GlobalKgb::build` 首先检查对合表所属内类是否与输入一致，不一致返回 `DatumMismatch`；对 `classification` 的归属则没有显式检查，由调用方保证。存储采用动态增长，预测大小仅作为分配提示。整体构造分为六个阶段，详见 [[GlobalKgb 的分阶段广度优先构造]]。^[global-kgb.md:49-76]

1. **登记 Cartan 类**：逐类调用 `table.add_cartan`。
2. **生成扭曲对合**：以恒等元为下标 0，按长度区间执行 BFS，根据 twisted commutation 判定选择 `table.cayley` 或 `table.cross`，最后核对生成数与表计数。
3. **生成包元数据**：为每个包计算长度、Cartan 类号和规范对合表达式的打印字。
4. **播种基本纤维**：枚举平方类子集与纤维群提升，将环面部分叠入，所有种子进入包 0。
5. **扩展 cross/Cayley 闭包**：按包区间执行 BFS，填充生成元状态、cross 和 Cayley 链接。
6. **生成打印头偏移**：累加正根的余根坐标得到 `dual_two_rho`，计算 `exp_2pi(dual_two_rho, 4).log_2pi()`。

上述阶段中的基本纤维种子以 `(identity_id, fingerprint)` 去重；重复触发 `"fundamental fiber distinctness"` 错误。平方类子集枚举使用 `checked_shl` 预算，基本余权位移与纤维群提升共同决定播种元素。^[global-kgb.md:55-76]

## 闭包不变量

cross 扩展要求长度差为偶数。若半长度差非零，则对环面部分执行简单反射，并标记为 `Complex`；虚根分支执行 `imaginary_cross_act`，由 `negative_at` 判定紧性；实根分支要求 cross 像等于自身。新指纹只能进入当前正在开启的新包，且 cross 两端必须具有相同的 Cartan 类号。^[global-kgb.md:67-72]

Cayley 链接只为 `ImaginaryNoncompact` 元素建立，目标的环面部分直接克隆。逆 Cayley 槽按写入顺序填充：首次写入占 `.0`，第二次占 `.1`。这套规则构成 [[cross 与 Cayley 闭包的根类型规则]] 的核心约束。^[global-kgb.md:72-74]

构造收尾扫描每个元素与生成元组合，保证其状态已经写入。来源据同一次迭代中 cross 写入先于状态写入这一顺序，推断 cross 槽不会再保留构造期哨兵 `usize::MAX`。^[global-kgb.md:78-80]

## 查询与打印

cross、Cayley 等表采用扁平存储，索引为 `x * semisimple_rank + generator`，访问器通过 `.get` 在越界时返回 `None`。调用时需要注意参数顺序：`status(element, generator)` 与 `cross(generator, element)` 相反。`torus_label()` 将 `log_2pi` 错误转为 `None`，而 `print_layout` 会传播同一错误。^[global-kgb.md:84-88]

`GlobalKgb` 仅派生 `Clone, Debug`，没有 `Eq`；快照比较借助 `GlobalKgbPrint`。打印实现复现字段填充宽度，缺失的 Cayley 链接显示为 `*`，具体接口和版式见 [[GlobalKgb 查询接口与 print_X 布局兼容]]。^[global-kgb.md:89-92]

## 测试与证据边界

来源记录了四个测试：单连通 A1、伴随 A1、单连通 B2 的 `print_X` 逐字节匹配，以及 B2 的结构不变量检查。B2 样例包含 17 个元素，包大小为 `[8,2,2,2,2,1]`，结构检查覆盖包字、cross 对合性和 Cayley 配对。^[global-kgb.md:94-100]

当前没有错误分支测试。半单秩为 0 的平凡群与一维环面有意未测，因为共享内类机制在空生成元集合上会 panic；维度匹配、非零分母和直接下标等隐式前置条件遭到违反时也可能 panic。此外，部分两倍分母计算使用普通乘法，存在 debug 溢出 panic 与 release 回绕的限制。^[global-kgb.md:101-105]

本页依据结构性源码阅读材料。来源未核对上游源码字节，也未在本次知识维护中执行 Atlas、Cargo、测试或 benchmark，因此上述测试锚点不构成本次运行结果或数学验收声明。相关覆盖说明见 [[GlobalKgb 的回归测试与证据边界]]。^[global-kgb.md:9-15, global-kgb.md:109-113]

## Sources

- [global-kgb.md](global-kgb.md) — 内类范围 KGB 图与 print_X 布局（global_kgb.rs）
