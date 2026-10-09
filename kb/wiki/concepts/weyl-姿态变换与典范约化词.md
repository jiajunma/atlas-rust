---
title: Weyl 姿态变换与典范约化词
summary: transform_srm 使用典范最左下降约化词，按 Complex、Real 或 Imaginary 状态执行交叉与反射或报错，最后归一化；双向共用同一词以形成逐字母逆变换。
sources:
  - block-access-modifier.md
kind: concept
createdAt: "2026-10-09T14:40:43.235Z"
updatedAt: "2026-10-09T14:40:43.235Z"
tags:
  - Weyl群
  - 表示论
  - 姿态变换
aliases:
  - weyl-姿态变换与典范约化词
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Weyl 姿态变换与典范约化词

Weyl 姿态变换由 `RepContext::transform_srm` 实现：它沿 Weyl 元素的词逐字母变换表示参数，并根据当前 KGB 元素的根类型更新参数。这一机制属于 [[BlockModifier 块修正子]] 切片，用于参数的相对化与标准参数恢复；来源记录时，该切片尚未接线到现存消费方。^[block-access-modifier.md:44-50, block-access-modifier.md:66-81, block-access-modifier.md:99-99]

## 典范词与双向变换

Rust 实现使用 `WeylElement::reduced_word` 给出的典范最左下降约化词，参见 [[基于左下降剥离的规范约化词]]。这与上游 `transform` 使用 `Weyl_group().word(w)`、即 transducer 随元素存储的词存在有意差异。实现文档声明该差异在目标域内无语义影响：两个方向采用同一典范词，使 `transform<false>` 成为 `transform<true>` 的逐字母逆；`make_relative_to` 与 `sr` 依赖这一性质。^[block-access-modifier.md:58-62]

## 逐字母作用规则

`transform_srm<LEFT_TO_RIGHT>(w, srm)` 由 `LEFT_TO_RIGHT` 选择施加方向，并在每一步查询 `kgb_status(x, s)`。对于 Complex 状态，它对 `x` 施加 cross，同时对有理权分子施加 offset 为零的简单反射；对于 Real 状态，`x` 保持不变，分子接受以 `−ρ_R` 为中心的仿射反射，offset 等于分母。遇到 Imaginary* 状态时返回 `RepInvariantViolation`；上游对应错误为 `Bad Weyl group element SRM transform`。全部字母处理后，在最终的 `x` 处执行 `real_unique` 归一化。^[block-access-modifier.md:66-70]

私有函数 `simple_reflect_numerator` 对有理权分子 \(v\) 执行
\[
v \leftarrow v-\alpha_s\bigl(\langle v,\operatorname{coroot}_s\rangle+\mathrm{offset}\bigr).
\]
分母保持不变，算术全程使用 checked 运算。^[block-access-modifier.md:83-85]

## 相对化与标准参数恢复

`make_relative_to(loc, srm0, bm, srm1)` 先进行 locator 部分的逆合成，再使用**更新后的** `bm.w` 执行 `transform<true>`，将 `srm1` 移回基姿态，最后把 `bm.shift` 设为两个 `gamma_lambda` 的整正交差。这将 [[定位器的相对姿态变换]] 与权的平移修正连接起来。^[block-access-modifier.md:76-78]

整正交差由 `make_diff_integral_orthogonal` 构造：从两代表的差中减去其在 \((1-\theta)X^*\) 中的固定原像，使结果与 `srm.gamma_lambda()` 的整根系正交。该路径经过 `IntegralSubsystem::integral`、`RepTable::integral_codec` 与 `theta_1_preimage`；差为零时短路，debug 构建另有正交性断言。^[block-access-modifier.md:72-75]

读取路径 `sr_with_modifier(srm, bm, gamma)` 按顺序加入 `bm.shift`、执行 `transform<false>(bm.w)`，最后调用 `to_standard`。其中 `shift_srm` 更新 `gamma_lambda` 后，在不变的 involution 处归一化。双向词变换与这一平移步骤共同支撑 [[块修正子的相对化与标准参数恢复]]。^[block-access-modifier.md:71-81]

## 测试证据与适用边界

来源列出两个集成测试。恒等修正子测试使用 A2 的 SL(3,R) fixture，KGB 尺寸为 4，检查 `sr_with_modifier` 与无修正子的 `sr` 一致。相对化往返测试中，锚点对的定位器在同一典范数据上碰撞，`bm.w` 的词为 `[1]`，shift 为 `[0,1]/4`；shift 与 transform 的往返精确恢复查询参数，而非仅相差根平移，完整 `sr_with_modifier` 也恢复查询参数本身。^[block-access-modifier.md:89-95]

这些记录属于结构性阅读与测试锚点说明，不构成数学或正确性验收。`transform_srm` 的分支仅经集成测试间接覆盖，`shift_srm` 与 `make_diff_integral_orthogonal` 没有独立单元测试；来源中的上游行号转录自代码注释，未核对上游字节，本次知识维护也未执行测试或 benchmark。^[block-access-modifier.md:9-13, block-access-modifier.md:99-105, block-access-modifier.md:109-115]

## Sources

- [只读块拓扑与块修正子（block_access.rs / block_modifier.rs）](block-access-modifier.md)
