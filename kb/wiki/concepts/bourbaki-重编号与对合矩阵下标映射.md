---
title: Bourbaki 重编号与对合矩阵下标映射
summary: perm[k] 指定第 k 个扁平化单根的输出矩阵下标，排列合法性由包装器验证，layout_involution 依赖调用方满足前置条件。
sources:
  - primitive-involution.md
kind: concept
createdAt: "2026-10-09T15:03:59.488Z"
updatedAt: "2026-10-09T21:04:41.677Z"
tags:
  - Bourbaki编号
  - 排列
  - 接口契约
aliases:
  - bourbaki-重编号与对合矩阵下标映射
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Bourbaki 重编号与对合矩阵下标映射
summary: perm[k] 将第 k 个扁平化单根映射到输出矩阵下标；排列合法性由包装器验证，layout_involution 依赖调用方满足前置条件。
sources:
  - primitive-involution.md
kind: concept
tags:
  - Bourbaki编号
  - 矩阵表示
  - 接口契约
aliases:
  - bourbaki-重编号与对合矩阵下标映射
---

# Bourbaki 重编号与对合矩阵下标映射

Bourbaki 重编号在 `layout_involution` 中规定扁平化单根位置对应的矩阵下标。函数输出单连通群基本权基上的行主序 `i32` 对合矩阵，其中 `perm[k]` 表示第 `k` 个扁平化单根的矩阵下标，对应上游 `Layout::d_perm`。^[primitive-involution.md:84-91]

## 映射方向与职责

`perm` 是用户提供的、针对扁平化单因子的 Bourbaki 重编号，其方向为“扁平位置 `k` → 输出矩阵下标 `perm[k]`”。排列合法性由包装器侧的 `checked_permutation` 验证；`layout_involution` 所属模块执行纯查表，不涉及 root datum、Weyl group 或 inner-class 构造管线。^[primitive-involution.md:17-25, primitive-involution.md:86-89]

查表使用两个位置指针：`r` 表示扁平图位置，`pos` 表示因子位置。内类字母决定各因子的对合模式，`perm` 决定这些顶点在输出矩阵中的下标。模式的完整说明见 [[单连通群基本权基上的逐字母对合表]]。^[primitive-involution.md:86-102]

## 对合模式与重编号

字母 `'c'` 产生恒等块；`'s'` 在 A 型上产生反对角矩阵，在奇秩 D 型上交换末两顶点，在 E6 上固定下标 1、3 并交换 0↔5、2↔4，在环面 T 上产生负恒等矩阵；`'u'` 交换末两顶点。这些位置均须结合 `perm[k]` 的映射理解。解析阶段会将 B、C、E7、E8、F、G 等类型的 `'s'` 坍缩为 `'c'`，因此相应的查表默认分支在遵守前置条件时不可达。^[primitive-involution.md:95-102, primitive-involution.md:138-139]

复型字母 `'C'` 将当前因子的 `rs` 个顶点与紧随因子的 `rs` 个顶点平行互换，并额外消耗一个因子。[[内类字母的字节解析与规范化]] 要求这两个因子相同且连续，为该交换提供前置保证。^[primitive-involution.md:74-75, primitive-involution.md:100-101]

## 前置条件与失败边界

`letters` 必须来自 `checked_inner_class_letters`，`perm` 必须是 `0..rank` 的排列。函数仅通过两道 `debug_assert_eq!` 检查 `perm.len() == rank` 和最终 `r == rank`，不返回 `Result`；违反前置条件可能导致下标越界 panic 或触发 `unreachable!`。排列合法性因此依赖包装器的验证，长度断言本身并不完成这一检查。^[primitive-involution.md:23-25, primitive-involution.md:86-91]

## 与一般格基转换的关系

Bourbaki 重编号规定查表输出的矩阵下标；独立函数 `on_basis` 则计算一般格基上的表示 `basis^-1 * matrix * basis`。该函数使用精确有理运算，将非方阵、奇异基、非整结果及条目无法转换为 `i32` 四类失败统一折叠为 `None`，包装器再将其标记为不兼容格。相关主题见 [[有理运算实现的整数矩阵精确换基]] 与 [[换基失败的统一不兼容格语义]]。^[primitive-involution.md:86-89, primitive-involution.md:104-113]

## 测试锚点与证据范围

测试使用因子 `[A1,A2]`、字母 `"cs"` 和 `perm [2,0,1]`，检查 Bourbaki 重编号对表输出的作用。按 `perm[k]` 的定义，三个扁平位置依次映射到矩阵下标 2、0、1。另一锚点是 A2 的 `'s'`／`'u'` 翻转矩阵 \(\begin{pmatrix}0&1\\1&0\end{pmatrix}\)，它在 `perm [1,0]` 下保持不变。^[primitive-involution.md:86-87, primitive-involution.md:122-127]

现有测试未覆盖 `'C'` 在非恒等 `perm` 下的行为，也未覆盖退化空输入。来源属于结构性源码阅读，不构成数学正确性验收；上游行号转录自代码注释，未独立核对上游字节。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。更多限制见 [[对合查表实现的证据范围与测试缺口]]。^[primitive-involution.md:132-149]

## Sources

- [primitive-involution.md](../../sources/primitive-involution.md)：内类字母解析与逐字母对合查表（`primitive_involution.rs`）。
