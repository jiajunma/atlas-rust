---
title: F₂ 上的位打包向量（ModTwoVector）
summary: 以动态 u64 数组表示有限维 F₂ 向量，支持异或与奇偶内积；重复下标偶次抵消，填充位保持为零，与无界整数表示分离。
sources:
  - mod-two.md
kind: concept
createdAt: "2026-10-09T15:01:57.388Z"
updatedAt: "2026-10-09T15:01:57.388Z"
tags:
  - 线性代数
  - 位打包
  - Rust
aliases:
  - f₂-上的位打包向量modtwovector
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# F₂ 上的位打包向量（ModTwoVector）

`ModTwoVector` 是有限向量空间 $\mathbb{F}_2^d$ 元素的位打包表示，属于服务 Cartan 纤维结构层的动态模二线性代数层。它刻意与 Malachite 整数层分离：打包字表示有限维向量，而不是无界整数。^[mod-two.md:12-20]

## 表示与不变量

其结构为 `ModTwoVector { dimension: usize, words: Vec<u64> }`，使用 `dimension` 记录维数，使用 `words` 存储打包位。每个构造器都会将维数范围之外的填充位清零，并保持这些位为零。派生的 `Ord` 因此具有健全的表示基础，但它仅是供 map 键使用的任意、确定全序，不是数学意义上的序。^[mod-two.md:17-20]

## 构造与位访问

`zero(dimension)` 创建零向量，所需字数为 `(d + 63) / 64`。其中加法使用 `checked_add` 检查；例如维数为 `usize::MAX` 时返回 `ArithmeticOverflow`，分配失败则返回 `AllocationFailed`。^[mod-two.md:22-23]

`from_ones(dimension, indices)` 对每个给定下标执行位翻转，而不是单纯置位，因此同一下标出现偶数次会抵消。测试中的下标序列 `[0,63,64,127,128,63]` 会使位 63 最终为 `false`。`bit(index)` 查询单个位，越界时返回 `None`。^[mod-two.md:24-26]

## 模二运算

`xor_assign` 与 `dot` 要求两侧向量维数一致，否则返回 `RankMismatch`。`dot` 具有 `pub(crate)` 可见性，实现 $\mathbb{F}_2$ 配对：逐字计算 `(l & r).count_ones() & 1`，再以 XOR 汇总奇偶性，最终返回 `parity == 1`。^[mod-two.md:26-28]

## 与子空间层的关系

[[最低主元索引的典范 RREF 子空间|ModTwoSubspace]] 使用 `ModTwoVector` 存储基向量，并以每行最低置位的位置作为主元索引。插入时先约化输入，再消去旧行中的新主元，使基保持与插入顺序无关的典范 RREF；这一结构进一步支持 [[F₂ 商空间的确定性代表元与陪集判定]]。^[mod-two.md:30-40]

[[F₂ 子商的低主元坐标（ModTwoSubquotient）]] 在共同环境空间中的两个子空间上构造典范商，使用低主元打包坐标。该子商类型刻意保持 crate 私有，公开数学接口由 `CartanFiber` 包装提供，内部坐标不作为序列化格式公开。^[mod-two.md:62-70]

## 测试与证据边界

源包列出的测试锚点包括零维、跨字边界、重复下标抵消、维数不匹配、130 维动态子空间，以及 `usize::MAX` 的溢出或分配拒绝。多数 `ArithmeticOverflow` 与 `AllocationFailed` 分支仍未覆盖，`dot` 在该文件内没有直接测试。^[mod-two.md:83-93]

这些信息来自源码结构性阅读及测试锚点整理；源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。^[mod-two.md:9-13, mod-two.md:103-108]

## Sources

- [mod-two.md](mod-two.md) — mod-2 线性代数：位打包向量与子空间。
