---
title: Weyl 群生成元的规范词构造
summary: 根反射、按 orth 坐标升序右乘的 R-群元素及复根反射乘积构成生成元，打印直接读取 canonical_word；上游一致性属于来源声明。
sources:
  - real-weyl.md
kind: concept
createdAt: "2026-10-09T15:07:49.296Z"
updatedAt: "2026-10-09T22:45:07.521Z"
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
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Weyl 群生成元的规范词构造
summary: 实 Weyl 层通过根反射、R-群反射乘积与复根反射乘积构造生成元，并直接使用 canonical_word 打印规范词，保留根列表、核生成元及乘法顺序。
sources:
  - real-weyl.md
kind: concept
tags:
  - Weyl群
  - 生成元
  - 规范词
aliases:
  - weyl-群生成元的规范词构造
provenanceState: extracted
---

# Weyl 群生成元的规范词构造

实 Weyl 层通过根反射及其乘积构造 `WeylElement`，再调用 `WeylElement::canonical_word` 输出生成元的规范词。源材料说明，这些词与上游经 `WeylGroup::word` 得到的 canonical word 相同，因此该模块未移植 `reflection_word`／`to_dominant` 机制。这一构造服务于[[实 Weyl 群与块稳定子的构造]]及其打印层。^[real-weyl.md:75-81]

## 生成元的构造

`RealWeylGenerators` 为每个列出的根构造一个 Weyl 元素：先由 `WeylAction::root_reflection` 得到根反射作用，再经 `WeylElement::from_action` 转为群元素。相关概念见 [[WeylAction 的对偶全格作用]]与 [[Weyl 元素的规范词]]。^[real-weyl.md:75-81]

R-群的每个核位向量对应一个反射乘积。构造时按 `orth` 下标升序遍历置位，并依次右乘相应根反射；因此，必须保留位向量坐标与 `orth` 根列表的对应关系，以及规定的右乘顺序。^[real-weyl.md:63-64, real-weyl.md:75-77]

复根对应的生成元为 $s_{rn}\,s_{\theta(rn)}$：先构造 `reflect(root)`，再右乘 `reflect(image)`，其中 `image` 是该根在对合下的像。打印使用所得群元素的规范词。^[real-weyl.md:77-81]

## 根列表与 R-群坐标

`RealWeyl` 的根列表保存原侧（primal）`RootId`。`imaginary` 与 `real` 按上游 `RootNbr` 顺序排列，排序键为高度及简单坐标的反向字典序；`complex` 则保留 `makeSimpleComplex` 的输出顺序。对偶侧的 `real_compact`、`real_orth` 通过余根向量映回原侧根，因为对偶根向量就是原侧余根向量。相关编号见 [[RootNumbering 根编号与 RootNbr 顺序]]。^[real-weyl.md:58-62]

单侧 fiber 构造中，`orth` 由非紧且与 `two_rho` 正交的根组成；这些根强正交，构成 $A_1^n$。每个 `orth` 根在 R-群位向量中占一个坐标。^[real-weyl.md:63-64, real-weyl.md:128-129]

`r_vectors` 将各 `orth` 根的 `m_alpha` 的 fiber 坐标按行注入 `ModTwoSubspace`，再按自由列升序构造核生成元。每个生成元的置位集合为 `[free] + {有 free 位的主元行}`，以保留上游核生成元的顺序。这里的自由列顺序决定生成元列表顺序，而单个向量的置位顺序决定反射乘积的构造顺序；参见 [[fiber grading 与 R-群核生成元]]。^[real-weyl.md:75-77, real-weyl.md:130-133]

两侧的归属需要区分：`imaginary_r` 保存原侧 R-群向量，`real_r` 保存对偶侧 R-群向量，即 `real_r: dual_side.r_vectors`。^[real-weyl.md:72-73]

## 规范词的打印

打印层直接读取 `WeylElement::canonical_word`。`format_word` 将空词打印为 `e`；非空词使用从 1 开始的生成元编号，并以逗号连接。完整输出要求每行终止、摘要与生成元节之间恰好一个空行、末节之后无多余字节，详见[[实 Weyl 群打印的字节兼容契约]]。^[real-weyl.md:79-81, real-weyl.md:151-157]

未移植 `reflection_word`／`to_dominant` 是模块声明的刻意偏差。另一个未移植入口是 `printDualRealWeyl`，原因是没有内建包装使用它；所需的 imaginary／real 生成元列表仍在计算，补齐该入口只涉及打印层。^[real-weyl.md:170-175]

## 测试证据与范围

测试注释声明，输出来自固定上游构建 `rev 4d3e9449`，于 2026-08-11 重新生成并逐字节复制进断言。七个 fixture 为 SL(2,R)、SU(2,1)、SL(3,R)、Sp(4,R)、SL(2,C)、SL(4,R) 和 SL(6,R)；其中 SL(6,R) 的输出锚定 rank-2 A 群按自由列升序排列的核生成元。^[real-weyl.md:161-168]

这些材料属于结构性源码阅读与测试锚点记录，不构成实 Weyl 层的数学验收。来源未核对上游字节，本次知识维护未执行 Atlas、Cargo、测试或 benchmark；预算耗尽路径与非 quasisplit 对偶形式的行为没有测试锚点。相关限制见[[实 Weyl 层的 oracle 测试与证据边界]]。^[real-weyl.md:9-13, real-weyl.md:177-188, real-weyl.md:192-196]

## Sources

- [real-weyl.md](../../sources/real-weyl.md) — 实 Weyl 群与块稳定子：real_weyl.rs 的构造、对偶 fiber 重放与打印层。
