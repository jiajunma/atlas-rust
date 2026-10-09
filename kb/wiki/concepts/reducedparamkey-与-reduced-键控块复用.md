---
title: ReducedParamKey 与 reduced 键控块复用
summary: 以姿态传输后的 KGB 元素、规范整数据编号及 Smith codec 余数组成私有稳定键，使不同 Weyl 姿态下匹配的查询复用已存块。
sources:
  - rep-table.md
kind: concept
createdAt: "2026-10-09T15:09:10.192Z"
updatedAt: "2026-10-09T19:36:15.923Z"
tags:
  - 块存储
  - 规范化
  - 键控复用
aliases:
  - reducedparamkey-与-reduced-键控块复用
  - R与R键
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# ReducedParamKey 与 reduced 键控块复用

`ReducedParamKey` 是共享公共块存储内部用于识别 reduced 参数的私有键。存储服务于一个实形式；当查询的积分子系统在某个 Weyl 姿态下与已存块匹配时，可复用该块，并通过查询相对的 `block_modifier` 记录姿态差。消费者仅获得稳定块句柄与查询相对代表元，reduced 键及其 Smith codec 保持私有。^[rep-table.md:19-27]

## 键的组成与规范化

键的结构为 `ReducedParamKey { x: KgbId, int_sys: u32, residue: u32 }`，提供 reduced 参数的哈希稳定身份，对应上游 `Reduced_param` 值 `(x, int_sys_nr, residue)`。三个字段分别表示姿态传输后的 KGB 元素、规范整数据编号和规范 codec 赋值的混合进制打包值。^[rep-table.md:21-24, rep-table.md:31-34]

| 字段 | 含义 |
| --- | --- |
| `x` | 经 locator attitude 传输后的 KGB 元素，由 `transform<true>(loc.w, srm)` 得到 |
| `int_sys` | 规范整数据的 id，对应 `locator::int_sys_nr` |
| `residue` | 规范 Smith codec 各赋值的混合进制打包值 |

构造 reduced 键时，`InnerClass::int_item` 在 Weyl 群作用下规范化整数据，`srm` 经 locator attitude 传输，`residue` 取自规范数据的 Smith codec。因此，键中的 `x` 使用传输后的 KGB 元素。相关概念见 [[典范整数据驻留]] 与 [[Weyl 姿态定位器]]。^[rep-table.md:21-24, rep-table.md:31-34]

## 块复用与查询相对姿态

查询匹配已存块后，`block_modifier` 记录查询到存储块的姿态差；来源将其对应到上游 `make_relative_to`。这一机制允许复用 Weyl 姿态下匹配的块，同时保留查询相对的代表元信息，参见 [[BlockModifier 块修正子]]。^[rep-table.md:24-27]

[[LocatedBlock 稳定块句柄与查询相对姿态]] 通过 `block()` 提供 `Arc<PartialBlock>`，通过 `raw_row()` 提供查询在存储块编号中的行号，并通过 `is_full()` 标识是否指向完整公共块。`prepared_query()` 在部分查找中保存规范化（normalised）查询，在完整查找中保存 dominant 查询；reduced 键与块相对代表元均从该参数算得。^[rep-table.md:36-41]

消费者可通过 `block_modifier()`、`relative_shift()` 和 `adapted_representative()` 获取查询相对已存块的姿态数据。只有当 `has_identity_generator_attitude()` 为真，即 modifier 的 `w` 与 `simple_pi` 均为恒等时，才可用平实中心位移直接读取存储行；仍假设恒等姿态的消费者受到显式门控。^[rep-table.md:25-27, rep-table.md:42-46]

## 查找入口

[[RepTableOwner 实形式资源所有者]] 提供两个查找入口：`lookup(query)` 解析或物化查询下方最小的部分块，`lookup_full_block(query)` 解析或物化包含查询的完整公共块。二者分别对应部分块与完整块的查找语义。^[rep-table.md:56-61]

## 证据范围

本页依据对 `rep_table.rs` 的结构性阅读，所读字节记录于 dirty 工作区快照。来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；块存储正确性属于其独立的 [[HPC 验收证据链]]。来源中的上游行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[rep-table.md:9-15, rep-table.md:72-80]

## Sources

- [rep-table.md](rep-table.md) — 共享块存储：reduced 键控复用与 RepTableOwner。
