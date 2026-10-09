---
title: Dynkin 分支的内类字母判定与 Complex 因子重排
summary: 依据扭转对分支的作用生成 c、u、s 或 C，并使 Complex 配对因子相邻、对齐 Bourbaki 槽位；跨越多个异秩因子的上游移位顺序尚无夹具覆盖。
sources:
  - layout-restricted-roots.md
kind: concept
createdAt: "2026-10-09T14:57:57.240Z"
updatedAt: "2026-10-09T14:57:57.240Z"
tags:
  - Dynkin图
  - 内类
  - 算法
aliases:
  - dynkin-分支的内类字母判定与-complex-因子重排
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Dynkin 分支的内类字母判定与 Complex 因子重排

`inner_class_letters` 根据 distinguished 对合在各 Dynkin 分支上的作用，确定内类字母，并在出现 Complex 因子对时调整因子顺序与 Bourbaki 置换。它属于 [[内类布局（InnerClassLayout）]] 的构造流程：布局记录按打印顺序排列的 Lie type 因子、内类字母，以及满足 `perm[k]` 为规范化序下第 `k` 个单根的 datum 下标的置换。一个 `'C'` 条目消耗两个 type 因子，且这两个因子相邻。^[layout-restricted-roots.md:17-22, layout-restricted-roots.md:29-35]

## 构造前提

`InnerClassLayout::build` 首先通过 `twist_permutation` 计算 distinguished 对合诱导的单根置换：逐根执行 `id_of`、`image`、回查与去重，任一步失败均返回 `LayoutInvariantViolation`。随后调用 `dynkin::classify`，再由 `inner_class_letters` 判定半单部分的内类字母，最后处理中心环面分支。^[layout-restricted-roots.md:24-27]

## 内类字母判定

判定逐个 Dynkin 分支进行。若扭转逐点固定该分支，字母为 `'c'`；若扭转保留该分支但并非逐点固定，则偶秩 D 型取 `'u'`，其他情形取 `'s'`；若扭转将该分支映到另一分支，则取 `'C'`，并进入 Complex 因子配对与重排流程。^[layout-restricted-roots.md:29-35]

这些规则用于半单 Dynkin 分支。中心环面字母另由商对合分类得到，按 `'c'`、`'C'`、`'s'` 的顺序追加；`IntegerLatticeBudget` 仅约束这一环面分支背后的 Smith 基计算，不用于半单数据。相关机制见 [[中心环面的商对合分类]]。^[layout-restricted-roots.md:17-22, layout-restricted-roots.md:37-41]

## Complex 因子配对与重排

对需要标记为 `'C'` 的分支，算法从后续分支中寻找包含 `twist[perm[offset]]` 的配对分支；若找不到，报告 `"non-matching Complex factor"`。找到后，通过旋转使两因子相邻，并将第二个因子的 Bourbaki 槽位改写为第一个因子的扭转像。因此，此步骤同时调整因子排列与单根置换，涉及 [[Bourbaki 顶点排序与置换语义]]。^[layout-restricted-roots.md:29-35]

实现保留了代码注释所述的上游移位顺序：先上移 `type` 切片，再在已经移位的切片上计算移位宽度。注释指出，这一顺序仅在 Complex 对跨越多个介于其间且秩不同的因子时可观测；当前没有构造夹具覆盖该情形。^[layout-restricted-roots.md:33-35]

## 测试锚点与证据边界

现有测试锚点包括：紧 A1 得到 `'c'`，twisted A2 得到 `'s'`，交换的 A1.A1 得到 `'C'` 且 `perm == [0, 1]`，D4 叉尖交换得到 `'u'`。Complex 因子重排另有 A1.B2.A1 夹具，其结果为 `perm == [0, 3, 1, 2]`。^[layout-restricted-roots.md:43-46]

A1.B2.A1 夹具提供了旋转重排的具体锚点，但未覆盖上述跨越多个不同秩中间因子的移位顺序问题。来源还明确指出，相关文件的错误分支没有失败路径测试；本材料属于结构性源码阅读，不构成数学或正确性验收。`atlas-types.w:2868-2941` 等上游位置仅转录自代码注释，未核对上游字节。^[layout-restricted-roots.md:10-13, layout-restricted-roots.md:29-35, layout-restricted-roots.md:79-82]

## Sources

- [layout-restricted-roots.md](layout-restricted-roots.md) — 内类布局与限制根系（layout.rs / restricted_roots.rs）
