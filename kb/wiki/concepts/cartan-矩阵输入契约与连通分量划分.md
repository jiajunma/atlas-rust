---
title: Cartan 矩阵输入契约与连通分量划分
summary: classify 校验方形、对角元、非对角元范围及零模式对称性，以 first-match 合并生成按最小顶点升序排列的分量，但不检查数值配对。
sources:
  - dynkin.md
kind: concept
createdAt: "2026-10-09T14:45:14.337Z"
updatedAt: "2026-10-09T22:27:36.058Z"
tags:
  - Dynkin分类
  - 输入校验
aliases:
  - cartan-矩阵输入契约与连通分量划分
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Cartan 矩阵输入契约与连通分量划分
summary: classify 检查 Cartan 矩阵的基本结构，通过 first-match 合并按最小顶点升序组织连通分量；这些检查不构成完整的数学合法性验收。
sources:
  - dynkin.md
kind: concept
tags:
  - Cartan矩阵
  - 连通分量
  - 输入校验
aliases:
  - cartan-矩阵输入契约与连通分量划分
---

# Cartan 矩阵输入契约与连通分量划分

`dynkin.rs` 中的 `classify` 接收 Cartan 矩阵，检查基本结构、划分连通分量，再进行 [[基于图结构的 Dynkin 单分量分类|Dynkin 类型识别]]与 Bourbaki 顶点排序。它是 crate 内部接口，返回 `Result<Vec<DynkinComponent>, StructureError>`；crate 外唯一入口为 `bourbaki_permutation`。^[dynkin.md:16-33]

## 输入契约

调用方应从已经检查的数据构造输入，使其满足 based-datum Cartan 不变量。`classify` 按行主序检查矩阵为方形、对角元为 2、非对角元属于 `-3..=0`，以及零模式对称性。非方形输入报 `NonSquareCartan`；其他所述不变量违规以 layout invariant 错误报告。^[dynkin.md:36-41]

零模式对称要求：若 `cartan[i][j] != 0`，则 `cartan[j][i] != 0`。此处不校验两个方向条目的数值配对，因此这些基础检查不能视为完整的 Cartan 合法性验证。规模为 0 的输入合法，空矩阵产生空分量列表。^[dynkin.md:38-41, dynkin.md:100-103]

## 邻接结构与连通分量合并

分类器以 `star[v]` 保存顶点 `v` 的全部邻居，并将满足 `entry < -1` 的条目记录到 `down_edges`，形式为 `(i, j, -entry)`，边标号为 2 或 3。邻接集合用于分量划分，多重边信息则参与后续单分量类型判定。^[dynkin.md:43-47, dynkin.md:58-62]

分量划分采用 first-fresh-vertex 顺序与 first-match 合并规则。顶点按编号升序处理：如果当前顶点的所有邻居编号均大于自身，则新建分量，孤立顶点也形成单点分量；否则，将当前顶点并入第一个与其邻居集相交的已有分量，再移除后续相交分量并合并进来。最终分量按各自最小顶点的升序排列。^[dynkin.md:43-47]

## 分量顺序与 Bourbaki 置换

每个 `DynkinComponent` 保存类型字母 `letter`、顶点支持集 `support: BTreeSet<usize>` 和位置向量 `position: Vec<usize>`。分量间顺序与分量内顶点顺序共同决定最终置换：`bourbaki_permutation` 按分量顺序拼接所有 `position`，其中 `result[i]` 是占据 Bourbaki 位置 `i` 的原 datum 顶点。参见 [[Bourbaki 顶点排序与置换语义]]。^[dynkin.md:25-30, dynkin.md:74-76]

输入编号还影响秩二分量的 B/C 标签。分类器按 `support` 升序取 `(i, j)`；当 `cartan[i][j] * cartan[j][i] == 2` 时，若 `cartan[i][j] == -1` 则判为 C，否则判为 B，并保持给定顺序。因此，不能因两种秩二系统同构而忽略有序 Cartan 条目对标签的影响。^[dynkin.md:51-56]

## 测试与证据边界

测试锚点包含块对角矩阵 A1+A2：分类字母为 `"AA"`，位置向量拼接为 `[0,1,2]`，体现 first-vertex 分量顺序；空矩阵得到空置换。来源记录的 6 个测试全部为成功路径，错误路径没有测试覆盖，E7/E8 与秩大于 4 的 D 型也没有测试锚点。参见 [[Dynkin 分类器的测试覆盖与证据边界]]。^[dynkin.md:87-103]

来源材料属于结构性源码阅读，不构成分类器的数学验收。“精确复现 upstream”是注释中的意图声明，上游引用未经过独立字节核对；本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[dynkin.md:9-12, dynkin.md:100-102, dynkin.md:111-115]

## Sources

- [dynkin.md](../../sources/dynkin.md) — Dynkin 分类器：连通分量、Bourbaki 置换与折叠 Cartan（dynkin.rs）。
