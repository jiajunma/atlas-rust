---
title: Cartan 矩阵输入契约与连通分量划分
summary: classify 检查方形、对角元 2、非对角元范围与零模式对称性，通过 first-match 合并生成按最小顶点升序排列的连通分量；这些检查不构成完整的数学合法性验收。
sources:
  - dynkin.md
kind: concept
createdAt: "2026-10-09T14:45:14.337Z"
updatedAt: "2026-10-09T14:45:14.337Z"
tags:
  - Cartan矩阵
  - 连通分量
  - 输入校验
aliases:
  - cartan-矩阵输入契约与连通分量划分
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Cartan 矩阵输入契约与连通分量划分

`dynkin.rs` 中的 `classify` 接收 Cartan 矩阵，先检查输入的基本结构，再划分连通分量，并对各分量进行 [[基于图结构的 Dynkin 单分量分类|Dynkin 类型识别]]与 Bourbaki 顶点排序。该函数为 `pub(crate)`，返回 `Result<Vec<DynkinComponent>, StructureError>`。^[dynkin.md:16-33]

## 输入契约

调用方应从已经检查的数据构造输入，使其满足 based-datum Cartan 不变量。`classify` 按行主序扫描，检查矩阵是否为方形、对角元是否为 2、非对角元是否属于整数范围 `-3..=0`，以及零模式是否对称。非方形输入使用 `NonSquareCartan` 错误；其他不变量违规以 layout invariant 错误报告。^[dynkin.md:36-41]

零模式对称要求：若 `cartan[i][j] != 0`，则 `cartan[j][i] != 0`。这一阶段不校验两个方向条目的数值配对，因此这些检查不能单独作为完整的 Cartan 合法性验证。规模为 0 的输入合法，空矩阵产生空分量列表。^[dynkin.md:38-41, dynkin.md:103-103]

## 邻接结构与连通分量

分类器以 `star[v]` 保存顶点 `v` 的全部邻居，并将所有满足 `entry < -1` 的条目记录到 `down_edges`，形式为有序三元组 `(i, j, -entry)`，其中边标号为 2 或 3。邻居集用于分量划分，带标号的有向边则参与后续类型判定。^[dynkin.md:43-47, dynkin.md:58-62]

分量划分采用 first-fresh-vertex 顺序与 first-match 合并规则：按顶点编号升序处理；若当前顶点的邻居编号均大于自身，则新建分量，孤立顶点也形成单点分量。否则，将当前顶点并入第一个与其邻居集相交的已有分量，再移除后续所有相交分量并将它们合并进来。最终分量按各自最小顶点的升序排列。^[dynkin.md:43-47]

## 顺序语义与后续分类

每个 `DynkinComponent` 保存类型字母 `letter`、顶点支持集 `support: BTreeSet<usize>` 和有序顶点向量 `position: Vec<usize>`。分量顺序与分量内部的 Bourbaki 顺序共同决定最终置换：`bourbaki_permutation` 按分量顺序拼接所有 `position`，其中 `result[i]` 表示占据 Bourbaki 位置 `i` 的原 datum 顶点。参见 [[Bourbaki 顶点排序与置换语义]]。^[dynkin.md:25-30, dynkin.md:74-76]

输入编号还会影响秩二分量的 B/C 标签。分类器按 `support` 升序取 `(i, j)`；当 `cartan[i][j] * cartan[j][i] == 2` 时，若 `cartan[i][j] == -1` 则判为 C，否则判为 B，并保持给定顺序。该规则体现了有序 Cartan 条目对分类标签的影响。^[dynkin.md:51-56]

## 测试与证据边界

成功路径测试包含块对角矩阵 A1+A2，其分类结果为 `"AA"`，拼接后的顶点位置为 `[0,1,2]`，体现 first-vertex 分量顺序；空矩阵测试得到空置换。现有 6 个测试全部覆盖成功路径，错误路径没有测试覆盖，相关范围见 [[Dynkin 分类器的测试覆盖与证据边界]]。^[dynkin.md:87-102]

来源材料属于结构性源码阅读，不构成分类器的数学验收。对上游行为“精确复现”的描述是代码注释的意图声明，材料未核对上游源码字节；本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[dynkin.md:9-12, dynkin.md:100-102, dynkin.md:111-115]

## Sources

- [dynkin.md](dynkin.md) — Dynkin 分类器：连通分量、Bourbaki 置换与折叠 Cartan（dynkin.rs）
