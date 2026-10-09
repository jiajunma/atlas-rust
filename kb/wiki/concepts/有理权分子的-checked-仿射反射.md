---
title: 有理权分子的 checked 仿射反射
summary: simple_reflect_numerator 在分母不变时以全程 checked 算术执行 v -= alpha_s * (<v, coroot_s> + offset)，支持普通简单反射及带偏移的仿射反射。
sources:
  - block-access-modifier.md
kind: concept
createdAt: "2026-10-09T14:40:35.633Z"
updatedAt: "2026-10-09T14:40:35.633Z"
tags:
  - 有理权
  - 仿射反射
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
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 有理权分子的 checked 仿射反射

有理权分子的 checked 仿射反射由私有函数 `simple_reflect_numerator` 实现，是块修正子中 Weyl 姿态变换的底层运算。它更新有理权的分子，保持分母不变，并在整个计算过程中使用 checked 算术。^[block-access-modifier.md:83-85]

## 运算公式

设有理权的分子为 \(v\)，简单根为 \(\alpha_s\)，对应简单余根为 \(\operatorname{coroot}_s\)，则更新规则为
\[
v \leftarrow v-\alpha_s\bigl(\langle v,\operatorname{coroot}_s\rangle+\mathrm{offset}\bigr).
\]
其中 `offset` 是加在配对值上的偏移量；分母不参与更新。源码包将这一实现对应到上游 `rootdata.h:610-611` 及 `617-618` 的 offset 变体。^[block-access-modifier.md:83-85]

## 在 Weyl 姿态变换中的作用

`RepContext::transform_srm<LEFT_TO_RIGHT>` 沿 Weyl 词逐字母操作，并依据 `kgb_status(x, s)` 选择分支。Complex 分支对 \(x\) 执行 cross，同时以 `offset = 0` 对分子作简单反射；Real 分支保持 \(x\) 不变，以分母作为 `offset`，实现以 \(-\rho_R\) 为中心的仿射反射。Imaginary* 分支返回 `RepInvariantViolation`。全部字母处理完毕后，在最终 \(x\) 处调用 `real_unique` 归一化。^[block-access-modifier.md:66-70]

这一运算属于 [[BlockModifier 块修正子]] 的姿态算术。实现使用 `WeylElement::reduced_word` 给出的典范最左下降约化词；两个方向使用同一典范词，使 `transform<false>` 成为 `transform<true>` 的逐字母逆。[[块修正子的相对化与标准参数恢复]] 中的 `make_relative_to` 与 `sr_with_modifier` 依赖这一逆向关系。^[block-access-modifier.md:58-62, block-access-modifier.md:76-81]

## 验证与证据边界

源码包记录了两个集成测试：恒等修正子下的标准参数恢复一致性，以及 SL(3,R) 锚点对的 `make_relative_to` 往返。后者涉及词 `[1]` 和 shift `[0,1]/4`，要求 shift 与 transform 往返精确回到查询参数，并由完整的 `sr_with_modifier` 恢复原参数。`transform_srm` 的分支覆盖通过集成测试间接进行。^[block-access-modifier.md:89-95, block-access-modifier.md:104-105]

这些记录属于结构性阅读证据，不构成数学或正确性验收；整个 block_modifier 切片在该来源记录时尚未接线、没有现存消费方。来源未核对所引上游文件的字节，本次知识维护也未执行测试或 benchmark。^[block-access-modifier.md:9-13, block-access-modifier.md:99-99, block-access-modifier.md:114-115]

## Sources

- [只读块拓扑与块修正子（block_access.rs / block_modifier.rs）](block-access-modifier.md)
