---
title: TitsCoset 逐步约化的类映射依据
summary: Rust 在各中间目标对合处约化，最终约化类不变的依据是操作在模二商上为类映射；本来源仅转述该注释论据，未作数学验收。
sources:
  - minimal-torus.md
kind: concept
createdAt: "2026-10-09T15:01:28.752Z"
updatedAt: "2026-10-10T00:43:55.667Z"
tags:
  - TitsCoset
  - 商空间
  - 证据边界
aliases:
  - titscoset-逐步约化的类映射依据
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: TitsCoset 逐步约化的类映射依据
summary: minimal_torus_part 在每个中间目标对合处约化，其最终约化类不变的依据是 TitsCoset 操作在模二商上为类映射；这一依据属于源码注释声明，尚未经本来源独立验证。
sources:
  - minimal-torus.md
kind: concept
tags:
  - TitsCoset
  - 模二商
  - 证据边界
aliases:
  - titscoset-逐步约化的类映射依据
provenanceState: extracted
---

# TitsCoset 逐步约化的类映射依据

`minimal_torus_part` 将给定强代表经逆 Cayley 与 based twisted 共轭下降到基本纤维。Rust 实现复用 stage-(c) 的 `TitsCoset` 操作，在每个中间环面部分的目标对合处约化；来源所述的上游实现则只在末尾约化。源码注释以这些操作是 mod-2 商上的类映射为依据，声明逐步约化不改变最终约化类。^[minimal-torus.md:22-29]

## 类映射依据与证据性质

这一依据关注的是最终的**约化类**：注释声明，相关操作在模二商上为类映射，因此可以在各步的目标对合处约化中间环面部分。注释将该性质关联到 stage-(e) KGB 枚举的验证；来源包明确未验证这一声明，也未核对所引用的上游文件字节。相关背景可参见 [[KGB 图与弱实形式]]。^[minimal-torus.md:10-13, minimal-torus.md:27-29]

## 在下降流程中的应用

初始环面部分来自 `factor − coch`：每个坐标必须为整数，否则报告 `"torus-part integrality"`；奇数坐标编码为 `torus_part` 的置位。随后，`grading_of_simples` 按单根配对偶性构造 `TitsCoset`；若 `table.lookup(twisted)` 缺失，则报告 `"synthetic involution coverage"`。^[minimal-torus.md:50-56]

下降循环在 Weyl 部分为恒等时停止。否则，算法按 `interface.outward()` 的迭代顺序选择首个左下降生成元：若其在当前对合下为 Real，则执行逆 Cayley，且 `Ok(None)` 也视为错误；否则执行 based twisted 共轭 `cross_pregated`。这一 [[强代表下降到基本纤维|下降过程]] 在每一步的目标对合处约化中间环面部分。^[minimal-torus.md:27-29, minimal-torus.md:56-60]

循环末尾仍执行幂等的 `coset.reduce`；当输入的 `twisted` 本来就在基本纤维时，这次末尾约化起决定作用。随后，算法构造 `coweight = coch + lift(tp)`，即在环面部分置位处给 `coch` 加 1，再进入 [[最小环面部分的 grading 轨道搜索]]，选出满足目标模式的最小环面部分。^[minimal-torus.md:60-76]

## 测试与证据边界

来源记录的四个测试锚点均为 rank 2、紧致内类。三个正例都满足 `coch == factor`，没有通过可区分的断言刻画非平凡运输行为；其中“等价因子选同一种子”测试与 A2 测试第一组输入完全相同，来源认为其独立价值仅在注释叙述中。^[minimal-torus.md:78-91]

非紧致 distinguished、非平凡运输、多数具名错误分支及 rank 大于 63 的资源门均未覆盖，详见 [[最小环面算法的测试覆盖边界]]。因此，本页保留“逐步约化不动最终类”为源码注释声明；来源的结构性阅读不构成数学或正确性验收，也未在本次维护中执行 Atlas、Cargo、测试或 benchmark。^[minimal-torus.md:93-100, minimal-torus.md:104-108]

## Sources

- [minimal-torus.md](../../sources/minimal-torus.md) — 合成实形的选定余特征与初始环面部分（minimal_torus.rs）。
