---
title: 重复 gcd 实现的语义漂移
summary: lattice.rs 与 global_kgb.rs 的私有 gcd_u64 对 gcd(0,0) 分别返回 1 与 0，但正分母约束使各自当前调用点仍安全。
sources:
  - lattice-types.md
kind: concept
createdAt: "2026-10-09T14:57:37.102Z"
updatedAt: "2026-10-10T00:40:42.747Z"
tags:
  - 代码维护
  - 精确算术
aliases:
  - 重复-gcd-实现的语义漂移
  - 重G实
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 重复 gcd 实现的语义漂移
summary: lattice.rs 与 global_kgb.rs 的私有 gcd_u64 对 gcd(0,0) 分别返回 1 与 0；分母恒正使各自当前调用点仍安全。
sources:
  - lattice-types.md
kind: concept
tags:
  - 代码维护
  - 整数算术
aliases:
  - 重复-gcd-实现的语义漂移
---

# 重复 gcd 实现的语义漂移

`lattice.rs` 与 `global_kgb.rs` 各自包含私有的 `gcd_u64` 实现，但对全零输入采用不同约定：前者返回 `left.max(1)`，因此 `gcd(0,0)=1`；后者返回 `left`，因此 `gcd(0,0)=0`。来源将这一差异记录为重复实现已开始发生语义漂移。^[lattice-types.md:69-71]

## 当前调用点的安全前提

来源判断两种实现的语义在各自调用点都安全，依据是**分母恒正**。这一安全判断限定于当前调用前提，并不表示两个实现对所有输入都具有相同语义。^[lattice-types.md:69-71]

在 `lattice.rs` 中，`RationalWeight` 使用 `Vec<i64>` 分子与 `i64` 公共分母表示有理权。构造器 `new` 执行 gcd 归一化，并拒绝非正分母：当 `denominator <= 0` 时，返回 `RepInvariantViolation { invariant: "rational weight denominator" }`。相关错误体系可参见 [[StructureError 统一错误分类学]]。^[lattice-types.md:17-20, lattice-types.md:43-47]

## 归一化与防御性检查

`RationalWeight` 的归一化时机随操作而异：`new` 在构造时归一化，`add`、`sub` 在交叉相乘形成公共分母后归一化，`scale` 在标量乘法后归一化。`halve` 只翻倍分母，刻意将归一化时机留给调用方；`normalized` 则重新执行构造器的 gcd 归一化。^[lattice-types.md:43-58]

复核还指出，`new` 中 `i64::try_from(gcd)` 与 `checked_div` 的两个溢出错误出口在当前不变量下不可达，属于防御性写法。^[lattice-types.md:67-68]

## 证据边界

上述结论来自结构性源码阅读与复核。来源记录两次阅读的源码字节及 SHA-256 相同，但未执行构建、测试或原版运行，不包含数学验收、性能或并行结论。格类型实现的正确性属于其独立的 [[HPC 验收证据链]]，本页不扩展该证据范围。^[lattice-types.md:9-13, lattice-types.md:83-89]

## Sources

- [lattice-types.md](../../sources/lattice-types.md) — 权格类型层：Weight、Coweight 与有理权。
