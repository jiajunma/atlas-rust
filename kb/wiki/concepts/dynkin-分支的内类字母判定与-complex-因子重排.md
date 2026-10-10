---
title: Dynkin 分支的内类字母判定与 Complex 因子重排
summary: 按扭转对分支的作用判定 c、u、s 或 C，并将 Complex 配对因子相邻排列；跨多个异秩中间因子的移位顺序缺少夹具覆盖。
sources:
  - layout-restricted-roots.md
kind: concept
createdAt: "2026-10-09T14:57:57.240Z"
updatedAt: "2026-10-10T00:41:03.503Z"
tags:
  - 内类
  - Dynkin分类
aliases:
  - dynkin-分支的内类字母判定与-complex-因子重排
  - D分C因
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Dynkin 分支的内类字母判定与 Complex 因子重排
summary: 根据扭转对 Dynkin 分支的作用确定内类字母，并将 Complex 配对因子相邻排列、调整 Bourbaki 槽位；跨越多个异秩中间因子的移位顺序尚无夹具覆盖。
sources:
  - layout-restricted-roots.md
kind: concept
tags:
  - Dynkin分类
  - 内类
  - 编号约定
aliases:
  - dynkin-分支的内类字母判定与-complex-因子重排
---

# Dynkin 分支的内类字母判定与 Complex 因子重排

`inner_class_letters` 根据 distinguished 对合诱导的扭转对各 Dynkin 分支的作用，生成内类字母 `'c'`、`'u'`、`'s'` 或 `'C'`，并调整 Complex 配对因子的顺序与 Bourbaki 槽位。它属于 [[内类布局（InnerClassLayout）]] 的构造流程；一个 `'C'` 条目对应两个相邻的 Lie type 因子。^[layout-restricted-roots.md:17-35]

## 构造前提与置换约定

`InnerClassLayout::build` 首先调用 `twist_permutation`，逐根执行 `id_of`、`image`、回查与去重，确定 distinguished 对合将每个单根映到哪个单根；任何一步失败均返回 `LayoutInvariantViolation`。随后依次执行 `dynkin::classify`、`inner_class_letters`，最后处理中心环面分支。^[layout-restricted-roots.md:24-27]

布局中的 Bourbaki 置换满足：`perm[k]` 是规范化顺序下第 `k` 个单根在 datum 中的下标。因此，Complex 因子的重排同时涉及 Lie type 因子次序与单根槽位；相关编号约定见 [[Bourbaki 顶点排序与置换语义]]。^[layout-restricted-roots.md:17-21, layout-restricted-roots.md:31-35]

## 内类字母判定

算法逐个 Dynkin 分支判定：若扭转逐点固定该分支，取 `'c'`；若扭转将该分支映到自身、但并非逐点固定，则偶秩 D 型取 `'u'`，其他情形取 `'s'`；若扭转将分支映到另一个分支，则取 `'C'`，并寻找配对因子。^[layout-restricted-roots.md:29-35]

这些规则处理半单部分。中心环面字母由 [[中心环面的商对合分类]] 另行确定，并按 `'c'`、`'C'`、`'s'` 的顺序追加。布局构造中的预算只约束中心环面字母背后的 Smith 基计算，不用于半单数据。^[layout-restricted-roots.md:17-22, layout-restricted-roots.md:37-41]

## Complex 因子配对与重排

当前分支被判为 `'C'` 时，算法在后续分支中寻找包含 `twist[perm[offset]]` 的分支；若找不到，报告 `"non-matching Complex factor"`。找到后，通过旋转使两个配对因子相邻，并将第二个因子的 Bourbaki 槽位改写为扭转像。^[layout-restricted-roots.md:31-35]

实现保留了来源所述的上游移位顺序：**先上移 `type` 切片，再在已移位的切片上计算移位宽度**。代码注释指出，这一顺序仅在 Complex 对跨越多个介于其间且秩不同的因子时可观测；当前没有构造夹具覆盖该情形。^[layout-restricted-roots.md:33-35]

## 测试锚点与覆盖缺口

来源列出的半单部分测试锚点包括：紧 A1 得到 `'c'`；twisted A2 得到 `'s'`；A1.A1 交换得到 `'C'`，具有两个因子且 `perm == [0, 1]`；D4 叉尖交换得到 `'u'`。A1.B2.A1 的 Complex 对旋转重排另有锚点，结果为 `perm == [0, 3, 1, 2]`。^[layout-restricted-roots.md:43-46]

A1.B2.A1 夹具覆盖了一个具体的旋转重排，但未覆盖上述跨越多个异秩中间因子的移位顺序。来源还指出，两份被阅读文件的错误分支均无失败路径测试；中心环面字母仅测试到 `'s'`，环面 `'c'`、`'C'` 尚无测试锚点。^[layout-restricted-roots.md:79-82]

## 证据边界

本页依据来源对 `layout.rs` 的结构性阅读，不构成数学或正确性验收。`inner_class_letters` 对应的上游位置 `atlas-types.w:2868-2941` 仅转录自代码注释，来源未核对上游字节；本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。上述测试锚点描述不能视为本次测试运行结果。^[layout-restricted-roots.md:9-13, layout-restricted-roots.md:29-35, layout-restricted-roots.md:89-93]

## Sources

- [layout-restricted-roots.md](../../sources/layout-restricted-roots.md) — 内类布局与限制根系（layout.rs / restricted_roots.rs）。
