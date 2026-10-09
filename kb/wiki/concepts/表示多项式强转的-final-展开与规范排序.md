---
title: 表示多项式强转的 final 展开与规范排序
summary: KType→KTypePol 经 finals_for 展开并合并项后按 K_type_pol 项序排序；Param→ParamPol 经 expand_final 展开并合并项后按 SR_poly 项序排序。
sources:
  - atlas-core-domain-dispatch.md
kind: concept
createdAt: "2026-10-09T14:29:10.342Z"
updatedAt: "2026-10-09T14:29:10.342Z"
tags:
  - 表示论
  - 类型转换
  - 规范化
aliases:
  - 表示多项式强转的-final-展开与规范排序
  - 表F展
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 表示多项式强转的 final 展开与规范排序

表示多项式强转由领域侧的 `coerce(tag, value, span)` 实现。`KpolK` 将 `KType` 转为 `KTypePol`，`PolP` 将 `Param` 转为 `ParamPol`；两条路径都先展开为 final 组分，再合并组分并按目标多项式的规范项序排序。^[atlas-core-domain-dispatch.md:40-52]

## KType 到 KTypePol

`KpolK` 经 `finals_for`（`K_repr`）展开 `KType`，得到 final 组分；随后逐项调用 `merge_ktype_term` 合并，最后按 canonical `K_type_pol` 项序排序。该路径可结合 [[KType 表示参数与规范化构造]] 与 [[finals_for 带号重数展开]] 阅读。源包标注的上游对应位置为 `atlas-types.w:5608-5617`。^[atlas-core-domain-dispatch.md:47-49]

## Param 到 ParamPol

`PolP` 经 `expand_final` 将 `Param` 展开为 final 标准组分，再合并并按 `SR_poly` 项序排序。展开逻辑对应 `repr.cpp:1299-1306`，强转对应 `atlas-types.w:7710-7717`；相关概念包括 [[StandardRepr 标准表示参数]] 与 [[可约点与标准参数 final 化]]。^[atlas-core-domain-dispatch.md:50-52]

## 强转边界与证据范围

上述流程属于强转的领域侧实现；语言侧的强转表注册位于 `coercions.rs`，重载解析位于 `typed.rs`。对于未知强转标签，`coerce` 返回运行时错误 `conversion '<tag>' is not implemented`。^[atlas-core-domain-dispatch.md:53-53, atlas-core-domain-dispatch.md:70-72]

本源包仅完成结构性阅读，不声称语言或数学验收；`merge_*`、`sort_*` 等适配器的内部实现也不在覆盖范围内。因此，本页说明展开、合并和排序的调用顺序及所用项序，不据此给出具体排序比较规则或更广泛的行为验收结论。^[atlas-core-domain-dispatch.md:9-13, atlas-core-domain-dispatch.md:63-69]

## Sources

- [领域派发与强转（domain_builtins.rs 中部）](atlas-core-domain-dispatch.md)
