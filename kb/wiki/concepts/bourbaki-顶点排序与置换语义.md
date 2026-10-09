---
title: Bourbaki 顶点排序与置换语义
summary: 各型通过端点选择、E 型长臂交换及 D 型短臂追加确定 position，拼接后的 result[i] 表示 Bourbaki 位置 i 对应的 datum 顶点。
sources:
  - dynkin.md
kind: concept
createdAt: "2026-10-09T14:45:47.400Z"
updatedAt: "2026-10-09T20:50:58.609Z"
tags:
  - Bourbaki编号
  - 置换
  - 根系
aliases:
  - bourbaki-顶点排序与置换语义
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# Bourbaki 顶点排序与置换语义

`bourbaki_permutation` 将 Cartan 矩阵的顶点按各连通分量的 Bourbaki 顺序排列，是 `dynkin.rs` 唯一对 crate 外公开的入口，供语言层按 Bourbaki 编号打印 grading。内部分类器为每个分量生成 `position` 向量，最终置换由这些向量依次拼接而成。^[dynkin.md:16-21, dynkin.md:74-76]

## 置换方向与分量顺序

返回向量满足：`result[i]` 是**占据 Bourbaki 位置 `i` 的原 datum 顶点编号**。它表示从 Bourbaki 位置到原顶点的对应关系，而非直接给出原顶点的新位置。^[dynkin.md:74-76]

连通分量按各分量最小顶点编号升序排列。分类器按顶点升序处理，通过 first-fresh-vertex 与 first-match 合并规则构造分量；各分量内部确定 `position` 后，再保持分量顺序拼接。输入检查与分量构造详见 [[Cartan 矩阵输入契约与连通分量划分]]。^[dynkin.md:43-47, dynkin.md:74-76]

## 秩一与秩二的排序

秩一分量归为 A 型。秩二分量先按 `support` 升序取顶点 `(i, j)`，再依据 `cartan[i][j] * cartan[j][i]` 判型：乘积为 1 时归为 A；乘积为 2 时，`cartan[i][j] == -1` 判为 C，否则判为 B，且保留给定顺序。因此，B2/C2 的标签由有序 Cartan 条目决定，相关约定见 [[秩二 Dynkin 分类的顺序约定]]。^[dynkin.md:51-56]

乘积为 3 时归为 G 型；若 `cartan[i][j] != -1`，则交换两个顶点的位置，使短根在前。^[dynkin.md:55-56]

## 高秩分量的起点与遍历

秩大于 2 时，分类器先根据顶点度数、分叉点和多重边识别类型，再选择遍历起点。多重边记录来自小于 `-1` 的 Cartan 条目，排序规则使用其中的 `lower`、`upper` 端点。判型条件详见 [[基于图结构的 Dynkin 单分量分类]]。^[dynkin.md:43-44, dynkin.md:58-69]

A 型从编号最小的端点开始。B 型从端点集中移除 `lower` 后取最小端点，C 型移除 `upper` 后取最小端点。F 型则从端点集中移除 `lower` 的邻居，再取最小端点。^[dynkin.md:64-69]

D4 可从任一端点开始；更高秩的 D 型排除与分叉点相邻的端点，从唯一剩余的长臂末端开始。D 型遍历若中断，只允许剩余分叉点旁的短臂顶点，并将它们按编号升序追加。^[dynkin.md:64-70]

E 型的短臂端点必须是唯一与分叉点相邻的端点。算法选择正交长臂端点；若首次选中的端点距离分叉点超过两步，则将其视为最长臂端点，改选另一条长臂。`position` 先放入“正交长臂端点、短臂端点”，再从公共邻居继续。^[dynkin.md:66-69]

完成起点及特殊前缀选择后，遍历每一步都取当前顶点编号最小的未访问邻居。这些排序规则保留了原顶点编号对选择结果的影响，包括 A 型与 D4 的端点选择以及 E 型的长臂交换。^[dynkin.md:16-19, dynkin.md:64-70]

## 测试锚点

B3 经 `[2,0,1]` 重标号后仍分类为 B，返回的 Bourbaki 置换可将重标号矩阵重建为标准形。A3 经 `[1,0,2]` 重标号后返回 `[1,0,2]`；D4 经 `[0,2,1,3]` 重标号、将分叉点从顶点 1 移至 2 后，返回 `[0,2,1,3]`。^[dynkin.md:92-95]

标准 D4、E6、F4 的测试分别得到按原编号排列的 `position`。块对角 A1+A2 分类为 `"AA"`，拼接后的 positions 为 `[0,1,2]`，体现分量的首顶点顺序。标准 B2/C2 返回恒等置换，G2 的两种指向覆盖交换与不交换，空矩阵返回空置换。^[dynkin.md:89-96]

## 证据边界

来源属于结构性源码阅读，不构成分类器的数学正确性验收。“精确复现 upstream”是实现注释的意图声明，材料未独立核对上游文件字节。六个测试均为成功路径，错误路径、E7/E8 以及秩大于 4 的 D 型缺少测试锚点；本次知识维护未执行 Atlas、Cargo、测试或 benchmark。详见 [[Dynkin 分类器的测试覆盖与证据边界]]。^[dynkin.md:9-12, dynkin.md:98-102, dynkin.md:111-115]

## Sources

- [dynkin.md](../../sources/dynkin.md) — Dynkin 分类器：连通分量、Bourbaki 置换与折叠 Cartan（dynkin.rs）。
