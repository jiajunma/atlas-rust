---
title: Bourbaki 重编号与对合矩阵下标映射
summary: perm[k] 指定第 k 个扁平化 Bourbaki 单根的输出矩阵下标，排列合法性由包装器验证，查表函数依赖调用方满足前置条件。
sources:
  - primitive-involution.md
kind: concept
createdAt: "2026-10-09T15:03:59.488Z"
updatedAt: "2026-10-10T00:45:45.316Z"
tags:
  - Bourbaki
  - 对合
  - 置换
aliases:
  - bourbaki-重编号与对合矩阵下标映射
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Bourbaki 重编号与对合矩阵下标映射
summary: perm[k] 将第 k 个扁平化单根位置映射到输出矩阵下标；排列合法性由包装器验证，layout_involution 依赖调用方满足前置条件。
sources:
  - primitive-involution.md
kind: concept
tags:
  - Bourbaki编号
  - 矩阵
  - 接口契约
aliases:
  - bourbaki-重编号与对合矩阵下标映射
---

# Bourbaki 重编号与对合矩阵下标映射

Bourbaki 重编号规定 `layout_involution` 中扁平化单根位置与输出矩阵下标的对应关系。函数输出单连通群基本权基上的行主序 `i32` 对合矩阵，其中 `perm[k]` 表示第 `k` 个扁平化单根的矩阵下标，对应上游 `Layout::d_perm`。^[primitive-involution.md:84-91]

## 映射方向与职责

`perm` 是用户提供的、针对扁平化单因子的 Bourbaki 重编号，映射方向为“扁平位置 `k` → 输出矩阵下标 `perm[k]`”。包装器侧的 `checked_permutation` 验证排列合法性；`layout_involution` 所属模块执行纯查表，不涉及 root datum、Weyl group 或 inner-class 构造管线。^[primitive-involution.md:17-25, primitive-involution.md:86-89]

查表使用两个位置指针：`r` 表示扁平图位置，`pos` 表示因子位置。内类字母决定各因子的对合模式，`perm` 指定相应顶点在输出矩阵中的位置。完整分派见 [[单连通群基本权基上的逐字母对合表]]。^[primitive-involution.md:86-102]

## 对合模式与重编号

字母 `'c'` 产生恒等块；`'s'` 在 A 型上产生反对角矩阵，在奇秩 D 型上交换末两顶点，在 E6 上固定局部下标 1、3 并交换 0↔5、2↔4，在环面 T 上产生负恒等矩阵；`'u'` 交换末两顶点。这些查表位置通过 `perm` 对应到输出矩阵下标。^[primitive-involution.md:86-102]

解析阶段会将 A1、B、C、偶秩 D、E7、E8、F、G 的 `'s'` 坍缩为 `'c'`。其中 B、C、E7、E8、F、G 在查表中的默认恒等分支，在遵守解析前置条件的调用下不可达，仅对绕过解析的输入起防御作用。^[primitive-involution.md:77-79, primitive-involution.md:138-139]

复型字母 `'C'` 将当前因子的 `rs` 个顶点与紧随因子的 `rs` 个顶点平行互换，并额外消耗一个因子。[[内类字母的字节解析与规范化]] 要求这两个因子相同且连续，为该交换提供前置保证。^[primitive-involution.md:74-75, primitive-involution.md:100-101]

## 前置条件与失败边界

`letters` 必须来自 `checked_inner_class_letters`，`perm` 必须是 `0..rank` 的排列。函数仅通过两道 `debug_assert_eq!` 检查 `perm.len() == rank` 和最终 `r == rank`，不返回 `Result`。违反前置条件可能导致下标越界 panic 或触发 `unreachable!`；排列合法性由包装器负责验证。^[primitive-involution.md:23-25, primitive-involution.md:86-91]

## 与一般格基转换的关系

Bourbaki 重编号规定查表输出的矩阵下标；独立函数 `on_basis` 则计算一般格基上的表示 \(B^{-1}MB\)，其中 \(M\) 为 `matrix`，\(B\) 为 `basis`。该函数使用有理逆和有理矩阵乘法，再逐项检查结果的整性及 `i32` 可表示性。^[primitive-involution.md:86-89, primitive-involution.md:104-111]

`on_basis` 将非方阵、奇异基、非整结果和转换失败四类情况统一折叠为 `None`，调用方无法区分，包装器将其统一标记为不兼容格。详见 [[换基失败的统一不兼容格语义]]。^[primitive-involution.md:111-113]

## 测试锚点与证据范围

测试使用因子 `[A1,A2]`、字母 `"cs"` 和 `perm [2,0,1]`，检查 Bourbaki 重编号对表输出的作用。另一个锚点是 A2 的 `'s'`／`'u'` 翻转矩阵 \(\begin{pmatrix}0&1\\1&0\end{pmatrix}\)，它在 `perm [1,0]` 下保持不变。^[primitive-involution.md:122-127]

现有测试未覆盖 `'C'` 在非恒等 `perm` 下的行为，也未覆盖退化空输入。来源属于结构性源码阅读，不构成数学正确性验收；上游行号转录自代码注释，未独立核对上游字节。该次知识维护未执行 Atlas、Cargo、测试或 benchmark，因此测试锚点不代表本次执行结果。更多限制见 [[对合查表实现的证据范围与测试缺口]]。^[primitive-involution.md:132-149]

## Sources

- [primitive-involution.md](../../sources/primitive-involution.md)：内类字母解析与逐字母对合查表（`primitive_involution.rs`）。
