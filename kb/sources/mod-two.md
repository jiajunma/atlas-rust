---
title: mod-2 线性代数：位打包向量与子空间
source: atlas-rust/mod-two
ingestedAt: 2026-10-03T10:32:42Z
---

# mod-2 线性代数：位打包向量与子空间

编辑状态：**结构性阅读。Kimi probe 的草案因摘录遗漏未注释的方法而偏薄，本包由
维护者直接读源补齐**；Kimi 调用记录与教训见快照。所读字节见
[`snapshots/2026-10-03-mod-two.json`](snapshots/2026-10-03-mod-two.json)
（`mod_two.rs` SHA-256
`945e867e328cb8188a78f5148134c03159618318d4b122588ff0afe1b540cade`，dirty
工作区）。

## ModTwoVector：$\mathbb{F}_2$ 上的位打包向量

`ModTwoVector { dimension: usize, words: Vec<u64> }`：有限向量空间元素的位
打包表示，**刻意与 Malachite 整数层分离**（其 words 编码有限向量空间的元素，
不是无界整数）。派生 `Ord` 是任意但确定的全序，仅供 map 键，不是数学序；
派生健全因为每个构造器把 `dimension` 以上的填充位清零并保持其为零。

- `zero(dimension)`：失败条件是 `word_count` 溢出或分配失败；
  `from_ones(dimension, indices)`：toggle 每个下标。
- `bit(index)`：越界返回 `None`；`xor_assign`：维数不匹配报
  `RankMismatch`；`dot`（`pub(crate)`）：$\mathbb{F}_2$ 配对——逐坐标乘积
  的奇偶。

## ModTwoSubspace：pivot 索引的 RREF 基

字段：`dimension`、`pivots: Vec<Option<ModTwoVector>>`、`rank`。基按
**pivot 位置索引**：`insert` 先 `reduce` 输入，取其最低置位作为新 pivot，并
把该 pivot 从旧行中消去——使基始终保持典范 RREF。`insert` 返回秩是否增加；
`contains` 即 `reduce` 后无剩余；`quotient_representative` 返回商类的确定性
行阶梯代表元（即 `reduce`）；`same_coset(left, right)` 判断 `left - right`
是否属于子空间；`quotient_dimension = dimension - rank`。

## ModTwoSubquotient：crate 私有的子商

`ModTwoSubquotient { numerator, denominator, complement_basis }`：两个
$\mathbb{F}_2$ 子空间在共同 ambient 空间中的典范商。**刻意 crate 私有**：
它拥有当前 Cartan-fiber 结构层使用的 low-pivot 打包坐标，而公开 API 是数学的
`CartanFiber` 包装，不是序列化格式。构造校验：维数一致、denominator 含于
numerator、补基恰好来自 denominator 缺 pivot 的位置；违反时报
`ModTwoSubquotientInvariantViolation`。

## 来源与限制

- 源码：[mod_two.rs](../../../crates/atlas-real-group/src/mod_two.rs)；
  阅读快照 [`2026-10-03-mod-two.json`](snapshots/2026-10-03-mod-two.json)。
- 关联：[紧致 grading](grading.md)、[整数格](integer-lattice.md)、
  [强实形式分类](strong-real.md)。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 起草经由本地 Kimi probe（无工具 profile，`kimi-code/k3-256k`；exit 0，
  130.0s）。因摘录按「文档注释附着」选择而漏掉未注释方法，草案偏薄；本包由
  维护者直接读源补齐。教训：该路由的摘录选择要包含裸签名清单，不能只带
  doc-comment 携带者。调用记录见快照的 `kimi_assist`。
