---
title: 实 Weyl 群与块稳定子的构造
summary: RealWeyl 按固定顺序校验 Cartan 类和实形代表，构造两侧 fiber 数据并汇总五个子系统类型；来源仅提供结构性阅读证据。
sources:
  - real-weyl.md
kind: concept
createdAt: "2026-10-09T15:07:44.476Z"
updatedAt: "2026-10-10T00:48:10.756Z"
tags:
  - 实Weyl群
  - 构造校验
aliases:
  - 实-weyl-群与块稳定子的构造
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
title: 实 Weyl 群与块稳定子的构造
summary: RealWeyl 按固定顺序校验 Cartan 类与实形代表，构造原侧和对偶侧的根子系统、紧根基及 R-群；实 Weyl 群与块稳定子通过对偶代表的选择区分。
sources:
  - real-weyl.md
kind: concept
tags:
  - 实Weyl群
  - 块稳定子
  - 不变量
aliases:
  - 实-weyl-群与块稳定子的构造
  - 实W群
provenanceState: extracted
---

# 实 Weyl 群与块稳定子的构造

`RealWeyl` 汇集指定 Cartan 类及原侧、对偶侧实形代表决定的简单虚根、实根、复根、子系统类型、紧根基和两个 R-群。实 Weyl 群与块稳定子共用构造入口：原侧均使用指定实形的代表，对偶侧分别使用伴随 fiber 的零元或指定对偶实形的代表。^[real-weyl.md:17-25, real-weyl.md:85-97]

## 上下文与代表元选择

`RealWeylContext` 借用原内类及其 Cartan 分类、对偶内类及其分类，并携带 `CartanClassificationBudget`。核心接口 `real_weyl(form, cartan, dual_form)` 接受内部 `WeakRealFormId`、`CartanId` 和可选对偶实形编号，返回 `Result<RealWeyl, _>`。`RealWeyl` 与 `RealWeylGenerators` 的字段为私有，通过 getter 访问。^[real-weyl.md:29-43, real-weyl.md:114-116, real-weyl.md:181-182]

打印包装通过 `ExternalFormOrder` 将外部实形编号转换为内部编号，越界返回 `IndexOutOfRange`。`block_stabilizer_print` 的 `dual_form` 属于对偶内类的外部编号体系；代表元选择的具体约定参见 [[实 Weyl 群与块稳定子的代表元选择]]。^[real-weyl.md:99-102]

## 固定构造顺序与错误传播

构造首先查询 Cartan 类；不存在时返回 `IndexOutOfRange { index: cartan.0, .. }`。随后在该类的标签表中定位原侧实形，无对应位置时返回 `RealFormNotDefinedOnCartan`。定位成功后，以 `partition.class_representative` 取得代表元 `x`；代表元缺失返回 `CartanClassificationInvariantViolation("real-form representative")`。^[real-weyl.md:85-89]

接着重建对偶 fiber 链，再选择代表元 `y`。`dual_form = None` 时使用对偶伴随 fiber 的零元，对应上游硬编码的 quasisplit 代表；指定对偶实形时，先定位对偶标签，再取得代表元。标签缺失返回 `RealFormNotDefinedOnCartan`，代表元缺失触发 `"dual real-form representative"` 不变量错误。^[real-weyl.md:90-93]

最后，原侧排序简单虚根与实根，计算复根简单基和单侧 fiber 数据；对偶侧计算 fiber 数据后，将对偶根映回原根数据。映射未命中触发 `"dual root correspondence"` 不变量错误。完成五个子系统的类型识别后，组装 `RealWeyl`。^[real-weyl.md:94-97]

## 对偶 fiber 链的临时重建

对偶 Cartan 对合 `−θ` 一般仅与典范对偶 Cartan 代表共轭，因此实现不能直接复用对偶分类中存储的 fiber。每次调用都会临时重建完整链，当前没有缓存，详见 [[对偶 Cartan fiber 链的临时重建]]。^[real-weyl.md:104-116]

重建依次经过 `dual_twisted_representative`、`CartanFiber::build`、`AdjointCartanFiber::build`、`CartanGradingData::build`、`WeakRealFormPartition::build`、`CayleyCrossDecomposition::build` 和 `RealFormLabels::build`。对偶扭曲代表通过右补最长 Weyl 元得到；对偶 fundamental 类缺失触发 `"dual fundamental class"` 不变量错误。各阶段使用上下文预算中的对应子预算，原侧则读取已经受预算约束的分类数据。^[real-weyl.md:106-116]

## 单侧 fiber 数据与 R-群

两侧共用 `fiber_side(root_system, involution, grading, element)`。正虚根先按上游键排序，再由基 grading 与代表元平移项判断非紧性：基 grading 为 \(\langle\alpha,\rho^\vee_{\mathrm{im}}\rangle\) 的奇偶，其中 \(2\rho^\vee_{\mathrm{im}}\) 是正虚余根之和；平移项是 ambient 代表与根的 datum-simple 坐标的模二点积。配对和 \(\sum_\beta \operatorname{bracket}(\alpha,\beta)\) 必须为偶数，否则触发 `"imaginary-simple coordinates"` 不变量错误。紧根累加进 `two_rho_ic`。^[real-weyl.md:118-127]

