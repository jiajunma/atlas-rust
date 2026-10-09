---
title: 环面对合的 dualPi0 子商构造
summary: dual_pi0 将 ker_F2(θ+1) 对饱和 +1 特征格的模二像取商，构成计算对偶分量群所需的源与目标空间。
sources:
  - topology-form-name.md
kind: concept
createdAt: "2026-10-09T15:13:48.209Z"
updatedAt: "2026-10-09T15:13:48.209Z"
tags:
  - 环面
  - 模二线性代数
  - 格理论
aliases:
  - 环面对合的-dualpi0-子商构造
  - 环D子
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 环面对合的 dualPi0 子商构造

`dualPi0` 将环面对合 \(\theta\) 的相关格数据表示为一个 \(\mathbf F_2\) 子商。在 `topology.rs` 中，私有函数 `dual_pi0` 构造这一子商，供对偶分量群的平凡性与秩计算使用。^[topology-form-name.md:16-38]

## 子商定义

设 \(L\) 为 \(\theta\) 作用的整数格，\(L^\theta\) 为其饱和 \(+1\) 特征格，\(\bar\theta\) 为模二诱导作用。源材料给出的构造可写为
\[
\operatorname{dualPi0}(\theta)
=
\frac{\ker_{\mathbf F_2}(\bar\theta+I)}
{\operatorname{im}(L^\theta\longrightarrow L/2L)}.
\]
分子是在模二空间中求得的核；分母则来自整数格上的饱和 \(+1\) 特征格，再取其模二像。构造时需要保留这两个层次的区别。^[topology-form-name.md:27-28]

## 在对偶分量群计算中的作用

实形式的连通性判定使用其最分裂 Cartan 的对偶分量群。构造取格基矩阵 \(B\)，列依次为简单余根和余权格 radical 的一组基，并将对合转运为
\[
i_{\mathrm{sw}}=B^t\theta B^{-t}.
\]
随后分别构造 \(\operatorname{dualPi0}(\theta)\) 与 \(\operatorname{dualPi0}(i_{\mathrm{sw}})\)。这一过程关联[[余根限制映射与对合的格基转运]]。^[topology-form-name.md:18-23, topology-form-name.md:31-36]

令 \(B_z\) 为将 \(B\) 的 radical 列清零后所得矩阵。私有类型 `CorootRestriction` 实现 `ModTwoAmbientMap`，在环境模二空间中按
\[
x\longmapsto B_z^t x\pmod 2
\]
计算限制映射。实现调用 `validate_induced_map_to`，验证该映射能够下降到两侧子商；这一步对应[[Ambient 映射的子商下降验证]]。^[topology-form-name.md:29-36]

对偶分量群是诱导映射
\[
\operatorname{dualPi0}(\theta)\longrightarrow
\operatorname{dualPi0}(i_{\mathrm{sw}})
\]
的核。实现逐个映射源子商的基代表并计算像秩：`dual_component_group_trivial` 判断像秩是否等于源维数，`dual_component_group_rank` 返回源维数减去像秩。核消失等价于诱导映射单射，也对应实形式连通；参见[[对偶分量群与实形式连通性]]。^[topology-form-name.md:18-23, topology-form-name.md:35-38]

## 构造约束与证据边界

外围管线先检查形状，再构造 radical 与基矩阵；无根 datum 分支将 radical 取为全格。对合转运使用精确有理求逆与矩阵乘法，并逐项检查结果的整性。`topology.rs` 不检查 \(\theta\) 是否满足对合条件，这一前提由调用方保证。^[topology-form-name.md:31-36, topology-form-name.md:85-87]

源材料列出五个拓扑测试锚点：单连通 A1 split、伴随 A1 split、单连通 A1 compact、伴随 B2 compact，以及 GL(2) 样式 split。其中，单连通 A1 split 的限制映射为恒等；伴随 A1 split 的 \(B=\begin{pmatrix}2\end{pmatrix}\) 在模二后消失，对应 PGL(2,R) 的两个分量；单连通 A1 compact 在 \(\theta=+1\) 时分子为零。`dual_component_group_rank` 没有直接测试，无根 datum 分支也未覆盖。^[topology-form-name.md:42-46]

这些结论的证据等级是结构性源码阅读。源材料中的上游 `tori::dualPi0` 与对偶分量群引用来自代码注释，未核对上游字节；本次知识维护也未执行 Atlas、Cargo、测试或 benchmark，不能据此宣称数学验收完成。^[topology-form-name.md:9-14, topology-form-name.md:91-95]

## Sources

- [topology-form-name.md](topology-form-name.md) — 对偶分量群平凡性与实形命名（topology.rs / form_name.rs）
