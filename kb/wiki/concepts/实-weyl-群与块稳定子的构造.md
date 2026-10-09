---
title: 实 Weyl 群与块稳定子的构造
summary: RealWeyl 根据 Cartan 类及原侧、对偶侧实形代表收集虚根、实根、复根、紧根基和 R-群，并按固定顺序校验索引与代表元。
sources:
  - real-weyl.md
kind: concept
createdAt: "2026-10-09T15:07:44.476Z"
updatedAt: "2026-10-09T15:07:44.476Z"
tags:
  - 实Weyl群
  - 块稳定子
  - Rust实现
aliases:
  - 实-weyl-群与块稳定子的构造
  - 实W群
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 实 Weyl 群与块稳定子的构造

`RealWeyl` 汇集指定 Cartan 类及实形代表所决定的虚根、实根、复根子系统、紧根基和两个 R-群。实 Weyl 群与块稳定子共用构造入口，区别在于对偶代表的选取：实 Weyl 群使用对偶伴随 fiber 的零元，块稳定子使用指定对偶实形的代表。^[real-weyl.md:17-25, real-weyl.md:83-97]

## 上下文与接口

`RealWeylContext` 借用原内类及其 Cartan 分类、对偶内类及其分类，并持有构造预算。核心接口 `real_weyl(form, cartan, dual_form)` 接受内部 `WeakRealFormId`、`CartanId` 和可选对偶实形编号，返回 `Result<RealWeyl, _>`；`RealWeyl` 与 `RealWeylGenerators` 的字段均为私有，通过 getter 暴露结果。^[real-weyl.md:29-43, real-weyl.md:181-182]

打印包装通过 `ExternalFormOrder` 将外部实形编号转换为内部编号；`block_stabilizer_print` 的 `dual_form` 属于对偶内类的外部编号体系。编号越界返回 `IndexOutOfRange`。^[real-weyl.md:99-102]

## 固定构造顺序

构造首先查询 Cartan 类；不存在时返回 `IndexOutOfRange`。随后在该类的标签表中定位实形，无对应位置时返回 `RealFormNotDefinedOnCartan`；定位成功后，从分区取得代表元 `x`，代表元缺失则返回 `CartanClassificationInvariantViolation("real-form representative")`。^[real-weyl.md:85-89]

接着重建对偶 fiber 链并选择 `y`。`dual_form = None` 时使用对偶伴随 fiber 的零元，对应上游硬编码的 quasisplit 代表；指定对偶实形时，先定位对偶标签，再取得代表元，分别检查实形是否在该 Cartan 上有定义以及代表元是否存在。^[real-weyl.md:90-93]

最后，原侧排序简单虚根与实根，计算复根简单基和单侧 fiber 数据；对偶侧计算相同的 fiber 数据，再将对偶根映回原根数据。对应关系缺失会触发 `"dual root correspondence"` 不变量错误。完成五个子系统的类型识别后，组装 `RealWeyl`。^[real-weyl.md:94-97]

## 对偶 fiber 链的重建

对偶 Cartan 对合 `−θ` 一般仅与典范对偶 Cartan 代表共轭，因此不能直接复用对偶分类中存储的 fiber。每次构造都临时重建完整链，当前没有缓存；详见 [[对偶 Cartan fiber 链的临时重建]]。^[real-weyl.md:104-116]

重建依次经过 `dual_twisted_representative`、`CartanFiber::build`、`AdjointCartanFiber::build`、`CartanGradingData::build`、`WeakRealFormPartition::build`、`CayleyCrossDecomposition::build` 和 `RealFormLabels::build`。其中对偶扭曲代表通过右补最长 Weyl 元得到；对偶 fundamental 类缺失触发 `"dual fundamental class"` 不变量错误。各阶段使用 `RealWeylContext.budget` 的对应子预算，原侧则读取已经预算约束的分类数据。^[real-weyl.md:106-116]

## 单侧 fiber 数据与 R-群

两侧共用 `fiber_side(root_system, involution, grading, element)`。它先按上游键排序正虚根，再由基 grading 与代表元平移项判断非紧性：基 grading 是配对 \(\langle\alpha,\rho^\vee_{\mathrm{im}}\rangle\) 的奇偶，其中 \(2\rho^\vee_{\mathrm{im}}\) 为正虚余根之和；平移项是 ambient 代表与根的 datum-simple 坐标的模二点积。计算要求相应配对和为偶数，否则返回 `"imaginary-simple coordinates"` 不变量错误。紧根累加为 `two_rho_ic`。^[real-weyl.md:118-127]

