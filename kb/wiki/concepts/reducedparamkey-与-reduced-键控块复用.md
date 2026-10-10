---
title: ReducedParamKey 与 reduced 键控块复用
summary: 私有键 (x, int_sys, residue) 由姿态传输后的 KGB 元素、规范整数据编号和 Smith codec 余数组成，使匹配的不同 Weyl 姿态查询复用已存块。
sources:
  - rep-table.md
kind: concept
createdAt: "2026-10-09T15:09:10.192Z"
updatedAt: "2026-10-10T00:49:43.065Z"
tags:
  - 块存储
  - 参数规范化
aliases:
  - reducedparamkey-与-reduced-键控块复用
  - R与R键
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: ReducedParamKey 与 reduced 键控块复用
summary: 私有键结合姿态传输后的 KGB 元素、规范整数据编号与 Smith codec 余数，使匹配的不同 Weyl 姿态查询复用已存块，并保留查询相对姿态。
sources:
  - rep-table.md
kind: concept
tags:
  - 块存储
  - 规范化
  - 键控复用
---

# ReducedParamKey 与 reduced 键控块复用

`ReducedParamKey` 是共享公共块存储内部用于识别 reduced 参数的私有键。该存储服务于一个实形式：当查询的积分子系统在某个 Weyl 姿态下与已存块匹配时，便复用该块，并以 `block_modifier` 记录查询到存储块的姿态差。reduced 键及其 Smith codec 保持私有，消费者只获得稳定块句柄与查询相对的代表元。^[rep-table.md:19-27]

## 键的组成与规范化

键的结构为 `ReducedParamKey { x: KgbId, int_sys: u32, residue: u32 }`，提供 reduced 参数的哈希稳定身份，对应上游 `Reduced_param` 值 `(x, int_sys_nr, residue)`。下表列出各字段的含义。^[rep-table.md:21-24, rep-table.md:31-34]

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `x` | `KgbId` | 经 locator attitude 传输后的 KGB 元素，即 `transform<true>(loc.w, srm)` |
| `int_sys` | `u32` | 规范整数据编号，即 `locator::int_sys_nr` |
| `residue` | `u32` | 规范 Smith codec 各赋值的混合进制打包值 |

构造 reduced 键时，`InnerClass::int_item` 在 Weyl 群作用下规范化整数据，`srm` 经 locator attitude 传输，`residue` 取自规范数据的 Smith codec。因此，键中的 `x` 是姿态传输后的 KGB 元素。相关主题见 [[典范整数据驻留]] 与 [[Weyl 姿态定位器]]。^[rep-table.md:21-24, rep-table.md:31-34]

## 块复用与查询相对姿态

复用已存块时，[[BlockModifier 块修正子]] 保存查询到存储块的姿态差，来源将其对应到上游 `make_relative_to`。仍假设恒等姿态的消费者受到显式门控。^[rep-table.md:24-27]

[[LocatedBlock 稳定块句柄与查询相对姿态]] 通过 `block()` 提供 `Arc<PartialBlock>`，通过 `raw_row()` 提供查询在存储块编号中的行号，并通过 `is_full()` 标识是否指向完整公共块。`prepared_query()` 在部分查找中保存规范化（normalised）查询，在完整查找中保存 dominant 查询；reduced 键与块相对代表元均从该参数计算。^[rep-table.md:36-41]

消费者通过 `block_modifier()`、`relative_shift()` 和 `adapted_representative()` 获取查询相对已存块的姿态数据。`has_identity_generator_attitude()` 为真，当且仅当 modifier 的 `w` 与 `simple_pi` 均为恒等；只有此时，消费者才可用平实中心位移直接读取存储行。^[rep-table.md:42-46]

## 查找入口

[[RepTableOwner 实形式资源所有者]] 提供两个查找入口：`lookup(query)` 解析或物化查询下方最小的部分块；`lookup_full_block(query)` 解析或物化包含查询的完整公共块。二者分别界定部分块与完整块的查找范围。^[rep-table.md:56-61]

## 证据范围

本页依据对 `rep_table.rs` 的结构性阅读，所读字节记录于 dirty 工作区快照 `2026-10-03-rep-table.json`。块存储正确性属于其独立的 HPC 证据链，来源包不重述或扩展该证据；本包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。^[rep-table.md:9-15, rep-table.md:80-80]

来源中的上游行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。`BlockLocator`、`IntegralDatumTable` 与 `ActiveKlCallback` 的展开属于后续来源包。^[rep-table.md:74-79]

## Sources

- [rep-table.md](../../sources/rep-table.md) — 共享块存储：reduced 键控复用与 RepTableOwner。
