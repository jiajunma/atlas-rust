---
title: mod-2 线性代数：位打包向量与子空间
source: atlas-rust/mod-two
ingestedAt: 2026-10-03T10:32:42Z
---

# mod-2 线性代数：位打包向量与子空间

编辑状态：**结构性阅读（两次：2026-10-03 初读、2026-10-06 重读，字节未变，
SHA-256 相同）；初读由维护者直接读源补齐（Kimi 摘录漏掉未注释方法），重读
经 Kimi probe 起草、维护者逐条对照源码核对后改写合并**。本包覆盖
`mod_two.rs`（818 行）：动态 F₂ 线性代数层，服务 Cartan 纤维结构层。上游
行号（bitvector.cpp 等）均转述自源码注释。

## ModTwoVector：$\mathbb{F}_2$ 上的位打包向量

`ModTwoVector { dimension: usize, words: Vec<u64> }`：有限向量空间元素的位
打包表示，**刻意与 Malachite 整数层分离**（其 words 编码有限向量空间的元素，
不是无界整数）。派生 `Ord` 是任意但确定的全序，仅供 map 键，不是数学序；
派生健全因为每个构造器把 `dimension` 以上的填充位清零并保持其为零。

- `zero(dimension)`：`word_count = (d + 63) / 64`，加法经 `checked_add`
  （`usize::MAX` → `ArithmeticOverflow`）；分配失败 → `AllocationFailed`。
- `from_ones(dimension, indices)`：toggle 每个下标——**重复下标偶次抵消**
  （测试锚定 `[0,63,64,127,128,63]` 使位 63 为 `false`）。
- `bit(index)`：越界返回 `None`；`xor_assign` / `dot`（`pub(crate)`）：
  维数不匹配报 `RankMismatch`。`dot` 是 $\mathbb{F}_2$ 配对——逐字
  `(l & r).count_ones() & 1` 折叠 XOR，结果为 `parity == 1`。

## ModTwoSubspace：pivot 索引的 RREF 基

字段：`dimension`、`pivots: Vec<Option<ModTwoVector>>`、`rank`。基按
**最低置位 pivot**（沿用 Atlas `BitVector::firstBit()`）索引：`insert` 先
`reduce` 输入，取其最低置位作为新 pivot，并把该 pivot 从旧行中消去——使基
始终保持典范 RREF，与插入顺序无关，为结构子商层提供确定性坐标。
`insert` 返回秩是否增加（`rank.checked_add(1)`，溢出 →
`ArithmeticOverflow`）；`contains` 即 `reduce` 后无剩余；
`quotient_representative` 返回商类的确定性行阶梯代表元（即 `reduce`）；
`same_coset(left, right)` 先查维数再判断差是否属于子空间；
`quotient_dimension = dimension - rank`。

`reduce`（私有）按 pivot **升序**扫描——缺失的早期 pivot 不意味着后期
pivot 缺失；典范基在每个 pivot 处已既约，升序扫描恰好清每个系数一次（注释
对齐上游 `normalSpanAdd`）。`right_kernel`（`pub(crate)`）：对每个自由坐标
生成 `[free] + {在 free 处置位的 pivot}` 并插入新子空间。`pivot_rows()`
按 pivot 升序产出 `(pivot, row)`——这正是上游 `Gauss_Jordan` 提交的排序
典范基（bitvector.cpp:673-697）；`real_weyl.rs` 的 R-group 核构造从这些行
读生成元位，而 `right_kernel` 会把它们重新约化为新子空间。
`basis_vectors()` 仅产出行（同一升序）。

## CanonicalModTwoSection：64 列掩码截面（2026-10-06 重读补充）

