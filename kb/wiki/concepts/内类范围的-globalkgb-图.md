---
title: 内类范围的 GlobalKgb 图
summary: GlobalKgb 枚举同一内类全部强实形的 KGB 元素并按扭对合分包，尚未移植任意 GlobalTitsElement 播种及 Bruhat/Hasse 层。
sources:
  - global-kgb.md
kind: concept
createdAt: "2026-10-09T14:49:10.247Z"
updatedAt: "2026-10-10T00:33:37.326Z"
tags:
  - kgb
  - rust
aliases:
  - 内类范围的-globalkgb-图
  - 内G图
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 内类范围的 GlobalKgb 图
summary: GlobalKgb 枚举同一内类全部强实形的 KGB 元素，按扭对合组织 tau 包，通过分阶段 BFS 建立 cross/Cayley 链接并提供 print_X 布局。
sources:
  - global-kgb.md
kind: concept
tags:
  - KGB
  - 强实形
  - Rust移植
aliases:
  - 内类范围的-globalkgb-图
---

# 内类范围的 GlobalKgb 图

`GlobalKgb` 枚举同一内类中**全部强实形**的 KGB 元素，并按扭对合划分为 tau 包。实现位于 `crates/atlas-real-group/src/global_kgb.rs`，移植上游 `kgb::global_KGB`，同时实现 `print_X` 的打印布局。当前不包含从任意 `GlobalTitsElement` 播种的第二构造器，也不包含 Bruhat/Hasse 层。^[global-kgb.md:10-15]

## 环面表示与去重

私有类型 `GlobalTorusElement` 用 `numerator: Vec<i64>` 和 `denominator: i64` 表示 $\exp(i\pi\cdot\mathrm{numerator}/\mathrm{denominator})$，坐标按模 $2\mathbb Z^{\mathrm{rank}}$ 理解。其表示保留算术历史：构造入口约化分子，简单反射后故意不再约化，因此打印中可以出现负分子。详见 [[全局环面元素的算术历史表示]]。^[global-kgb.md:19-29]

去重指纹将 `log_2pi` 的分子投影到 $\theta+I$ 饱和像的适应基各列，再对分母取 `rem_euclid`，只保留 `diagonal.len()` 个分量。源码注释以同一饱和像的基之间存在幺模变换说明指纹的无损性；格条目预算限制为 64 位，以保障 `i128` 累加不溢出。详见 [[基于饱和像适应基的 KGB 去重指纹]]。^[global-kgb.md:33-36]

## 分阶段构造

`GlobalKgb::build` 首先检查 `table.inner_class()` 是否与输入内类一致，不一致返回 `DatumMismatch`；`classification` 的归属没有显式检查，由调用方保证。存储采用动态增长，预测大小仅作为分配提示。^[global-kgb.md:49-53]

构造先逐 Cartan 类调用 `table.add_cartan`，再以恒等元为下标 0，按长度区间执行 BFS。对每个生成元与父元素，依据 twisted commutation 判定选择 `table.cayley` 或 `table.cross`，最后核对生成数与表计数。随后为每个包生成长度、Cartan 类号和规范对合表达式的打印字。^[global-kgb.md:55-62]

基本纤维播种枚举平方类子集，以基本余权生成位移 `rcw`，再叠入纤维群的各个 lift；全部种子进入包 0。平方类子集数量使用 `checked_shl` 进行预算检查，种子以 `(identity_id, fingerprint)` 去重，冲突时报 `"fundamental fiber distinctness"`。相关背景见 [[基本纤维与平方类播种]]。^[global-kgb.md:63-66]

随后按包区间执行 cross/Cayley 闭包 BFS，填充生成元状态与链接。最后累加正根的余根坐标得到 `dual_two_rho`，通过 `exp_2pi(dual_two_rho, 4).log_2pi()` 生成打印头偏移。完整阶段说明见 [[GlobalKgb 的分阶段广度优先构造]]。^[global-kgb.md:67-76]

## 闭包不变量

cross 扩展要求长度差为偶数。若半长度差 $d=\Delta/2\ne0$，则执行 `simple_reflect` 并标记为 `Complex`；虚根分支执行 `imaginary_cross_act`，由 `negative_at` 判定紧性；实根分支要求 cross 像等于自身。新指纹只能进入当前正在开启的新包，且 cross 两端的 Cartan 类号必须一致。^[global-kgb.md:67-72]

Cayley 链接仅为 `ImaginaryNoncompact` 元素建立，环面部分原样克隆；逆 Cayley 槽的首次写入占 `.0`，第二次写入占 `.1`。^[global-kgb.md:72-74]

构造收尾扫描每个元素与生成元组合，保证状态已经写入。来源根据同一次迭代中 cross 写入先于状态写入的顺序，进一步推断 cross 槽不会残留构造期哨兵 `usize::MAX`；这是来源给出的推断。^[global-kgb.md:78-80]

## 查询与打印

cross、Cayley 等表按 `x * semisimple_rank + generator` 扁平存储，访问器通过 `.get` 在越界时返回 `None`。调用时需注意参数顺序：`status(element, generator)` 与 `cross(generator, element)` 相反。`torus_label()` 将 `log_2pi` 错误转为 `None`，而 `print_layout` 传播同一错误，两条路径的错误语义不同。^[global-kgb.md:84-88]

`GlobalKgb` 仅派生 `Clone, Debug`，没有 `Eq`；快照比较借助 `GlobalKgbPrint`。`render` 复现字段填充宽度：元素号宽度为 `digits(size−1)`，Cartan 类号与长度的宽度取末行位数，标签宽度为 `3·lattice_rank+3`，缺失的 Cayley 链接打印为 `*`。详见 [[GlobalKgb 查询接口与 print_X 布局兼容]]。^[global-kgb.md:89-92]

## 测试与证据边界

来源记录了四个测试：单连通 A1、伴随 A1、单连通 B2 的 `print_X` 逐字节匹配，以及 B2 结构不变量检查。三个打印样例分别有 5、3、17 行；B2 包含负分子标签 `[0,-1]/2`，结构检查覆盖包大小 `[8,2,2,2,2,1]`、包字、cross 对合性和 Cayley 配对。^[global-kgb.md:96-100]

当前没有错误分支测试。半单秩为 0 的平凡群与一维环面有意未测，因为共享内类机制在空生成元集合上会 panic。维度匹配、非零分母和直接下标等隐式前置条件被违反时也会 panic；部分 `2 * denominator` 使用普通乘法，存在 debug 溢出 panic、release 回绕的限制。详见 [[GlobalKgb 的回归测试与证据边界]]。^[global-kgb.md:101-105]

本页依据结构性源码阅读材料。来源中的上游行号仅转录自代码注释，未核对上游字节；本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。因此，测试锚点描述不代表本次执行结果，也不构成数学验收声明。^[global-kgb.md:9-15, global-kgb.md:109-113]

## Sources

- [global-kgb.md](../../sources/global-kgb.md) — 内类范围 KGB 图与 print_X 布局（global_kgb.rs）。