紧根集合经 `simple_basis` 得到 `compact_basis`；`orth` 选取非紧且与 `two_rho` 正交的根，这些根强正交并构成 \(A_1^n\)。`simple_basis` 保留上游的特殊扫描行为：候选因反射结果非正而移除自身时，整个外层扫描终止，后续候选不再检查；调用方只传入正根。^[real-weyl.md:128-137]

R-群生成向量通过各 `orth` 根的 `m_alpha` 的 fiber 坐标求核得到。实现将这些坐标按行注入 `ModTwoSubspace`，按自由列升序生成核基：每个向量置位自由列自身，以及含有该自由列位的主元行所对应的位置。每个 `orth` 条目占一个位坐标，核生成元顺序与上游约定一致。^[real-weyl.md:63-64, real-weyl.md:130-133]

两个 R-群字段的来源需要明确区分：`real_r` 保存对偶侧的 `r_vectors`，`imaginary_r` 保存原侧向量，参见 [[实 Weyl 群 R-群数据的两侧归属]]。^[real-weyl.md:72-73]

## 根列表、复根选择与类型识别

所有根列表均保存原侧的 `RootId`。`imaginary` 与 `real` 按上游 `RootNbr` 键排序，即高度与简单坐标的反向字典序；`complex` 保留 `makeSimpleComplex` 的输出顺序。对偶侧的 `real_compact` 与 `real_orth` 利用“对偶根向量等于原侧余根向量”的对应关系映回原侧。^[real-weyl.md:58-62]

`simple_complex` 从同时正交于正虚根之和、正实根之和的正根中求简单基，再做 Dynkin 分类。对合成对的分量只保留一个：对当前分量向后扫描，删除首个含有与当前分量最低根之像非正交顶点的后续分量，每次只删除一个。^[real-weyl.md:139-142]

`real_type` 与 `real_compact_type` 使用转置的子系统 Cartan 矩阵，其余三个类型使用非转置矩阵；B/C 类型互换在此发生。来源记录的测试锚点为 C2 根数据上的 `Sp(4,R)` Cartan #3 输出 B2 型 `W^R`。^[real-weyl.md:66-70]

相关辅助函数 `twisted_orbit_size` 以虚、实、复三个子系统的 Weyl 群阶之积作为稳定子阶，其中复因子只取每个对合配对中的一个分量。整除性失败触发 `"integral twisted orbit size"` 不变量错误。^[real-weyl.md:144-147]

## Weyl 生成元与展示

每个列出的根经 `WeylAction::root_reflection` 和 `WeylElement::from_action` 转换为 Weyl 元素。每个 R-群核向量对应一个反射乘积，按 `orth` 坐标升序遍历置位并右乘；复根生成元为 \(s_{\mathrm{rn}}s_{\theta(\mathrm{rn})}\)，先构造根反射，再右乘其对合像的反射。生成元按构造即具有规范词，打印使用 `canonical_word`，参见 [[Weyl 元素的规范词]]。^[real-weyl.md:75-81]

打印层分别以 `W^C.((A.W_ic) x W^R)` 和 `W^C.((A_i.W_ic) x (A_r.W_rc))` 展示实 Weyl 群与块稳定子的结构。空词打印为 `e`，非空词使用从 1 开始的生成元编号并以逗号连接；标点、空行和行终止均属于字节契约，详见 [[实 Weyl 群打印的字节兼容契约]]。^[real-weyl.md:149-157]

## 证据与实现边界

来源是对 `real_weyl.rs` 的结构性阅读，不构成数学正确性验收；上游文件与行号来自代码注释，未核对上游字节。测试注释声明，七个 fixture 的输出取自固定上游构建 `rev 4d3e9449`，于 2026-08-11 重新生成并逐字节复制到断言，覆盖 B/C 互换、复因子展示前缀、秩二 R-群核生成元顺序，以及部分编号和实形定义错误路径。^[real-weyl.md:9-13, real-weyl.md:159-168]

预算耗尽路径与非 quasisplit 对偶实形行为没有测试锚点。每次重建对偶链是性能线索，其取舍是否有意尚未确认，来源不作性能声明。两处 `.expect("checked cartan id")` 被描述为按构造不可达的潜在 panic 点；`simple_complex` 的部分索引依赖 Dynkin 分类输出与根基长度一致。^[real-weyl.md:177-188]

实现未移植上游 `reflection_word`／`to_dominant` 机器、打印末尾的调试尺寸断言及其 Weyl 群阶计算，也未移植 `printDualRealWeyl` 入口。后者没有内建包装使用，其所需的虚根与实根生成元列表仍在计算。来源记录的知识维护未执行 Atlas、Cargo、测试或 benchmark。^[real-weyl.md:170-175, real-weyl.md:192-196]

## Sources

- [real-weyl.md](../../sources/real-weyl.md) — 实 Weyl 群与块稳定子：`real_weyl.rs` 的构造、对偶 fiber 重放与打印层。
