---
title: per-involution (1-θ)X* 图像基对：播种、传送与坐标
source: atlas-rust/real-projection
ingestedAt: 2026-10-03T10:32:42Z
---

# per-involution (1-θ)X* 图像基对：播种、传送与坐标

编辑状态：**结构性阅读（两次：2026-10-03 初读、2026-10-06 重读，字节未变，
SHA-256 相同）；两份草案均经 Kimi probe 起草、维护者逐条对照源码核对后改写合并**。
本包解释 `real_projection.rs`（566 行）的基对与传送纪律；其正确性属于它自己的 HPC
证据链（signed-projection 与 fundamental-lattice gate 等），本包不重述也不
扩展。所读字节见
[`snapshots/2026-10-03-real-projection.json`](snapshots/2026-10-03-real-projection.json)
（初读）与
[`snapshots/2026-10-06-real-projection-matreduc.json`](snapshots/2026-10-06-real-projection-matreduc.json)
（重读，同一 SHA-256 `bd76f5d1…`，与 matreduc.rs 同包进行）。

## 定位：什么是 (lift_mat, M_real)

每个 involution 的 $(1-\theta)X^*$ 图像基对（上游 `InvolutionTable::record`
对，involutions.h:104-105）：`lift_mat` 是 $1-\theta$ 像的列 echelon 基
（$n \times r$），`m_real`（$r \times n$）把像元素表为该基下的坐标。二者
满足 $\mathrm{lift\_mat} \cdot \mathrm{m\_real} = 1 - \theta$
（involutions.h:105）。

## 基的不唯一性与播种/传送纪律

基**并非**由 $\theta$ 唯一决定。上游纪律：在 Cartan 轨道的 canonical
involution 处由 $1-\theta$ 的 echelon 归约播种（`InvolutionTable::add_involution`，
involutions.cpp:196-208），再沿 cross-action BFS 传送（`add_cross`，
involutions.cpp:242-243）。播种端的 echelon 归约逐操作复现上游
`matreduc::column_echelon`（matreduc.h:129）及其 gcd sweep（matreduc.h:70）——
因为被选出的 `lambda-rho` 代表元与 `y_lift` 的符号依赖精确的图像基。

`build(theta)` 是 `column_echelon` 作用于 $1-\theta$ 的移植，增量跟踪列操作
矩阵及其逆；上游随后取该逆矩阵的前 $r$ 行作为 `M_real`。行扫描**自底向上**，
每行 `gcd_sweep(row, limit)` 后主元落列 `limit-1`；零列擦除时核列逐列向右端
轮转（已停放列不再动）。

`gcd_sweep` 的符号纪律（2026-10-06 重读补充）：复制局部行后取最小绝对值
主元；**负主元取正并在记录矩阵置 `ops[mindex][mindex] = −1`**——注释称
E6 involution-187 的分解只有带这个记录符号才成立；消元用 `div_euclid`
（负余数会选出负主元、反转典范像基定向）；ops 经 `apply_column_ops`
（= 上游 `column_apply` 的前 `limit` 列）同时作用于 `a` 与 `col`。
`invert_integer_matrix` 用欧几里得行消元求幺模逆：主元选列内绝对值最小者、
对角须为 ±1（−1 整行取负），末尾逐项验证 `M·M⁻¹ = I`；
`check_against` 在 `build` 收尾自校验
（`RepInvariantViolation { invariant: "image basis factorization" }`）。

## 传送与坐标接口

- `transported(reflection)`：沿一条 cross 边（一个单反射）传送——
  `lift_mat' = reflection * lift_mat`，`m_real' = m_real * reflection`，
  保持分解不变式 $(sL)(Ms) = s(1-\theta)s = 1-\theta'$；用 checked i64
  算术，溢出报 `ArithmeticOverflow`。传送所得基与对 $1-\theta'$ 重新 echelon
  归约的结果相差列符号/列次序——这正是上游把基存放在 record 中携带而非
  重算的原因。
- `check_against(theta)`：校验 $\mathrm{lift\_mat} \cdot \mathrm{m\_real}
  = 1 - \theta$。
- `coordinates(weight)`：$(1-\theta)v$ 在图像基下的坐标，即 $M_{real} \cdot
  v$（involutions.h:211）；`lift(coordinates)`：`lift_mat * coordinates`，
  把坐标映回 $X^*$（involutions.cpp:346-356）。

阅读观察（2026-10-06 重读）：`transported` 只校验反射矩阵的方阵性，**不**
校验其阶 = n，也不重跑 `check_against`（入表侧由 `push_record` 对新鲜 θ
对账，见 [Twisted involution 表](involution-table.md)）；`coordinates` 的
zip 截断使短 `weight` 静默按零；`lift` 对过长坐标直接下标会 panic；
`invert_integer_matrix` 的消元用**非受检**普通算术，与文件其余的全 checked
风格不对称；分配受检性也不对称（`try_reserve_exact` vs `vec!`/`to_vec`/
`collect`）。

测试锚点（2026-10-06 重读补充，4 个）：带符号 gcd 扫描对 original3840186
的逐字锚定（`[[8,-12],[4,-6]]` → pivot 2、image `[[0,4],[0,2]]`、columns
`[[-3,2],[-2,1]]`）；斜环面（θ=`[[-7,12],[-4,7]]`）与斜乘积的像基字面量
（`lift_mat [[4],[2]]`、`m_real [[2,-3]]`）；零/满像边界（恒等 → rank 0，
−I → 满秩、`lift_mat = 2I`）。`transported` 无测试。

## 来源与限制

- 源码：[real_projection.rs](../../crates/atlas-real-group/src/real_projection.rs)；
  阅读快照
  [`2026-10-03-real-projection.json`](snapshots/2026-10-03-real-projection.json)
  （初读）与
  [`2026-10-06-real-projection-matreduc.json`](snapshots/2026-10-06-real-projection-matreduc.json)
  （重读，同一 SHA-256 `bd76f5d1…`）。
- 上游行号均转述自源码注释（involutions.h/involutions.cpp/matreduc.h），
  未独立重读上游，随版本演进可能漂移。
- 关联：[Twisted involution 表](involution-table.md)、
  [表示参数上下文](rep-context.md)、[整数格](integer-lattice.md)、
  [Tits 元素](tits-element.md)。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 初读起草经由本地 Kimi probe（exit 0，74.5s）；重读同样经 Kimi probe
  （1200s 期限，exit 0，455.9s），其 gcd_sweep 符号纪律、幺模逆流程与
  panic/不对称观察均精确，已并入正文。调用记录见两份快照的 `kimi_assist`。
