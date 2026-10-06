---
title: KLV 多项式的存储与逐列计算
source: atlas-rust/kl-polynomial-table
ingestedAt: 2026-10-03T10:32:42Z
---

# KLV 多项式的存储与逐列计算

编辑状态：**结构性阅读（两次：2026-10-03 初读、2026-10-06 重读，字节未变，
SHA-256 相同）；两份草案均经 Kimi probe 起草、维护者逐条对照源码核对后改写合并**。
本包解释 `kl_polynomial.rs`（多项式引擎）与 `kl_table.rs`（按块存储与填充）
的代码结构；KLV 计算的正确性属于它自己的 HPC 证据链（F4/E6 修复、rank6
inventory 等），本包不重述也不扩展。所读字节见
[`snapshots/2026-10-03-kl-polynomial-table.json`](snapshots/2026-10-03-kl-polynomial-table.json)
（初读）与
[`snapshots/2026-10-06-lattice-kl-polynomial.json`](snapshots/2026-10-06-lattice-kl-polynomial.json)
（重读，仅覆盖 `kl_polynomial.rs`，与 lattice.rs 同包进行）。

## 多项式表示

KLV 多项式 $P_{x,y}$ 是变量 $q$ 上的多项式；上游约定整系数、非负系数、
首项系数为 1，用 `SafePoly<KLCoeff>` 存储。`KlPol(Vec<i32>)` 镜像这一布局：
低次项在前，**零多项式是空向量**，非零多项式的最高次系数必非零（每次运算
经 `trim` 维持）。`degree()` 对零多项式返回 0（`polynomials.h` 的约定），
调用方需要用 `is_zero()` 区分它与常数多项式。非负性与首一性是算法输出满足
的外层性质，不是类型层维护的不变量；系数是 `i32`，没有独立的溢出防护——
溢出防护属于调用链的错误预算，不属于本类型。

## 运算集：恰好够递归与 μ-修正

模块只提供 KLV 递归需要的运算，每个都有明确的上游对应：

- `add`/`sub`（polynomials.h 的 `add`/`subtract`）；
- `shift()`：乘以 $1+q$（kl.cpp:409 的 `safeAdd(Pxy,1)`）；
- `add_shifted(other, d)`：$P + q^d\cdot other$，complex descent 递归项
  $P_{sx,sy} + q\,P_{x,sy}$（kl.cpp:416）；
- `sub_shifted(other, d, mu)`：$P - \mu\, q^d\, other$，μ-修正项
  （kl.cpp:504-512 的 `safeSubtract`）；
- `add_shifted_scaled(other, d, mu)`：μ-求和贡献（kl.cpp:834-836）；
- `scaled(factor)`：标量倍乘；
- `divide_by_2()`：KLV 语境下恰整除，任一奇系数报
  `StructureError::RepInvariantViolation`（"KL polynomial parity"），对应
  kl.cpp:702 的 `safeDivide(2)`；
- `quotient_by_1_plus_q(bound)`：合成除法恢复 $(1+q)P$ 的商，交错部分和并
  截断到次数界（kl.cpp:711）；
- `evaluate_at_minus_one()`：$q=-1$ 求值，系数的交错和
  （repr.cpp:1953-1955）；
- `from_coefficients`：逐系数构造（`ext_kl` 的 `extract_M` 等路径需要）。

只有 `divide_by_2`/`quotient_by_1_plus_q` 之外的运算永不失败；
`StructureError` 只从整性检查进入。

复核备注（2026-10-06 重读标记，均属实并保留）：

- `coefficient()` 的文档写「panics if out of range」，但实现越界返回 0——
  且 `add`/`sub` 的正确性依赖此行为：文档过期，以实现为准；
- `quotient_by_1_plus_q` 的函数体**恒 `Ok`**，返回 `Result` 仅为签名形态
  （与上游 `safe_quotient_by_1_plus_q` 调用形态对齐）；
- `KlHashTable` derive 了 `Default`：`KlHashTable::default()` 得到**空池**
  （无 0/1 号种子），与 `new()` 语义不同——若存在 `default()` 调用点，池
  索引约定（0=零、1=一）将被破坏；
- 算术全部是不检查的 `i32` 普通运算（含 `index + d` 的下标计算），没有
  `ArithmeticOverflow` 通道——与 `lattice.rs` 的全检查风格对立；是否可接受
  取决于系数上界，本包不断言。

## 去重池

`KlHashTable`（上游 `KL_hash_Table`）把多项式内容去重到 `KlIndex`：
`pool: Vec<KlPol>` 加 `HashMap<KlPol, usize>`；`zero` 固定在索引 0、
`one`（常数 1）固定在索引 1，与上游 `KLStore{Zero, One}` 初始化
（kl.cpp:100-101）一致。`match_pol` 插入即去重；`kl_pol` 访问器返回的就是
这个池索引，调用方经 `pool()` 取回多项式本体。

