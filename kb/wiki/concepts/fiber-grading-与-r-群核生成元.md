---
title: fiber grading 与 R-群核生成元
summary: fiber_side 结合基 grading 与模二平移识别紧根，并从强正交非紧根的 m_alpha 数据按自由列升序构造 R-群核生成元。
sources:
  - real-weyl.md
kind: concept
createdAt: "2026-10-09T15:07:49.305Z"
updatedAt: "2026-10-09T22:45:07.330Z"
tags:
  - 分级
  - R群
  - 模二线性代数
aliases:
  - fiber-grading-与-r-群核生成元
  - FG与R
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: fiber grading 与 R-群核生成元
summary: fiber_side 通过基 grading 与模二平移判定虚根紧性，从强正交非紧根的 m_alpha 数据按自由列升序构造 R-群核生成元；imaginary_r 属于原侧，real_r 属于对偶侧。
sources:
  - real-weyl.md
kind: concept
tags:
  - 纤维
  - R-群
  - 模二线性代数
aliases:
  - fiber-grading-与-r-群核生成元
---

# fiber grading 与 R-群核生成元

`fiber_side` 是[[实 Weyl 群与块稳定子的构造]]中原侧与对偶侧共用的辅助过程，输入为 `(root_system, involution, grading, element)`。它判定虚根的紧性，构造紧根简单基、强正交非紧根列表 `orth`，并生成以 `orth` 条目为坐标的 R-群核向量。^[real-weyl.md:118-133]

## 虚根 grading 与紧性判定

正虚根先按上游根编号键排序，即按高度与简单坐标的反向字典序排列。非紧性判定结合基 grading 的线性扩张与代表元给出的模二平移项。^[real-weyl.md:58-64, real-weyl.md:122-127]

基 grading 取 \(\langle\alpha,\rho^\vee_{\mathrm{im}}\rangle\) 的奇偶，其中 \(2\rho^\vee_{\mathrm{im}}\) 为正虚余根之和。实现要求相应的 \(\sum_\beta \operatorname{bracket}(\alpha,\beta)\) 为偶数；若为奇数，返回 `"imaginary-simple coordinates"` 不变量错误。平移项由 `parity_dot` 计算，是 ambient 代表与根的 datum-simple 坐标的模二点积。^[real-weyl.md:122-127]

紧根累加进 `two_rho_ic`，再通过 `simple_basis(compact)` 得到 `compact_basis`。`orth` 选取非紧且与 `two_rho` 正交的根；这些根强正交，构成 \(A_1^n\) 子系统。^[real-weyl.md:126-129]

`simple_basis` 保留上游的特殊扫描行为：候选因自身反射结果非正而被移除时，整个外层扫描立即终止，后续候选不再检查。调用方只传入正根。^[real-weyl.md:135-137]

## R-群核生成元与确定性顺序

`r_vectors` 将每个 `orth` 根的 `m_alpha`（fiber 坐标）按行注入 `ModTwoSubspace`，据此构造核生成元。输出位向量的每个坐标对应一个 `orth` 条目；相关坐标主题见 [[m_alpha 的模二归约与伴随投影]]。^[real-weyl.md:63-64, real-weyl.md:130-133]

核生成元按自由列升序产生。对每个自由列 `free`，置位该自由坐标，以及所有在 `free` 位上非零的主元行所对应的位置。这保留了来源所述上游 `BitMatrix::kernel` 的生成元顺序。^[real-weyl.md:130-133]

构造 `RealWeylGenerators` 时，每个核向量对应一个 Weyl 元素乘积：按 `orth` 索引升序遍历置位，依次右乘相应根反射。根反射经 `WeylAction::root_reflection` 与 `WeylElement::from_action` 构造；打印采用 `WeylElement::canonical_word`，参见 [[Weyl 元素的规范词]]。^[real-weyl.md:75-81]

## 原侧与对偶侧的归属

`imaginary_r` 保存原侧 R-群向量，`real_r` 保存 `dual_side.r_vectors`，即对偶侧向量。`RealWeyl` 的根列表统一保存原侧 `RootId`：对偶侧的 `real_compact` 与 `real_orth` 通过余根向量映回原侧，因为对偶根向量就是原侧余根向量；核向量的坐标仍对应相应 `orth` 列表的条目。^[real-weyl.md:58-73]

两侧共用 `fiber_side`，但数据来源不同。原侧读取已经过预算约束的 `classification`；对偶侧在每次 `real_weyl` 调用时临时重建整条 fiber 链，无缓存，并使用 `RealWeylContext.budget` 的相应子预算。详见 [[对偶 Cartan fiber 链的临时重建]]。^[real-weyl.md:104-120]

## 测试与证据边界

来源记录的逐字节输出测试包含 SL(6,R) 的 rank-2 A 群，锚定按自由列升序排列的核生成元。测试注释声明 oracle 来自固定上游构建 `rev 4d3e9449`，输出于 2026-08-11 重新生成并逐字节复制进断言；相关范围见 [[实 Weyl 层的 oracle 测试与证据边界]]。^[real-weyl.md:159-168]

本页依据结构性源码阅读，不构成实 Weyl 层的数学正确性验收。来源中的上游行号转录自代码注释，未核对上游字节；预算耗尽路径与非 quasisplit 对偶形式的行为没有测试锚点。对偶链无缓存仅是已知性能线索，来源未给出性能结论。^[real-weyl.md:9-13, real-weyl.md:177-188]

## Sources

- [real-weyl.md](../../sources/real-weyl.md) — 实 Weyl 群与块稳定子：real_weyl.rs 的构造、对偶 fiber 重放与打印层。
