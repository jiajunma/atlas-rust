---
title: Cartan 矩阵的精确有理有限型检查
summary: 沿连通分量传播精确有理对称化比例，再以 LDLᵀ 式分解检查正主元；其与有限型的等价性仅记录为代码意图。
sources:
  - root-datum-dual.md
kind: concept
createdAt: "2026-10-09T15:09:44.830Z"
updatedAt: "2026-10-09T22:47:21.017Z"
tags:
  - Cartan矩阵
  - 精确线性代数
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
  - 精确算术
  - 有限型
aliases:
  - cartan-矩阵的精确有理有限型检查
---

# Cartan 矩阵的精确有理有限型检查

Cartan 矩阵的精确有理有限型检查是 [[BasedRootDatum：带基根数据与构造不变量]] 的构造校验环节，由 `root_datum.rs` 中的 `validate_cartan` 与 `is_finite_type` 实现。核心计算使用 `malachite::Rational`，先沿连通分量传播对称化比例，再检查加权矩阵的 LDLᵀ 式分解主元。来源将“该检查等价于有限型”列为代码意图，未作数学验收。^[root-datum-dual.md:39-65]

## 输入校验与错误顺序

`validate_cartan` 按固定顺序检查输入：首先拒绝非方阵，返回 `StructureError::NonSquareCartan`；随后要求对角元为 2、非对角元非正；再要求零模式对称，即不能出现 `(entry == 0) != (transpose == 0)`；最后执行有限型检查。后三项失败均返回 `StructureError::InvalidCartanMatrix`。^[root-datum-dual.md:53-58]

这是 `from_simple_data` 的首道门控。通过后，构造器才检查 `lattice_rank` 是否至少等于 `semisimple_rank`、简单根与余根的数量及坐标维数是否匹配，并逐项验证根与余根的配对是否等于 Cartan 矩阵对应元素。^[root-datum-dual.md:39-46]

## 两遍精确有理计算

第一遍按 Cartan 矩阵的连通分量播种，每个分量的起始比例 `scale` 为 1，再沿非零 Cartan 边传播相邻顶点的预期比例。计算公式如下；若传播结果发生冲突，立即返回 `false`。^[root-datum-dual.md:60-63]

$$
\operatorname{expected}
=
\operatorname{scale}[\mathrm{row}]
\frac{C[\mathrm{row}][\mathrm{col}]}
     {C[\mathrm{col}][\mathrm{row}]}.
$$

第二遍对 `scale` 加权矩阵进行 LDLᵀ 式分解，任一主元满足 `pivot ≤ 0` 即返回 `false`。两遍计算均采用精确有理数；实现中的两处 `.expect(...)` 依赖内部不变量：待处理的 `pending` 顶点必已具有 `scale`。^[root-datum-dual.md:60-65]

## 边界与构造复用

空 Cartan 矩阵通过全部检查，因为相关循环空转。这与纯环面的合法输入一致：`from_simple_data` 允许空 Cartan 矩阵配合正 `lattice_rank`，表示没有根的纯环面。^[root-datum-dual.md:48-58]

`BasedRootDatum` 区分整个约化环面的秩 `lattice_rank` 与简单根个数 `semisimple_rank`，二者仅在无中心环面时相等。该类型明确不设全局秩上限，来源以 A33 测试作为这一设计的锚点。^[root-datum-dual.md:18-21]

[[对偶根数据与对偶内类构造]] 中的 `dual_datum` 转置 Cartan 矩阵、互换简单根与余根后，再调用 `BasedRootDatum::from_simple_data`，因此复用包括有限型检查在内的全部构造门控。该接口仍保持可失败，来源未将已校验根数据的对偶重校验必然成功作为已验收结论。^[root-datum-dual.md:108-112]

## 证据范围

本页依据结构性源码阅读，记录实现的检查顺序、计算流程及失败条件。来源未证明 `is_finite_type` 与有限型的等价性，也未作数学或性能验收；本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[root-datum-dual.md:9-14, root-datum-dual.md:204-216]

## Sources

- [root-datum-dual.md](../../sources/root-datum-dual.md) — BasedRootDatum 与对偶内类构造（root_datum.rs / dual.rs）。
