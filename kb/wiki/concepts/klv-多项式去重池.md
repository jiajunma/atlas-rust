---
title: KLV 多项式去重池
summary: KlHashTable 通过向量与哈希表将多项式内容映射为池索引，new 固定零与一的索引为 0 和 1，而派生 Default 生成的空池不满足该种子约定。
sources:
  - kl-polynomial-table.md
kind: concept
createdAt: "2026-10-09T14:54:56.489Z"
updatedAt: "2026-10-09T14:54:56.489Z"
tags:
  - 内容去重
  - 多项式存储
  - Rust设计
aliases:
  - klv-多项式去重池
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# KLV 多项式去重池

KLV 多项式去重池 `KlHashTable` 将内容相同的多项式映射到同一个 `KlIndex`，使 [[KLV 表的逐列存储与句柄设计|KLV 表]]通过池索引引用多项式。它对应上游的 `KL_hash_Table`。^[kl-polynomial-table.md:68-72]

## 存储与访问

池由 `pool: Vec<KlPol>` 和 `HashMap<KlPol, usize>` 组成，分别保存多项式本体及其内容到索引的映射。`match_pol` 在插入时去重；表的 `kl_pol` 访问器返回池索引，调用方通过 `pool()` 取得多项式本体。^[kl-polynomial-table.md:68-72]

池中的 `KlPol` 采用低次项在前的 `Vec<i32>` 表示：零多项式为空向量，非零多项式经 `trim` 保证最高次系数非零。这些表示约定属于 [[KLV 多项式的表示与不变量]]；非负性与首一性是算法输出的外层性质，并非类型维护的不变量。^[kl-polynomial-table.md:21-27]

## 初始化与固定索引

`KlHashTable::new()` 建立两个固定种子：索引 `0` 对应零多项式，索引 `1` 对应常数多项式 `1`，与上游 `KLStore{Zero, One}` 的初始化约定一致。^[kl-polynomial-table.md:59-61, kl-polynomial-table.md:68-72]

这两个索引直接参与 [[KLV 表的 primitive 投影与访问语义]]：当投影位置超出列长时，若 `x` 恰好投影到 `y` 自身，`kl_pol(x, y)` 返回索引 `1`，表示 \(P_{y,y}=1\)；否则返回索引 `0`，表示零多项式。^[kl-polynomial-table.md:89-93]

`KlHashTable` 派生的 `Default` 与 `new()` 语义不同：`KlHashTable::default()` 得到空池，不包含上述种子。如果调用方使用 `default()`，就会破坏“索引 `0` 为零、索引 `1` 为一”的约定；来源仅指出这一风险，没有确认存在此类调用点。^[kl-polynomial-table.md:59-61]

## 测试与证据边界

来源列出的测试检查了池种子序号：`get(0)` 为零多项式，`get(1).as_slice() == &[1]`。但 `match_pol` 的去重路径和 `get` 的越界行为仍属于未测试范围，因此种子测试不能作为去重行为已获完整验证的证据。^[kl-polynomial-table.md:118-123]

本页依据代码结构阅读记录。来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；其中的上游行号转述自源码注释，未独立重读上游。^[kl-polynomial-table.md:130-135]

## Sources

- [kl-polynomial-table.md](kl-polynomial-table.md) — KLV 多项式的存储与逐列计算。