紧根集合通过 `simple_basis` 得到 `compact_basis`；`orth` 选取非紧且与 `two_rho` 正交的根，这些根强正交并构成 \(A_1^n\)。`simple_basis` 保留了上游的扫描行为：候选因反射结果非正而移除自身时，整个外层扫描随即终止，后续候选不再检查；调用方只传入正根。^[real-weyl.md:128-137]

R-群生成向量来自各 `orth` 根的 `m_alpha` 在 fiber 坐标中的核计算。实现将这些坐标按行注入 `ModTwoSubspace`，按自由列升序生成核基：每个向量置位自由列自身，以及包含该自由列位的主元行所对应的位置。每个 `orth` 条目占一个位坐标；相关背景见 [[fiber grading 与 R-群核生成元]]。^[real-weyl.md:63-64, real-weyl.md:130-133]

## 根编号、复根与子系统类型

`RealWeyl` 中的根列表统一保存原根数据的 `RootId`。`imaginary` 与 `real` 按上游 `RootNbr` 键排序，即先比较高度，再比较简单坐标的反向字典序；`complex` 保留 `makeSimpleComplex` 的输出顺序。对偶侧的 `real_compact` 与 `real_orth` 利用“对偶根向量等于原侧余根向量”的对应关系映回原侧。^[real-weyl.md:58-62]

复根简单基从同时正交于正虚根之和、正实根之和的正根中提取，再进行 Dynkin 分类。对合成对的分量只保留一个：对当前分量向后扫描，删除首个含有与当前分量最低根之像非正交顶点的后续分量，每次只删除一个。参见 [[复根子系统选择与扭曲轨道大小]]。^[real-weyl.md:139-142]

`real_type` 与 `real_compact_type` 使用转置的子系统 Cartan 矩阵，另三个类型使用非转置矩阵；B/C 类型互换在此发生。例如，源材料记录的测试锚点为 C2 根数据上的 `Sp(4,R)` Cartan #3 输出 B2 型 `W^R`。此外，`real_r` 保存对偶侧 R-群向量，`imaginary_r` 保存原侧向量，不能仅凭字段名称判断其来源侧。^[real-weyl.md:66-73]

## Weyl 生成元与展示

每个列出的根经 `WeylAction::root_reflection` 和 `WeylElement::from_action` 转换为 Weyl 元素。每个 R-群核向量对应一个反射乘积，按 `orth` 坐标升序遍历置位并右乘；复根生成元为 \(s_{\mathrm{rn}}s_{\theta(\mathrm{rn})}\)，先构造根反射，再右乘其对合像的反射。生成元通过 `canonical_word` 输出规范词，相关概念见 [[Weyl 元素的规范词]]。^[real-weyl.md:75-81]

打印层分别以 `W^C.((A.W_ic) x W^R)` 和 `W^C.((A_i.W_ic) x (A_r.W_rc))` 展示实 Weyl 群与块稳定子的结构。空词打印为 `e`，非空词使用从 1 开始的生成元编号并以逗号连接；标点、空行和行终止属于明确的字节契约，详见 [[实 Weyl 群打印的字节兼容契约]]。^[real-weyl.md:149-157]

## 证据与覆盖边界

源材料是对 `real_weyl.rs` 的结构性阅读，不构成数学正确性验收；其中上游文件与行号来自代码注释，未核对上游字节。测试注释记录了七个 fixture 的逐字节输出断言，覆盖 B/C 互换、复因子的展示前缀、秩二 R-群的核生成元顺序，以及部分编号和实形定义错误路径。参见 [[实 Weyl 层的 oracle 测试与证据边界]]。^[real-weyl.md:9-13, real-weyl.md:159-168]

预算耗尽路径与非 quasisplit 对偶实形行为没有测试锚点。每次重建对偶链只是已知的性能线索，不能据此作性能结论；两个 `.expect("checked cartan id")` 被描述为按构造不可达的潜在 panic 点。上游调试用尺寸断言及 `printDualRealWeyl` 打印入口未移植。^[real-weyl.md:170-188]

## Sources

- [real-weyl.md](real-weyl.md)
