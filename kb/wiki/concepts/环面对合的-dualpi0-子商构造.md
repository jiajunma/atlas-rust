---
title: 环面对合的 dualPi0 子商构造
summary: dual_pi0 构造 ker_F₂(θ+1) 对饱和 +1 特征整数格的模二像的子商，对合前提由调用方保证。
sources:
  - topology-form-name.md
kind: concept
createdAt: "2026-10-09T15:13:48.209Z"
updatedAt: "2026-10-10T00:53:37.251Z"
tags:
  - 拓扑
  - 模二线性代数
  - 整数格
aliases:
  - 环面对合的-dualpi0-子商构造
  - 环D子
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 环面对合的 dualPi0 子商构造
summary: dual_pi0 构造模二空间中 θ+I 的核对饱和 +1 特征整数格的模二像的子商，用于对偶分量群的平凡性与秩计算。
sources:
  - topology-form-name.md
kind: concept
tags:
  - 环面对合
  - 模二线性代数
  - 子商
---

# 环面对合的 dualPi0 子商构造

`dualPi0` 将环面对合的格数据表示为一个 \(\mathbf F_2\) 子商。`topology.rs` 中的私有函数 `dual_pi0` 构造这一空间，供对偶分量群的限制映射、平凡性判定和秩计算使用。^[topology-form-name.md:27-38]

## 子商定义

设 \(L\) 为对合 \(\theta\) 作用的整数格，\(L^\theta\) 为其饱和 \(+1\) 特征格，\(\bar\theta\) 为模二诱导作用，则
\[
\operatorname{dualPi0}(\theta)
=
\frac{\ker_{\mathbf F_2}(\bar\theta+I)}
{\operatorname{im}(L^\theta\longrightarrow L/2L)}.
\]
分子是模二空间中的核，分母是饱和 \(+1\) 特征**整数格**的模二像；构造保留了整数格与模二空间这两个层次。^[topology-form-name.md:27-28]

## 限制映射与对偶分量群

令 \(B\) 的列依次为简单余根和余权格 radical 的一组基。对合转运到单连通覆盖权格上的矩阵为
\[
i_{\mathrm{sw}}=B^t\theta B^{-t}.
\]
实现分别构造 \(\operatorname{dualPi0}(\theta)\) 与 \(\operatorname{dualPi0}(i_{\mathrm{sw}})\)，作为限制映射的源与目标。换基过程参见 [[余根限制映射与对合的格基转运]]。^[topology-form-name.md:18-23, topology-form-name.md:31-36]

将 \(B\) 的 radical 列清零得到 \(B_z\)。私有类型 `CorootRestriction` 实现 `ModTwoAmbientMap`，在环境模二空间中计算
\[
x\longmapsto B_z^t x\pmod 2.
\]
计算采用逐输出列奇偶累加；随后通过 `validate_induced_map_to` 验证该映射能够下降到两侧子商。^[topology-form-name.md:29-36]

对偶分量群是诱导限制映射
\[
\operatorname{dualPi0}(\theta)
\longrightarrow
\operatorname{dualPi0}(i_{\mathrm{sw}})
\]
的核。实现逐个映射源子商的基代表并计算像秩：`dual_component_group_trivial` 判断像秩是否等于源维数，`dual_component_group_rank` 返回源维数减去像秩。对实形式的最分裂 Cartan 使用这一构造时，核消失等价于实形式连通，参见 [[对偶分量群与实形式连通性]]。^[topology-form-name.md:18-23, topology-form-name.md:31-38]

## 构造约束与实现风险

外围管线先检查形状，再计算 radical 并组装 \(B\)；无根 datum 分支将 radical 取为全格。矩阵分量经 `i32::try_from` 转换，转运计算使用 `invert_rational` 和两次 `rational_product`，并逐项通过 `integral_entry` 检查整性。`topology.rs` 不校验 \(\theta\) 是否为对合，这一前提由调用方保证。^[topology-form-name.md:31-36, topology-form-name.md:85-87]

平凡性与秩两个函数复制了整段计算管线，仅最后的返回值不同。来源将此标记为同步漂移风险，即维护时两处实现可能发生不一致。^[topology-form-name.md:37-40]

## 测试与证据边界

来源列出五个拓扑测试锚点。单连通 A1 split 中 \(B=\begin{pmatrix}1\end{pmatrix}\)，限制映射为恒等，对应连通情形；伴随 A1 split 中 \(B=\begin{pmatrix}2\end{pmatrix}\)，限制映射在模二后消失，对应 PGL(2,R) 的两个分量。单连通 A1 compact 在 \(\theta=+1\) 时分子为零；另有伴随 B2 compact，以及环面坐标被清零、对偶分量群秩为一的 GL(2) 样式 split。^[topology-form-name.md:42-45]

这些锚点属于外围拓扑计算的测试覆盖：`dual_component_group_rank` 没有直接测试，无根 datum 分支也未覆盖。^[topology-form-name.md:42-46]

本页依据结构性源码阅读，不构成数学验收。上游 `tori::dualPi0` 等引用仅转录自代码注释，来源未核对上游字节；本次知识维护未执行 Atlas、Cargo、测试或 benchmark，因此上述测试描述不代表本次执行结果。^[topology-form-name.md:9-14, topology-form-name.md:91-95]

## Sources

- [topology-form-name.md](../../sources/topology-form-name.md) — 对偶分量群平凡性与实形命名（topology.rs / form_name.rs）。
