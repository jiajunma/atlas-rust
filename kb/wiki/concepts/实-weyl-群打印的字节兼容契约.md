---
title: 实 Weyl 群打印的字节兼容契约
summary: 打印层固定分解头行、摘要、生成元节及冒号和空行规则，空词打印 e，非空词使用从 1 开始的逗号分隔编号。
sources:
  - real-weyl.md
kind: concept
createdAt: "2026-10-09T15:08:12.630Z"
updatedAt: "2026-10-09T22:45:40.054Z"
tags:
  - 打印契约
  - 兼容性
  - 实Weyl群
aliases:
  - 实-weyl-群打印的字节兼容契约
  - 实W群
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 实 Weyl 群打印的字节兼容契约
summary: 实 Weyl 群与块稳定子的打印契约固定头行、摘要、生成元规范词、冒号差异及换行布局；回归断言提供有限 fixture 的逐字节输出锚点。
sources:
  - real-weyl.md
kind: concept
tags:
  - 输出格式
  - 兼容性
  - 字节契约
aliases:
  - 实-weyl-群打印的字节兼容契约
  - 实W群
provenanceState: extracted
---

# 实 Weyl 群打印的字节兼容契约

[[实 Weyl 群与块稳定子的构造|实 Weyl 群与块稳定子]]的打印层以 `RealWeylPrint` 保存头行、摘要和生成元节，再由 `render()` 输出字符串。字节兼容契约涵盖固定文本、生成元词、标点、行终止符及空行布局，测试中的逐字节断言为这些可观测行为提供锚点。^[real-weyl.md:35-37, real-weyl.md:149-168]

## 固定头行与摘要

实 Weyl 群的头行固定为 `real weyl group is W^C.((A.W_ic) x W^R), where:`，随后有 4 行摘要。块稳定子的头行固定为 `block stabilizer is W^C.((A_i.W_ic) x (A_r.W_rc)), where:`，随后有 5 行摘要。`W^C` 非平凡时，其摘要带有 `isomorphic to ` 前缀，包括末尾空格。^[real-weyl.md:151-154]

子系统类型也是输出契约的一部分。`real_type` 和 `real_compact_type` 使用转置的子系统 Cartan 矩阵，B/C 互换由此进入输出：例如 C2 datum 上的 `Sp(4,R)` Cartan #3 打印 `W^R is a Weyl group of type B2`，测试以 `(2,3)` 为锚点。^[real-weyl.md:66-70]

## 生成元词、顺序与标点

生成元节保留上游不一致的冒号约定：`generators for A`、`generators for A_i` 和 `generators for A_r` 不带冒号，而 `W^C`、`W_ic`、`W^R` 和 `W_rc` 对应的生成元节标题带冒号。^[real-weyl.md:154-155]

`format_word` 将空词打印为 `e`；非空词使用从 1 开始的生成元编号，以逗号连接。打印读取 `WeylElement::canonical_word`；来源说明，生成元在构造时已具有与上游 `WeylGroup::word` 一致的规范词，因此没有移植 `reflection_word` 与 `to_dominant` 机制。相关背景见 [[Weyl 元素的规范词]]。^[real-weyl.md:75-81, real-weyl.md:156-157]

生成元的构造顺序也需保留。每个列出的根对应一个 Weyl 反射；R-群的每个核位向量对应一个反射乘积，按 `orth` 坐标的置位升序右乘，核生成元本身按自由列升序产生。复根生成元为 \(s_{rn}\cdot s_{\theta(rn)}\)，先构造根的反射，再右乘其对合像的反射。^[real-weyl.md:75-79, real-weyl.md:130-133]

## 换行与空行

`render()` 要求每行均有行终止符，摘要与生成元节之间恰好一个空行，末节之后没有多余字节。末行的行终止符属于契约，末节后的额外空行则不属于契约。^[real-weyl.md:156-157]

## 打印入口与编号

`real_weyl_print(form, cartan)` 与 `block_stabilizer_print(form, cartan, dual_form)` 接收外部形式编号，通过 `ExternalFormOrder` 转换为内部 `WeakRealFormId`；越界形式编号返回 `IndexOutOfRange`。块稳定子入口的 `dual_form` 使用对偶内类的外部编号。^[real-weyl.md:38-43, real-weyl.md:99-102]

实 Weyl 群打印使用对偶伴随 fiber 的零元作为 `y`，对应上游硬编码的 quasisplit 代表；块稳定子打印则使用指定对偶形式的代表元。形式在所选 Cartan 类上无定义时返回 `RealFormNotDefinedOnCartan`。^[real-weyl.md:85-93]

## 回归证据与适用边界

测试注释声明，探针 `/tmp/probe_rw_all.at` 对照固定上游构建 `rev 4d3e9449`，输出于 2026-08-11 重新生成并逐字节复制进断言。七个 fixture 为 `SL(2,R)`、`SU(2,1)`、`SL(3,R)`、`Sp(4,R)`、`SL(2,C)`、`SL(4,R)` 和 `SL(6,R)`。关键锚点包括 B/C 互换、`SL(2,C)` 的 `W^C is isomorphic to ... A1` 前缀，以及 `SL(6,R)` 的 rank-2 A 群自由列升序核生成元，详见 [[实 Weyl 层的 oracle 测试与证据边界]]。^[real-weyl.md:161-168]

当前实现未移植打印末尾的 `#ifndef NDEBUG` 尺寸断言及 `weylsize` 计算，也未移植 `printDualRealWeyl`，原因是没有内建包装使用后者。后者所需的虚根、实根生成元列表仍在计算，来源将补齐该功能描述为打印层改动。^[real-weyl.md:170-175]

这些记录属于结构性阅读与测试锚点说明，不代表实 Weyl 层已经完成数学验收。来源包未核对上游文件字节，预算耗尽路径及非 quasisplit 对偶形式行为没有测试锚点；该次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[real-weyl.md:9-13, real-weyl.md:179-188, real-weyl.md:192-196]

## Sources

- [real-weyl.md](../../sources/real-weyl.md) — 实 Weyl 群与块稳定子：real_weyl.rs 的构造、对偶 fiber 重放与打印层。
