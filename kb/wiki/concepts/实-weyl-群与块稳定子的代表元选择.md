---
title: 实 Weyl 群与块稳定子的代表元选择
summary: 两者共用实形代表 x；实 Weyl 群使用对偶伴随 fiber 零元作为 y，块稳定子使用指定对偶实形代表，打印入口先转换外部形式编号。
sources:
  - real-weyl.md
kind: concept
createdAt: "2026-10-09T21:07:07.662Z"
updatedAt: "2026-10-09T21:07:07.662Z"
tags:
  - 实-Weyl-群
  - 块稳定子
  - 实形式
aliases:
  - 实-weyl-群与块稳定子的代表元选择
  - 实W群
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# 实 Weyl 群与块稳定子的代表元选择

实 Weyl 群与块稳定子的构造共用 `RealWeylContext::real_weyl`，以 Cartan 类、实形式及可选的对偶实形式确定代表元 \(x\) 与 \(y\)。两者的区别在于对偶侧：实 Weyl 群使用对偶伴随 fiber 的零元，块稳定子使用指定对偶实形式的代表元。^[real-weyl.md:17-25, real-weyl.md:38-43, real-weyl.md:85-97]

## 实形式编号与 primal 代表元

打印接口接收外部形式编号，并通过 `ExternalFormOrder` 转换为内部 `WeakRealFormId`；编号越界返回 `IndexOutOfRange`。`block_stabilizer_print` 的 `dual_form` 属于**对偶内类**的外部编号体系。相关编号约定见 [[弱实形式的外部编号与严格排序]]。^[real-weyl.md:99-102]

进入 `real_weyl` 后，构造首先查找指定 Cartan 类；类不存在时返回 `IndexOutOfRange { index: cartan.0, .. }`。随后查找实形式在该类标签表中的局部位置；若不存在，则返回 `RealFormNotDefinedOnCartan`，对应包装层的 “Cartan class not defined for real form”。^[real-weyl.md:83-87]

primal 代表元由 `x = partition.class_representative(局部位置)` 取得。因此，代表元选择依赖该 Cartan 类中的局部标签位置；若标签定位成功却无法取得代表元，则返回 `CartanClassificationInvariantViolation("real-form representative")`。这一过程与 [[弱实形式的局部到全局标签映射]] 相衔接。^[real-weyl.md:85-89]

## 对偶代表元的两种选择

取得 \(x\) 后，构造先重建当前 Cartan 类对应的对偶 fiber 链，再依据 `dual_form: Option<WeakRealFormId>` 选择 \(y\)。当参数为 `None` 时，\(y\) 取对偶伴随 fiber 的零元；来源将其对应到上游硬编码的 quasisplit 代表。这是实 Weyl 群打印所用的选择。^[real-weyl.md:22-24, real-weyl.md:90-91]

当参数为 `Some(dual_form)` 时，构造在对偶标签中定位指定形式，再提取其代表元。这是块稳定子打印所用的选择。若对偶标签缺失，返回 `RealFormNotDefinedOnCartan`；若代表元缺失，则返回带 `"dual real-form representative"` 的不变量错误。^[real-weyl.md:22-24, real-weyl.md:91-93]

## 为什么重建对偶 fiber

对偶 Cartan 对合 \(-\theta\) 一般仅与典范对偶 Cartan 代表元共轭，因此实现不能直接复用对偶分类存储中的 fiber。每次调用都从 `dual_twisted_representative` 出发，以对偶 Weyl 群的最长元作右侧补乘，临时重建与当前代表元相配的对偶链。详见 [[对偶 Cartan fiber 链的临时重建]] 与 [[跨对偶的 Cartan 类对应与扭曲代表元]]。^[real-weyl.md:104-110]

重建依次经过 `CartanFiber`、`AdjointCartanFiber`、`CartanGradingData`、`WeakRealFormPartition`、`CayleyCrossDecomposition` 和 `RealFormLabels`；缺少对偶 fundamental 类时返回 `"dual fundamental class"` 不变量错误。各阶段使用 `RealWeylContext.budget` 的相应子预算，而 primal 侧读取已经预算构造的分类数据。该链目前无缓存。^[real-weyl.md:109-116]

## 代表元如何进入后续构造

两侧共用 `fiber_side(root_system, involution, grading, element)`。非紧虚根判定包含由 ambient 代表元与根的 datum-simple 坐标计算的模二点积平移项，因此代表元进入 grading 求值，继而参与紧根基、正交非紧根集合及 R-群向量的构造；参见 [[虚根的 noncompact grading]]。^[real-weyl.md:118-133]

对偶侧得到的根通过余根向量对应映回 primal `RootId`，对应失败返回 `"dual root correspondence"` 不变量错误。最终 `real_r` 保存对偶侧的 R-群向量，`imaginary_r` 保存 primal 侧的向量；解释代表元与结果的关系时，需要保留这一侧别约定。^[real-weyl.md:58-64, real-weyl.md:72-73, real-weyl.md:94-97]

## 测试与证据边界

来源记录的测试锚点覆盖形式在指定 Cartan 类上无定义时的 `RealFormNotDefinedOnCartan`，以及形式或 Cartan 编号越界时的 `IndexOutOfRange`。但预算耗尽路径和非 quasisplit 对偶形式的行为没有测试锚点，不能据此认定所有对偶代表元选择均已得到验证。^[real-weyl.md:167-168, real-weyl.md:188-188]

本说明依据 `real_weyl.rs` 的结构性阅读，不构成实 Weyl 层的数学验收。来源中的上游行号转录自代码文档注释，未核对上游字节；本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[real-weyl.md:9-13, real-weyl.md:192-196]

## Sources

- [real-weyl.md](../../sources/real-weyl.md)：实 Weyl 群与块稳定子：real_weyl.rs 的构造、对偶 fiber 重放与打印层。
