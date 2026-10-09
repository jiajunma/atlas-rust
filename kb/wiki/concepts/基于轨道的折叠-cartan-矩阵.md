---
title: 基于轨道的折叠 Cartan 矩阵
summary: folded_cartan 按轨道构造折叠根与折叠余根，并依 crate 的配对约定累加 cartan[a][b]；它检查索引越界，但将轨道完整性、不交性、覆盖性及输出合法性留给调用方。
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

`folded_cartan` 根据 Cartan 矩阵与 `ExtGen` 轨道描述计算折叠矩阵，是 `dynkin.rs` 中的 crate 内部接口。源码注释将它对应到 upstream `DynkinDiagram::folded`，实际采用 `rootdata.cpp` 中的 `cofold` Cartan 公式；注释声称两者的边重数相同，但来源材料未验证这一点。^[dynkin.md:78-85]

## 接口与构造

接口接收整数 Cartan 矩阵及轨道列表，返回整数矩阵或错误；其可见性为 `pub(crate)`。^[dynkin.md:25-32]

```rust
pub(crate) fn folded_cartan(
    cartan: &[Vec<i32>],
    orbits: &[ext_block::ExtGen],
) -> Result<Vec<Vec<i32>>, _>
```

折叠单根取轨道成员之和。折叠单余根的选择依赖成员是否可交换：可交换成员对应的长度 2 轨道取第一个成员的余根；不可交换成员对应的长度 3 情形取两者余根之和。实现按 `ExtGenKind::One`、`Three` 和其他情况分支，文件中未显式出现 `Two` 分支名称。^[dynkin.md:80-81, dynkin.md:106-107]

## 矩阵条目与下标方向

crate 的原始矩阵约定为 \(\operatorname{cartan}[a][b]=\langle\alpha_a,\alpha_b^\vee\rangle\)。折叠矩阵的 \(C(i,j)\) 对第 \(j\) 个轨道的折叠根与第 \(i\) 个轨道的折叠余根进行配对，即把对应根成员 \(a\) 与余根成员 \(b\) 的 `cartan[a][b]` 相加。因此，输出的第一下标 \(i\) 选择余根轨道，第二下标 \(j\) 选择根轨道；解释结果时须保留这一方向。^[dynkin.md:82-83]

## 校验与调用方责任

轨道成员下标越界时，函数返回 `IndexOutOfRange { index: a.max(b) }`。它不校验轨道的完整性、不交性或覆盖性，也不校验输出是否为合法 Cartan 矩阵；输出合法性依赖调用方。相关输入约束可参见 [[Cartan 矩阵输入契约与连通分量划分]]，但不能据此假定折叠接口执行了相同校验。^[dynkin.md:83-85, dynkin.md:106-107]

## 证据边界

本页依据对 `dynkin.rs` 的结构性阅读，不代表分类器或折叠公式的数学验收。上游文件位置仅转录自源码注释，未核对上游字节；此次知识维护也未执行 Atlas、Cargo、测试或 benchmark。来源列出的六个测试均为成功路径，所列锚点未包含折叠矩阵测试，覆盖背景见 [[Dynkin 分类器的测试覆盖与证据边界]]。^[dynkin.md:9-12, dynkin.md:87-102, dynkin.md:111-115]

## Sources

- [dynkin.md](dynkin.md) — Dynkin 分类器：连通分量、Bourbaki 置换与折叠 Cartan（dynkin.rs）
