---
title: 弱实形式标签与外部编号（real_form_labels.rs / real_form_order.rs）
source: atlas-rust/real-form-labels-order
ingestedAt: 2026-10-05T23:59:00Z
---

# 弱实形式标签与外部编号（real_form_labels.rs / real_form_order.rs）

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `crates/atlas-real-group/src/` 的 `real_form_labels.rs`（609 行）与
`real_form_order.rs`（674 行）。两文件互不导入（在所给字节内）；本包是
结构性阅读，不声称标签/编号层的数学验收；上游引用（output.cpp:71-140、
gradings.cpp:51-76、cartanclass.cpp:929-948 等）仅转录自代码注释。

## real_form_labels.rs：逐 Cartan 的实形式标签

`RealFormLabels` 把一个 Cartan 的弱实类（局部类）映射到 fundamental
partition 的类（inner class 的全局实形式编号）：`label(k)` = 局部类 k（按
`classes()` 顺序）的全局标签。构造门控按序运行：datum 一致性
（`DatumMismatch`）；fundamental grading 必须位于本 inner class 的
distinguished involution、Cartan grading 必须位于分解的合成对合（均
`CartanFiberInvolutionMismatch`）；分解必须经此 distinguished 因子化
（`DistinguishedInvolutionMismatch`）；分区-fiber 探针（外来 fiber 经元素
provenance 路径以 `CartanFiberMismatch` 失败）。

计算机制（grading-based legacy mechanism）：

1. **Cayley 回拉翻转位**：Cartan 侧逐虚单根计算翻转奇偶——使
   `root + alpha` 为根的 Cayley 根 alpha 的个数为奇则翻转。
2. **根列表运送**：`cartan_imaginary ++ cayley` 经 cross 作用运送；每个像
   必须对 distinguished 为 imaginary（否则 `"fundamental imaginary"` 违规）。
3. **基 grading 扩展** `base_grading_extension`（`pub(crate)`）：基本 grading
   在任意虚根处的值 = 其虚单基坐标系数和的奇偶；系数精确求解**转置的**
   bracket 索引子 Cartan 系统（行 j 把每个基根与余根 j 配对）；整性不是
   虚根性判据（非虚根的投影也可能为整），故根类被显式门控。
4. **求解**：增广 `ModTwoSubspace`（list_len 个根位置 + 维数个哨兵位）表达
   fundamental fiber 基代表与运送根的关系；逐局部类：右端 =（回拉后的
   grading XOR 翻转位）接 Cayley 块的全 noncompact，与扩展基 grading 求差；
   `quotient_representative` 余量在根位置有置位 → `ImpossibleGrading`；哨兵
   位 XOR 出 ambient 代表 → fundamental 分区 `class_of`。
5. **锚点**：首标签必须是 fundamental quasisplit 类（否则
   `"quasisplit anchor"` 违规；空分区必然触发）。

测试锚点（6 个）：A2 反射 Cartan 只标 quasisplit；fundamental Cartan 给出
恒等映射；sc A1 分裂 Cartan 标 quasisplit；B2 与 twisted A2 的每个 Cartan
都与锚点相关（标签计数、quasisplit 锚、值域）；交叉/外来输入的三条拒绝
路径；基扩展的手算奇偶锚点（A2/B2 各两个根）加实根拒绝。

## real_form_order.rs：外部（解释器面）编号

`ExternalFormOrder` 是内部弱实形编号与 upstream 外部输出编号的双射
（`FormNumberMap`，output.cpp:71-140）：按 **depth** 升序（distinguished
involution 处极大正交 noncompact 虚根集的大小，gradings.cpp:51-76），平局用
`specialGrading` 的 PARTITION 重载（cartanclass.cpp:929-948）作为
twist-fixed 简单生成元上的无符号位集（生成元 0 = 最低位）比较。上游用
不稳定 `std::sort`，本移植**断言严格 (depth, grading) 序**，任何并列报
响亮的不变量违例（`"strict (depth, grading) order"`），quasisplit 必须居末
（`"quasisplit last"`）。注意：「compact（深度 0）为 external 0」是文档
声明，代码并无对首项深度为 0 的显式断言。

