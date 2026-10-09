---
title: Cayley/Cross 的下降剥离算法
summary: 验证 datum 与 w∘δ 的一致性后，按生成器升序选择首个下降，依据 Real 或 Complex 根类执行不同反射步骤，并在步进前检查剥离预算；终止性论证仅为源码声明。
sources:
  - cayley-cross.md
kind: concept
createdAt: "2026-10-09T14:43:52.687Z"
updatedAt: "2026-10-09T14:43:52.687Z"
tags:
  - 根系
  - 算法
  - 资源预算
aliases:
  - cayleycross-的下降剥离算法
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Cayley/Cross 的下降剥离算法

Cayley/Cross 的下降剥离算法用于分解 twisted involution：先逐步剥离下降生成元，再逆序重放所记录的字母，得到 cross word 与 Cayley 根集合。分解的构造不变量是：从 distinguished involution 出发，按序执行 cross word 的 twisted 共轭，再依次左乘 Cayley 根反射，可以复现输入；每个 Cayley 根在对应重放步骤中必须为 imaginary。^[cayley-cross.md:17-24, cayley-cross.md:35-53]

## 输入校验与生成元准备

构造首先校验来源一致性：输入 Weyl 作用的 datum 不匹配时返回 `DatumMismatch`；存储的对合必须恰为 \(w\circ\delta\)，且 weight 与 coweight 两侧矩阵都要一致，否则返回 `DistinguishedInvolutionMismatch`。这一校验也支撑终止性论证，因为它迫使 \(w^{-1}=\delta w\delta\)。^[cayley-cross.md:28-31]

随后建立简单根表 `simple_ids`，并计算生成元上的 twist 置换：`twist[g]` 表示 \(\delta\) 将第 \(g\) 个简单根映到的生成元下标。若找不到对应生成元，返回 `InvalidBasedAutomorphism`。这里的生成元下标与 `RootId` 是不同的编号，输出 `cross_word` 存储前者。^[cayley-cross.md:17-19, cayley-cross.md:32-34]

## 下降选择与剥离规则

剥离循环采用“外部编号最小的下降优先”：按生成元下标升序扫描，选择第一个使 \(\delta(\theta(\alpha_g))\) 的简单根坐标全部不大于零的生成元。若不存在下降，但当前作用仍非单位，则报告 `"peeling termination"` 不变量错误。^[cayley-cross.md:35-37]

找到下降后，算法根据根类型记录字母并更新当前作用 `current`。Real 根记录 Cayley 字母，更新为 \(s_g\circ\mathrm{current}\)；Complex 根记录 Cross 字母，更新为 \(s_g\circ\mathrm{current}\circ s_{\mathrm{twist}[g]}\)。若下降被分类为 Imaginary，则报告 `"descent kind"` 不变量错误；源码注释声明 imaginary 根不可能是下降。相关根类型见[[对合下的虚根、实根与复根分类]]。^[cayley-cross.md:40-43]

步数预算在找到下降之后、实际步进之前检查：当 `steps == max_peeling_steps` 时，返回 `CayleyCrossResourceLimit { resource: "peeling steps" }`。因此，无须剥离的输入可以在预算为零时通过；Complex 分支虽然复合两次反射，预算仍只计一步。^[cayley-cross.md:37-42]

## 逆序收集与根集合规范化

剥离结束后，算法逆序重放字母记录。遇到 Cayley 字母，将 `simple_ids[g]` 加入 Cayley 根集合；遇到 Cross 字母，将生成元下标加入 `cross_word`，并用该反射变换所有已收集的 Cayley 根。^[cayley-cross.md:44-45]

收集完成后先检查两两正交，要求两个方向的 bracket 都为零。随后进行[[Cayley 根的长根化与强正交规范化]]：对于和仍为根的正交短根对，以其和、差两个长根替换；若差不是根，则报告 `"B2 pair"`。每次替换严格增加长根数，这是源码给出的终止依据。最后逐根取正并升序排序，访问器文档保证输出根集合强正交、为正根且有序。^[cayley-cross.md:46-50]

## 重放验证与跨实现比较

构造最后从 identity 开始，按序施加 cross 字母，再按排序后的 Cayley 根依次重放。每次左乘根反射之前，都检查该根在当前重放步骤中为 imaginary，否则报告 `"Cayley root imaginary"`；终态若不等于输入，则报告 `"replay equality"`。^[cayley-cross.md:51-53]

最终的 `CayleyCrossDecomposition` 保存 Cayley 根列表、生成元下标组成的 `cross_word`、重放复合得到的 `cross_action`，以及输入 `twisted` 的克隆。不同移植实现的分解部分并不唯一：Atlas 按内部 transducer 顺序剥离，因此跨实现比较应使用重放不变量或 label 级结果，不能直接比较原始 Cayley 根集合与 cross word。^[cayley-cross.md:17-24]

## 测试与证据边界

来源记录的测试锚点包括：两种 distinguished involution 下的空 identity 分解；A1×A1 的 Cayley 与 Cross 示例；A2 最长元得到 Cayley 根 \([1,1]\) 与 `cross_word [0]`；B2 长根化的生效与不变情形；以及 A2、B2 twisted involution 全枚举中的重放相等和 Cayley 根两两正交检查。预算为零、不同 datum 和不同 backing 的负路径也有精确错误匹配。^[cayley-cross.md:55-59]

这些记录属于结构性阅读，不构成数学正确性验收。源码关于每步长度减少 1 或 2、长根化终止以及 imaginary 根不能成为下降的声明在来源中仅被转述；六种 `CayleyCrossInvariantViolation` 不变量错误均无专门负测试，A1×A1 的 swap 用例也只固定 cross word 长度而未固定内容。本次知识维护未执行测试或 benchmark。^[cayley-cross.md:95-99, cayley-cross.md:107-111]

## Sources

- [cayley-cross.md](cayley-cross.md)：`cayley_cross.rs` 与 `involution_classification.rs` 的结构性阅读来源包。
