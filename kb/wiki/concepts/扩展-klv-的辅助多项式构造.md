---
title: 扩展 KLV 的辅助多项式构造
summary: qk_plus_1、qk_minus_1 与 qk_minus_q 分别构造 1+q^k、q^k−1 与 q^k−q，pol_mul_spol 支持带符号 SPol 与 KlPol 相乘。
sources:
  - extended-kl.md
kind: concept
createdAt: "2026-10-09T19:28:53.997Z"
updatedAt: "2026-10-09T19:28:53.997Z"
tags:
  - 扩展KLV
  - 多项式运算
aliases:
  - 扩展-klv-的辅助多项式构造
  - 扩K的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 扩展 KLV 的辅助多项式构造

扩展 KLV 的辅助多项式构造位于 `ext_kl.rs`，提供三种返回 `KlPol` 的多项式构造函数，以及带符号多项式与 `KlPol` 的乘积操作。它们属于扩展（twisted）KLV 多项式表的实现；该表在结构上镜像普通 KLV 表，但采用 extended-block 版本的递归。^[extended-kl.md:19-24, extended-kl.md:64-68]

## 三种基本构造

`qk_plus_1(k)` 构造 \(1+q^k\)，`qk_minus_1(k)` 构造 \(q^k-1\)，`qk_minus_q(k)` 构造 \(q^k-q\)。三者均返回 `KlPol`；来源标注的上游对应位置依次为 `ext_kl.cpp:178-184`、`186-192` 和 `194-200`。^[extended-kl.md:66-67]

## 带符号乘积

`pol_mul_spol` 计算带符号的 `SPol` 与 `KlPol` 的乘积，来源将其对应到上游 `ext_kl.cpp:209-212` 等位置。^[extended-kl.md:68-68]

扩展表复用共享的 `KlHashTable`，其条目是系数为 `i32` 的 `KlPol`。表的 primitivisation 符号另存于 `prim_flip` 位图，不打包进池索引；`kl_pol_index` 返回 `(KLIndex, bool)`，查询 `p(x,y)` 时若需翻转符号，则通过 `scaled(-1)` 得到结果。相关机制见 [[KLV 多项式去重池]] 与 [[Primitivisation 索引与符号传播]]。^[extended-kl.md:28-35, extended-kl.md:72-80]

## 证据边界

来源仅明确列出这些辅助函数的公式、返回类型及乘积操作，没有展开其具体实现或测试结果。本材料属于结构性阅读，未执行构建、测试或原版运行；扩展 KL 的正确性由独立的 [[HPC 验收证据链]] 承载，不能由这些函数说明推导出数学验收、性能或并行结论。^[extended-kl.md:9-15, extended-kl.md:64-68, extended-kl.md:114-114]

## Sources

- [extended-kl.md](extended-kl.md) — 扩展 KLV 多项式表：primitivisation 符号与逐列存储。
