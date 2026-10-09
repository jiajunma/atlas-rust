---
title: Cartan 矩阵的精确有理有限型检查
summary: 先沿连通分量传播精确有理对称化比例，再以 LDLᵀ 式分解检查正主元；与有限型的等价性仅为代码意图，未获本包数学验收。
sources:
  - root-datum-dual.md
kind: concept
createdAt: "2026-10-09T15:09:44.830Z"
updatedAt: "2026-10-09T21:08:38.258Z"
tags:
  - Cartan矩阵
  - 精确算术
  - 有限型
aliases:
  - cartan-矩阵的精确有理有限型检查
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Cartan 矩阵的精确有理有限型检查
summary: 使用精确有理数传播 Cartan 矩阵的对称化比例，再以 LDLᵀ 式分解检查正主元；与有限型的等价性属于代码意图，来源未作数学验收。
sources:
  - root-datum-dual.md
kind: concept
tags:
  - Cartan矩阵
  - 精确计算
  - 有限型
aliases:
  - cartan-矩阵的精确有理有限型检查
---

# Cartan 矩阵的精确有理有限型检查

Cartan 矩阵的精确有理有限型检查是 [[BasedRootDatum：带基根数据与构造不变量]] 的构造校验环节，由 `root_datum.rs` 中的 `validate_cartan` 与 `is_finite_type` 实现。核心计算使用 `malachite::Rational`，先传播对称化比例，再检查加权矩阵分解中的主元。来源将“该检查等价于有限型”列为代码意图，未作数学验收。^[root-datum-dual.md:39-65]

## 输入校验与错误顺序

`validate_cartan` 按固定顺序拒绝不合法输入：首先检查是否为方阵，失败返回 `NonSquareCartan`；其次要求对角元为 2、非对角元非正；随后要求零模式对称，即不能出现 `(entry == 0) != (transpose == 0)`；最后执行有限型检查。后三项失败均返回 `InvalidCartanMatrix`。^[root-datum-dual.md:53-58]

这是 `from_simple_data` 的首道门控。通过后，构造器才检查 `lattice_rank` 是否至少等于简单根个数、根与余根的数量及坐标维数是否匹配，并逐项验证根与余根的配对等于 Cartan 矩阵对应元素。^[root-datum-dual.md:39-46]

## 两遍精确有理计算

第一遍按 Cartan 矩阵的连通分量播种，每个分量的起始缩放系数为 1。沿非零 Cartan 边传播时，相邻顶点的预期系数按 `expected = scale[row] * C[row][col] / C[col][row]` 计算；若传播结果发生冲突，立即返回 `false`。^[root-datum-dual.md:60-63]

$$
\operatorname{expected}
=
\operatorname{scale}[\mathrm{row}]
\frac{C[\mathrm{row}][\mathrm{col}]}
     {C[\mathrm{col}][\mathrm{row}]}.
$$

第二遍对 scale 加权矩阵进行 LDLᵀ 式分解，任一主元满足 `pivot ≤ 0` 即返回 `false`。两遍均采用精确有理运算；实现中的两处 `.expect(...)` 依赖内部不变量：待处理的 pending 顶点必已具有 scale。^[root-datum-dual.md:60-65]

## 边界与构造复用

空 Cartan 矩阵通过全部检查，因为相关循环空转。这与纯环面的合法输入一致：`from_simple_data` 允许空 Cartan 矩阵配合正 `lattice_rank`，表示没有根的纯环面。^[root-datum-dual.md:48-58]

`BasedRootDatum` 区分整个格的 `lattice_rank` 与简单根个数 `semisimple_rank`，并明确不设全局秩上限；来源以 A33 测试作为这一设计的锚点。^[root-datum-dual.md:18-21]

[[对偶根数据与对偶内类构造]] 中的 `dual_datum` 转置 Cartan 矩阵、互换简单根与余根后，再调用 `BasedRootDatum::from_simple_data`，因此复用包括有限型检查在内的全部构造门控。该接口仍保持可失败，来源未将对偶重校验必然成功作为已验收结论。^[root-datum-dual.md:108-112]

## 证据范围

本页依据结构性源码阅读，记录实现的校验顺序、计算流程及失败条件。来源未证明 `is_finite_type` 与有限型的等价性，也未作数学正确性或性能结论；本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[root-datum-dual.md:9-14, root-datum-dual.md:204-216]

## Sources

- [root-datum-dual.md](../../sources/root-datum-dual.md) — BasedRootDatum 与对偶内类构造（root_datum.rs / dual.rs）。
