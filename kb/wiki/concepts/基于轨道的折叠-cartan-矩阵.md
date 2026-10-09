---
title: 基于轨道的折叠 Cartan 矩阵
summary: folded_cartan 按轨道构造折叠根与余根并累加 cartan[a][b]，仅检查下标越界，轨道完整性、不交性、覆盖性及输出合法性由调用方保证。
sources:
  - dynkin.md
kind: concept
createdAt: "2026-10-09T14:45:36.116Z"
updatedAt: "2026-10-09T20:50:57.167Z"
tags:
  - Cartan矩阵
  - 轨道折叠
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
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 基于轨道的折叠 Cartan 矩阵
summary: folded_cartan 按 ExtGen 轨道构造折叠根与折叠余根，依指定下标方向累加 Cartan 条目；它检查索引越界，但不验证轨道划分及输出矩阵的合法性。
sources:
  - dynkin.md
kind: concept
createdAt: "2026-10-09T14:45:36.116Z"
updatedAt: "2026-10-09T14:45:36.116Z"
tags:
  - Cartan矩阵
  - 根系折叠
  - 接口契约
aliases:
  - 基于轨道的折叠-cartan-矩阵
  - 基C矩
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 基于轨道的折叠 Cartan 矩阵

`folded_cartan` 根据原始 Cartan 矩阵与 `ExtGen` 轨道列表计算折叠矩阵，是 `dynkin.rs` 中的 `pub(crate)` 接口。源码注释将其对应到上游 `DynkinDiagram::folded`，实际采用 `rootdata.cpp` 中的 `cofold` Cartan 公式；注释声称两者的边重数相同，但来源材料未验证这一点。^[dynkin.md:25-32, dynkin.md:78-85]

## 折叠根与折叠余根

折叠单根取轨道成员之和。折叠单余根的选择取决于成员是否可交换：可交换成员对应的长度 2 轨道取第一个成员的余根；不可交换成员对应的长度 3 情形取两者余根之和。实现仅按 `ExtGenKind::One`、`Three` 和其他情况分支，本文件未显式出现 `Two` 分支名称。^[dynkin.md:80-81, dynkin.md:106-107]

## 矩阵条目与下标方向

crate 的原始矩阵约定为 \(\operatorname{cartan}[a][b]=\langle\alpha_a,\alpha_b^\vee\rangle\)。折叠矩阵条目 \(C(i,j)\) 对第 \(j\) 个轨道的折叠根与第 \(i\) 个轨道的折叠余根进行配对，累加相应的 `cartan[a][b]`。因此，输出的第一下标 \(i\) 选择余根轨道，第二下标 \(j\) 选择根轨道；这一方向是解释输出时必须保留的约定。^[dynkin.md:82-83]

## 校验与调用方责任

轨道成员下标越界时，函数返回 `IndexOutOfRange { index: a.max(b) }`。它不校验轨道的完整性、不交性或覆盖性，也不校验输出是否为合法 Cartan 矩阵；输出合法性依赖调用方。^[dynkin.md:83-85, dynkin.md:106-107]

同一模块的 `classify` 另有方形、对角元、非对角元范围及零模式对称性检查，详见 [[Cartan 矩阵输入契约与连通分量划分]]。这些属于分类接口的输入契约，不能据此认定 `folded_cartan` 执行了同样的检查。^[dynkin.md:36-41, dynkin.md:78-85]

## 证据边界

本页依据对 `dynkin.rs` 的结构性阅读，不构成分类器或折叠公式的数学验收。上游位置仅转录自源码注释，未核对上游字节；本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[dynkin.md:9-12, dynkin.md:111-115]

来源列出的六个测试均为成功路径，其锚点涉及类型分类与 Bourbaki 置换，未列出折叠矩阵测试。错误路径同样没有测试覆盖，相关背景见 [[Dynkin 分类器的测试覆盖与证据边界]]。^[dynkin.md:87-102]

## Sources

- [dynkin.md](../../sources/dynkin.md) — Dynkin 分类器：连通分量、Bourbaki 置换与折叠 Cartan（dynkin.rs）
