---
title: 重复 gcd 实现的语义漂移
summary: lattice.rs 与 global_kgb.rs 的私有 gcd_u64 对 gcd(0,0) 分别返回 1 与 0，体现重复实现的语义漂移，但正分母约束使各自当前调用点仍安全。
sources:
  - lattice-types.md
kind: concept
createdAt: "2026-10-09T14:57:37.102Z"
updatedAt: "2026-10-09T14:57:37.102Z"
tags:
  - 最大公约数
  - 代码维护
  - 不变量
aliases:
  - 重复-gcd-实现的语义漂移
  - 重G实
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 重复 gcd 实现的语义漂移

`lattice.rs` 与 `global_kgb.rs` 各自包含私有的 `gcd_u64` 实现，但零输入的返回约定已经不同：前者返回 `left.max(1)`，因此 `gcd(0,0)=1`；后者返回 `left`，因此 `gcd(0,0)=0`。这是重复实现发生语义漂移的具体表现。^[lattice-types.md:69-71]

## 当前安全性与调用前提

源材料明确指出，两种实现的语义在各自调用点都安全，依据是分母恒正。因此，已观察到的差异并不构成这些调用点存在错误的证据；其安全性依赖于调用处的不变量。^[lattice-types.md:69-71]

在 `lattice.rs` 中，`RationalWeight` 使用 `Vec<i64>` 分子与 `i64` 公共分母表示有理权。构造器 `new` 执行 gcd 归一化，并拒绝非正分母：`denominator <= 0` 会产生 `RepInvariantViolation { invariant: "rational weight denominator" }`。这一检查提供了归一化所需的正分母前提，相关错误分类可参见 [[StructureError 统一错误分类学]]。^[lattice-types.md:17-20, lattice-types.md:43-47]

## 归一化的时机

`RationalWeight` 并非每次操作后都立即归一化：`add`、`sub` 在交叉相乘形成公共分母后归一化，`scale` 在标量乘法后归一化；`halve` 则只翻倍分母，刻意把归一化时机留给调用方，后续可用 `normalized` 重新调用构造器归一化。因此，理解 gcd 的作用还需要区分正分母约束与具体操作的归一化时机。^[lattice-types.md:43-58]

## 证据边界

上述差异是源材料结构性阅读中的复核结论。该材料未执行构建、测试或原版运行，不提供数学验收或性能结论；不能把“当前调用点安全”的源码判断扩展为更广泛的运行验证。相关证据要求可参见 [[HPC 验收证据链]]。^[lattice-types.md:9-13, lattice-types.md:61-71, lattice-types.md:89-93]

## Sources

- [lattice-types.md](../../sources/lattice-types.md)
