---
title: 有理权分子的 checked 仿射反射
summary: simple_reflect_numerator 在分母不变时以全程 checked 算术执行 v -= alpha_s * (<v, coroot_s> + offset)，支持普通与带偏移的简单反射。
sources:
  - block-access-modifier.md
kind: concept
createdAt: "2026-10-09T14:40:35.633Z"
updatedAt: "2026-10-09T20:47:58.422Z"
tags:
  - 有理权
  - 反射
  - 算术安全
aliases:
  - 有理权分子的-checked-仿射反射
  - 有C仿
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 有理权分子的 checked 仿射反射
summary: simple_reflect_numerator 以全程 checked 算术更新有理权分子，保持分母不变，并通过 offset 支持简单反射与仿射反射。
sources:
  - block-access-modifier.md
kind: concept
tags:
  - 有理权
  - 仿射反射
  - 算术安全
---

# 有理权分子的 checked 仿射反射

`simple_reflect_numerator` 是 [[BlockModifier 块修正子]] 姿态算术中的私有函数。它对有理权的分子执行带偏移的简单反射，保持分母不变，全程采用 checked 算术。^[block-access-modifier.md:44-50, block-access-modifier.md:83-85]

## 运算公式

设有理权分子为 \(v\)，简单根为 \(\alpha_s\)，对应简单余根为 \(\operatorname{coroot}_s\)。更新规则为
\[
v \leftarrow v-\alpha_s\bigl(\langle v,\operatorname{coroot}_s\rangle+\mathrm{offset}\bigr).
\]
`offset` 加在根余根配对值上，分母不参与更新。来源将该实现对应到上游 `rootdata.h:610-611` 与 `617-618` 的 offset 变体。^[block-access-modifier.md:83-85]

## 在 Weyl 姿态变换中的使用

`RepContext::transform_srm<LEFT_TO_RIGHT>` 沿 Weyl 词逐字母操作，以 `kgb_status(x, s)` 决定反射方式。Complex 分支对 \(x\) 执行 cross，并以 `offset = 0` 对分子执行简单反射；Real 分支保持 \(x\) 不变，以有理权的分母作为 `offset`，执行以 \(-\rho_R\) 为中心的仿射反射。Imaginary* 分支返回 `RepInvariantViolation`。`LEFT_TO_RIGHT` 选择施加方向，所有字母处理完毕后在最终 \(x\) 处执行 `real_unique` 归一化。^[block-access-modifier.md:66-70]

实现使用 `WeylElement::reduced_word` 给出的典范最左下降约化词，而上游使用 transducer 随元素存储的词。两个变换方向使用同一典范词，使 `transform<false>` 成为 `transform<true>` 的逐字母逆；来源将这一差异声明为对目标域无语义影响。^[block-access-modifier.md:58-62]

这一逆向关系用于 [[块修正子的相对化与标准参数恢复]]：`make_relative_to` 以更新后的 `bm.w` 执行 `transform<true>`，将参数移回基姿态，再确定整正交差；`sr_with_modifier` 则先加 `bm.shift`，再执行 `transform<false>`，最后调用 `to_standard` 恢复标准参数。^[block-access-modifier.md:76-81]

## 测试与证据边界

来源记录了两个集成测试：恒等修正子使 `sr_with_modifier` 与无修正子的 `sr` 一致；SL(3,R) 锚点对的 `make_relative_to` 往返使用词 `[1]` 与 shift `[0,1]/4`，要求 shift 与 transform 往返精确回到查询参数，并由完整的 `sr_with_modifier` 恢复原参数。`transform_srm` 的分支覆盖通过这些集成测试间接进行。^[block-access-modifier.md:89-95, block-access-modifier.md:104-105]

本页依据结构性源码阅读，不构成数学或正确性验收。来源记录时，整个 block_modifier 切片尚未接线、没有现存消费方；上游引用仅转录自代码注释，未核对上游字节，本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[block-access-modifier.md:9-13, block-access-modifier.md:99-99, block-access-modifier.md:114-115]

## Sources

- [只读块拓扑与块修正子（block_access.rs / block_modifier.rs）](block-access-modifier.md)
