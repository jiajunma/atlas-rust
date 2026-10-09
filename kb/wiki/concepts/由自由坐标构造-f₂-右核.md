---
title: 由自由坐标构造 F₂ 右核
summary: right_kernel 为每个自由坐标结合对应主元系数构造核向量，再插入新子空间形成重新约化的典范核基。
sources:
  - mod-two.md
kind: concept
createdAt: "2026-10-09T15:02:24.770Z"
updatedAt: "2026-10-09T15:02:24.770Z"
tags:
  - 线性代数
  - 核空间
  - 高斯消元
aliases:
  - 由自由坐标构造-f₂-右核
  - 由F右
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 由自由坐标构造 F₂ 右核

`ModTwoSubspace::right_kernel` 从 $\mathbb{F}_2$ 子空间的典范 RREF 基出发，为每个自由坐标构造一个核向量，再将这些向量插入新子空间。该方法为 `pub(crate)` 接口，返回的核通过新子空间的消元过程重新规范化。^[mod-two.md:42-49]

## 基与自由坐标

`ModTwoSubspace` 按每行的最低置位位置索引主元（pivot）。插入向量时，先用现有基约化输入，再选择最低置位作为新主元，并从旧行中消去该主元，从而保持与插入顺序无关的典范 RREF。`pivot_rows()` 按主元升序提供 `(pivot, row)`，`basis_vectors()` 以同一顺序提供基向量。相关结构见[[最低主元索引的典范 RREF 子空间]]。^[mod-two.md:30-49]

## 构造规则

对于每个自由坐标 $f$，`right_kernel` 构造一个向量：在 $f$ 处置位，并在所有“其基行在 $f$ 处置位”的主元位置置位。若以 $r_p$ 表示主元为 $p$ 的基行、以 $e_i$ 表示第 $i$ 个坐标的单位向量，则这一规则可写为
\[
v_f=e_f+\sum_{\substack{p\text{ 为主元}\\r_p[f]=1}}e_p.
\]
每个 $v_f$ 随后插入新的 `ModTwoSubspace`。^[mod-two.md:44-48]

新子空间的插入操作会重新约化这些生成向量，因此应区分按自由坐标直接构造的向量与最终返回的典范基。源材料特别指出，`pivot_rows()` 保留原子空间按主元排序的基行，而 `right_kernel` 将构造出的核向量重新约化为新子空间。^[mod-two.md:32-35, mod-two.md:45-49]

## 与配对及调用方的关系

底层向量采用[[F₂ 上的位打包向量（ModTwoVector）|ModTwoVector]]。其 `dot` 方法实现 $\mathbb{F}_2$ 配对：逐字计算交集置位数的奇偶性，再以 XOR 合并结果；维数不匹配时报 `RankMismatch`。^[mod-two.md:15-28]

源材料还记录了与生成元读取有关的区别：`real_weyl.rs` 的 R-group 核构造从 `pivot_rows()` 提供的排序基行读取生成元位，而 `right_kernel` 会重新约化。涉及这些生成元时，应保留这一接口差异；相关主题见[[fiber grading 与 R-群核生成元]]。^[mod-two.md:45-48]

## 测试与证据边界

源文件的 12 个测试包含右核计算，也覆盖插入后的 RREF 保持、跨多字依赖消元及 130 维动态子空间。源材料未给出右核测试的具体输入与断言，并明确指出 `dot` 没有文件内直接测试。^[mod-two.md:83-93]

这些信息来自结构性源码阅读；该来源包没有执行构建、测试或原版运行，也不提供数学验收、性能或并行结论。^[mod-two.md:9-13, mod-two.md:95-103]

## Sources

- [mod-two.md](mod-two.md) — mod-2 线性代数：位打包向量与子空间。
