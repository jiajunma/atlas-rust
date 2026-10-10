---
title: 扭曲对合的 Cayley/Cross 分解
summary: 分解先重放 cross 共轭，再逐个检查 Cayley 根为虚根并左乘反射以恢复输入；分解跨实现不唯一，差分应比较重放不变量或标签。
sources:
  - cayley-cross.md
kind: concept
createdAt: "2026-10-09T14:43:41.739Z"
updatedAt: "2026-10-10T02:22:37.906Z"
tags:
  - 扭曲对合
  - Cayley变换
  - 差分验证
aliases:
  - 扭曲对合的-cayleycross-分解
  - 扭C分
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 扭曲对合的 Cayley/Cross 分解

`CayleyCrossDecomposition` 将[[扭曲对合（TwistedInvolution）]]表示为一段 cross 作用与一组 Cayley 根反射。从 distinguished involution 出发，先按序对 cross word 执行 twisted 共轭，再依次左乘 Cayley 根的反射，可以恢复输入。每个 Cayley 根在其重放步骤上必须为 imaginary；构造时会验证这些不变量。^[cayley-cross.md:17-24]

## 表示与比较契约

分解保存四部分：`cayley_roots` 为 `RootId` 列表，`cross_word` 为**生成元下标词，而非 RootId 列表**，`cross_action` 为重放复合得到的 `WeylAction`，`twisted` 为输入的克隆。^[cayley-cross.md:17-19]

Cayley 根与 cross word 在不同移植实现之间并不唯一：Atlas 按其内部 transducer 顺序进行剥离。因此，跨实现比较必须使用重放不变量或标签级结果，不能要求原始分解部分一致。^[cayley-cross.md:22-24]

## 构造流程

### 输入校验

`build` 首先检查 `twisted.weyl_action().datum()` 是否匹配，不匹配则返回 `DatumMismatch`。随后比较 weight 与 coweight 两侧矩阵，要求存储对合恰为 \(w\circ\delta\)，否则返回 `DistinguishedInvolutionMismatch`。来源指出，此条件迫使 \(w^{-1}=\delta w\delta\)，为终止性论证提供前提。^[cayley-cross.md:28-31]

构造器逐个取得简单根的 `simple_ids`，再建立生成元置换 `twist[g]`，记录 \(\delta\) 将第 \(g\) 个简单根映到哪个生成元下标。若无法找到对应下标，则返回 `InvalidBasedAutomorphism`。^[cayley-cross.md:32-34]

### 下降剥离与预算

[[Cayley/Cross 的下降剥离算法]]按生成元下标升序选择首个 descent，判据是 \(\delta(\theta(\alpha_g))\) 的简单坐标全部不大于零。若没有 descent，而当前作用仍非单位，则触发 `"peeling termination"` 不变量错误。^[cayley-cross.md:35-37]

每步操作由根类型决定：Real 根产生 Cayley 字母，并将当前作用更新为 \(s_g\circ\mathrm{current}\)；Complex 根产生 Cross 字母，并更新为 \(s_g\circ\mathrm{current}\circ s_{\mathrm{twist}[g]}\)。Imaginary 根触发 `"descent kind"` 错误；代码注释声明 imaginary 根不可能成为 descent。^[cayley-cross.md:40-43]

预算检查发生在找到 descent 之后、执行步骤之前。当 `steps == max_peeling_steps` 时，返回资源名为 `"peeling steps"` 的 `CayleyCrossResourceLimit`。因此，无需剥离的输入可在零预算下通过；Complex 步骤虽然复合两次反射，仍只计一个预算步骤。^[cayley-cross.md:37-43]

### 逆序收集与根集规范化

剥离结束后，算法逆序重放记录的字母。Cayley 字母将 `simple_ids[g]` 追加到 Cayley 根集；Cross 字母追加到 `cross_word`，同时反射所有已收集的 Cayley 根。^[cayley-cross.md:44-45]

随后进行[[Cayley 根的长根化与强正交规范化]]。`ensure_pairwise_orthogonal` 要求每对根在两个方向上的 bracket 均为零；`long_orthogonalize` 将和为根的正交短根对，即 B2 对，替换为其和与差这两个长根。差必须也是根，否则触发 `"B2 pair"` 错误。来源以每次替换严格增加长根数说明终止性。最后逐根通过 `positive_form` 取正并升序排序；访问器文档保证输出根集强正交、全为正根且升序排列。^[cayley-cross.md:46-50]

### 重放验证

最终验证从 identity 出发，按序施加 cross 字母，再遍历排序后的 Cayley 根。每步先检查该根在当前重放状态下为 imaginary，否则触发 `"Cayley root imaginary"`，随后左乘其根反射。终态必须等于输入，否则触发 `"replay equality"`。^[cayley-cross.md:51-53]

## 测试与证据边界

来源列出的测试锚点包括：两种 distinguished involution 下的 identity 空分解；A1×A1 中 identity 配合 \(s_0\) 得到一个 Cayley 根，swap 配合 \(s_0\circ s_1\) 得到一个 cross 字母；A2 最长元 \(s_0s_1s_0\) 得到坐标为 \([1,1]\) 的 Cayley 根及 `cross_word [0]`。B2 的短根、长根 pinning 两例分别覆盖长根化生效与无需变换的路径。^[cayley-cross.md:55-58]

A2、B2 的 twisted involution 全枚举测试检查分解成功、重放相等及 Cayley 根两两正交；三条负路径精确匹配零预算、异 datum 与异 backing 的错误。不过，六种 `CayleyCrossInvariantViolation` 不变量串均无专门负测试，A1×A1 swap 案例也仅固定 `cross_word.len() == 1`，未固定字母内容。^[cayley-cross.md:58-59, cayley-cross.md:97-99]

本页依据结构性源码阅读，不构成数学或正确性验收。剥离每步长度下降一或二、长根计数严格递增带来的终止性、imaginary 根不可能为 descent，以及跨实现分解不唯一，均是来源对文档或注释声明的转述，未在此次维护中独立验证。此次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[cayley-cross.md:95-96, cayley-cross.md:107-111]

## Sources

- [cayley-cross.md](../../sources/cayley-cross.md) — Cayley/Cross 分解与整对合分类（cayley_cross.rs / involution_classification.rs）。
