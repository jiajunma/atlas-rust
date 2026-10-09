---
title: fiber grading 与 R-群核生成元
summary: fiber_side 通过基 grading 和模二平移区分紧根与非紧根，从强正交非紧根的 m_alpha 数据求核，并按自由列升序产生 R-群位向量；real_r 属于对偶侧。
sources:
  - real-weyl.md
kind: concept
createdAt: "2026-10-09T15:07:49.305Z"
updatedAt: "2026-10-09T15:07:49.305Z"
tags:
  - fiber
  - R群
  - 模二线性代数
aliases:
  - fiber-grading-与-r-群核生成元
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# fiber grading 与 R-群核生成元

`fiber_side` 是实 Weyl 群构造中供原侧与对偶侧共用的辅助过程。它接收 `(root_system, involution, grading, element)`，通过虚根的 grading 判定紧性，构造紧根简单基、强正交非紧根列表 `orth`，以及以 `orth` 为坐标的 R-群核生成元。^[real-weyl.md:118-133]

## 虚根 grading 与紧性判定

正虚根先按 upstream 键排序；该键采用根的高度与简单坐标的反向字典序。非紧性判定使用基 grading 的线性扩张，并叠加 fiber 元素给出的模二平移项。^[real-weyl.md:58-64, real-weyl.md:122-127]

基 grading 取配对 \(\langle\alpha,\rho^\vee_{\mathrm{im}}\rangle\) 的奇偶，其中 \(2\rho^\vee_{\mathrm{im}}\) 是正虚余根之和。实现要求相应的 \(\sum_\beta \operatorname{bracket}(\alpha,\beta)\) 为偶数；若为奇数，返回 `"imaginary-simple coordinates"` 不变量错误。平移项由 ambient 代表与根的 datum-simple 坐标作 mod-2 点积得到，对应辅助函数 `parity_dot`。^[real-weyl.md:122-127]

判定为紧的根累加到 `two_rho_ic`，紧根集合经 `simple_basis(compact)` 得到 `compact_basis`。`orth` 则选取非紧且与 `two_rho` 正交的根；这些根强正交，构成 \(A_1^n\) 子系统。^[real-weyl.md:126-129]

`simple_basis` 保留了上游的特殊扫描行为：当候选因自身反射结果非正而被移除时，整个外层扫描终止，后续候选不再检查。这里的调用方只传入正根。^[real-weyl.md:135-137]

## R-群核生成元及其顺序

`r_vectors` 的构造把每个 `orth` 根的 `m_alpha`（fiber 坐标）按行注入 `ModTwoSubspace`。因此，[[m_alpha 的模二归约与伴随投影]]提供的坐标参与核计算，而输出核向量的每个坐标对应一个 `orth` 条目。^[real-weyl.md:63-64, real-weyl.md:130-133]

核生成元按自由列的升序生成。对每个自由列 `free`，生成元置位于 `[free]`，并加入所有在 `free` 位上非零的主元行所对应的位置。这保留了 upstream `BitMatrix::kernel` 的生成元顺序；核基的排列也是兼容行为的一部分。^[real-weyl.md:130-133]

构造 `RealWeylGenerators` 时，每个 R-群核向量对应一个 Weyl 元素乘积：按 `orth` 索引升序遍历置位，并依次右乘相应根反射。打印使用 `WeylElement::canonical_word`，可结合[[Weyl 元素的规范词]]理解最终生成元词的表示。^[real-weyl.md:75-81]

## 原侧与对偶侧的归属

`RealWeyl` 中 R-群字段的归属需要区分两侧：`imaginary_r` 保存原侧的 R-群向量，而 `real_r` 保存 `dual_side.r_vectors`。对偶侧的 `real_compact` 与 `real_orth` 根列表则通过余根向量映回原侧 `RootId`；对偶根向量就是原侧余根向量。^[real-weyl.md:58-73]

两侧共用 `fiber_side`，但对偶 fiber 链在每次 `real_weyl` 调用时临时重建且无缓存；原侧读取已有、经过预算约束的分类数据。相关构造背景见[[对偶 Cartan fiber 链的临时重建]]与[[实 Weyl 群与块稳定子的构造]]。^[real-weyl.md:104-120]

## 测试与证据边界

来源记录的逐字节输出测试包含 SL(6,R) 的 rank-2 A 群，专门覆盖按自由列升序排列的核生成元。测试注释声明 oracle 来自固定 upstream 构建 `rev 4d3e9449`，输出于 2026-08-11 重新生成并复制进断言。^[real-weyl.md:159-168]

这些内容属于结构性源码阅读，不构成数学正确性验收；上游行号引用来自代码注释，来源包未核对上游字节。预算耗尽路径与非 quasisplit 对偶形式行为没有测试锚点，证据范围可参见[[实 Weyl 层的 oracle 测试与证据边界]]。^[real-weyl.md:177-188]

## Sources

- [real-weyl.md](real-weyl.md) — 实 Weyl 群与块稳定子：real_weyl.rs 的构造、对偶 fiber 重放与打印层。
