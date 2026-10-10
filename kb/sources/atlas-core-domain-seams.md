---
title: 领域层接缝：值提取器、alcove 助手与 Weyl 词/生成元校验（domain_builtins.rs 三处缝隙）
source: atlas-rust/atlas-core-domain-seams
ingestedAt: 2026-10-10T09:00:00Z
---

# 领域层接缝（domain_builtins.rs 的三处缝隙）

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包补齐
`domain_builtins.rs`（22771 行）既有各包之间的三个区间：值提取器组
（2538–2653）、alcove/FPP 助手（5924–6085）、Weyl 词/生成元校验与值
冻结（9606–9723）。除此之外该文件无其他未覆盖生产区间：上部值层
（domain-values）、1628–2527 构造管线、2657–2933 形变缓存、
3022–5005 + 7992–9606 SCC/根表（`as_integer` 在 9554，属此区间）、
5005–5924 根编号/alcove 机器、6102–7992 中心分类器、9730–12263
校验与打印、12263–18442 派发组织，各有专包；166 臂的逐臂数学与
`cfg(test)` 回归库（18430 起）按既定范围不入包。对应
[阅读快照](snapshots/2026-10-10-atlas-core-domain-seams.json)。
结构性阅读，不声称语言或数学验收。

## 值提取器组（2538–2653）

派发臂与构造管线共用的 `Value` → 领域类型提取器：

- `as_lie_type`（2538）：`LieType` 直传；`String` 经 `parse_lie_type`
  解析——这是上游隐式 string→LieType 强转的落点；其他值报
  `expected a Lie type, found …`（Type 类诊断）。
- `as_usize`（2550）：BigInt → usize 收窄，失败报
  `expected a nonnegative machine integer`。
- `as_matrix`（2558）：先查方阵——`Value::Matrix` 用类型化矩阵的
  rows/cols 报 `expected a square mat; received a {}x{} matrix`，
  嵌套表路径报 `expected a square mat`；两条文案不同。
- `explicit_datum_matrix`（2580）：对类型化 `Matrix` 返回 `Cow::Borrowed`
  零拷贝；嵌套表回退先按行提取再按列主序装配 `Matrix::from_columns`；
  空表报运行时错误
  `Implicit conversion to matrix for an empty set of vectors`。
  **空维保留**：`as_matrix_rows`（2604）把 0xN 类型化矩阵表示为 N 个空
  行，使下游维度校验不会把 0xN 误当 0x0——这是 empty-root-datum 证据
  （原版接受 Nx0 简单根/余根矩阵）的适配层，回归锚点是
  `zero_row_matrices_do_not_collapse_into_zero_by_zero_shapes`（19076）。
  矩形校验先于逐元素 i32 收窄（`matrix entry out of range`）。
- 消费链：显式 RootDatum 臂在 12462–12468 同时用 `as_matrix_rows`
  （lattice 基）与 `explicit_datum_matrix`（简单根/余根）；另有多臂
  （8076、8204、8299、8945、9052、9861 等）直接消费
  `as_matrix_rows`。
- `merge_ktype_term`（2657）只是 `merge_pol_term` 的 (KType, SplitValue)
  特化（同 K 型合并、零系数丢弃），语义归形变缓存包。

## alcove/FPP 助手（5924–6085）

- `bareiss_det`（5971）：无分数 Bareiss 消元。主元为零时只在**下方**
  找行交换并翻符号；找不到则行列式为 0。每步以 `previous`（上一主元）
  整除——Sylvester 恒等式保证整除精确。空矩阵返回 1；结果为
  `sign * a[n-1][n-1]`。
- `adjugate_det`（5932）：行列式走 `bareiss_det`；伴随矩阵按余子式
  展开——`adj[i][j] = (-1)^{i+j}·M[j][i]`（注释明确指出伴随是余子式
  矩阵的转置；代码外层下标命名 `column`、内层 `row`，符号用
  `(row+column)%2`，与转置写法一致）。注释给出适用范围：秩 ≤ 9；
  `adj/det` 即逆矩阵。空矩阵返回 `(vec![], 1)`。全部算术在 i64。
- 消费链：`CenterClassifier::new`（6105）与分类器内部（6272 弃用 det、
  6296 两者皆用）——中心分类器包的「adjugate/行列式表示与上游
  `C_denom` 一致」即落在这两个助手身上。
