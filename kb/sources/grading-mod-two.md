---
title: Cartan 紧性分级表与动态 F₂ 线性代数层（grading.rs / mod_two.rs）
source: atlas-rust/grading-mod-two
ingestedAt: 2026-10-06T07:00:00Z
---

# Cartan 紧性分级表与动态 F₂ 线性代数层（grading.rs / mod_two.rs）

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `grading.rs`（601 行）与 `mod_two.rs`（818 行）。grading 是
mod_two 的直接客户（`Grading(ModTwoVector)`、`ModTwoSubspace` 消元、
`dot` 配对）。本包是结构性阅读，不声称数学验收；上游引用
（gradings.cpp:20-23、cartanclass.cpp:172、bitvector.cpp 等）仅转录自注释。

## mod_two.rs：动态 F₂ 层

- `ModTwoVector { dimension, words: Vec<u64> }`：位打包、动态尺寸（刻意与
  Malachite 整数层分离）。`from_ones` 逐位 toggle——**重复下标偶次抵消**
  （测试锚定）。派生 Ord/Hash 是确定性 map-key 序而非数学序，其正确性依赖
  「所有构造器清零 padding 位」。
- `ModTwoSubspace`：低主元约定（沿用 Atlas `BitVector::firstBit()`），每次
  `insert` 后基在**每个主元处既约**（RREF）——与插入顺序无关，为结构
  子商层提供确定性坐标。`reduce` 按主元升序扫描（缺失的早期主元不意味着
  后期主元缺失；注释对齐 `normalSpanAdd`）。`right_kernel`：每个自由坐标
  生成 `[free] + {在 free 处置位的主元}`。`pivot_rows`（升序）正是上游
  `Gauss_Jordan` 提交的排序典范基（bitvector.cpp:673-697）——
  `real_weyl.rs` 的 R-group 核构造读这些行，而 `right_kernel` 会重新约化。
- `CanonicalModTwoSection`（上游 `BinaryMap::section` 限制到像）：保留首批
  独立输入列、**丢弃依赖列**（即使其源标记在增广空间独立——此区分固定了
  可观测的实形种子代表）；源坐标打包进 u64（>64 列报
  `ResourceLimitExceeded{limit:64}`），目标向量动态。`solve` 命中时返回
  掩码。穷举测试钉住确定性：全部 2¹² 个 3×4 映射 × 8 目标，解 = 数值最
  小源掩码。
- `ModTwoSubquotient`（pub(crate)；公开数学 API 是 `CartanFiber` 包装）：
  `new` 校验维度一致、rank 单调、分母 ⊆ 分子、补基 = 主元位未被分母占用
  的分子行；`canonical_representative`（不在分子 → `NotInModTwoSubspace`）、
  `to_coordinates`/`ambient_representative`（低主位读/写）、
  `validate_induced_map_to`（分子与分母**两侧**基向量的像都要落位，否则
  `CartanFiberMapDoesNotDescend{relation: "numerator"|"denominator"}`——只
  查商基代表会让正规形选择「看似」定义了映射）。
- `ModTwoAmbientMap` trait + `#[cfg(test)] ModTwoLinearMap` 稠密测试辅助。

## grading.rs：一个 Cartan 类的紧性分级表

`Grading(ModTwoVector)`：位 i 对应属主模型简单虚根列表第 i 项（=
`RootInvolutionData::imaginary_simple_roots` 的确定性根序拷贝），置位 =
**非紧**（gradings.cpp:20-23）。与 CartanFiber/AdjointCartanFiber 的 ambient
mod-two 坐标刻意分型（A2+恒等时三者维度相同，只能靠类型区分）。

`CartanGradingData::build(root_system, root_involution, adjoint)`：ambient
纤维刻意不作参数（值相等无法表达纤维身份）——门槛是
`adjoint.ambient_fiber().involution() != root_involution.involution()` →
`CartanFiberInvolutionMismatch`。逐虚根收集：`m_alpha` = 余根在 ambient
纤维的 mod-2 像；伴随 `m_alpha` 经投影（`Pi(y)_j = ⟨α_j, y⟩` 正是其
bracket 向量，故配对只保留投影内单一实现）；`simple_mod_two` = 单根坐标的
奇性位（`% 2 != 0` 含负奇，B2 测试锚定）。`base_grading` 全 1（quasisplit
规范化：基点把一切简单虚根判非紧），`grading_shifts[i]` = 伴随基代表与各
单根奇性向量的 F₂ 配对。`ensure_faithful_shifts`：shift 列线性相关 →
`GradingShiftsNotFaithful`（上游是 cartanclass.cpp:172 的断言，这里改为
无条件拒绝；注释称无已知公共路径可达相关列，纯防御）。

`grading(element)`：典范代表后逐根取 **`!dot`**（基点全 1 与配对值的 XOR）。
`element_from_grading(target)`：增广消元——每列 = 分级位 + 一个标记位
（记录是哪一列 shift），右端 = target 的紧位（= target XOR 全 1 基），
余数低 imaginary_rank 位有置位 → `ImpossibleGrading`；标记位读出的组合经
`xor_assign` 汇总为 ambient 代表。唯一性 = 构造期的忠实性不变量。

`try_capacity`（pub(crate)）在此文件定义：全 crate 的 Vec 预分配预算通道
（`AllocationFailed` 映射）。

## 测试锚点与限制

grading 9 个测试：SC A1 quasisplit 规范化（m_alpha 非平凡、伴随平凡、
双向往返）；A2 恒等的四元素双射（根序 index 0 = α₂，shift 为置换矩阵）；
A2 扭转拒全紧分级（`ImpossibleGrading`）；A1×A1 交换的 imaginary_rank 0；
中心余权坐标区分 m_alpha 与伴随像；B2 负奇余根坐标归约；外来输入三连拒；
注入相关列被拒；33 个 A1 因子保持动态（突破 32/64 位打包限制）。
mod_two 12 个测试：130 维动态子空间（超越 Atlas 打包秩上限）、依赖检测、
商代表/陪集、零维与跨字边界、RREF 保持、右核、子商低主元坐标、
usize::MAX 的溢出/分配拒绝、2¹² 穷举截面 oracle、64 列边界、130 维动态
目标。

未覆盖：build 的两处 `IndexOutOfRange`、多数 `ArithmeticOverflow`/
`AllocationFailed` 分支、`ModTwoSubquotientInvariantViolation`（5 处）、
`NotInModTwoSubspace`、`CartanFiberMapDoesNotDescend` 两个 relation 值；
`dot` 无 mod_two 内直接测试；`validate_induced_map_to` 在两文件内无调用方。

## 来源与限制

精确读取身份见
[`2026-10-06-grading-mod-two.json`](snapshots/2026-10-06-grading-mod-two.json)：
绑定 Git base、两文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe
（无工具档案）以两文件完整字节起草（1300s 期限，exit 0，458.1s），维护者
对照源码逐条核对改写。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。
