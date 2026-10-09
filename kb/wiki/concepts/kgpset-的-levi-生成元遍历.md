---
title: kgp_set 的 Levi 生成元遍历
summary: kgp_set 在 theta-stable 化后沿实单根对应的 Levi 生成元执行位图限界 BFS，静默跳过无法映射的生成元，并依赖调用方保证实根分支的奇偶前提。
sources:
  - ktype.md
kind: concept
createdAt: "2026-10-09T14:56:48.502Z"
updatedAt: "2026-10-09T22:36:21.321Z"
tags:
  - K型
  - Levi子群
  - 广度优先搜索
aliases:
  - kgpset-的-levi-生成元遍历
  - K的L生
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: kgp_set 的 Levi 生成元遍历
summary: kgp_set 先将输入变为 theta-stable，再按实单根对应的 Levi 生成元执行位图限界 BFS；无法映射的根被静默跳过，实根分支依赖调用方保证 final/semifinal 前提。
sources:
  - ktype.md
kind: concept
tags:
  - K型
  - Levi子群
  - 广度优先搜索
aliases:
  - kgpset-的-levi-生成元遍历
---

# kgp_set 的 Levi 生成元遍历

`kgp_set` 是 `ktype.rs` 中的 K 型遍历过程，来源标注其对应上游 `K_repr.cpp:398–464`。它先将输入变为 theta-stable，再以所得元素的实单根作为 Levi 生成元，通过广度优先搜索（BFS）遍历 KGB 元素。^[ktype.md:56-61]

## 起点与生成元选择

遍历首先调用 `made_theta_stable`，耗尽复下降。该变形以图大小作为“慷慨”的终止界，相关错误为 `"theta-stable termination"`；参见 [[K 型变形与终止预算]]。^[ktype.md:39-40]

随后，`kgp_set` 将 theta-stable 元的实单根映射为 Levi 生成元下标。无法映射回生成元下标的根会被**静默跳过**。^[ktype.md:56-57]

## BFS 与分支规则

BFS 使用 `present` 位图限制重复入队，保证每个 KGB 元素至多入队一次。^[ktype.md:57-58]

Real 分支使用 `shift = eval/2`，但**不检查奇偶性**。final/semifinal 前提由调用方负责；源码注释称 wrapper 会先检查，遍历内部并不承担这项验证。相关条件见 [[K 型谓词链与调用前提]]。^[ktype.md:58-60]

Real 分支先处理 `second`，再处理 `first`。注释给出的理由是 `first` 更可能已经插入，因此后试。Complex 分支调用反射时，第三参数取 `0`。^[ktype.md:60-61]

## 证据与覆盖边界

来源属于对 `ktype.rs` 的结构性阅读，不声称数学验收；上游行号仅转录自注释。测试覆盖清单将 `kgp_set` 整体和全部终止预算错误列为未覆盖项，详见 [[K 型实现的测试锚点与证据边界]]。^[ktype.md:10-14, ktype.md:72-74]

该来源所记录的知识维护未执行 Atlas、Cargo、测试或 benchmark，因此上述说明属于源码阅读结论，不能作为 `kgp_set` 完整运行验证的证据。^[ktype.md:83-87]

## Sources

- [ktype.md](../../sources/ktype.md)：K 型值与 RepContext 谓词/变形（ktype.rs）。
