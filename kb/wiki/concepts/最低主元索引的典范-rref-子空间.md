---
title: 最低主元索引的典范 RREF 子空间
summary: ModTwoSubspace 按最低置位索引主元，通过升序消元及插入后的旧行消元维持与插入顺序无关的典范基。
sources:
  - mod-two.md
kind: concept
createdAt: "2026-10-09T15:02:13.888Z"
updatedAt: "2026-10-09T22:40:34.672Z"
tags:
  - 模二线性代数
  - 高斯消元
  - 确定性坐标
aliases:
  - 最低主元索引的典范-rref-子空间
  - 最R子
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 最低主元索引的典范 RREF 子空间
summary: ModTwoSubspace 以最低置位坐标索引主元，通过升序约化和旧行消元维护与插入顺序无关的典范 RREF 基，为成员判定、商代表和子商坐标提供确定性。
sources:
  - mod-two.md
kind: concept
tags:
  - 有限域
  - 线性代数
  - 规范化
---

# 最低主元索引的典范 RREF 子空间

`ModTwoSubspace` 表示 $\mathbb{F}_2$ 上的子空间，以基向量的**最低置位坐标**作为主元（pivot）索引，维护典范的既约行阶梯形（RREF）基。该基与插入顺序无关，为结构子商层提供确定性坐标；最低主元约定沿用 Atlas 的 `BitVector::firstBit()`。^[mod-two.md:30-35]

## 存储与插入不变量

子空间保存环境维数 `dimension`、主元行数组 `pivots: Vec<Option<ModTwoVector>>` 和秩 `rank`。行向量使用 [[F₂ 上的位打包向量（ModTwoVector）]] 表示，其存储编码有限向量空间元素，与无界整数层分离。^[mod-two.md:17-20, mod-two.md:32-35]

`insert` 先用 `reduce` 约化输入，再取剩余向量的最低置位作为新主元，并从旧行中消去该主元，以保持典范 RREF。方法返回秩是否增加；秩增长使用 `rank.checked_add(1)`，溢出时返回 `ArithmeticOverflow`。^[mod-two.md:33-37]

## 升序约化与商代表

私有方法 `reduce` 按主元索引**升序**扫描。早期主元缺失不意味着后续主元也缺失，不能据此提前结束扫描。典范基在每个主元处已经既约，升序扫描恰好清除每个主元系数一次；源码注释将这一行为对应到上游 `normalSpanAdd`。^[mod-two.md:42-44]

`contains` 通过约化后是否无剩余判断成员关系；`quotient_representative` 直接返回约化结果，作为商类的确定性行阶梯代表元。`same_coset(left, right)` 先检查维数，再判断两向量之差是否属于子空间；`quotient_dimension` 返回 `dimension - rank`。相关主题见 [[F₂ 商空间的确定性代表元与陪集判定]]。^[mod-two.md:37-40]

## 右核与基的遍历

`right_kernel` 的可见性为 `pub(crate)`。它对每个自由坐标构造一个向量：在该自由坐标置位，并在所有于该自由坐标处置位的原基行所对应的主元坐标置位。随后将这些向量插入新子空间，重新约化为右核的基。^[mod-two.md:44-48]

`pivot_rows()` 按主元升序产出 `(pivot, row)`，`basis_vectors()` 以相同顺序仅产出行向量。前者对应上游 `Gauss_Jordan` 提交的排序典范基；`real_weyl.rs` 的 R-group 核构造从这些行读取生成元位。这里的上游位置来自源码注释的转述。^[mod-two.md:12-13, mod-two.md:45-49]

## 子商坐标中的作用

[[F₂ 子商的低主元坐标（ModTwoSubquotient）]] 使用共同环境空间中的分子、分母子空间构造典范商。构造时检查维数一致、分母秩不超过分子秩、分母每个基向量属于分子，以及补基恰好来自分母缺失主元的位置；违反条件返回 `ModTwoSubquotientInvariantViolation`。^[mod-two.md:64-70]

子商的 `to_coordinates` 与 `ambient_representative` 按补基最低置位读写位。这套低主元打包坐标刻意保持为 crate 私有实现；公开 API 是数学意义上的 `CartanFiber` 包装，并非序列化格式。^[mod-two.md:64-74]

## 测试与证据边界

源材料列出的测试锚点包括 130 维动态子空间、依赖检测与维数不符、跨多字依赖消元、确定性商代表与陪集、插入后直接检查私有 `pivots` 的 RREF 保持，以及右核计算和子商低主元坐标。多数 `ArithmeticOverflow`／`AllocationFailed` 分支、五处 `ModTwoSubquotientInvariantViolation` 以及 `NotInModTwoSubspace` 未被所列测试覆盖。^[mod-two.md:85-92]

本页依据结构性源码阅读整理。源包未执行构建、测试或原版运行，因此测试锚点仅说明源码中存在相应测试，不代表本次已验证其运行结果，也不构成数学验收、性能或并行结论。^[mod-two.md:9-13, mod-two.md:103-108]

## Sources

- [mod-two.md](../../sources/mod-two.md) — mod-2 线性代数：位打包向量与子空间。
