---
title: kgp_set 的 Levi 生成元遍历
summary: kgp_set 先构造 theta-stable 元，再以其实单根为 Levi 生成元开展位图限界 BFS；无法映射的生成元被跳过，实根分支的奇偶前提由调用方承担。
sources:
  - ktype.md
kind: concept
createdAt: "2026-10-09T14:56:48.502Z"
updatedAt: "2026-10-09T14:56:48.502Z"
tags:
  - 表示论
  - 广度优先搜索
  - 调用契约
aliases:
  - kgpset-的-levi-生成元遍历
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# kgp_set 的 Levi 生成元遍历

`kgp_set` 是 `ktype.rs` 中的 K 型遍历过程，源码注释对应上游 `K_repr.cpp:398–464`。它先将输入变为 theta-stable，再选取实单根作为 Levi 生成元，通过广度优先搜索（BFS）遍历 KGB 元素。^[ktype.md:56-61]

## 起点与生成元选择

遍历首先调用 `made_theta_stable`，耗尽复下降；该变形以图大小作为“慷慨”的终止界，相关错误为 `"theta-stable termination"`。随后，`kgp_set` 取 theta-stable 元的实单根作为 Levi 生成元；无法映射回生成元下标的根会被静默跳过。相关变形约束见 [[K 型变形与终止预算]]。^[ktype.md:39-40, ktype.md:56-57]

## BFS 与分支规则

BFS 使用 `present` 位图限制入队，使每个 KGB 元素至多入队一次。这是遍历的去重与限界机制。^[ktype.md:57-58]

Real 分支使用 `shift = eval/2`，但不检查奇偶性。final/semifinal 前提由调用方负责；源码注释称 wrapper 会先检查这些前提，因此不能把该除法理解为遍历内部已经完成了前提验证。相关条件见 [[K 型谓词链与调用前提]]。^[ktype.md:58-60]

处理候选像时，Real 分支先处理 `second`，再处理 `first`；注释给出的理由是 `first` 更可能已经插入。Complex 分支调用反射时，第三参数取 `0`。这些顺序和参数约定属于实现细节。^[ktype.md:60-61]

## 证据与覆盖边界

来源是对 `ktype.rs` 的结构性阅读，不构成数学验收。现有测试未覆盖 `kgp_set` 整体，也未覆盖全部终止预算错误；本次知识维护没有执行 Atlas、Cargo、测试或 benchmark。因此，上述内容描述的是源码中记录的行为，不能据此宣称遍历已经通过完整运行验证。参见 [[K 型实现的测试锚点与证据边界]]。^[ktype.md:10-14, ktype.md:72-74, ktype.md:83-87]

## Sources

- [ktype.md](../../sources/ktype.md)：K 型值与 RepContext 谓词/变形（ktype.rs）。
