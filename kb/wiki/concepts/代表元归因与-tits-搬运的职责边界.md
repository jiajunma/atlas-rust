---
title: 代表元归因与 Tits 搬运的职责边界
summary: 归因 helper 仅接受分类已存储的代表元；一般 twisted involution 的归约需借助 table-backed Tits cross actions 同步搬运 torus factor，后续 minimal_torus_part 下降仍依赖分离的 inverse-Cayley 操作。
sources:
  - weak-real-form.md
kind: concept
createdAt: "2026-10-09T15:15:58.723Z"
updatedAt: "2026-10-09T15:15:58.723Z"
tags:
  - Tits作用
  - Cartan分类
  - 系统设计
aliases:
  - 代表元归因与-tits-搬运的职责边界
  - 代T搬
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 代表元归因与 Tits 搬运的职责边界

`weak_real_form_at_representative` 是弱实形式的代表元级归因内核：它要求输入的 `twisted` 恰好是 `CartanClassification` 已存储的某个代表元，再由环面因子确定局部 grading，并通过该 Cartan 的标签取得全局弱实形式编号。将一般 twisted involution 移到代表元所需的 Tits 搬运属于另一项职责。^[weak-real-form.md:58-72, weak-real-form.md:80-82]

## 代表元归因的计算路径

归因首先对环面因子施加 dual fixed-point projection：
\[
v\longmapsto \frac{v+v\theta}{2},
\]
其中采用 Atlas 的行向量、右乘约定。投影值的整数配对使平方中心化；对虚单根，偶数配对标记其为 noncompact，从而确定 grading。^[weak-real-form.md:58-63]

整性检查覆盖**每个单根**，并且先于虚根 grading 的提取；非整值配对报 `InvalidStrongTorusFactor`。因此，即使实 Cartan 的虚根基为空，也不能绕过这一检查。随后依次执行 `element_from_grading`、局部 `class_of` 和 `labels().label(local)`，完成[[弱实形式的局部到全局标签映射]]。^[weak-real-form.md:74-82]

局部分类来自 adjoint Cartan fiber 的 $W_{im}$ 轨道划分；每条轨道对应该 Cartan involution 处的一个弱实形式。划分本身保存类表和确定性代表元，而实形标签保存在 `RealFormLabels` 中，详见[[弱实形式的伴随 Cartan 纤维轨道划分]]。^[weak-real-form.md:20-23]

## Tits 搬运负责什么

对于一般 twisted involution，移到分类代表元时，必须通过 table-backed Tits cross actions **一并搬运 torus factor**。`weak_real_form_at_representative` 不会隐式构建或扩展所需表，其调用前提仍是输入已处于存储的代表元位置。后续 `minimal_torus_part` 的下降还依赖目前分离的 inverse-Cayley 操作，可参见[[最小环面部分的 grading 轨道搜索]]。^[weak-real-form.md:65-72]

## 来源一致性与验证顺序

由于 `CartanClassification` 不保留 `InnerClass` 句柄，归因函数从分类的第一个代表元重构归一化 distinguished involution，作为显式 provenance 门；不匹配时报 `DatumMismatch`。它还检查 `twisted` 的 datum 和 distinguished-involution 分解，相关分解不匹配报 `DistinguishedInvolutionMismatch`。这些检查构成[[弱实形式归因的来源与整性门控]]的一部分。^[weak-real-form.md:65-71]

验证依次经过原始因子长度检查（长度须等于格秩，否则报 `RankMismatch`）、provenance 门、`twisted` 的 datum 门、$w\cdot\delta=\theta$ 分解门、代表元查找、投影、整性检查，最后才提取 grading 并查询标签。分解门通过 `compose_matrices` 检查 weight 与 coweight 双矩阵；代表元查找位于投影之前。^[weak-real-form.md:74-82]

## 证据范围

来源记录了合成 A1 归因、秩与半整投影拒绝、外来同秩分类优先报 `DatumMismatch`，以及 A2 半整投影等测试锚点。但该来源包仅进行了结构性阅读，未执行构建、测试或原版运行；其描述不构成一般 Tits 搬运流程的运行验证，也不扩展既有 HPC 证据链。上游对应关系来自源码注释中的引用，未独立重读上游。^[weak-real-form.md:9-16, weak-real-form.md:84-91, weak-real-form.md:101-106]

## Sources

- [weak-real-form.md](weak-real-form.md) — 弱实形式划分：adjoint fiber 的 $W_{im}$ 轨道。
