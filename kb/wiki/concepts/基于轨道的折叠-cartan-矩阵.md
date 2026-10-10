---
title: 基于轨道的折叠 Cartan 矩阵
summary: folded_cartan 按轨道构造折叠根与余根并累加 cartan[a][b]，检查下标越界，但轨道完整性、不交性、覆盖性及输出合法性由调用方保证。
sources:
  - dynkin.md
kind: concept
createdAt: "2026-10-09T14:45:36.116Z"
updatedAt: "2026-10-10T00:30:50.257Z"
tags:
  - Cartan矩阵
  - 根系折叠
  - 调用契约
aliases:
  - 基于轨道的折叠-cartan-矩阵
  - 基C矩
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 基于轨道的折叠 Cartan 矩阵
summary: folded_cartan 按 ExtGen 轨道构造折叠根与余根，依指定下标方向累加 Cartan 条目；它检查索引越界，但不验证轨道划分及输出矩阵的合法性。
sources:
  - dynkin.md
kind: concept
tags:
  - Cartan矩阵
  - 根系折叠
aliases:
  - 基于轨道的折叠-cartan-矩阵
provenanceState: extracted
---

# 基于轨道的折叠 Cartan 矩阵

`folded_cartan` 接收原始 Cartan 矩阵与 `ExtGen` 轨道列表，返回折叠矩阵，是 `dynkin.rs` 中仅限 crate 内使用的接口。源码注释将其对应到上游 `DynkinDiagram::folded`，实际采用 `rootdata.cpp` 中的 `cofold` Cartan 公式；注释声称两者边重数相同，但来源材料未验证这一点。^[dynkin.md:25-32, dynkin.md:78-85]

## 折叠根与余根

折叠单根取轨道成员之和。折叠单余根按成员是否可交换构造：可交换成员的长度 2 轨道取第一个成员的余根；不可交换的长度 3 情形取两者余根之和。实现仅区分 `ExtGenKind::One`、`Three` 和其他情况，文件中未显式出现 `Two` 名称。^[dynkin.md:80-81, dynkin.md:106-107]

## 矩阵条目与下标方向

原始矩阵采用约定 \(\operatorname{cartan}[a][b]=\langle\alpha_a,\alpha_b^\vee\rangle\)。输出条目 \(C(i,j)\) 对第 \(j\) 个轨道的折叠根与第 \(i\) 个轨道的折叠余根进行配对，累加相应的 `cartan[a][b]`。因此，输出的第一下标选择余根轨道，第二下标选择根轨道。^[dynkin.md:82-83]

## 校验与调用契约

轨道下标越界时，函数返回 `IndexOutOfRange { index: a.max(b) }`。它不检查轨道的完整性、不交性或覆盖性，也不验证输出是否为合法 Cartan 矩阵；输出合法性依赖调用方。^[dynkin.md:83-85, dynkin.md:106-107]

同一模块的 `classify` 检查矩阵方形、对角元为 2、非对角元属于 `-3..=0`，以及零模式对称性。这些是分类接口的输入检查，相关说明见 [[Cartan 矩阵输入契约与连通分量划分]]；来源未将这些检查列为 `folded_cartan` 的行为。^[dynkin.md:36-41, dynkin.md:78-85]

## 测试与证据边界

来源列出的六个测试均为成功路径，覆盖类型分类与 Bourbaki 置换，未列出折叠矩阵测试；错误路径没有测试覆盖。参见 [[Dynkin 分类器的测试覆盖与证据边界]]。^[dynkin.md:87-102]

本页依据对 `dynkin.rs` 的结构性阅读，不构成数学验收。上游引用仅转录自源码注释，未核对上游文件字节；本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[dynkin.md:9-12, dynkin.md:111-115]

## Sources

- [dynkin.md](../../sources/dynkin.md) — Dynkin 分类器：连通分量、Bourbaki 置换与折叠 Cartan（dynkin.rs）。
