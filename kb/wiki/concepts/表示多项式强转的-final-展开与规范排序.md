---
title: 表示多项式强转的 final 展开与规范排序
summary: KType 与 Param 分别经 finals_for 和 expand_final 展开成 final 组分，合并同类项后按 K_type_pol 或 SR_poly 的规范项序排序。
sources:
  - atlas-core-domain-dispatch.md
kind: concept
createdAt: "2026-10-09T14:29:10.342Z"
updatedAt: "2026-10-09T20:32:57.201Z"
tags:
  - 表示参数
  - 类型转换
  - 多项式
aliases:
  - 表示多项式强转的-final-展开与规范排序
  - 表F展
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 表示多项式强转的 final 展开与规范排序
summary: KType→KTypePol 经 finals_for 展开并合并项后按 K_type_pol 项序排序；Param→ParamPol 经 expand_final 展开并合并项后按 SR_poly 项序排序。
sources:
  - atlas-core-domain-dispatch.md
kind: concept
tags:
  - 表示论
  - 类型转换
  - 规范化
---

# 表示多项式强转的 final 展开与规范排序

表示多项式强转由领域侧的 `coerce(tag, value, span)` 实现。`KpolK` 将 `KType` 转为 `KTypePol`，`PolP` 将 `Param` 转为 `ParamPol`；两条路径都先展开为 final 组分，再合并项，最后按目标多项式的规范项序排序。^[atlas-core-domain-dispatch.md:40-52]

## KType 到 KTypePol

`KpolK` 经 `finals_for`（`K_repr`）将 `KType` 展开为 final 组分，随后逐项调用 `merge_ktype_term` 合并，最后按 canonical `K_type_pol` 项序排序。源包标注的上游对应位置为 `atlas-types.w:5608-5617`；相关概念见 [[KType 表示参数与规范化构造]] 和 [[finals_for 带号重数展开]]。^[atlas-core-domain-dispatch.md:47-49]

## Param 到 ParamPol

`PolP` 经 `expand_final` 将 `Param` 展开为 final 标准组分，再合并并按 `SR_poly` 项序排序。展开逻辑对应 `repr.cpp:1299-1306`，强转对应 `atlas-types.w:7710-7717`；相关概念见 [[StandardRepr 标准表示参数]] 和 [[可约点与标准参数 final 化]]。^[atlas-core-domain-dispatch.md:50-52]

## 强转边界

上述流程属于强转的领域侧实现。语言侧的强转表注册位于 `coercions.rs`，重载解析位于 `typed.rs`，可结合 [[有序强制转换注册表]] 阅读。对于未知强转标签，`coerce` 返回运行时错误 `conversion '<tag>' is not implemented`。^[atlas-core-domain-dispatch.md:53-53, atlas-core-domain-dispatch.md:70-72]

## 证据范围

源包仅完成结构性阅读，不声称语言或数学验收；`merge_*`、`sort_*` 等适配器的内部实现不在其覆盖范围内。因此，本页说明展开、合并和排序的调用顺序及所用项序，不据此给出具体排序比较规则，也不将结构性阅读视为行为验收。^[atlas-core-domain-dispatch.md:9-13, atlas-core-domain-dispatch.md:63-69]

## Sources

- [领域派发与强转（domain_builtins.rs 中部）](atlas-core-domain-dispatch.md)