- `root_vertex_simple`（6002）：本文件注释引 `alcoves.cpp:345-408`；
  取 wall component 的 `labels_for_component`，第一个 label-1 墙作为
  「最低余根」墙剔除，其余墙生成有限部分；用 `inverse_cartan` 求
  **转置**子 Cartan 的精确逆，先以 `numer·floors` 试未移位解，坐标不
  全被 `denom` 整除时依次对每个其余 label-1 墙加一列重试（`labels_one`
  循环），全部失败报 `no root lattice vertex found for alcove
  component`。系数乘根向量累加进结果。
- **同一上游函数的两份移植并存**：`atlas-real-group/alcove.rs:643` 另有
  一个 `root_vertex_simple`（引 `alcoves.cpp:347-412`，行号区间与本文件
  注释略有出入，两处都是源码注释转述，未独立重读上游）。
  **完整逐行对账（2026-10-10）**：算法逐步等价——转置构造逐元相同
  （两边都是 `bracket(gen[j], gen[i])` 置于 [i][j]）、剔除首个 label-1
  墙、重试语义相同（alcove.rs 预生成 attempts 列表，本文件惰性
  `try_vertex(None)` 后逐列 `Some(column)`，顺序与数值一致）、整性判据
  相同（每个分子坐标被分母整除）、累加数学相同（系数×根坐标求和）。
  差异四处，前三处为已知记录，第四处为本次新发现：
  错误通道（`Result<_, String>` vs `StructureError`）、预算纪律（无 vs
  `try_reserve_exact`）、bracket 失败处理（`unwrap_or(0)` 静默置零 vs
  `?` 传播；两侧调用点都只传同一分量的已枚举根，实际不可达）、
  **溢出纪律**：本文件版用 `coefficient as i32` 截断窄化并以普通
  `*`/`+` 累加（溢出时静默回绕出错误顶点），alcove.rs 版用
  `checked_mul`/`checked_add` 再累加、逐坐标 `i32::try_from` 收窄。
  第四处是潜在的健壮性缺口而非已证实的错误结果：系数来自子 Cartan
  逆乘以小整数墙取值，目前没有展现出溢出的输入；若可达，正确行为是
  报错而非回绕。列为后续 HPC 探针候选（需构造输入），不构成修复授权。
  本文件版的唯一调用方是派发臂 `"alcove_root_vertex"`（13275/13315）。

## Weyl 词/生成元校验与值冻结（9606–9723）

AFTER-v5 落地的 Weyl 身份纪律在派发侧的配套助手：

- `require_weyl_compatible`（9606）：不兼容即运行时错误
  `Weyl group mismatch`；兼容性本身是抽象群 `Arc` 身份（见
  domain-values 包）。
- `weyl_replayed_in_left`（9620）：把右操作数的**外生成元规范词**在左
  owner 的坐标系里逐步 `right_multiply_simple` 重放；外来根置换永不直接
  比较或复合。
- `weyl_elements_equal`（9634）：先查兼容，再把重放结果与左元素的
  词级置换比较。消费方是 Weyl `=` 关系臂（855）。
- `weyl_elt_value`（9642）：值冻结点——构造 `WeylEltValue` 时一次性算
  出规范约化词（上游 `WeylGroup::word`，weyl.cpp:944-957，行号转述自
  源码注释）。消费方如 6694。
- `check_weyl_word`（9660）：引 `atlas-types.w:2344-2359`。逐条先
  `as_integer`；负数报 `Negative integer where unsigned is required`；
  u64 收窄失败报 `Integer value to big for conversion`（源码如此，注意
  与别处的 `too big` 写法并存——两处措辞都是源码转述，未独立重读上游）；
  ≥ semisimple rank 报
  `Illegal Weyl word entry {i} (should be <{r})`。消费方：validate
  （10106）。
- `check_weyl_generator`（9690）：引 `atlas-types.w:2447-2476`。**先**
  收窄到上游机器 `int`（i32），再做有符号范围检查——负 i32 经
  `usize::try_from` 失败同样落到
  `Generator {g} out of range for Weyl group (should be <{r})`。
  消费方：validate（10097）。
- `check_generator`（9710）：按实形式上下文的 semisimple rank 检查；
  报 `Illegal root index: {g}`，注释说明沿用上游
  `get_reflection_index`（atlas-types.w:4481-4489）的用户下标措辞；
  posroot/负下标是记录在案的 phase-1 暂缓。

## 边界声明

本包是对当前工作区字节的结构性阅读（git base 与哈希见快照）；上游
C++/CWEB 行号均转述自 Rust 源码注释，未独立重读上游，可能随版本漂移。
值提取器与校验助手的措辞锚点以 HPC 语料门为行为权威；两份
`root_vertex_simple` 的并存与差异是阅读观察，不是缺陷判定或修复授权。
