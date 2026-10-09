---
title: 最长 Weyl 元的下坡行走
summary: 从源码计算的 2ρ 出发，反复选择首个正余根配对的生成元并左合成反射，目标为 −2ρ，超预算或无法推进均报不变量错误。
sources:
  - root-datum-dual.md
kind: concept
createdAt: "2026-10-09T15:10:08.133Z"
updatedAt: "2026-10-09T22:47:25.768Z"
tags:
  - Weyl群
  - 算法
  - 资源预算
aliases:
  - 最长-weyl-元的下坡行走
  - 最W元
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 最长 Weyl 元的下坡行走
summary: longest_action 按生成元升序选择正余根配对的反射并左合成，将 two_rho 计算的权送到其负值；超预算或无法推进时报不变量错误，数学与复杂度声明尚未验收。
sources:
  - root-datum-dual.md
kind: concept
tags:
  - Weyl群
  - 最长元
  - 资源预算
aliases:
  - 最长-weyl-元的下坡行走
  - 最W元
provenanceState: extracted
---

# 最长 Weyl 元的下坡行走

`longest_action` 是 `dual.rs` 中定位最长 Weyl 元的自由函数，通过逐步反射返回将 $2\rho$ 送到 $-2\rho$ 的 `WeylAction`，无需枚举整个 Weyl 群。代码文档将其对应到上游 `rd.to_dominant(-rd.twoRho())`；来源仅转录这一对应关系，未核对上游源码字节。^[root-datum-dual.md:10-14, root-datum-dual.md:103-125]

## 初始权与反射选择

辅助函数 `two_rho` 使用长度为 `lattice_rank` 的 `i64` 累加器，只累加满足 `is_positive(id) == Some(true)` 的根，其他条目（包括 `None`）均跳过。累加采用 `checked_add`，最后逐分量通过 `i32::try_from` 转换。将所得权认作 $2\rho$ 是代码意图，来源未独立验收这一恒等关系。^[root-datum-dual.md:116-119, root-datum-dual.md:213-215]

行走以该权为起点，以其负值为目标。每轮按生成元编号升序扫描，选择第一个与当前权具有正余根配对的简单反射，将其左合成到累计作用，并作用于当前权。生成元选择顺序和左合成方向均属于实现约定。^[root-datum-dual.md:116-123]

实现读取 `semisimple_rank` 与 `simple_coroots`，通过 `WeylGroup`／`WeylAction` 完成反射，不调用 `BasedRootDatum::reflect_weight`。相关作用表示见 [[WeylAction 的对偶全格作用]]。^[root-datum-dual.md:186-188]

## 步数预算与失败行为

每轮执行 `steps += 1`，随后检查 `steps > weyl_budget || !advanced`。预算允许恰好 `weyl_budget` 步；超过预算或本轮无法推进时，均返回 `StructureError::LayoutInvariantViolation { invariant: "longest Weyl element" }`。这两种失败使用同一个不变量错误。^[root-datum-dual.md:119-124]

算术保护并不统一：`two_rho` 使用带检查的 `i64` 加法，但行走内层的余根配对采用未检查的 `i64` 累加。来源未确认这种差异是否有意，因此不能将整个行走描述为所有整数运算均经过溢出检查。^[root-datum-dual.md:117-127]

## 在对偶构造中的用途

[[对偶根数据与对偶内类构造]] 使用最长元作用 $W_0$，先形成乘积 $M=qW_0$，再构造对偶权作用 $-M^t$ 与余权作用 $-M$。其中 `weyl_budget` 只约束最长元定位，`root_budget` 约束对偶根系闭包。^[root-datum-dual.md:129-139]

`dual_cartan_correspondence` 的公开参数 `_weyl_budget` 是未使用的遗留参数；该流程中的最长元行走实际以 `dual.root_system().roots().len()`，即已枚举的对偶根数，作为预算。^[root-datum-dual.md:141-154]

内部函数 `dual_twisted_representative` 将原代表元的规范 Weyl 字在对偶 datum 上逐生成元重放，末步左合成 `longest`，再结合对偶 distinguished involution 构造 `TwistedInvolution`。该路径保留两种格作用及 distinguished-involution 来源信息，可结合 [[Weyl 元素兼容性与跨坐标词重放]] 阅读。^[root-datum-dual.md:157-164]

## 复杂度声明与证据边界

代码注释称，下坡行走步数恰好等于最长元的约化长度，并将其描述为相对于上游 transducer `O(rank)` 实现的等价 `O(length)` 行走。这是注释中的复杂度声明，来源未作验收，不能据此推导实测速度比较。相关表示见 [[Weyl 群的紧凑 Transducer 表示]]。^[root-datum-dual.md:123-125, root-datum-dual.md:213-216]

本页依据结构性源码阅读材料，不构成数学验收。来源未对 `two_rho` 的数学恒等、最长元行走的复杂度或对偶实形式计数的数学正确性作验收结论；本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[root-datum-dual.md:204-216]

## Sources

- [root-datum-dual.md](../../sources/root-datum-dual.md) — BasedRootDatum 与对偶内类构造（root_datum.rs / dual.rs）。
