---
title: Weyl 群生成元的规范词构造
summary: 根反射、按 orth 坐标升序右乘的 R-群元素及复根反射乘积共同构成生成元，打印直接使用 canonical_word，无需移植上游词转换机制。
sources:
  - real-weyl.md
kind: concept
createdAt: "2026-10-09T15:07:49.296Z"
updatedAt: "2026-10-09T15:07:49.296Z"
tags:
  - Weyl群
  - 生成元
  - 规范词
aliases:
  - weyl-群生成元的规范词构造
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Weyl 群生成元的规范词构造

实 Weyl 层通过根反射及其乘积构造 `WeylElement`，再用 `WeylElement::canonical_word` 输出生成元的规范词。源材料说明，该词与 upstream 经 `WeylGroup::word` 得到的 canonical word 相同，因此 Rust 实现未移植 `reflection_word`／`to_dominant` 机制。该过程服务于[[实 Weyl 群与块稳定子的构造]]及其打印层。^[real-weyl.md:75-81]

## 生成元的构造

`RealWeylGenerators` 为每个列出的根构造一个 Weyl 元素：先调用 `WeylAction::root_reflection` 得到根反射作用，再经 `WeylElement::from_action` 转为群元素。这连接了 [[WeylAction 的对偶全格作用]]与 [[Weyl 元素的规范词]]。^[real-weyl.md:75-81]

R-群的每个核位向量对应一个生成元乘积。构造时按 `orth` 下标升序遍历置位，并依次右乘相应根反射。因此，位向量的坐标顺序也决定了乘积的构造顺序。^[real-weyl.md:63-64, real-weyl.md:75-77]

复根对应的生成元为 \(s_{rn}\,s_{\theta(rn)}\)：先构造 `reflect(root)`，再右乘 `reflect(image)`。这个顺序是源码规定的构造顺序。^[real-weyl.md:77-81]

## 根列表与 R-群坐标顺序

`RealWeyl` 的根列表统一保存 primal `RootId`。`imaginary` 与 `real` 按 upstream `RootNbr` 顺序排列，排序键为高度及简单坐标的反向字典序；`complex` 则保留 `makeSimpleComplex` 的输出顺序。对偶侧的 `real_compact`、`real_orth` 通过余根向量映回 primal 根。相关编号背景见 [[RootNumbering 根编号与 RootNbr 顺序]]。^[real-weyl.md:58-62]

每个 `orth` 根占据 R-群位向量的一个坐标。构造核生成元时，按自由列升序，为每个自由列生成置位集合 `[free] + {有 free 位的主元行}`，从而保留 upstream 核生成元的顺序。这里 `imaginary_r` 来自 primal 侧，而 `real_r` 来自对偶侧；这些向量与其对应的 `orth` 坐标必须配套解释。^[real-weyl.md:63-64, real-weyl.md:72-73, real-weyl.md:130-133]

## 规范词的打印

打印层读取 `WeylElement::canonical_word`，由 `format_word` 将空词表示为 `e`，非空词则使用从 1 开始的生成元编号，并以逗号连接。完整输出还规定每行终止、摘要与生成元节之间恰好一个空行，以及末节之后无多余字节；详见[[实 Weyl 群打印的字节兼容契约]]。^[real-weyl.md:79-81, real-weyl.md:149-157]

## 测试证据与范围

测试注释声明，断言中的输出取自固定 upstream 构建 `rev 4d3e9449`，于 2026-08-11 重新生成并逐字节复制。七个 fixture 包括 SL(2,R)、SU(2,1)、SL(3,R)、Sp(4,R)、SL(2,C)、SL(4,R) 和 SL(6,R)；其中 SL(6,R) 的输出锚定了 rank-2 A 群按自由列升序排列的核生成元。^[real-weyl.md:159-168]

这些材料属于结构性源码阅读及测试锚点记录，不构成实 Weyl 层的数学验收。来源未核对 upstream 字节，本次知识维护也未执行测试或 benchmark；预算耗尽路径与非 quasisplit 对偶形式的行为没有测试锚点。相关限制见[[实 Weyl 层的 oracle 测试与证据边界]]。^[real-weyl.md:9-13, real-weyl.md:177-188, real-weyl.md:192-196]

## Sources

- [real-weyl.md](real-weyl.md)
