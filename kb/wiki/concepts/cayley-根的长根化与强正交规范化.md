---
title: Cayley 根的长根化与强正交规范化
summary: 逆序重放中由 cross 字母反射已收集的 Cayley 根，再验证双向正交、将 B2 正交短根对替换为长根和差，并取正排序；强正交是访问器文档声明的输出保证。
sources:
  - cayley-cross.md
kind: concept
createdAt: "2026-10-09T14:43:50.078Z"
updatedAt: "2026-10-09T14:43:50.078Z"
tags:
  - 根系
  - 强正交
  - 规范化
aliases:
  - cayley-根的长根化与强正交规范化
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Cayley 根的长根化与强正交规范化

Cayley 根的长根化是 `CayleyCrossDecomposition::build` 的后处理步骤：对收集到的 Cayley 根先检查两两正交，再将特定正交短根对替换为长根，最后取正并排序。访问器文档声明，输出 Cayley 根具有强正交、均为正根、升序排列三项性质。^[cayley-cross.md:46-50]

## 输入与处理流程

在[[扭曲对合的 Cayley/Cross 分解]]中，下降剥离所得字母按逆序重放：遇到 Cayley 字母，将对应简单根 `simple_ids[g]` 加入集合；遇到 Cross 字母，则追加 `cross_word`，并反射所有已收集的 Cayley 根。因此，后处理面对的是经过 Cross 作用搬运的根集合。^[cayley-cross.md:44-45]

`ensure_pairwise_orthogonal` 首先检查任意两个根在两个方向上的 bracket 均为零。随后，`long_orthogonalize` 寻找和仍为根的正交短根对，即源码所称的 B2 对，并将其替换为长根的和与差。差也必须是根，否则触发 `"B2 pair"` 不变量错误。^[cayley-cross.md:46-48]

按源码说明，每次替换都严格增加集合中的长根数，从而保证长根化过程终止。替换完成后，逐根调用 `positive_form` 取正，再按升序排序，形成输出的规范化根列表。^[cayley-cross.md:48-50]

## 重放不变量与规范化边界

后处理结束后，构造器重新验证整个分解：从 identity 出发依次施加 Cross 字母，再依次处理已经排序的 Cayley 根。每个 Cayley 根在其重放步骤上必须为 imaginary，随后左乘该根的反射；若根类型检查失败，报 `"Cayley root imaginary"`，若终态不等于输入，则报 `"replay equality"`。^[cayley-cross.md:51-53]

这里的规范化不意味着不同实现会产生相同的 Cayley 根列表或 `cross_word`。源文档明确指出，两部分在不同 port 之间不唯一，Atlas 的剥离顺序依赖其内部 transducer 顺序；跨实现比较应采用重放不变量或 label 级结果，而不能直接比较这些原始分解部分。^[cayley-cross.md:17-24]

## 测试与证据边界

现有测试中，B2 的短根与长根 pinning 两个案例分别锚定长根化实际生效和无需修改的情形；A2、B2 的 twisted involution 全枚举测试检查分解成功、重放相等及 Cayley 根两两正交。这些检查的明确范围是两两正交，不能将其扩大表述为独立验证了全部强正交保证。^[cayley-cross.md:55-59]

该来源属于结构性源码阅读，不构成数学或正确性验收；长根计数严格递增所支持的终止性仍是对源码声明的转述。包括 `"B2 pair"` 在内的六种 `CayleyCrossInvariantViolation` 不变量错误均没有专门的负向测试，本次知识维护也未运行测试或 benchmark。^[cayley-cross.md:95-99, cayley-cross.md:107-111]

## Sources

- [cayley-cross.md](cayley-cross.md) — Cayley/Cross 分解与整对合分类（`cayley_cross.rs` / `involution_classification.rs`）。