## 表的布局

`KlTable` 为一个块（block）计算并存储 $P_{x,y}$（kl.h:64-173）。按列组织：
列 $y$ 存 `d_KL[y]`，按下标 $x$ 在 $y$ 的 descent set 中的 primitive-index
位置索引，存放池索引；μ-系数存 `d_mu[y]`，只记非零对
`MuPair { x, coef }`（kl.h:41-45）。`holes[y]` 为 `true` 表示该列尚未计算。

实际存储类型是 `#[doc(hidden)]` 的 `KlTableHandle<B: BlockTopology>`
（字段：`support: KlSupport<B>`、`holes`、`columns`、`mu_columns`、`pool`）。
`KlTable<'a, B = &'a BlockGraph>` 只是源码兼容别名；`SharedKlTable =
KlTableHandle<Arc<PartialBlock>>` 持有共享句柄。`new` 从 `&BlockGraph` 借用
构造，`from_handle` 从任意句柄构造。

## 访问语义

- `kl_pol(x, y)`：`x` 先经 `primitive_index_of` 投影到 $y$ 的 descent set；
  `x` 越界或列 `y` 不存在时报错。投影位置超出列长（含 $\ell(x) \ge \ell(y)$
  与 primitivisation 失败）时，按 kl.cpp:129-131 返回：若 `x` 恰好投影到
  `y` 自身则返回恒等 $P_{y,y}=1$（池索引 1），否则零多项式（池索引 0）。
  `UndefBlock`（`x == support.size()`）作为哨兵映射到原语计数。
- `mu(x, y)`：列不存在返回 `None`；否则线性查找 `MuPair`——表中只存非零
  μ，查不到也是 `None`，调用方不能据此区分「μ=0」与「未填充」。
- `mu_column(y)` 返回该列全部非零 μ-对；`prim_map(y)` 是该列的非零-KL 位图
  （kl.cpp:223-232）。

## 填充算法

`fill(limit)` 计算 `[0, limit)` 的列；`limit == 0` 表示全部填满
（kl.cpp:188-221），已填充列跳过，是幂等的逐列推进。每列
`fill_kl_column`（kl.cpp:350-363）先为该列的 descent set 准备 primitive
索引，然后分派：

- 存在直接递归（`first_direct_recursion` 找到第一个使 `y` 有 complex
  descent 或 real type-I descent 的生成元 $s$）时走 `recursion_column`，
  再 `complete_primitives`；
- 否则走 `new_recursion_column`，按 `x` 区分 recursion.pdf 的
  "nice and real" 与 "endgame" 两种情形——μ-修正由此进入一般路径。

两条路径的公式与判定条件不在本包展开；endgame 的历史修复
（`first_endgame_pair`、real-II cross 的 `UndefBlock` 边界）有自己的
tests-first 证据链，见 [项目交接记录](../../../docs/HANDOFF.md) 的相应段落。

## 来源与限制

- 测试锚点（2026-10-06 重读补充）：kl_polynomial.rs 的 4 个测试——池种子
  序号（`get(0)` 为零、`get(1).as_slice() == &[1]`）、`shift` 展开
  （`(1+2q)(1+q) = 1+3q+2q²`）、`q=−1` 交错和（`[1,−1,2] → 4`；
  `(1+q)² → 0`）、`sub_shifted` 单项（`(1+q) − q·1 = 1`）。未测面广：
  `add`/`sub`、`add_shifted`、`scaled`、`divide_by_2` 错误分支、
  `quotient_by_1_plus_q`、`match_pol` 去重路径、`get` 越界等。
- 源码：[kl_polynomial.rs](../../../crates/atlas-real-group/src/kl_polynomial.rs)、
  [kl_table.rs](../../../crates/atlas-real-group/src/kl_table.rs)；阅读快照
  [`2026-10-03-kl-polynomial-table.json`](snapshots/2026-10-03-kl-polynomial-table.json)
  （初读）与
  [`2026-10-06-lattice-kl-polynomial.json`](snapshots/2026-10-06-lattice-kl-polynomial.json)
  （重读，仅 `kl_polynomial.rs`，同一 SHA-256 `de27bf1e…`）。
- 上游行号均转述自源码注释（kl.cpp/kl.h/polynomials.h/repr.cpp），未独立
  重读上游，随版本演进可能漂移。
- 关联：[KGB 图结构](kgb-graph-structure.md)（块与链接的底层来源）、
  [根坐标与格坐标](../wiki/math/root-coordinates.md)；`KlSupport`、
  `BlockTopology`、`PartialBlock` 的展开属于后续来源包。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 初读起草经由本地 Kimi probe（exit 0，90.7s）；重读同样经 Kimi probe
  （600s 期限，exit 0，326.1s），其四条复核备注与测试锚点普查均精确，已
  并入正文。调用记录见两份快照的 `kimi_assist`。
