---
title: 实 Weyl 群与块稳定子的代表元选择
summary: 两者共享原侧实形代表 x；实 Weyl 群的 y 取对偶伴随 fiber 零元，块稳定子的 y 取指定对偶实形代表。
sources:
  - real-weyl.md
kind: concept
createdAt: "2026-10-09T21:07:07.662Z"
updatedAt: "2026-10-10T00:48:05.242Z"
tags:
  - 实Weyl群
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
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 实 Weyl 群与块稳定子的代表元选择
summary: 两者共用实形代表 x；实 Weyl 群以对偶伴随 fiber 零元为 y，块稳定子使用指定对偶实形代表，打印入口先转换外部形式编号。
sources:
  - real-weyl.md
kind: concept
tags:
  - 实-Weyl-群
  - 块稳定子
  - 实形式
aliases:
  - 实-weyl-群与块稳定子的代表元选择
  - 实W群
provenanceState: extracted
---

# 实 Weyl 群与块稳定子的代表元选择

实 Weyl 群与块稳定子共用 `RealWeylContext::real_weyl` 构造入口，以 Cartan 类、实形式及可选的对偶实形式确定代表元 $x$ 与 $y$。两者共用 primal 侧实形代表 $x$，区别在于对偶侧：实 Weyl 群取对偶伴随 fiber 的零元作为 $y$，块稳定子取指定对偶实形式的代表元。^[real-weyl.md:17-25, real-weyl.md:38-43, real-weyl.md:85-97]

## 编号转换与实形代表元

打印接口接收外部形式编号，通过 `ExternalFormOrder` 转换为内部 `WeakRealFormId`；形式编号越界返回 `IndexOutOfRange`。`block_stabilizer_print` 的 `dual_form` 属于**对偶内类**的外部编号体系，相关约定见 [[弱实形式的外部编号与严格排序]]。^[real-weyl.md:99-102]

进入 `real_weyl` 后，首先查找指定 Cartan 类；类不存在时返回 `IndexOutOfRange { index: cartan.0, .. }`。随后查找实形式在该类标签表中的局部位置；若没有对应位置，返回 `RealFormNotDefinedOnCartan`，对应包装层的 “Cartan class not defined for real form”。^[real-weyl.md:83-87]

代表元由 `x = partition.class_representative(局部位置)` 取得，因此选择依据是该 Cartan 类标签表中的局部位置。若标签定位成功但代表元缺失，则返回 `CartanClassificationInvariantViolation("real-form representative")`。相关标签关系可结合 [[弱实形式的局部到全局标签映射]] 阅读。^[real-weyl.md:85-89]

## 对偶代表元的两种选择

取得 $x$ 后，构造先调用 `dual_side(cartan_class)` 重建对偶 fiber 链，再根据 `dual_form: Option<WeakRealFormId>` 选择 $y$。当参数为 `None` 时，取对偶伴随 fiber 的零元；来源将其对应到上游硬编码的 quasisplit 代表。这是实 Weyl 群打印使用的选择。^[real-weyl.md:22-24, real-weyl.md:88-91]

当参数为 `Some(dual_form)` 时，先在对偶标签中定位指定形式，再取得其代表元。这是块稳定子打印使用的选择。对偶标签定位失败返回 `RealFormNotDefinedOnCartan`；代表元缺失则返回带 `"dual real-form representative"` 的不变量错误。^[real-weyl.md:22-24, real-weyl.md:91-93]

## 对偶 fiber 为何需要重建

对偶 Cartan 对合 $-\theta$ 一般仅与典范对偶 Cartan 代表元共轭，因此实现不能直接复用对偶分类存储的 fiber。每次调用都从 `dual_twisted_representative` 出发，以 `longest_action(对偶, weyl_budget)` 右补最长元，再临时重建整条链；详见 [[对偶 Cartan fiber 链的临时重建]]。^[real-weyl.md:104-110]

重建顺序为 `CartanFiber::build` → `AdjointCartanFiber::build` → `CartanGradingData::build` → `WeakRealFormPartition::build` → `CayleyCrossDecomposition::build` → `RealFormLabels::build`。缺少对偶 fundamental 类时返回 `"dual fundamental class"` 不变量错误。各阶段使用 `RealWeylContext.budget` 的相应子预算，primal 侧则读取已经预算构造的 `classification`。^[real-weyl.md:110-116]

该对偶链目前每次调用都会重建，没有缓存。来源未确认这一取舍是否有意，也未据此提出性能结论。^[real-weyl.md:186-187]

## 代表元对后续构造的影响

两侧共用 `fiber_side(root_system, involution, grading, element)`。非紧虚根判定的 grading 包含一个平移项：ambient 代表元与根的 datum-simple 坐标之间的模二点积 `parity_dot`。这一判定随后用于构造紧根基、正交非紧根集合 `orth`，以及由各 `orth` 根的 `m_alpha` 计算的 R-群核向量。^[real-weyl.md:118-133]

对偶侧的根列表通过余根向量对应映回 primal `RootId`；对应失败返回 `"dual root correspondence"` 不变量错误。最终 `real_r` 保存**对偶侧**的 R-群向量，`imaginary_r` 保存 **primal 侧**的向量，解释输出时必须保留这一归属约定，参见 [[实 Weyl 群 R-群数据的两侧归属]]。^[real-weyl.md:58-64, real-weyl.md:72-73, real-weyl.md:94-97]

## 测试与证据边界

来源记录的错误路径测试锚点覆盖形式在指定 Cartan 类上无定义时的 `RealFormNotDefinedOnCartan`，以及形式或 Cartan 编号越界时的 `IndexOutOfRange`。预算耗尽路径和非 quasisplit 对偶形式的行为没有测试锚点，因此现有记录不能说明所有对偶代表元选择均已得到验证。^[real-weyl.md:167-168, real-weyl.md:188-188]

本页依据 `real_weyl.rs` 的结构性阅读，不构成实 Weyl 层的数学验收。来源中的上游行号转录自代码文档注释，未核对上游字节；该次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[real-weyl.md:9-13, real-weyl.md:192-196]

## Sources

- [real-weyl.md](../../sources/real-weyl.md)：实 Weyl 群与块稳定子：real_weyl.rs 的构造、对偶 fiber 重放与打印层。