`DepthTables`：预计算正虚根与其基非紧奇偶（`M = C^T` 的有理逆列和；整性
闸门 `"integral imaginary-simple coordinates"`）。`depth` 为贪心极大正交集：
选取保持 `positive_imaginary` 顺序的首个 noncompact 根；与其不正交的候选
被移出 noncompact；正交但其和仍为根的候选（非单 lace 型的短根对）被
**翻转** compactness。两处源码观察（2026-10-10 对照源码确认，均为清理
候选而非缺陷）：`DepthTables::build` 在 182–185 行有一段零效果空转循环
（`for (slot, value) in column.iter().enumerate() { let _ = slot; let _ =
value; }`，紧随其后才是真正的列和累加）；`weight_sum`（454–466）返回
`Result<Option<Weight>, _>`，但每个 Ok 路径都是 `Some`（秩不符走 Err），
唯一调用点（264 行）的 `if let Some(sum)` 恒取该分支——Option 层是残留，
两端均无强制。

`verified_generator_map`：twist-fixed 简单生成元（升序）与伴随 fiber 维数
必须相等；逐位校验**实际有序基**（非抽象双射——`special_grading_key` 按此
数值顺序比较掩码）；生成元下标 ≥ 127（`MAX_KEY_GENERATORS`）拒绝。

`special_grading_key`（PARTITION 重载）：以类代表为种子，升序扫描
`0..2^dimension`，`>=` 替换——取得最大 popcount 中的**最高** fiber 下标；
在 fiber 秩内取补，再把补集 unslice 到 twist-fixed 简单生成元上
（生成元 0 = LSB）。`MAX_MASK_BITS` 限制 fiber 宽度。

访问器：`internal`/`external` 越界返回 `None`；`quasisplit_external()` =
`form_count().saturating_sub(1)`；`special_grading(external)` 返回该形的
tiebreak 位集（位 g = 生成元 g，1 = noncompact imaginary——位含义来自
文档/字段注释）。

测试锚点（6 个）：E6 twisted 生成元坐标回归（期望来自所给置换而非 Rust
输出；`verified_generator_map == [1, 3]`）；SL(2) 的 compact 0/split 末与
KGB 大小 1、3；Spin(5) 的 [1,4,11]；等秩 A2 的 SU(3) 先于 SU(2,1)；
twisted A2 的单一 quasisplit 形；Spin(8) 的同深度并列经 grading tiebreak
严格排序（compact 首、split 末）。

## 两文件的接口关系

互不导入。概念上编号域相容：`label` 的输出与 `external` 的输入同为
fundamental 分区的 `WeakRealFormId`——「局部类 → fundamental 内部编号 →
外部编号」的串联在类型上可行，但字节内无代码或测试执行这种组合（推断，
未验证）。两者都以 fundamental 分区的 `quasisplit_class()` 为锚点
（labels：首标签；order：居末断言）。

## 限制与未覆盖面

- 不做数学/正确性验收；上游行号与「compact 为 external 0」均为注释声明。
- 未测试分支众多：labels 侧的门控拒绝大多有锚点，但 `ImpossibleGrading`、
  `"quasisplit anchor"` 失败无锚点；order 侧几乎全部失败路径无锚点。
- `special_grading_key` 的 `0..(1 << dimension)` 全枚举是实现事实，其安全性
  依赖 `MAX_MASK_BITS`（数值在本包外定义，见 weak-real-form 包）。
- labels 测试仅触及 A1/A2/B2；order 测试触及 A1/A2/B2/D4/E6（E6 仅生成元
  映射）。两文件无任何组合测试。

## 来源与限制

精确读取身份见
[`2026-10-06-real-form-labels-order.json`](snapshots/2026-10-06-real-form-labels-order.json)：
绑定 Git base、两文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe
（无工具档案）以两文件完整字节起草，维护者对照源码逐条核对改写。本次
知识维护未执行 Atlas、Cargo、测试或 benchmark。
