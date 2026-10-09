---
title: F₂ 商空间的确定性代表元与陪集判定
summary: 将向量按子空间典范基约化得到确定性商类代表元，并通过两向量之差是否属于子空间判断是否同陪集。
sources:
  - mod-two.md
kind: concept
createdAt: "2026-10-09T15:02:19.204Z"
updatedAt: "2026-10-09T15:02:19.204Z"
tags:
  - 商空间
  - 典范代表元
  - 线性代数
aliases:
  - f₂-商空间的确定性代表元与陪集判定
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# F₂ 商空间的确定性代表元与陪集判定

`ModTwoSubspace` 通过典范既约行阶梯形（RREF）基，为 $\mathbb{F}_2$ 商空间提供确定性代表元。`quotient_representative` 将向量约化为其商类的代表元，`same_coset` 则通过判断两个向量之差是否属于子空间来判定陪集相同。商空间维数为 `dimension - rank`。^[mod-two.md:30-40]

## 典范基与代表元

子空间以 `dimension`、`pivots: Vec<Option<ModTwoVector>>` 和 `rank` 存储，基行按最低置位主元索引。插入向量时，先用已有基约化输入，再取剩余向量的最低置位作为新主元，并从旧行中消去该主元。这样得到的 RREF 基与插入顺序无关，为确定性商坐标提供基础，详见 [[最低主元索引的典范 RREF 子空间]]。^[mod-two.md:30-40]

`quotient_representative(vector)` 直接返回私有方法 `reduce` 的结果。约化按主元升序扫描；某个早期主元缺失，并不意味着后续主元也缺失，因此扫描仍需继续。由于典范基在每个主元处都已既约，这一过程恰好清除每个主元系数一次，得到确定性的行阶梯代表元。^[mod-two.md:38-44]

## 成员与陪集判定

`contains(vector)` 判断向量约化后是否为零，即检验其是否属于子空间。`same_coset(left, right)` 先检查维数，再判断两者之差是否属于子空间。对于共同环境空间中的子空间 $S$，其判定条件可写为
\[
x+S=y+S\quad\Longleftrightarrow\quad x-y\in S.
\]
这使成员判定、商代表元计算和陪集判定共享同一套约化机制。^[mod-two.md:32-44]

## 在子商中的使用

[[F₂ 子商的低主元坐标（ModTwoSubquotient）|ModTwoSubquotient]] 将这一机制用于共同环境空间中的分子与分母子空间。构造时检查维数一致、分母秩不超过分子秩，以及分母的每个基向量都属于分子；补基则来自分母缺少主元的位置。该类型刻意保持 crate 私有，公开数学接口由 `CartanFiber` 包装提供，其低主元打包坐标并非序列化格式。^[mod-two.md:62-70]

子商的 `canonical_representative(vector)` 首先检查向量是否属于分子；若不属于，返回 `NotInModTwoSubspace`，否则调用分母的 `quotient_representative`。`to_coordinates` 与 `ambient_representative` 再按补基最低置位读取或写入坐标位，将商代表元与子商坐标衔接起来。^[mod-two.md:72-74]

## 测试与证据边界

源文件的测试锚点包括确定性商代表元与陪集判定、插入后的 RREF 保持、跨多字的依赖消元、130 维动态子空间，以及子商低主元坐标。覆盖缺口包括 `ModTwoSubquotientInvariantViolation` 的五处检查及 `NotInModTwoSubspace` 错误分支。^[mod-two.md:83-93]

这些信息来自源码结构性阅读及测试内容核对；来源材料未执行构建、测试或原版运行，因此不构成数学验收或性能结论。^[mod-two.md:9-13, mod-two.md:103-108]

## Sources

- [mod-two.md](../../sources/mod-two.md) — mod-2 线性代数：位打包向量与子空间。
