---
title: Cayley 根的长根化与强正交规范化
summary: 逆序重放运输 Cayley 根，验证双向正交后将 B2 正交短根对替换为长根和差，再取正排序；强正交保证属于来源文档声明。
sources:
  - cayley-cross.md
kind: concept
createdAt: "2026-10-09T14:43:50.078Z"
updatedAt: "2026-10-10T00:29:25.261Z"
tags:
  - 根系
  - Cayley变换
  - 规范化
aliases:
  - cayley-根的长根化与强正交规范化
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Cayley 根的长根化与强正交规范化
summary: Cayley 根经逆序重放收集、双向正交检查及 B2 短根对长根化，再取正排序；强正交是访问器文档声明的输出保证。
sources:
  - cayley-cross.md
kind: concept
tags:
  - 根系
  - 强正交
  - 规范化
aliases:
  - cayley-根的长根化与强正交规范化
---

# Cayley 根的长根化与强正交规范化

Cayley 根的长根化是 `CayleyCrossDecomposition::build` 的后处理步骤：先检查已收集根的两两正交性，再将特定的正交短根对替换为长根，最后取正并排序。访问器文档声明，输出根列表具有**强正交、均为正根、升序排列**三项性质。^[cayley-cross.md:46-50]

## 根的收集与运输

在[[扭曲对合的 Cayley/Cross 分解]]中，[[Cayley/Cross 的下降剥离算法|下降剥离]]得到的字母按逆序重放。遇到 Cayley 字母时，将对应简单根 `simple_ids[g]` 加入集合；遇到 Cross 字母时，将生成元下标追加到 `cross_word`，并反射所有已收集的 Cayley 根。后处理因此作用于经过这些 Cross 反射运输的根；`cross_word` 存储生成元下标，而非 `RootId`。^[cayley-cross.md:17-19, cayley-cross.md:44-45]

## 正交检查与长根化

`ensure_pairwise_orthogonal` 要求任意两个根在两个方向上的 bracket 均为零。随后，`long_orthogonalize` 处理和仍为根的正交短根对，即来源所称的 B2 对。对于这样的根对 $\alpha,\beta$，算法以长根 $\alpha+\beta$ 与 $\alpha-\beta$ 替换它们；差也必须为根，否则触发 `"B2 pair"` 不变量错误。^[cayley-cross.md:46-48]

源码给出的终止性理由是：每次替换都严格增加长根数。替换完成后，算法逐根调用 `positive_form` 取正，再按升序排序。来源仅转述这一终止性论述，未作独立验证。^[cayley-cross.md:48-50, cayley-cross.md:95-96]

## 重放验证与规范化边界

后处理完成后，构造器从 identity 出发，按序施加 Cross 字母，再依次处理已经排序的 Cayley 根。每个根在其重放步骤上必须为 imaginary，随后左乘该根的反射；根类型不符时触发 `"Cayley root imaginary"`，终态不等于输入时触发 `"replay equality"`。这些检查验证后处理后的分解仍能重建输入。^[cayley-cross.md:51-53]

这一规范化不保证不同移植实现产生相同的 Cayley 根列表或 `cross_word`。来源明确指出，两部分在不同实现之间不唯一，Atlas 按内部 transducer 顺序进行剥离。因此，跨实现比较必须采用重放不变量或 label 级结果，不能直接比较原始分解部分。^[cayley-cross.md:17-24]

## 测试与证据边界

B2 的短根、长根 pinning 两个测试案例分别锚定长根化生效与无需替换的情形。A2、B2 的 twisted involution 全枚举测试检查分解成功、重放相等及 Cayley 根两两正交。来源列出的测试检查是两两正交；强正交则是访问器文档声明的输出保证。^[cayley-cross.md:46-50, cayley-cross.md:55-59]

来源属于结构性源码阅读，不构成数学或正确性验收。包括 `"B2 pair"` 在内的六种 `CayleyCrossInvariantViolation` 不变量错误均无专门负向测试；本次知识维护未执行 Atlas、Cargo、测试或 benchmark，因此测试锚点描述不代表本次执行结果。^[cayley-cross.md:95-99, cayley-cross.md:107-111]

## Sources

- [cayley-cross.md](../../sources/cayley-cross.md) — Cayley/Cross 分解与整对合分类（`cayley_cross.rs` / `involution_classification.rs`）。
