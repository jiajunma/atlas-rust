---
title: Cartan 矩阵的精确有理有限型检查
summary: 以精确有理数沿连通分量传播对称化比例，再通过 LDLᵀ 式分解检查正主元；其与有限型的等价性在来源中仅为代码意图，未经数学验收。
sources:
  - root-datum-dual.md
kind: concept
createdAt: "2026-10-09T15:09:44.830Z"
updatedAt: "2026-10-09T15:09:44.830Z"
tags:
  - Cartan矩阵
  - 精确计算
  - 有限型
aliases:
  - cartan-矩阵的精确有理有限型检查
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Cartan 矩阵的精确有理有限型检查

Cartan 矩阵的精确有理有限型检查是 `BasedRootDatum` 构造校验的一部分，由 `root_datum.rs` 中的 `validate_cartan` 与 `is_finite_type` 实现。检查使用 `malachite::Rational` 进行精确有理运算；其与有限型的等价性是代码意图，所提供的来源仅完成结构性阅读，不构成数学验收。^[root-datum-dual.md:39-65]

## 输入校验与检查顺序

`validate_cartan` 按固定顺序检查矩阵：先拒绝非方阵，返回 `NonSquareCartan`；再检查对角元是否均为 2、非对角元是否均非正；随后检查零模式是否对称，即是否存在 `(entry == 0) != (transpose == 0)`；最后执行有限型检查。后三类检查失败均返回 `InvalidCartanMatrix`。^[root-datum-dual.md:53-58]

这一步是 [[BasedRootDatum：带基根数据与构造不变量]] 的首道构造门控。通过后，构造器才继续检查 ambient 格秩、简单根与余根的数量及坐标维数，并逐项验证根与余根的配对等于 Cartan 矩阵对应元素。^[root-datum-dual.md:39-46]

## 两遍精确有理计算

第一遍按 Cartan 矩阵的连通分量传播缩放系数：每个分量以 `scale = 1` 播种，沿非零 Cartan 边按下式计算相邻顶点的预期系数；若传播结果发生冲突，立即返回 `false`。^[root-datum-dual.md:60-63]

$$
\operatorname{expected}
=
\operatorname{scale}[\mathrm{row}]
\frac{C[\mathrm{row}][\mathrm{col}]}
     {C[\mathrm{col}][\mathrm{row}]}.
$$

第二遍对 scale 加权矩阵进行 LDLᵀ 式分解，任何主元满足 `pivot ≤ 0` 都返回 `false`。两遍计算均使用精确有理数。实现中两处 `.expect(...)` 依赖内部不变量：待处理的 pending 顶点必已具有 scale。^[root-datum-dual.md:60-65]

## 边界与构造复用

空 Cartan 矩阵通过全部检查，因为相关循环空转。这与纯环面边界一致：`from_simple_data` 允许空 Cartan 配合正 `lattice_rank`，表示没有根的纯环面。^[root-datum-dual.md:48-58]

[[对偶根数据与对偶内类构造]] 中的 `dual_datum` 会转置 Cartan 矩阵、互换简单根与余根，再调用 `BasedRootDatum::from_simple_data`，因此复用包括有限型检查在内的全部构造门控。其接口仍保持可失败，来源未将重校验必然成功作为已验收结论。^[root-datum-dual.md:108-112]

## 证据范围

来源明确将“`is_finite_type` 检查等价于有限型”列为代码意图，未给出数学正确性或性能结论；本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。因此，本页记录的是实现的检查流程及失败条件。^[root-datum-dual.md:204-216]

## Sources

- [root-datum-dual.md](root-datum-dual.md) — BasedRootDatum 与对偶内类构造（root_datum.rs / dual.rs）
