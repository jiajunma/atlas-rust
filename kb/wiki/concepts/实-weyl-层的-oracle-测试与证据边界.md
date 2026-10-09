---
title: 实 Weyl 层的 oracle 测试与证据边界
summary: 源包记录七组 fixture 的逐字节 oracle 断言及错误路径锚点，但仅属结构性阅读，未核对上游字节或完成数学验收，预算耗尽与非 quasisplit 对偶形式仍缺测试锚点。
sources:
  - real-weyl.md
kind: concept
createdAt: "2026-10-09T15:08:24.716Z"
updatedAt: "2026-10-09T15:08:24.716Z"
tags:
  - 测试证据
  - oracle
  - 覆盖限制
aliases:
  - 实-weyl-层的-oracle-测试与证据边界
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 实 Weyl 层的 oracle 测试与证据边界

实 Weyl 层的 oracle 测试以固定 upstream 构建的输出为基准，通过逐字节断言覆盖实 Weyl 群与块稳定子的部分打印行为。现有源材料属于对 `crates/atlas-real-group/src/real_weyl.rs` 的结构性阅读，不构成该层的数学验收；其中 upstream 文件与行号均转录自代码注释，未核对上游字节。^[real-weyl.md:9-13, real-weyl.md:161-168]

## Oracle 来源与 fixture 范围

测试注释声明，探针 `/tmp/probe_rw_all.at` 对照固定 upstream 构建 `rev 4d3e9449`，其输出于 2026-08-11 重新生成，并逐字节复制进断言。以下七个 fixture 是该注释列出的覆盖范围；表中两个计数分别表示弱实形式数和 Cartan 类数。^[real-weyl.md:161-165]

| InnerClass fixture | 弱实形式数 | Cartan 类数 |
|---|---:|---:|
| SL(2,R) | 2 | 2 |
| SU(2,1) | 2 | 2 |
| SL(3,R) | 1 | 2 |
| Sp(4,R) | 3 | 4 |
| SL(2,C) | 1 | 1 |
| SL(4,R) | 2 | 3 |
| SL(6,R) | 2 | 4 |

## 关键测试锚点

**对偶子系统的 B/C 互换。** `Sp(4,R)` 的 `(2,3)` 测试锚定：在 C2 datum 上，Cartan #3 的输出为 `W^R is a Weyl group of type B2`。实现中，`real_type` 与 `real_compact_type` 使用转置的子系统 Cartan 矩阵；该断言覆盖了对偶侧类型计算在打印结果中的表现。相关构造见 [[实 Weyl 群与块稳定子的构造]]。^[real-weyl.md:66-70, real-weyl.md:165-166]

**复因子的打印前缀。** `SL(2,C)` 的输出锚定 `W^C is isomorphic to ... A1`，覆盖非平凡 `W^C` 所带的 `isomorphic to ` 前缀。完整格式约定见 [[实 Weyl 群打印的字节兼容契约]]。^[real-weyl.md:151-157, real-weyl.md:165-166]

**R-群核生成元的顺序。** `SL(6,R)` 的 rank-2 A 群输出锚定自由列升序的核生成元。实现为每个自由列生成一个位向量，其置位包含该自由列以及含有该自由位的主元行；每个核位向量再对应一个 Weyl 元素乘积，按 `orth` 坐标升序右乘。因此，这一锚点涉及生成元的可观测顺序。^[real-weyl.md:75-81, real-weyl.md:130-133, real-weyl.md:165-167]

**错误分类。** 已列出的错误路径锚点包括：形式在指定 Cartan 上无定义时返回 `RealFormNotDefinedOnCartan`，形式或 Cartan 编号越界时返回 `IndexOutOfRange`。这些锚点与 [[实 Weyl 群打印的前置检查]] 相关，但不覆盖全部构造失败路径。^[real-weyl.md:167-168, real-weyl.md:188-188]

## 字节契约与实现边界

打印层保留头行、摘要行数、生成元节标题的冒号差异以及空行布局。`format_word` 将空词打印为 `e`，非空词使用从 1 开始的编号并以逗号连接；`render` 要求每行终止，摘要与节之间恰好一个空行，末节之后无多余字节。这些格式细节属于逐字节兼容契约。^[real-weyl.md:149-157]

生成元打印使用 `WeylElement::canonical_word`，未移植 upstream 的 `reflection_word`／`to_dominant` 机制。模块还明确省略了打印末尾的调试尺寸断言及 Weyl 群阶计算，并未移植 `printDualRealWeyl` 包装。因此，解释测试范围时需要保留这些实现差异。^[real-weyl.md:75-81, real-weyl.md:170-175]

## 证据不能支持的结论

预算耗尽路径及非 quasisplit 对偶形式的行为没有测试锚点。[[对偶 Cartan fiber 链的临时重建]] 在每次 `real_weyl` 调用时执行且无缓存，但源材料仅将其列为性能线索，未确认该取舍是否有意，也不作性能声明。^[real-weyl.md:186-188]

源码阅读还记录了两个按构造不可达的 `.expect("checked cartan id")`，以及 `simple_complex` 中依赖 Dynkin 分类输出与根基长度一致的索引操作。这些属于阅读观察，不能视为已由 oracle 验证的安全性结论。^[real-weyl.md:183-185]

精确读取身份由 `snapshots/2026-10-06-real-weyl.json` 记录，绑定 Git base、文件字节 SHA-256 与草案调用记录。维护者对照源码逐条核对并改写了草案，但本次知识维护没有执行 Atlas、Cargo、测试或 benchmark；页面应区分代码中已有的测试断言、注释声明的 oracle 来源与本次实际完成的结构性阅读。^[real-weyl.md:190-196]

## Sources

- [real-weyl.md](real-weyl.md)：实 Weyl 群与块稳定子：real_weyl.rs 的构造、对偶 fiber 重放与打印层。
