---
title: 由自由坐标构造 F₂ 右核
summary: 对每个自由坐标结合对应主元行系数生成核向量，再插入新子空间，得到重新约化的典范核基。
sources:
  - mod-two.md
kind: concept
createdAt: "2026-10-09T15:02:24.770Z"
updatedAt: "2026-10-10T00:44:42.317Z"
tags:
  - 模二线性代数
  - 核空间
  - 典范基
aliases:
  - 由自由坐标构造-f₂-右核
  - 由F右
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 由自由坐标构造 F₂ 右核
summary: right_kernel 为每个自由坐标结合主元行系数生成核向量，再插入新子空间，形成重新约化的典范核基。
sources:
  - mod-two.md
kind: concept
tags:
  - 模二线性代数
  - 核空间
  - 算法
---

# 由自由坐标构造 F₂ 右核

`ModTwoSubspace::right_kernel` 是 crate 内可见的右核构造方法。它从 $\mathbb{F}_2$ 子空间的典范 RREF 基出发，为每个自由坐标生成核向量，再将这些向量插入新子空间，得到重新约化的典范核基。^[mod-two.md:42-49]

## 主元与基的约定

`ModTwoSubspace` 以基行的**最低置位位置**作为主元（pivot）。`insert` 先约化输入，取剩余向量的最低置位作为新主元，再从旧行中消去该主元，使基保持与插入顺序无关的典范 RREF。相关不变量见 [[最低主元索引的典范 RREF 子空间]]。^[mod-two.md:30-35]

`pivot_rows()` 按主元升序产出 `(pivot, row)`，`basis_vectors()` 按同一顺序产出基行。内部 `reduce` 也按主元升序扫描；早期位置没有主元，并不意味着后续位置没有主元。由于典范基在各主元处已经既约，升序扫描恰好将每个主元系数清除一次。^[mod-two.md:42-49]

## 自由坐标构造规则

设 $r_p$ 是主元位于 $p$ 的基行，$e_i$ 是第 $i$ 个坐标的单位向量。对于每个自由坐标 $f$，`right_kernel` 构造 $v_f=e_f+\sum_{p:\,r_p[f]=1}e_p$：在自由坐标 $f$ 处置位，并在所有满足 $r_p[f]=1$ 的主元位置置位；运算在 $\mathbb{F}_2$ 上进行。^[mod-two.md:44-48]

每个生成的 $v_f$ 随后插入新的 `ModTwoSubspace`。插入会继续维护典范 RREF，因此最终返回的核基经过重新约化，不能将按自由坐标生成的原始向量直接视为返回的典范基行。^[mod-two.md:32-35, mod-two.md:44-49]

## 配对与基行读取

底层 [[F₂ 上的位打包向量（ModTwoVector）|ModTwoVector]] 使用动态位打包表示有限向量空间元素。其 `dot` 方法实现 $\mathbb{F}_2$ 配对：逐字计算 `(l & r).count_ones() & 1`，再用 XOR 合并奇偶性；维数不匹配时返回 `RankMismatch`。^[mod-two.md:17-28]

源材料区分了排序基行的读取与右核的重新约化：`real_weyl.rs` 的 R-group 核构造从 `pivot_rows()` 提供的基行读取生成元位，而 `right_kernel` 会将生成向量重新约化为新子空间。相关背景见 [[fiber grading 与 R-群核生成元]]。^[mod-two.md:45-49]

## 测试与证据边界

源文件的 12 个测试包含右核计算，并覆盖插入后的 RREF 保持、跨多字依赖消元和 130 维动态子空间。来源未列出右核测试的具体输入与断言，并明确指出 `dot` 没有文件内直接测试。^[mod-two.md:83-93]

本页依据结构性源码阅读。来源包未执行构建、测试或原版运行，因此上述测试信息仅描述源码中的测试锚点，不构成测试通过、数学验收、性能或并行结论。^[mod-two.md:9-13, mod-two.md:103-108]

## Sources

- [mod-two.md](../../sources/mod-two.md) — mod-2 线性代数：位打包向量与子空间。
