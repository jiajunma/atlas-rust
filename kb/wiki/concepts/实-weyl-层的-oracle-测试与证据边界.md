---
title: 实 Weyl 层的 oracle 测试与证据边界
summary: 来源记录七组 fixture 的逐字节 oracle 断言及部分错误路径，但本次仅作结构性阅读，未核对上游字节或完成数学验收，预算耗尽与非 quasisplit 对偶形式仍缺测试锚点。
sources:
  - real-weyl.md
kind: concept
createdAt: "2026-10-09T15:08:24.716Z"
updatedAt: "2026-10-09T22:45:41.325Z"
tags:
  - 回归测试
  - 证据边界
  - 兼容性
aliases:
  - 实-weyl-层的-oracle-测试与证据边界
  - 实W层O测
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 实 Weyl 层的 oracle 测试与证据边界
summary: 七组 fixture 提供逐字节输出断言与错误路径锚点；oracle 来源由测试注释声明，源包仅完成结构性阅读，预算耗尽与非 quasisplit 对偶形式仍缺测试锚点。
sources:
  - real-weyl.md
kind: concept
tags:
  - 测试证据
  - oracle
  - 覆盖限制
aliases:
  - 实-weyl-层的-oracle-测试与证据边界
provenanceState: extracted
---

# 实 Weyl 层的 oracle 测试与证据边界

实 Weyl 层以固定 upstream 构建的输出作为 oracle，通过逐字节断言覆盖实 Weyl 群与块稳定子的部分打印行为。源包对 `crates/atlas-real-group/src/real_weyl.rs` 的工作属于结构性阅读，不构成数学验收；其中的上游文件位置与对应关系转录自代码注释，未独立核对上游字节。^[real-weyl.md:9-13, real-weyl.md:159-168]

## Oracle 来源与 fixture 范围

测试注释声明，探针 `/tmp/probe_rw_all.at` 对照固定 upstream 构建 `rev 4d3e9449`，输出于 2026-08-11 重新生成并逐字节复制进断言。注释列出以下七个 fixture；两个计数分别表示弱实形式数和 Cartan 类数。^[real-weyl.md:161-165]

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

**对偶子系统的 B/C 互换。** `Sp(4,R)` 的 `(2,3)` 测试锚定：在 C2 datum 上，Cartan #3 打印 `W^R is a Weyl group of type B2`。实现中，`real_type` 与 `real_compact_type` 使用转置的子系统 Cartan 矩阵，B/C 互换由此进入输出；相关背景见 [[实 Weyl 群与块稳定子的构造]]。^[real-weyl.md:66-70, real-weyl.md:165-166]

**复因子的打印前缀。** `SL(2,C)` 的断言包含 `W^C is isomorphic to ... A1`，锚定非平凡 `W^C` 所带的 `isomorphic to ` 前缀。该细节属于 [[实 Weyl 群打印的字节兼容契约]]。^[real-weyl.md:151-157, real-weyl.md:165-166]

**R-群核生成元的顺序。** `SL(6,R)` 的 rank-2 A 群断言锚定自由列升序的核生成元。实现对每个自由列生成一个位向量，置位包括该自由列以及含有该自由位的主元行；每个核位向量对应一个 Weyl 元素乘积，按 `orth` 坐标升序右乘。^[real-weyl.md:75-81, real-weyl.md:130-133, real-weyl.md:165-167]

**错误分类。** 已列出的错误路径锚点包括：形式在指定 Cartan 上无定义时返回 `RealFormNotDefinedOnCartan`，形式或 Cartan 编号越界时返回 `IndexOutOfRange`。源包同时明确记录，预算耗尽路径与非 quasisplit 对偶形式的行为没有测试锚点。^[real-weyl.md:167-168, real-weyl.md:188-188]

## 字节契约与移植范围

打印层保留头行、摘要行数、节标题的冒号差异和空行布局：RealWeyl 有四行摘要，BlockStabilizer 有五行；`generators for A`、`A_i`、`A_r` 无冒号，而相应 Weyl 子群标题带冒号。空词打印为 `e`，非空词以从 1 开始的生成元编号用逗号连接；每行终止，摘要与节之间恰好一个空行，末节之后无多余字节。^[real-weyl.md:149-157]

生成元词按构造即为 canonical，打印使用 `WeylElement::canonical_word`，未移植上游 `reflection_word`／`to_dominant` 机制。模块还省略了打印末尾的调试尺寸断言及其 Weyl 群阶计算，并未移植 `printDualRealWeyl` 包装；后者所需的 imaginary／real 生成元列表仍在计算。^[real-weyl.md:75-81, real-weyl.md:170-175]

## 未覆盖行为与证据限制

[[对偶 Cartan fiber 链的临时重建]] 在每次 `real_weyl` 调用时执行且无缓存。源包未确认这一取舍是否有意，仅将其列为性能线索，不作性能声明；现有测试锚点也未覆盖预算耗尽与非 quasisplit 对偶形式的行为。^[real-weyl.md:104-116, real-weyl.md:186-188]

结构性阅读记录了两个按构造不可达的 `.expect("checked cartan id")`，以及 `simple_complex` 中依赖 Dynkin 分类输出与根基长度一致的索引操作。这些记录属于源码阅读观察，源包并未提供相应的安全性验收结论。^[real-weyl.md:179-185]

精确读取身份由 `snapshots/2026-10-06-real-weyl.json` 记录，绑定 Git base、文件字节 SHA-256 与 Kimi 调用记录。草案经过维护者对照源码逐条核对改写，但该次知识维护没有执行 Atlas、Cargo、测试或 benchmark。因此，已有断言、注释声明的 oracle 来源和实际完成的结构性阅读应分别理解，不能据此宣称实 Weyl 层已通过数学验收。^[real-weyl.md:9-13, real-weyl.md:190-196]

## Sources

- [real-weyl.md](../../sources/real-weyl.md)：实 Weyl 群与块稳定子：real_weyl.rs 的构造、对偶 fiber 重放与打印层。
