---
title: 重复 gcd 实现的语义漂移
summary: lattice.rs 与 global_kgb.rs 的私有 gcd_u64 对 gcd(0,0) 分别返回 1 与 0，但正分母约束使各自当前调用点仍安全。
sources:
  - lattice-types.md
kind: concept
createdAt: "2026-10-09T14:57:37.102Z"
updatedAt: "2026-10-09T22:36:56.945Z"
tags:
  - 代码维护
  - 整数算术
aliases:
  - 重复-gcd-实现的语义漂移
  - 重G实
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 重复 gcd 实现的语义漂移
summary: lattice.rs 与 global_kgb.rs 的私有 gcd_u64 对 gcd(0,0) 分别返回 1 与 0；正分母不变量使各自当前调用点仍安全。
sources:
  - lattice-types.md
kind: concept
tags:
  - 最大公约数
  - 代码维护
  - 不变量
---

# 重复 gcd 实现的语义漂移

`lattice.rs` 与 `global_kgb.rs` 各自包含私有的 `gcd_u64` 实现，但零输入的返回约定不同：前者返回 `left.max(1)`，因此 `gcd(0,0)=1`；后者返回 `left`，因此 `gcd(0,0)=0`。源材料将这一边界差异记录为重复实现已开始发生语义漂移。^[lattice-types.md:69-71]

## 当前安全性与调用前提

两种实现的语义在各自当前调用点都安全，依据是分母恒正。这一结论限定于现有调用前提，不能据此认为两个函数对所有输入都具有相同语义。^[lattice-types.md:69-71]

在 `lattice.rs` 中，`RationalWeight` 以 `Vec<i64>` 分子和 `i64` 公共分母表示有理权。构造器 `new` 执行 gcd 归一化，并拒绝非正分母：`denominator <= 0` 会产生 `RepInvariantViolation { invariant: "rational weight denominator" }`。错误分类可参见 [[StructureError 统一错误分类学]]。^[lattice-types.md:17-20, lattice-types.md:43-47]

## 归一化与防御性检查

`RationalWeight` 的 gcd 归一化时机取决于操作：`new` 在构造时归一化，`add`、`sub` 在交叉相乘形成公共分母后归一化，`scale` 在标量乘法后归一化。`halve` 则只翻倍分母，刻意将归一化留给调用方；`normalized` 可重新执行构造器的 gcd 归一化。^[lattice-types.md:43-58]

复核还指出，`new` 中 `i64::try_from(gcd)` 与 `checked_div` 的两个溢出错误出口在当前不变量下不可达，属于防御性写法。^[lattice-types.md:67-68]

## 证据边界

上述差异来自结构性源码阅读与复核。来源记录两次阅读的源码字节及 SHA-256 相同，但未执行构建、测试或原版运行，不包含数学验收、性能或并行结论。“当前调用点安全”是这一证据范围内的源码判断，不能替代独立的 [[HPC 验收证据链]]。^[lattice-types.md:9-13, lattice-types.md:69-71, lattice-types.md:83-89]

## Sources

- [lattice-types.md](../../sources/lattice-types.md) — 权格类型层：Weight、Coweight 与有理权。
