---
title: 环面对合的 dualPi0 子商构造
summary: dual_pi0 将 ker_F₂(θ+1) 对饱和 +1 特征格的模二像取商，为对偶分量群限制映射提供源与目标空间。
sources:
  - topology-form-name.md
kind: concept
createdAt: "2026-10-09T15:13:48.209Z"
updatedAt: "2026-10-09T21:11:54.471Z"
tags:
  - 环面对合
  - 模二线性代数
  - 子商
aliases:
  - 环面对合的-dualpi0-子商构造
  - 环D子
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# 环面对合的 dualPi0 子商构造

`dualPi0` 将环面对合 \(\theta\) 的格数据表示为一个 \(\mathbf F_2\) 子商。`topology.rs` 中的私有函数 `dual_pi0` 构造这一子商，供对偶分量群的平凡性与秩计算使用。^[topology-form-name.md:16-38]

## 子商定义

设 \(L\) 为对合 \(\theta\) 作用的整数格，\(L^\theta\) 为其饱和 \(+1\) 特征格，\(\bar\theta\) 为模二诱导作用。子商定义为
\[
\operatorname{dualPi0}(\theta)
=
\frac{\ker_{\mathbf F_2}(\bar\theta+I)}
{\operatorname{im}(L^\theta\longrightarrow L/2L)}.
\]
分子是在模二空间中计算的核；分母则先取整数格上的饱和 \(+1\) 特征格，再取其模二像。两者的计算层次需要区分。^[topology-form-name.md:27-28]

## 限制映射与对偶分量群

计算取矩阵 \(B\)，其列依次为简单余根和余权格 radical 的一组基，并将对合转运到单连通覆盖权格上：
\[
i_{\mathrm{sw}}=B^t\theta B^{-t}.
\]
随后分别构造 \(\operatorname{dualPi0}(\theta)\) 与 \(\operatorname{dualPi0}(i_{\mathrm{sw}})\)，作为限制映射的源与目标。相关换基过程见[[余根限制映射与对合的格基转运]]。^[topology-form-name.md:18-23, topology-form-name.md:31-36]

令 \(B_z\) 为将 \(B\) 的 radical 列清零后得到的矩阵。私有类型 `CorootRestriction` 实现 `ModTwoAmbientMap`，在环境模二空间中计算
\[
x\longmapsto B_z^t x\pmod 2.
\]
实现通过 `validate_induced_map_to` 检查该映射能否下降到两侧子商；相关接口契约见[[线性映射下降到子商的条件验证]]。^[topology-form-name.md:29-36]

对偶分量群是诱导映射
\[
\operatorname{dualPi0}(\theta)
\longrightarrow
\operatorname{dualPi0}(i_{\mathrm{sw}})
\]
的核。实现逐个映射源子商的基代表并计算像秩：`dual_component_group_trivial` 判断像秩是否等于源维数，`dual_component_group_rank` 返回源维数减去像秩。使用实形式最分裂 Cartan 的数据时，核为零等价于实形式连通，参见[[对偶分量群与实形式连通性]]。^[topology-form-name.md:18-23, topology-form-name.md:35-38]

## 构造约束

外围管线先检查形状，再计算 radical 并组装 \(B\)；无根 datum 分支将 radical 取为全格。矩阵分量经 `i32::try_from` 转换，对合转运使用 `invert_rational` 和两次 `rational_product`，再逐项通过 `integral_entry` 检查整性。`topology.rs` 不校验 \(\theta\) 是否为对合，这一前提由调用方保证。^[topology-form-name.md:31-36, topology-form-name.md:85-87]

平凡性与秩两个函数复制了同一整段计算管线，仅最终返回值不同。源材料将这一实现结构标记为同步漂移风险。^[topology-form-name.md:37-40]

## 测试与证据边界

源材料列出五个拓扑测试锚点：单连通 A1 split、伴随 A1 split、单连通 A1 compact、伴随 B2 compact，以及 GL(2) 样式 split。单连通 A1 split 的 \(B=\begin{pmatrix}1\end{pmatrix}\)，限制映射为恒等；伴随 A1 split 的 \(B=\begin{pmatrix}2\end{pmatrix}\) 在模二后消失，对应 PGL(2,R) 的两个分量；单连通 A1 compact 在 \(\theta=+1\) 时分子为零。GL(2) 样式 split 的环面坐标被清零，得到秩一对偶分量群。^[topology-form-name.md:42-45]

`dual_component_group_rank` 没有直接测试，无根 datum 分支也未覆盖。以上内容来自结构性源码阅读，不构成数学验收；上游 `tori::dualPi0` 等引用转录自代码注释，未核对上游字节，本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[topology-form-name.md:9-14, topology-form-name.md:45-46, topology-form-name.md:91-95]

## Sources

- [topology-form-name.md](../../sources/topology-form-name.md) — 对偶分量群平凡性与实形命名（topology.rs / form_name.rs）
