---
title: KLV 多项式去重池
summary: KlHashTable 按多项式内容去重，new 将零与一固定在索引 0 和 1，而派生 Default 产生无种子的空池。
sources:
  - kl-polynomial-table.md
kind: concept
createdAt: "2026-10-09T14:54:56.489Z"
updatedAt: "2026-10-10T00:38:21.011Z"
tags:
  - KLV
  - 去重
  - 存储
aliases:
  - klv-多项式去重池
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: KLV 多项式去重池
summary: KlHashTable 按多项式内容去重并返回池索引；new 固定零与一的索引为 0 和 1，派生 Default 则产生无种子的空池。
sources:
  - kl-polynomial-table.md
kind: concept
tags:
  - KLV多项式
  - 去重存储
aliases:
  - klv-多项式去重池
---

# KLV 多项式去重池

`KlHashTable` 将内容相同的 KLV 多项式映射到同一个 `KlIndex`，供 [[KLV 表的逐列存储与句柄设计|KLV 表]]通过池索引引用多项式。它对应上游的 `KL_hash_Table`。^[kl-polynomial-table.md:68-79]

## 存储与访问

池由 `pool: Vec<KlPol>` 和 `HashMap<KlPol, usize>` 组成，分别保存多项式本体及其内容到索引的映射。`match_pol` 在插入时去重；表的 `kl_pol` 访问器返回池索引，调用方通过 `pool()` 取回多项式本体。表按列存储的 KL 数据保存这些索引。^[kl-polynomial-table.md:68-79]

`KlPol` 使用低次项在前的 `Vec<i32>`：零多项式为空向量，非零多项式经 `trim` 保持最高次系数非零。非负性与首一性属于算法输出的外层性质，并非类型维护的不变量，详见 [[KLV 多项式的表示与不变量]]。^[kl-polynomial-table.md:21-27]

## 初始化与固定索引

`KlHashTable::new()` 建立两个固定种子：索引 `0` 对应零多项式，索引 `1` 对应常数多项式 `1`。这一约定与上游 `KLStore{Zero, One}` 的初始化一致。^[kl-polynomial-table.md:59-61, kl-polynomial-table.md:68-72]

这两个固定索引参与 [[KLV 表的 primitive 投影与访问语义]]：当 `kl_pol(x, y)` 的 primitive 投影位置超出列长时，若 `x` 恰好投影到 `y` 自身，则返回索引 `1`，表示 \(P_{y,y}=1\)；否则返回索引 `0`，表示零多项式。^[kl-polynomial-table.md:89-93]

**派生的 `Default` 与 `new()` 不等价。** `KlHashTable::default()` 得到空池，不包含零与一种子。来源将其记录为条件性风险：若存在这样的调用点，固定索引约定将被破坏；来源未确认此类调用点存在。^[kl-polynomial-table.md:59-61]

## 测试与证据边界

来源列出的测试锚点检查池种子序号：`get(0)` 为零多项式，`get(1).as_slice() == &[1]`。`match_pol` 的去重路径与 `get` 的越界行为仍列在未测试范围内，因此种子测试不能作为这些行为已获验证的证据。^[kl-polynomial-table.md:118-123]

本页依据结构性源码阅读记录。来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；上游行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[kl-polynomial-table.md:9-17, kl-polynomial-table.md:130-135]

## Sources

- [kl-polynomial-table.md](../../sources/kl-polynomial-table.md) — KLV 多项式的存储与逐列计算。
