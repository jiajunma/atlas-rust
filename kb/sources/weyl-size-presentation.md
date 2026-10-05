---
title: Weyl 群阶识别与实形展示层（weyl_size.rs / presentation.rs）
source: atlas-rust/weyl-size-presentation
ingestedAt: 2026-10-06T02:00:00Z
---

# Weyl 群阶识别与实形展示层（weyl_size.rs / presentation.rs）

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `crates/atlas-real-group/src/` 的 `weyl_size.rs`（262 行）与
`presentation.rs`（186 行）。两文件互不导入；本包是结构性阅读，不声称这两层
的数学验收；上游引用（cartanclass.cpp:1046-1064、realredgp.cpp:68-80、
atlas-types.w:3566-3575）仅转录自代码注释，本包未核对上游字节。

## weyl_size.rs：由 Cartan 类型识别求 Weyl 群阶

唯一入口 `weyl_order_of_cartan`（`pub(crate)`）：按需 twisted-共轭划分
（task #9）把每个轨道大小与上游阶商公式 `|W| / (|W_im| × |W_re| × |W_cx|)`
核对（cartanclass.cpp:1046-1064）——只需要被识别 Cartan 矩阵的 Weyl 群的
**阶**。B/C 取向无关（同为 2ⁿn!），识别因此退化为分量拆分 + 分支形状分析。
阶用精确 `Integer` 算术：分量乘积在 crate 的动态秩范围内早已溢出 u128。

流程：逐行方形检查（`NonSquareCartan`）→ 连通分量按「非零非对角链接」
BFS（零对角行/列 = 环面因子，贡献阶 1，既不作为种子也永不满足邻居条件）→
逐分量 `component_order` 相乘。

`component_order` 按最大边重数 `m = C_ij·C_ji` 分派（唯一的受检算术是
`checked_mul`，溢出 → `ArithmeticOverflow`）：m=3 → 秩 2 为 12（G2）否则
拒绝；m=2 → 度 > 2 拒绝；秩 4 且双键位于两个内节点之间 → 1152（F4），否则
`2^rank·rank!`（B/C 链同阶）；m=1 → 度 > 3 或多叉拒绝；无叉 →
`(rank+1)!`（A_n）；单叉按排序分支长度匹配：[1,1,_] → D_n
（`2^n n! / 2`）、[1,2,2] → 51840（E6）、[1,2,3] → 2903040（E7）、
[1,2,4] → 696729600（E8），其余拒绝。`branch_lengths` 从唯一的度-3 节点
向外走；无环检测（含环输入可能不终止而非报错——阅读观察，无防护）。

已知留白（阅读观察）：单节点分量对任何非零对角值都返回 2；符号不一致的
非对角对（乘积为负）计入度数但不提升 `max_multiplicity`（初值 1 只取
max），即被当作单连通处理；rank > 4 的双键链不检查双键位置。

测试锚点：经典系列（A4=120、B5=C5=3840、D5=1920）、例外型（G2=12、
F4=1152、E6=51840）、夹一行环面的 A1×A1=4、空系统=1。

## presentation.rs：实形展示层

`RealFormPresentation`（五个 `pub` 字段）：`printType` 的 Lie 代数名 +
上游 `RealReductiveGroup::construct` 的四个状态位（realredgp.cpp:68-80）：
`IsCompact`/`IsSplit` 把 most-split Cartan 对合 `ms_tau` 与 ±1 逐元素比较；
`IsQuasisplit` 只按 external 编号 == `quasisplit_external()` 判定；
`IsConnected` 委托 `topology::dual_component_group_trivial`（对偶分量群的
平凡性）。

`build_presentations` 按 external 编号顺序逐形计算。注意顺序（阅读观察）：
compact/split 的逐元素扫描**先于** `ms_tau` 的秩检查（仅查行数）——失败时
结果被整体丢弃，无可观察副作用。错误统一为 `LayoutInvariantViolation`，
四个 reason 字符串（"external form number"/"most split Cartan"（两处
共用）/"most split involution rank"/"special grading"）。

测试锚点（3 个）：sc A1：compact 的 "su(2)" 先于 split 的 "sl(2,R)"；
adjoint A1 的 split 形不连通；sc B2 三个形：compact "so(5)" 居首、
split=quasisplit 的 "so(3,2)"（= Sp(4,R)）居末。

## 两文件的接口关系

互不导入。共享仅 `StructureError`（注意两条导入路径写法不同：
`crate::StructureError` vs `crate::error::StructureError`——同一类型的
再导出）。架构关联仅见于模块注释（task #9 阶商校验 vs 展示状态位）；
`CartanClassification` 内部是否调用 `weyl_order_of_cartan` 在这两份字节中
不可见。

## 限制与未覆盖面

- 不做数学/正确性验收；阶数字面量与上游行号均为注释声明。
- weyl_size：所有错误分支、E7/E8 分支、B4/C4（rank 4 非 F4）分支均无测试；
  `factorial` 的 `Result` 签名无可失败操作（预留）。
- presentation：非恒等对合（非 split 内类）、quasisplit 但非 split 的形、
  四个不变量分支均无测试。

## 来源与限制

精确读取身份见
[`2026-10-06-weyl-size-presentation.json`](snapshots/2026-10-06-weyl-size-presentation.json)：
绑定 Git base、两文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe
（无工具档案）以两文件完整字节起草（exit 0，292.1s），维护者对照源码逐条
核对改写。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。
