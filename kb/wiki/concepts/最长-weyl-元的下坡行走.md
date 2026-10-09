---
title: 最长 Weyl 元的下坡行走
summary: longest_action 从源码计算的 2ρ 出发，按生成元顺序选择正配对反射并左合成，目标为 −2ρ，超预算或无法推进时报不变量错误；数学与复杂度声明尚未验收。
sources:
  - root-datum-dual.md
kind: concept
createdAt: "2026-10-09T15:10:08.133Z"
updatedAt: "2026-10-09T15:10:08.133Z"
tags:
  - Weyl群
  - 最长元
  - 预算控制
aliases:
  - 最长-weyl-元的下坡行走
  - 最W元
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 最长 Weyl 元的下坡行走

`longest_action` 通过逐步反射的下坡行走定位最长 Weyl 元，返回将 $2\rho$ 送到 $-2\rho$ 的 `WeylAction`。该函数位于 `dual.rs`，其文档将这一过程对应到上游 `rd.to_dominant(-rd.twoRho())`；来源材料仅转录了该对应关系，未核对上游源码字节。^[root-datum-dual.md:114-125, root-datum-dual.md:10-14]

## 初始权与反射选择

辅助函数 `two_rho` 使用长度为 `lattice_rank` 的 `i64` 累加器，只累加满足 `is_positive(id) == Some(true)` 的根；`Some(false)` 与 `None` 均跳过。累加采用 `checked_add`，最终逐分量通过 `i32::try_from` 转换。来源将其结果用作 $2\rho$，但未独立验收这一恒等关系。^[root-datum-dual.md:116-119, root-datum-dual.md:213-215]

行走以该权为起点，以其负值为目标。每轮按生成元编号升序扫描，选择第一个与当前权具有正余根配对的简单反射，将该反射左合成到累计作用，并作用于当前权。因此，生成元扫描顺序与左合成方向都是实现流程的一部分。^[root-datum-dual.md:116-123]

实现读取 `semisimple_rank` 和 `simple_coroots`，反射通过 `WeylGroup`／`WeylAction` 完成，并不调用 `BasedRootDatum::reflect_weight`。相关格作用可参见 [[WeylAction 的对偶全格作用]]。^[root-datum-dual.md:186-188]

## 步数预算与失败行为

每轮执行 `steps += 1`，随后检查 `steps > weyl_budget || !advanced`。预算允许恰好 `weyl_budget` 步；超过预算或本轮无法推进时，均返回 `StructureError::LayoutInvariantViolation { invariant: "longest Weyl element" }`，而不是分别报告两种错误。^[root-datum-dual.md:119-124]

算术检查存在需要保留的边界：`two_rho` 使用带检查的 `i64` 加法，但行走内层的余根配对采用未检查的 `i64` 累加。来源未确认这种差异是否有意，因此不能将整个行走描述为所有整数运算均经过溢出检查。^[root-datum-dual.md:117-127]

## 在对偶构造中的用途

[[对偶根数据与对偶内类构造]] 使用最长元作用 $W_0$，先形成乘积 $M=qW_0$，再构造对偶权作用 $-M^t$ 与余权作用 $-M$。这里 `weyl_budget` 仅约束最长元定位，`root_budget` 则约束对偶根系闭包，两者职责不同。^[root-datum-dual.md:129-139]

`dual_cartan_correspondence` 的公开参数 `_weyl_budget` 已成为未使用的遗留参数；该流程中的最长元行走实际以 `dual.root_system().roots().len()`，即已枚举的对偶根数，作为预算。^[root-datum-dual.md:141-154]

## 复杂度声明与证据边界

代码注释称，下坡行走步数恰好等于最长元的约化长度，并将其描述为相对于上游 transducer `O(rank)` 实现的等价 `O(length)` 行走。这是注释中的复杂度声明，来源并未作性能验收；不能据此推导实测运行时间或速度比较。相关表示见 [[Weyl 群的紧凑 Transducer 表示]]。^[root-datum-dual.md:123-125, root-datum-dual.md:213-216]

本页依据结构性阅读材料。该材料未对 `two_rho` 的数学恒等、最长元行走的复杂度或相关对偶计算作数学、性能及正确性结论，也未执行 Atlas、Cargo、测试或 benchmark。^[root-datum-dual.md:204-216]

## Sources

- [root-datum-dual.md](root-datum-dual.md) — BasedRootDatum 与对偶内类构造（root_datum.rs / dual.rs）
