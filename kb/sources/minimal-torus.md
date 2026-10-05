---
title: 合成实形的选定余特征与初始环面部分（minimal_torus.rs）
source: atlas-rust/minimal-torus
ingestedAt: 2026-10-06T01:10:00Z
---

# 合成实形的选定余特征与初始环面部分（minimal_torus.rs）

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `crates/atlas-real-group/src/minimal_torus.rs`（651 行）。它是结构性
阅读，不声称该层的数学验收；上游引用（innerclass.cpp:1318-1327、
realredgp.cpp:212-309、atlas-types.w:3534-3545 等）仅转录自代码注释，本包
未核对上游字节。

## 概览

两个 `pub` 函数服务上游 `real_form_value::build` 的自定义种子分支：

- `elected_square_root`：`real_form_of` 的 `coch` 输出（强数据的移位平方的
  `stable_log`，由投影后的环面因子算出）——自定义 `RealReductiveGroup`
  构造器保存的 `g_rho_check`。
- `minimal_torus_part`：`realredgp::minimal_torus_part`（realredgp.cpp:
  212-309）：把给定强代表经逆 Cayley 与 based twisted 共轭下降到基本纤维，
  然后游走基本虚 grading 轨道，选出数值最小的环面部分，其 datum-单根的
  紧致性模式复现目标弱实形。

下降复用 stage-(c) 的 `TitsCoset` 操作，在每个中间环面部分的目标对合处
约化；上游只在末尾约化。文档声明：这些操作是 mod-2 商上的类映射（stage-(e)
KGB 枚举所验证的性质），故逐步约化不动最终约化类（注释声明，本包不验）。
本文件无类型/trait/impl 定义；`MAX_MASK_BITS = 63` 给轨道游走的 u64
seen-set 编码限定格秩上限（与弱实形 mask 游走同一纪律）。

## elected_square_root 的流程

门：`factor` 长度 ≠ 格秩 → `RankMismatch`；`twisted` 的像置换长度 ≠ 根数 →
`DatumMismatch`。由 `twisted.reduced_word` 重建矩阵级作用（按
`word.iter().rev()` 逐生成元左复合单反射），并以「`WeylElement::from_action`
往返等于 twisted」校验钉住字合成方向。随后按序运输：先 distinguished 对合的
余权矩阵，后元素的余权作用矩阵（注释：`square_shifted = t + w.delta_tr(t)`
的 Weyl 作用在对偶侧）。逐坐标取 `(factor + after_w)/2` 得半平方，构造
`delta + I`，调 `stable_log(half_square, plus_one, budget)`（预算是唯一
透传）。

## minimal_torus_part 的流程

入口门序：table 的 inner class 不符 → `DatumMismatch`；coch/factor 长度 →
`RankMismatch`（`actual` 取两者较大）；置换长度 → `DatumMismatch`；
rank > 63 → `SeedResourceLimit { resource: "mask bits" }`。

1. **初始环面部分**：`factor − coch` 逐坐标须为整数（否则
   `"torus-part integrality"`），奇数坐标置位得 `torus_part`（即上游
   `(torus_factor − coch).normalize()` 整化后的 `TorusPart`）。
2. **下降到基本纤维**（realredgp.cpp:230-243）：`grading_of_simples`（逐
   单根配对偶性；上游 `RatCoweight::dot` 的整性断言在此为具名门）构造
   `TitsCoset`；`table.lookup(twisted)` 缺失报
   `"synthetic involution coverage"`。循环：Weyl 部分为恒等则停；否则按
   `interface.outward()` 迭代序取**首个**左下降生成元（注释：对应上游
   `WeylGroup::leftDescent` 的外部编号首个非零抛物片段）；该生成元在当前
   对合下为 Real 则逆 Cayley（`Ok(None)` 也是错误），否则 based twisted
   共轭（`cross_pregated`）。末尾 `coset.reduce`（幂等；twisted 本就在基本
   纤维时起决定作用）。
3. **余权提升**：`coweight = coch + lift(tp)`（torus_part 置位处加 1）。
4. **目标模式**：基本纤维（`CartanId(0)`）的虚单根基上：`start` = 各根与
   coweight 的偶配对（置位 = 紧致，同上游 `start_grading`）；
   `constrained` = 该基根是否 datum-单根；`target` = 受约束处该实形的
   **非紧致性**（翻转使存储位合 crate 的 `Grading` 约定）；
   `m_alpha` = 各基余根的奇偶向量；`grading_shift` = 基间配对 mod 2 矩阵。
   一处有意的翻译差异（注释声明）：上游把目标 grading 切片位与基本单虚
   基的**前导** datum-单根条目配对（依赖其根编号把 datum 单根排在最前）；
   crate 的虚根基有自己的确定性根序，改为**逐位置**配对；上游前导段不变量
   成立时（特别地，平凡 distinguished 下）两者一致。
5. **轨道游走**（realredgp.cpp:271-299）：LIFO 栈 + `BTreeSet<u64>` 去重
   （上限 2^rank）：候选 = 受约束位置与目标一致的状态；在每个**置位**
   grading 位上以 `m_alpha[i]` 平移并按 `grading_shift[i]` 翻转 grading。
   空候选在上游是断言，此处为具名错误 `"minimal torus part candidates"`。
6. **选举**：候选取最小（`ModTwoVector` 序 = 等维位向量的整数序）。

## 测试锚点（4 个，均 rank 2、紧致内类）

- A1.T1 中心因子：对合翻转半单坐标、中心坐标承载整个环面因子；
  `coch == factor`、环面部分为零（冻结锚点
  `weak_real_form_a1_t1_central_probe`）。
- A2 非典范种子：因子 [1/3,2/3]（配 s0）与 [2/3,1/3]（配 s1）各自保持，
  环面部分分别为 [0] 与 [1] 置位（冻结锚点
  `weak_real_form_a2_noncanonical_probe`）。
- 等价因子选同一种子（语言层投影的等价类）。
- 门控拒绝：长度不符、空 InvolutionTable 的具名拒绝（非静默 miss）。

注意：三个正例测试都满足 `coch == factor`，故运输非平凡时的行为没有被
可区分的断言刻画（阅读观察）；测试 3 与测试 2 第一组输入完全相同，其独立
价值仅在注释叙述中。

## 限制与未覆盖面

- 不做数学/正确性验收；上游行号与「逐步约化不动最终类」「逐位置配对一致
  性」均为注释声明。
- 测试未覆盖：非紧致 distinguished、多数具名错误分支、rank > 63 的预算门、
  非平凡运输。
- `encode` 的函数体不会失败（`Result` 签名为预留）；`minimal_torus_part`
  无显式预算参数（仅 `MAX_MASK_BITS` 与分配防护）。

## 来源与限制

精确读取身份见
[`2026-10-06-minimal-torus.json`](snapshots/2026-10-06-minimal-torus.json)：
绑定 Git base、文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe（无
工具档案）以完整文件字节起草（540s 期限，exit 0，414.4s），维护者对照源码
逐条核对改写。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。
