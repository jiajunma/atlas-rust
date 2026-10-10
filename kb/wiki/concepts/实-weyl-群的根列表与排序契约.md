---
title: 实 Weyl 群的根列表与排序契约
summary: RealWeyl 统一保存 primal RootId，虚根与实根按 upstream RootNbr 排序，复根保留构造顺序，对偶根列表通过余根向量映回 primal。
sources:
  - real-weyl.md
kind: concept
createdAt: "2026-10-09T21:07:08.423Z"
updatedAt: "2026-10-09T21:07:08.423Z"
tags:
  - 实-Weyl-群
  - 根编号
  - 数据表示
aliases:
  - 实-weyl-群的根列表与排序契约
  - 实W群
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# 实 Weyl 群的根列表与排序契约

`RealWeyl` 的根列表统一保存原始根数据一侧（primal）的 `RootId`，但不同列表遵循不同的排序规则：`imaginary` 与 `real` 按上游 `RootNbr` 序排列，`complex` 保留 `makeSimpleComplex` 的输出顺序；对偶侧根列表则先通过余根向量对应映回 primal。列表次序还决定 R-群位向量的坐标含义及生成元乘积的构造顺序。^[real-weyl.md:58-64, real-weyl.md:75-81]

## 根编号与列表顺序

`imaginary` 与 `real` 保存简单虚根、简单实根，并经 `sort_by_upstream_key` 排序。排序键由根的高度（height）与简单根坐标的反向字典序组成，对应 `cartan_classification::upstream_positive_key` 所表达的上游 `RootNbr` 次序。^[real-weyl.md:58-60, real-weyl.md:94-97]

`complex` 是这一排序规则的例外：它保留 `simple_complex` 对应的 `makeSimpleComplex` 输出顺序。该算法先取与正虚根和、正实根和都正交的正根，求简单基并进行 Dynkin 分类；对合成对的分量只保留一个。具体删除规则是向后扫描，删除第一个含有与当前分量最低根之像非正交顶点的后续分量，每次只删除一个。^[real-weyl.md:60-61, real-weyl.md:139-142]

根基提取还保留 [[simple_basis 的提前终止扫描语义]]：若候选因反射结果非正而移除自身，整个外层扫描立即终止，后续候选不再检查。这里的调用方只传入正根。^[real-weyl.md:135-137]

## 对偶根映射与类型方向

对偶侧的 `real_compact` 与 `real_orth` 最终也使用 primal `RootId`。映射依据是“对偶根向量等于 primal 余根向量”：对偶侧完成 `fiber_side` 后，`primal_roots_of_dual` 通过这一对应关系寻找 primal 根；未找到对应根时，返回 `"dual root correspondence"` 不变量错误。^[real-weyl.md:58-62, real-weyl.md:94-96]

统一根编号不改变子系统的对偶配对方向。`real_type` 与 `real_compact_type` 使用 `subsystem_cartan(..., transposed=true)`，即取 primal 子系统 Cartan 矩阵的转置，因此会出现 B/C 类型互换；其余三个类型使用 `transposed=false`。来源中的测试锚点为 C2 datum 上的 `Sp(4,R)` Cartan #3，其输出包含 `W^R is a Weyl group of type B2`。^[real-weyl.md:66-70]

## `orth` 次序与 R-群坐标

两侧共用的 `fiber_side` 先按上游键排序正虚根，再判定紧性并提取紧根简单基。`orth` 由非紧且与 `two_rho` 正交的根组成；这些根强正交，构成 $A_1^n$。R-群位向量的每个坐标对应一个 `orth` 条目，因此必须连同根列表顺序解释。^[real-weyl.md:63-64, real-weyl.md:120-129]

`r_vectors` 将各个 `orth` 根的 `m_alpha`（fiber 坐标）按行注入 `ModTwoSubspace`，按自由列升序生成核基。每个核向量置位于该自由列，以及含有该自由列位的主元行对应位置；这一规则保留上游核生成元顺序，可结合 [[由自由坐标构造 F₂ 右核]] 阅读。^[real-weyl.md:130-133]

字段归属也需与坐标次序一起保留：`imaginary_r` 保存 primal 侧的 R-群向量，`real_r` 保存对偶侧的 `dual_side.r_vectors`。^[real-weyl.md:72-73]

## 从列表到生成元词

`RealWeylGenerators` 为每个列出的根构造一个反射元素，经过 `WeylAction::root_reflection` 与 `WeylElement::from_action` 转换。每个 R-群核向量对应一个乘积：按 `orth` 索引升序访问置位项，并依次右乘相应反射。复根生成元则按 $s_r s_{\theta(r)}$ 构造，即先取根反射，再右乘其对合像的反射。^[real-weyl.md:75-79]

生成元打印直接使用 `WeylElement::canonical_word`，关联 [[Weyl 元素的规范词]]；来源说明这些词按构造已经典范化，因此未移植 `reflection_word`／`to_dominant` 机制。打印时空词为 `e`，非空词使用从 1 开始的生成元编号，以逗号连接。^[real-weyl.md:79-81, real-weyl.md:156-157]

## 测试与证据边界

来源记载的逐字节输出锚点包括 `Sp(4,R)` 的 B/C 互换、`SL(2,C)` 的复因子摘要，以及 `SL(6,R)` 中秩为 2 的 A 群自由列升序核生成元。测试注释声明这些断言来自固定上游构建 `rev 4d3e9449` 的探针输出，于 2026-08-11 重新生成并复制。^[real-weyl.md:161-168]

本页依据 `real_weyl.rs` 的结构性阅读，不构成数学验收；上游行号转录自代码注释，来源维护未核对上游字节，也未执行 Atlas、Cargo、测试或 benchmark。预算耗尽路径与非 quasisplit 对偶形式的行为没有测试锚点。^[real-weyl.md:9-13, real-weyl.md:188-196]

## Sources

- [real-weyl.md](../../sources/real-weyl.md) — 实 Weyl 群与块稳定子：`real_weyl.rs` 的构造、对偶 fiber 重放与打印层。
