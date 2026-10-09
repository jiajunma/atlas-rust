---
title: 实 Weyl 群打印的字节兼容契约
summary: 打印层固定群分解头行、摘要、生成元节、冒号差异及换行布局，空词输出 e，非空词使用从 1 开始的逗号分隔编号。
sources:
  - real-weyl.md
kind: concept
createdAt: "2026-10-09T15:08:12.630Z"
updatedAt: "2026-10-09T15:08:12.630Z"
tags:
  - 输出格式
  - 兼容性
  - 字节契约
aliases:
  - 实-weyl-群打印的字节兼容契约
  - 实W群
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 实 Weyl 群打印的字节兼容契约

实 Weyl 群与块稳定子的打印层通过 `RealWeylPrint` 保存头行、摘要与生成元节，并由 `render()` 输出字符串。兼容契约涵盖固定文本、生成元词、标点、换行和空行；源码中的测试以逐字节断言锚定输出。^[real-weyl.md:35-37, real-weyl.md:149-168]

## 固定头行与摘要

实 Weyl 群的头行如下，随后有 4 行摘要：^[real-weyl.md:151-154]

```text
real weyl group is W^C.((A.W_ic) x W^R), where:
```

块稳定子的头行如下，随后有 5 行摘要：^[real-weyl.md:151-154]

```text
block stabilizer is W^C.((A_i.W_ic) x (A_r.W_rc)), where:
```

`W^C` 非平凡时，其摘要包含 `isomorphic to ` 前缀。子系统类型也属于可观测输出：`real_type` 与 `real_compact_type` 使用转置的子系统 Cartan 矩阵，因此可能发生 B/C 互换；例如 C2 datum 上的 `Sp(4,R)` Cartan #3 打印 `W^R is a Weyl group of type B2`。^[real-weyl.md:66-70, real-weyl.md:153-154]

## 生成元词与标点

生成元节保留上游的冒号差异：`generators for A`、`generators for A_i`、`generators for A_r` 不带冒号，而 `W^C:`、`W_ic:`、`W^R:`、`W_rc:` 对应的节标题带冒号。统一这些标点会改变约定的输出字节。^[real-weyl.md:154-155]

`format_word` 将空词打印为 `e`；非空词使用从 1 开始的生成元编号，以逗号连接。打印使用 `WeylElement::canonical_word`，与上游通过 `WeylGroup::word` 得到的规范词一致；实现因此没有移植 `reflection_word` 与 `to_dominant` 机制。相关概念见 [[Weyl 元素的规范词]]。^[real-weyl.md:75-81, real-weyl.md:156-157]

生成元的构造顺序同样影响输出。R-群的每个核位向量对应一个反射乘积，按 `orth` 坐标的置位升序右乘；核生成元按自由列升序生成。复根生成元则为 \(s_{rn}\cdot s_{\theta(rn)}\)，先构造根的反射，再右乘其对合像的反射。^[real-weyl.md:75-79, real-weyl.md:130-133]

## 换行与空行

`render()` 要求每行均有行终止符，摘要与生成元节之间恰好一个空行，末节之后没有多余字节。因此，末行的行终止符属于契约，而末节后的额外空行不属于契约。^[real-weyl.md:156-157]

## 回归证据与适用边界

测试注释声明，探针 `/tmp/probe_rw_all.at` 对照固定上游构建 `rev 4d3e9449`，输出于 2026-08-11 重新生成并逐字节复制到断言中。七个 fixture 覆盖 `SL(2,R)`、`SU(2,1)`、`SL(3,R)`、`Sp(4,R)`、`SL(2,C)`、`SL(4,R)` 与 `SL(6,R)`；关键锚点包括 B/C 互换、`SL(2,C)` 的 `W^C is isomorphic to ... A1` 前缀，以及 `SL(6,R)` 的 rank-2 A 群自由列升序核生成元。参见 [[实 Weyl 层的 oracle 测试与证据边界]]。^[real-weyl.md:161-168]

当前实现未移植打印末尾的 `#ifndef NDEBUG` 尺寸断言及 `weylsize` 计算，也未移植 `printDualRealWeyl`，因为没有内建包装使用后者；其所需的虚根、实根生成元列表仍然计算。^[real-weyl.md:170-175]

这些内容是结构性阅读与测试锚点的记录，不代表实 Weyl 层已完成数学验收。来源包未核对上游文件字节，预算耗尽路径和非 quasisplit 对偶形式行为也没有测试锚点；本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[real-weyl.md:9-13, real-weyl.md:179-188, real-weyl.md:192-196]

## Sources

- [real-weyl.md](real-weyl.md) — 实 Weyl 群与块稳定子：real_weyl.rs 的构造、对偶 fiber 重放与打印层。
