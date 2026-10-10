---
title: KlSupport 的拓扑构造门控
summary: 构造期集中验证秩容量、长度存在且非降、逐生成元拓扑数据完整及链接目标合法，避免列填充时因这些不变量失效而 panic。
sources:
  - kl-support.md
kind: concept
createdAt: "2026-10-09T14:55:39.214Z"
updatedAt: "2026-10-10T00:39:00.391Z"
tags:
  - KL
  - 不变量
aliases:
  - klsupport-的拓扑构造门控
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: KlSupport 的拓扑构造门控
summary: KlSupport 在构造期集中检查秩容量、长度非降、拓扑数据存在性与链接目标合法性；本原索引另有准备前提和未显式验证的顺序假定。
sources:
  - kl-support.md
kind: concept
tags:
  - 输入验证
  - 拓扑不变量
aliases:
  - klsupport-的拓扑构造门控
---

# KlSupport 的拓扑构造门控

`KlSupport<B: BlockTopology>` 为块元素预计算下降集、good-ascent 集与 length-stop 表，并提供懒填充的本原索引表。构造器 `new` 首先调用 `validate_topology`，在可失败的构造边界集中检查拓扑不变量，避免这些不变量失效时在列填充深处触发 panic。^[kl-support.md:16-21, kl-support.md:32-39]

## 构造校验

构造门控要求 rank ≤ 32；每个元素的长度必须存在，且按元素顺序非降；每个元素、每个生成元对应的 descent、cayley 和 inverse_cayley 数据必须存在；cross 与 Cayley 像的目标都必须位于块内。^[kl-support.md:32-35]

秩上限对应 [[RankFlags：简单生成元位集]] 的 `u32` 存储容量。`RankFlags` 的 `set` 和 `is_set` 自身不做边界检查，因此在 `KlSupport` 的使用路径上，由构造门控保证 rank ≤ 32。^[kl-support.md:23-28]

## 校验后的预计算

通过校验后，构造器逐元素分类生成元：`is_descent()` 为真时加入下降集；否则，仅当类型不是 `ImaginaryTypeII` 时加入 good-ascent 集。`ImaginaryTypeII` 既不是下降，也不是 good ascent。本原性由这些集合判定：`x` 对给定下降集本原，当且仅当其 good-ascent 集与该下降集的交集为空，详见[[下降集、good ascent 与本原性]]。^[kl-support.md:19-20, kl-support.md:35-37, kl-support.md:41-43]

长度非降是构造期强制的不变量，也是 [[length-stop 长度边界表]] 的构建依据：`length_stop[l]` 表示首个长度 ≥ `l` 的元素，表末尾追加块大小 `size`。本原索引 `prim_index` 则采用懒填充。^[kl-support.md:32-39]

## 门控的边界

本原索引表采用降序扫描，并沿 `unique_ascent` 链继承索引。该过程依赖“上升像序号更大”的假定；来源将其标为阅读观察，并明确指出没有显式防护。这与构造期强制检查的长度非降保证不同。^[kl-support.md:49-52, kl-support.md:70-71]

本原索引访问还有独立的调用前提：`prim_index`、`nr_of_primitives`、`col_size` 和 `self_index` 都要求先对同一下降集调用 `prepare_prim_index`，否则会因 map 缺键而 panic。只有 `prim_index` 的文档注释显式写出这一前置条件。^[kl-support.md:53-55]

## 测试与证据范围

使用 `FakeTopology` 替身的单元测试覆盖三条构造拒绝路径：rank 为 33、长度序列不满足非降要求，以及 cross 链接目标越界。`new` 的成功路径、全部判定方法和整个本原索引机制没有单元测试；来源说明这些部分经 KL 层的 HPC 门覆盖。相关限制见 [[KL 支撑层的测试覆盖与证据边界]]。^[kl-support.md:57-62]

本页依据 `kl_support.rs` 的结构性阅读，不构成数学或正确性验收。来源未核对上游引用的文件字节，也未读取 `crate::block` 中 `BlockDescent` 的完整变体及 `is_descent()` 定义；该次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[kl-support.md:10-12, kl-support.md:66-69, kl-support.md:75-79]

## Sources

- [kl-support.md](../../sources/kl-support.md) — 逐块 KL 支撑数据：KlSupport 与 RankFlags（kl_support.rs）
