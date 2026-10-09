---
title: Bourbaki 重编号与对合矩阵下标映射
summary: perm[k] 将第 k 个扁平化单根映射到输出矩阵下标；排列合法性由包装器 checked_permutation 验证，layout_involution 依赖调用方满足前置条件。
sources:
  - primitive-involution.md
kind: concept
createdAt: "2026-10-09T15:03:59.488Z"
updatedAt: "2026-10-09T15:03:59.488Z"
tags:
  - Bourbaki编号
  - 矩阵表示
  - 接口契约
aliases:
  - bourbaki-重编号与对合矩阵下标映射
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Bourbaki 重编号与对合矩阵下标映射

Bourbaki 重编号在 `layout_involution` 中决定扁平化单根位置对应的矩阵下标。该函数输出单连通群基本权基上的行主序 `i32` 对合矩阵，其中 `perm[k]` 表示第 `k` 个扁平化单根的矩阵下标，对应上游的 `Layout::d_perm`。^[primitive-involution.md:84-91]

## 映射方向与职责

`perm` 是用户提供的、针对扁平化单因子的 Bourbaki 重编号，由包装器侧的 `checked_permutation` 验证。其方向是“扁平位置 `k` → 矩阵下标 `perm[k]`”。`layout_involution` 所属模块仅执行纯查表，不涉及 root datum、Weyl group 或 inner-class 构造管线。^[primitive-involution.md:17-25, primitive-involution.md:86-89]

查表使用两个位置指针：`r` 表示扁平图位置，`pos` 表示因子位置。内类字母决定各因子的对合模式，`perm` 则决定这些位置在输出矩阵中的下标；相关模式见 [[单连通群基本权基上的逐字母对合表]]。^[primitive-involution.md:86-102]

## 对合模式与重编号

逐字母查表时，`'c'` 产生恒等块；`'s'` 在 A 型上给出反对角，在奇秩 D 型上交换末两顶点，在 E6 上固定位置 1、3 并交换 0↔5、2↔4，在环面 T 上给出负恒等；`'u'` 交换末两顶点。这些模式中的顶点位置须结合 `perm[k]` 的下标含义理解。^[primitive-involution.md:86-102]

复型字母 `'C'` 将当前因子的 `rs` 个顶点与紧随因子的 `rs` 个顶点平行互换，并额外消耗一个因子。[[内类字母的字节解析与规范化]] 要求这两个因子相同且连续，从而为该交换提供前置保证。^[primitive-involution.md:74-75, primitive-involution.md:100-101]

## 前置条件与失败边界

`letters` 必须来自 `checked_inner_class_letters`，`perm` 必须是 `0..rank` 的排列。函数通过两道 `debug_assert_eq!` 检查 `perm.len() == rank` 和最终 `r == rank`，但不返回 `Result`；违反前置条件可能导致下标越界 panic 或进入 `unreachable!`。因此，包装器的排列验证是调用契约的一部分，不能由查表函数的长度断言替代。^[primitive-involution.md:23-25, primitive-involution.md:86-91]

## 与一般换基的区别

Bourbaki 重编号规定查表结果的矩阵下标；一般格基上的矩阵转换则由独立的 `on_basis` 完成，计算 `basis^-1 * matrix * basis`。该换基使用精确有理运算，遇到非方阵、奇异基、非整结果或无法转换为 `i32` 的条目时统一返回 `None`，由包装器重标为不兼容格。^[primitive-involution.md:86-89, primitive-involution.md:104-113]

## 测试证据与限制

测试使用因子 `[A1,A2]`、字母 `"cs"` 和 `perm [2,0,1]`，明确检查 Bourbaki 重编号作用于表输出；按映射定义，三个扁平位置分别落到矩阵下标 2、0、1。另一个锚点是 A2 的 `'s'`／`'u'` 翻转矩阵 `[[0,1],[1,0]]`，它在 `perm [1,0]` 下保持不变。^[primitive-involution.md:86-87, primitive-involution.md:122-127]

现有测试未覆盖 `'C'` 在非恒等 `perm` 下的行为，也未覆盖退化空输入。来源仅完成结构性阅读，不构成数学正确性验收；所列上游位置来自代码注释，未独立核对上游字节。相关边界见 [[对合查表实现的证据范围与测试缺口]]。^[primitive-involution.md:132-141]

## Sources

- [primitive-involution.md](primitive-involution.md)：内类字母解析与逐字母对合查表（`primitive_involution.rs`）。
