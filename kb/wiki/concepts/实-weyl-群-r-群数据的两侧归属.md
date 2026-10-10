---
title: 实 Weyl 群 R-群数据的两侧归属
summary: RealWeyl 的 real_r 保存对偶侧 R-群向量，imaginary_r 保存 primal 侧向量；每个位向量的坐标对应所属侧有序 orth 根列表。
sources:
  - real-weyl.md
kind: concept
createdAt: "2026-10-10T00:48:33.123Z"
updatedAt: "2026-10-10T00:48:33.123Z"
tags:
  - 实Weyl群
  - R群
  - 对偶
aliases:
  - 实-weyl-群-r-群数据的两侧归属
  - 实W群R
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

# 实 Weyl 群 R-群数据的两侧归属

`RealWeyl` 的 R-群数据按 primal 与对偶两侧分别计算：`imaginary_r` 保存 primal 侧的 R-群向量，`real_r` 保存对偶侧的 `dual_side.r_vectors`。理解这些字段时，必须同时追踪计算所在的一侧及其对应的 `orth` 根列表。^[real-weyl.md:63-73]

## 两侧计算与根编号

两侧共用 `fiber_side(root_system, involution, grading, element)`。primal 侧使用实形代表 `x`；对偶侧使用代表 `y`：未指定对偶实形时，`y` 为对偶伴随 fiber 的零元，即上游硬编码的 quasisplit 代表；指定对偶实形时，则通过对偶标签定位并取得代表元。参见 [[实 Weyl 群与块稳定子的代表元选择]]。^[real-weyl.md:85-96, real-weyl.md:118-120]

`RealWeyl` 中的根列表统一保存 **primal `RootId`**。因此，对偶侧计算得到的 `real_compact` 与 `real_orth` 必须通过余根向量映回 primal：对偶根向量就是 primal 余根向量。若映射失败，构造返回 `"dual root correspondence"` 不变量错误。根编号统一并不改变 `real_r` 来自对偶侧这一归属。^[real-weyl.md:58-73, real-weyl.md:94-96]

对偶侧还影响子系统类型：`real_type` 与 `real_compact_type` 使用转置的子系统 Cartan 矩阵，B/C 类型互换由此进入。这与根列表最终保存 primal 编号是不同层面的约定。^[real-weyl.md:66-70]

## R-群位向量的坐标与顺序

`fiber_side` 先按 grading 判定虚根的紧性，累加紧根得到 `two_rho_ic`，并求出紧根简单基 `compact_basis`；`orth` 则选取非紧且与 `two_rho` 正交的根。来源将这些根描述为强正交根，构成 $A_1^n$。^[real-weyl.md:122-129]

每个 R-群位向量的一个坐标对应一个 `orth` 条目。计算 `r_vectors` 时，将各 `orth` 根的 `m_alpha` fiber 坐标按行注入 `ModTwoSubspace`，再构造核生成元。每个自由列按升序产生一个生成元，置位集合为 `[free]` 加上所有含该自由列位的主元行；这一顺序复现上游核生成元顺序。相关背景见 [[m_alpha 的模二归约与伴随投影]]。^[real-weyl.md:63-64, real-weyl.md:130-133]

因此，R-群向量的解释依赖所属侧的 `orth` 列表及其顺序；它的坐标含义是这些根的选择位，而不是独立于根列表的 ambient 坐标。^[real-weyl.md:63-64, real-weyl.md:130-133]

## 从位向量到 Weyl 生成元

`RealWeylGenerators` 为每个 R-群核位向量构造一个 Weyl 元素：按 `orth` 下标升序遍历置位，将相应根反射依次右乘。单个根反射通过 `WeylAction::root_reflection` 与 `WeylElement::from_action` 构造，最终打印 `WeylElement::canonical_word`。这使位坐标顺序直接参与生成元乘积的构造。参见 [[Weyl 元素的规范词]]。^[real-weyl.md:75-81]

## 对偶 fiber 的来源与预算

对偶侧的数据不能直接复用对偶分类存储中的 fiber，因为所需的对偶 Cartan 对合 $-\theta$ 一般只是典范对偶 Cartan 代表元的共轭。每次 `real_weyl` 调用都会临时重建对偶 fiber、grading、弱实形式分区及标签链，不使用缓存；各步使用 `RealWeylContext.budget` 的相应子预算，primal 侧则读取已有的 `classification`。详见 [[对偶 Cartan fiber 链的临时重建]]。^[real-weyl.md:104-116, real-weyl.md:186-187]

## 测试与证据边界

测试注释记载，逐字节输出断言来自固定上游构建 `rev 4d3e9449` 的探针输出。其中，`SL(6,R)` 锚定 rank-2 A 群的自由列升序核生成元；`Sp(4,R)` 的 `(2,3)` 用例锚定 C2 datum 上实根子系统打印为 B2 的转置配对行为。预算耗尽路径及非 quasisplit 对偶形式的行为没有测试锚点。^[real-weyl.md:161-168, real-weyl.md:188-188]

本页依据 `real_weyl.rs` 的结构性阅读，不构成数学验收。来源中的上游行号仅转录自代码注释，未核对上游字节；本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[real-weyl.md:9-13, real-weyl.md:192-196]

## Sources

- [real-weyl.md](../../sources/real-weyl.md) — 实 Weyl 群与块稳定子：`real_weyl.rs` 的构造、对偶 fiber 重放与打印层。