上游 `BinaryMap::section` 选举限制到其像：保留**首批独立输入列**，依赖列
**丢弃**（即使其源标记在增广空间中独立——注释称这一区分固定了可观测的
实形种子代表，正如 bitvector.cpp 的 section 遗忘零化列）。源坐标打包进
`u64` 掩码（`columns.len() > 64` 报 `ResourceLimitExceeded { limit: 64 }`），
目标向量仍动态。`solve(target)`：按行升序消元并累积解掩码；
`Ok(remainder.is_zero().then_some(solution))`——目标落在保留列张成内时
`Some(掩码)`，否则 `None`。一次分解服务同一映射的所有目标。穷举测试钉住
确定性：全部 `2^12` 个 3×4 映射 × 8 个目标，解 = 数值最小源掩码。

## ModTwoSubquotient：crate 私有的子商

`ModTwoSubquotient { numerator, denominator, complement_basis }`：两个
$\mathbb{F}_2$ 子空间在共同 ambient 空间中的典范商。**刻意 crate 私有**：
它拥有当前 Cartan-fiber 结构层使用的 low-pivot 打包坐标，而公开 API 是数学的
`CartanFiber` 包装，不是序列化格式。构造校验：维数一致、
`denominator.rank <= numerator.rank`、denominator 的每个基向量被
numerator `contains`、补基恰好来自 denominator 缺 pivot 的位置；违反时报
`ModTwoSubquotientInvariantViolation`。

- `canonical_representative(vector)`：不在分子 → `NotInModTwoSubspace`；
  否则 `denominator.quotient_representative`。
- `to_coordinates` / `ambient_representative`：按补基最低置位读/写位。
- `validate_induced_map_to(target, map)`：分子的每个基向量的像须在
  `target.numerator` 中，分母的每个基向量的像须在 `target.denominator`
  中，否则 `CartanFiberMapDoesNotDescend { relation: "numerator" |
  "denominator" }`——注释：只查商基代表会让正规形选择在源分母向量的目标类
  非零时「看似」定义了映射。
- `ModTwoAmbientMap` trait 供域层在稠密矩阵属多余分配时直接实现；
  `#[cfg(test)] ModTwoLinearMap` 是稠密测试辅助。

## 测试锚点（2026-10-06 重读补充）

12 个测试：130 维动态子空间（超越 Atlas 打包秩上限）；依赖检测与维数不符；
确定性商代表与陪集；零维与跨字边界（含重复下标抵消）；跨多字的依赖消元；
插入后 RREF 保持（直接读私有 `pivots` 断言）；右核计算；子商低主元坐标；
`usize::MAX` 的溢出/分配拒绝（不做不可失败分配）；2¹² 穷举截面 oracle；
64 列边界（64 通过、65 拒绝）；130 维动态目标。未覆盖：多数
`ArithmeticOverflow`/`AllocationFailed` 分支、
`ModTwoSubquotientInvariantViolation`（5 处）、`NotInModTwoSubspace`、
`CartanFiberMapDoesNotDescend` 两个 relation 值；`dot` 无文件内直接测试；
`validate_induced_map_to` 在文件内无调用方。

## 来源与限制

- 源码：[mod_two.rs](../../../crates/atlas-real-group/src/mod_two.rs)；
  阅读快照 [`2026-10-03-mod-two.json`](snapshots/2026-10-03-mod-two.json)
  （初读）与 [`2026-10-06-grading-mod-two.json`](snapshots/2026-10-06-grading-mod-two.json)
  （重读，同一 SHA-256 `945e867e…`，重读与 grading.rs 同包进行）。
- 关联：[紧致 grading](grading.md)、[整数格](integer-lattice.md)、
  [强实形式分类](strong-real.md)。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 初读 Kimi probe 草案因摘录按「文档注释附着」选择而漏掉未注释方法，由
  维护者直接读源补齐（教训已记：摘录要包含裸签名清单）；重读 Kimi probe
  （1300s 期限，exit 0，458.1s）的 CanonicalModTwoSection/right_kernel/
  pivot_rows 细节与 12 个测试锚点均精确，已并入正文。调用记录见两份快照的
  `kimi_assist`。
