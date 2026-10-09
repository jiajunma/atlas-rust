---
title: 最低主元索引的典范 RREF 子空间
summary: ModTwoSubspace 通过升序消元及插入后的旧行消元保持与插入顺序无关的典范既约行阶梯基，支持秩与成员判定。
sources:
  - mod-two.md
kind: concept
createdAt: "2026-10-09T15:02:13.888Z"
updatedAt: "2026-10-09T15:02:13.888Z"
tags:
  - 线性代数
  - 高斯消元
  - 典范基
aliases:
  - 最低主元索引的典范-rref-子空间
  - 最R子
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 最低主元索引的典范 RREF 子空间

`ModTwoSubspace` 表示 $\mathbb{F}_2$ 上的子空间，以每个基向量的**最低置位坐标**作为主元（pivot）索引，维护典范的简化行阶梯形（RREF）基。该基与向量插入顺序无关，为结构子商层提供确定性坐标；最低主元的选择沿用 Atlas `BitVector::firstBit()` 的约定。^[mod-two.md:30-35]

## 存储与插入不变量

子空间包含环境维数 `dimension`、主元行数组 `pivots: Vec<Option<ModTwoVector>>` 和秩 `rank`。每个基向量采用 [[F₂ 上的位打包向量（ModTwoVector）]] 表示，数组按最低置位主元索引各行。^[mod-two.md:17-20, mod-two.md:32-35]

`insert` 首先约化输入向量，再取剩余向量的最低置位作为新主元，并从所有旧行中消去该主元，使基始终保持典范 RREF。方法返回秩是否增加；秩通过 `rank.checked_add(1)` 更新，溢出时返回 `ArithmeticOverflow`。^[mod-two.md:33-37]

## 升序约化与商代表

私有方法 `reduce` 按主元索引升序扫描。缺失某个较早主元并不意味着后续主元也缺失，因此不能据此停止扫描。由于典范基在每个主元处已经既约，升序扫描恰好清除每个主元系数一次；源码注释将这一行为对齐到上游 `normalSpanAdd`。^[mod-two.md:42-44]

`contains` 通过约化后是否无剩余来判断向量是否属于子空间。`quotient_representative` 直接返回约化结果，作为商类的确定性行阶梯代表元；`same_coset(left, right)` 先检查维数，再判断两向量之差是否属于子空间。商空间维数为 `dimension - rank`，相关语义见 [[F₂ 商空间的确定性代表元与陪集判定]]。^[mod-two.md:37-40]

## 右核与基的遍历顺序

`right_kernel` 对每个自由坐标构造一个向量：在该自由坐标置位，并在原基中所有于该自由坐标处置位的行所对应的主元坐标置位。随后将这些向量插入新子空间，使右核自身也按同一规则约化。该方法的可见性为 `pub(crate)`。^[mod-two.md:44-48]

`pivot_rows()` 按主元升序产出 `(pivot, row)`，对应上游 `Gauss_Jordan` 提交的排序典范基；`basis_vectors()` 以相同顺序仅产出行向量。`real_weyl.rs` 的 R-group 核构造直接从这些行读取生成元位，因此基行的顺序与形态是调用方使用的数据约定。^[mod-two.md:45-49]

## 子商层中的作用

[[F₂ 子商的低主元坐标（ModTwoSubquotient）]] 使用两个共同环境空间中的子空间构造典范商。其构造要求分母包含于分子，补基恰好来自分母缺失主元的位置；坐标转换按补基的最低置位读写位。该低主元打包坐标属于 crate 私有实现，公开接口是数学意义上的 `CartanFiber` 包装，并非序列化格式。^[mod-two.md:64-74]

## 测试与证据边界

源材料列出的测试锚点包括：130 维动态子空间、依赖检测与维数不符、跨多字依赖消元、确定性商代表与陪集、插入后直接检查私有 `pivots` 的 RREF 保持，以及右核计算。这些锚点覆盖了动态维数、约化行为和典范基维护的部分关键性质。^[mod-two.md:83-89]

该源包属于结构性源码阅读记录，未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；多数 `ArithmeticOverflow` 与 `AllocationFailed` 分支也未被所列测试覆盖。^[mod-two.md:9-13, mod-two.md:89-93, mod-two.md:103-108]

## Sources

- [mod-two.md](mod-two.md) — mod-2 线性代数：位打包向量与子空间。
