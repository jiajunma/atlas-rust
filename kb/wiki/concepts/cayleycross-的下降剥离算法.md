---
title: Cayley/Cross 的下降剥离算法
summary: 先校验 datum 与 w∘δ 的双矩阵一致性，再按外部生成元顺序选择首个下降执行 Cayley 或 Cross，并在步进前检查剥离预算。
sources:
  - cayley-cross.md
kind: concept
createdAt: "2026-10-09T14:43:52.687Z"
updatedAt: "2026-10-09T22:26:36.996Z"
tags:
  - 算法
  - Weyl群
  - 资源预算
aliases:
  - cayleycross-的下降剥离算法
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Cayley/Cross 的下降剥离算法
summary: 验证 datum 与 w∘δ 的双矩阵一致性后，按外部生成元升序选择首个下降，依据根类型执行 Cayley 或 Cross 剥离，再逆序收集并重放验证；预算按剥离步数计量。
sources:
  - cayley-cross.md
kind: concept
tags:
  - 算法
  - Weyl群
  - 资源预算
aliases:
  - cayleycross-的下降剥离算法
provenanceState: extracted
---

# Cayley/Cross 的下降剥离算法

Cayley/Cross 的下降剥离算法用于构造[[扭曲对合的 Cayley/Cross 分解]]。它逐步剥离下降生成元，逆序处理记录的字母，再规范化 Cayley 根并验证重放结果。构造所验证的核心不变量是：从 distinguished involution 出发，先按序执行 cross word 的 twisted 共轭，再依次左乘 Cayley 根反射，能够复现输入；每个 Cayley 根在对应重放步骤中必须为 imaginary。^[cayley-cross.md:17-24, cayley-cross.md:35-53]

## 输入校验与生成元准备

构造首先检查来源一致性：`twisted.weyl_action().datum()` 不匹配时返回 `DatumMismatch`；存储的对合必须恰为 \(w\circ\delta\)，且 weight 与 coweight 两侧矩阵都要一致，否则返回 `DistinguishedInvolutionMismatch`。来源说明，这一门控也支撑终止性论证，因为它迫使 \(w^{-1}=\delta w\delta\)。^[cayley-cross.md:28-31]

随后建立简单根编号表 `simple_ids`，并计算生成元上的 twist 置换：`twist[g]` 表示 \(\delta\) 将第 \(g\) 个简单根映到的生成元下标。若找不到对应生成元，返回 `InvalidBasedAutomorphism`。这里必须区分生成元下标与 `RootId`：`cross_word` 存储前者，`cayley_roots` 存储后者。^[cayley-cross.md:17-19, cayley-cross.md:32-34]

## 下降选择与剥离规则

循环采用“外部编号最小的下降优先”：按生成元下标升序扫描，选择第一个使 \(\delta(\theta(\alpha_g))\) 的简单根坐标全部不大于零的生成元。若不存在下降，但当前作用仍非单位，则返回 `"peeling termination"` 不变量错误。^[cayley-cross.md:35-37]

找到下降并通过预算检查后，算法依据[[对合下的虚根、实根与复根分类|根类型]]记录字母、更新当前作用 `current`。Real 根记录 Cayley 字母，并更新为 \(s_g\circ\mathrm{current}\)；Complex 根记录 Cross 字母，并更新为 \(s_g\circ\mathrm{current}\circ s_{\mathrm{twist}[g]}\)。Imaginary 分支返回 `"descent kind"` 不变量错误；源码注释声明 imaginary 根不可能是下降。^[cayley-cross.md:37-43]

预算检查发生在找到下降之后、实际步进之前：当 `steps == max_peeling_steps` 时，返回 `CayleyCrossResourceLimit { resource: "peeling steps" }`。因此，无须剥离的输入可以在预算为零时通过；Complex 分支虽然复合两次反射，预算仍只计一步。^[cayley-cross.md:37-42]

## 逆序收集与根集合规范化

剥离完成后，算法逆序处理字母记录。遇到 Cayley 字母，将 `simple_ids[g]` 追加到 Cayley 根集合；遇到 Cross 字母，将生成元下标追加到 `cross_word`，并反射所有已收集的 Cayley 根。^[cayley-cross.md:44-45]

收集完成后，`ensure_pairwise_orthogonal` 检查根两两正交，要求两个方向的 bracket 都为零。随后进行[[Cayley 根的长根化与强正交规范化]]：对于和仍为根的正交短根对，即 B2 对，以其和、差两个长根替换；若差不是根，返回 `"B2 pair"` 不变量错误。源码以每次替换严格增加长根数作为终止依据。最后逐根取正并升序排序，访问器文档保证输出根集合强正交、为正根且有序。^[cayley-cross.md:46-50]

## 重放验证与输出契约

最终验证从 identity 出发，按序施加 cross 字母，再依次处理排序后的 Cayley 根。每次左乘根反射前，检查该根在当前重放步骤中为 imaginary，否则返回 `"Cayley root imaginary"`；终态若不等于输入，则返回 `"replay equality"`。^[cayley-cross.md:51-53]

`CayleyCrossDecomposition` 保存 `cayley_roots`、`cross_word`、重放复合得到的 `cross_action`，以及输入 `twisted` 的克隆。来源明确指出，Cayley 根与 cross word 在不同移植实现之间并不唯一，Atlas 按其内部 transducer 顺序剥离。因此，跨实现差分必须比较重放不变量或 label 级结果，不能直接比较这些原始分解部分。^[cayley-cross.md:17-24]

## 测试与证据边界

来源记录的测试锚点包括：两种 distinguished involution 下的空 identity 分解；A1×A1 的 Cayley 与 Cross 示例；A2 最长元 \(s_0s_1s_0\) 得到 Cayley 根 \([1,1]\) 与 `cross_word [0]`；B2 长根化生效与不作改变的两种情形；以及 A2、B2 twisted involution 全枚举中的分解、重放相等和根两两正交检查。预算为零、不同 datum、不同 backing 的三条负路径也有精确错误匹配。^[cayley-cross.md:55-59]

这些记录属于结构性源码阅读，不构成数学正确性验收。每步长度减少 1 或 2、长根计数递增保证终止、imaginary 根不能成为下降，以及跨实现分解不唯一等声明均仅作转述，未独立验证。六种 `CayleyCrossInvariantViolation` 不变量错误均无专门负测试；A1×A1 的 swap 用例只固定 `cross_word.len() == 1`，未固定字母内容。^[cayley-cross.md:95-99]

来源的精确读取身份由 `2026-10-06-cayley-cross.json` 快照记录，绑定 Git base、所读文件字节的 SHA-256 与草案调用记录；维护者对照源码逐条核对改写。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[cayley-cross.md:107-111]

## Sources

- [cayley-cross.md](../../sources/cayley-cross.md)：Cayley/Cross 分解与整对合分类（`cayley_cross.rs` / `involution_classification.rs`）。
