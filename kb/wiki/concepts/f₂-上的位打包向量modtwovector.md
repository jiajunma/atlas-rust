---
title: F₂ 上的位打包向量（ModTwoVector）
summary: 以动态 u64 数组表示有限维 F₂ 向量，重复下标按奇偶抵消，异或与内积检查维数，并保持填充位为零。
sources:
  - mod-two.md
kind: concept
createdAt: "2026-10-09T15:01:57.388Z"
updatedAt: "2026-10-10T00:44:09.311Z"
tags:
  - 模二线性代数
  - 位向量
  - Rust
aliases:
  - f₂-上的位打包向量modtwovector
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: F₂ 上的位打包向量（ModTwoVector）
summary: 以动态 u64 数组表示有限维 F₂ 向量，保持填充位为零，支持重复下标抵消、异或与奇偶内积，并检查维数、溢出和分配错误。
sources:
  - mod-two.md
kind: concept
tags:
  - 模二线性代数
  - 位向量
  - Rust设计
aliases:
  - f₂-上的位打包向量modtwovector
provenanceState: extracted
---

# F₂ 上的位打包向量（ModTwoVector）

`ModTwoVector` 表示有限向量空间 $\mathbb{F}_2^d$ 的元素，属于服务 Cartan 纤维结构层的动态模二线性代数层。它刻意与 Malachite 整数层分离：打包字编码有限维向量，而不是无界整数。^[mod-two.md:12-20]

## 表示与不变量

结构为 `ModTwoVector { dimension: usize, words: Vec<u64> }`，以维数和动态字数组保存向量。每个构造器均将 `dimension` 以上的填充位清零，并保持其为零。派生的 `Ord` 因而具有健全的表示基础，但它只是供 map 键使用的任意、确定全序，不是数学序。^[mod-two.md:17-20]

## 构造与位访问

`zero(dimension)` 创建零向量，所需字数为 `(d + 63) / 64`。其中加法使用 `checked_add`，传入 `usize::MAX` 时返回 `ArithmeticOverflow`；分配失败返回 `AllocationFailed`。^[mod-two.md:22-23]

`from_ones(dimension, indices)` 对每个下标执行位翻转，因此重复下标出现偶数次会抵消。例如，测试输入 `[0,63,64,127,128,63]` 使位 63 最终为 `false`。`bit(index)` 查询单个位，越界时返回 `None`。^[mod-two.md:24-26]

## 模二运算

`xor_assign` 执行异或运算，`dot` 计算 $\mathbb{F}_2$ 配对；维数不匹配时返回 `RankMismatch`。`dot` 的可见性为 `pub(crate)`，逐字计算 `(l & r).count_ones() & 1`，再以 XOR 汇总奇偶性，最终返回 `parity == 1`。^[mod-two.md:26-28]

## 与子空间及子商的关系

[[最低主元索引的典范 RREF 子空间|ModTwoSubspace]] 用 `ModTwoVector` 保存基向量，以每行最低置位作为主元索引。插入时先约化输入，再从旧行中消去新主元，使基保持与插入顺序无关的典范 RREF。约化也用于成员判定和确定性商类代表元计算，为结构子商层提供确定性坐标。^[mod-two.md:30-40]

[[F₂ 子商的低主元坐标（ModTwoSubquotient）]] 在共同环境空间中的两个子空间上构造典范商，使用低主元打包坐标。该类型刻意保持 crate 私有，公开数学接口由 `CartanFiber` 包装提供；内部坐标不是公开序列化格式。^[mod-two.md:62-74]

[[保留首批独立列的典范截面（CanonicalModTwoSection）]] 则区分源坐标与目标向量：源坐标装入 `u64` 掩码，因此输入列数最多为 64；目标向量仍采用动态表示。这一列数限制属于截面类型，而不是动态目标向量的维数限制。^[mod-two.md:51-60]

## 测试与证据边界

源包列出的测试锚点包括零维、跨字边界、重复下标抵消、维数不匹配、130 维动态子空间、跨多字依赖消元，以及 `usize::MAX` 的溢出或分配拒绝。多数 `ArithmeticOverflow` 与 `AllocationFailed` 分支尚未覆盖，`dot` 在文件内没有直接测试。^[mod-two.md:83-93]

上述信息来自源码结构性阅读和测试锚点整理。源包未执行构建、测试或原版运行，不包含数学验收、性能或并行结论；测试覆盖描述不代表本次执行结果。^[mod-two.md:9-13, mod-two.md:103-108]

## Sources

- [mod-two.md](../../sources/mod-two.md) — mod-2 线性代数：位打包向量与子空间。
