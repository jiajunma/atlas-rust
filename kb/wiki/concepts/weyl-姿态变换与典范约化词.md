---
title: Weyl 姿态变换与典范约化词
summary: transform_srm 双向使用同一典范最左下降约化词，按复根、实根或虚根执行交叉反射、仿射反射或报错，最后在目标位置归一化。
sources:
  - block-access-modifier.md
kind: concept
createdAt: "2026-10-09T14:40:43.235Z"
updatedAt: "2026-10-09T22:24:27.981Z"
tags:
  - Weyl群
  - 表示参数
  - 规范化
aliases:
  - weyl-姿态变换与典范约化词
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Weyl 姿态变换与典范约化词
summary: transform_srm 使用典范最左下降约化词，按 KGB 根类型逐字母更新表示参数；双向共用同一词，支撑块修正子的相对化与标准参数恢复。
sources:
  - block-access-modifier.md
kind: concept
tags:
  - Weyl群
  - 表示参数
  - 姿态变换
aliases:
  - weyl-姿态变换与典范约化词
---

# Weyl 姿态变换与典范约化词

Weyl 姿态变换由 `RepContext::transform_srm` 实现，沿 Weyl 元素的词逐字母更新表示参数，并根据当前 KGB 元素的根类型选择作用。它属于 [[BlockModifier 块修正子]] 切片，用于参数相对化与标准参数恢复；来源记录时，该切片尚未接线，无现存消费方。^[block-access-modifier.md:44-50, block-access-modifier.md:66-81, block-access-modifier.md:99-99]

## 典范词与作用方向

Rust 使用 `WeylElement::reduced_word` 给出的典范最左下降约化词，相关概念见 [[基于左下降剥离的规范约化词]]。上游 `transform` 使用 `Weyl_group().word(w)`，即 transducer 随元素存储的词。这是一处有意偏差，实现文档声明其在目标域内无语义差异。^[block-access-modifier.md:58-62]

两个方向共用同一典范词，使 `transform<false>` 成为 `transform<true>` 的逐字母逆；`make_relative_to` 与 `sr` 依赖这一性质。`transform_srm<LEFT_TO_RIGHT>(w, srm)` 通过 `LEFT_TO_RIGHT` 选择字母的施加方向。^[block-access-modifier.md:58-70]

## 按根类型逐字母变换

每一步查询 `kgb_status(x, s)`。在 Complex 状态下，对 KGB 元素 `x` 施加 cross，并对有理权分子执行 offset 为零的简单反射。在 Real 状态下，`x` 保持不变，分子执行以 \(-\rho_R\) 为中心的仿射反射，offset 等于分母。^[block-access-modifier.md:66-69]

遇到 Imaginary* 状态时，变换返回 `RepInvariantViolation`；上游对应异常为 `Bad Weyl group element SRM transform`。处理完所有字母后，在最终 `x` 处执行 `real_unique` 归一化。^[block-access-modifier.md:66-70]

私有函数 `simple_reflect_numerator` 对有理权分子 \(v\) 执行 \(v\leftarrow v-\alpha_s\bigl(\langle v,\operatorname{coroot}_s\rangle+\mathrm{offset}\bigr)\)，分母保持不变，全程使用 checked 算术。^[block-access-modifier.md:83-85]

## 相对化与标准参数恢复

`make_relative_to(loc, srm0, bm, srm1)` 先执行 locator 部分的逆合成，再使用**更新后的** `bm.w` 执行 `transform<true>`，把 `srm1` 移回基姿态，最后将 `bm.shift` 设为两个 `gamma_lambda` 的整正交差。这一步连接了 [[定位器的相对姿态变换]] 与权的平移修正。^[block-access-modifier.md:76-78]

整正交差由 `make_diff_integral_orthogonal` 构造：从两代表的差中减去其在 \((1-\theta)X^*\) 中的固定原像，使结果与 `srm.gamma_lambda()` 的整根系正交。该过程经过 `IntegralSubsystem::integral`、`RepTable::integral_codec` 与 `theta_1_preimage`；差为零时短路，debug 构建包含正交性断言。^[block-access-modifier.md:72-75]

恢复路径 `sr_with_modifier(srm, bm, gamma)` 依次加入 `bm.shift`、执行 `transform<false>(bm.w)`，最后调用 `to_standard`。`shift_srm` 则在更新 `gamma_lambda` 后，于不变的 involution 处归一化。相关流程见 [[块修正子的相对化与标准参数恢复]]。^[block-access-modifier.md:71-81]

## 测试锚点与证据边界

来源列出两个集成测试。恒等修正子测试使用 A2 的 SL(3,R) fixture，KGB 尺寸为 4，检查 `sr_with_modifier` 与无修正子的 `sr` 一致。相对化往返测试中，锚点对的定位器在同一典范数据上碰撞，`bm.w` 的词为 `[1]`，shift 为 `[0,1]/4`；shift 与 transform 的往返精确恢复查询参数，而非仅相差根平移，完整 `sr_with_modifier` 也恢复查询参数本身。^[block-access-modifier.md:89-95]

`transform_srm` 的分支覆盖通过集成测试间接进行，`shift_srm` 与 `make_diff_integral_orthogonal` 没有独立单元测试。这些测试锚点说明不构成数学或正确性验收。^[block-access-modifier.md:99-105]

本页依据结构性源码阅读；来源中的上游行号仅转录自代码注释，未核对上游字节。精确读取身份由来源所列快照绑定 Git base 与文件字节 SHA-256；本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[block-access-modifier.md:9-13, block-access-modifier.md:109-115]

## Sources

- [只读块拓扑与块修正子（block_access.rs / block_modifier.rs）](../../sources/block-access-modifier.md)
