---
title: 实投影像基对与精确整数矩阵约化（real_projection.rs / matreduc.rs）
source: atlas-rust/real-projection-matreduc
ingestedAt: 2026-10-06T07:50:00Z
---

# 实投影像基对与精确整数矩阵约化（real_projection.rs / matreduc.rs）

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `real_projection.rs`（566 行）与 `matreduc.rs`（755 行）。两文件
互不调用，但同属上游 `utilities/matreduc` 血统的不同分支，共享同一 gcd
扫描骨架而溢出制度相反（i64 checked vs i32 wrapping）。本包是结构性阅读，
不声称数学验收；上游行号仅转录自注释。

## real_projection.rs：(1−θ)X\* 的像基对

`RealProjection { lift_mat, m_real }`（pub(crate) 字段）= 上游
`InvolutionTable::record` 对（involutions.h:104-105），满足
`lift_mat * m_real == 1 − θ`。基**不由 θ 唯一确定**：上游在 Cartan 轨道
典范对合处播种再沿 cross BFS 运输，故选举的 λ−ρ 代表元与 y_lift 符号依赖
精确像基——这是逐操作复刻 `matreduc::column_echelon`（含其 gcd 扫描）的
动机。

- `build(theta)`：a = 1−θ（checked）；行扫描**自底向上**，每行
  `gcd_sweep(row, limit)`，主元落列 `limit−1`；零列擦除时核列逐列向右端
  轮转（已停放列不再动）；`m_real = col.inverse().block(0,0,r,n)`，其中
  `invert_integer_matrix` 用欧几里得行消元求幺模逆（主元选列内绝对值最
  小者；对角须为 ±1，−1 整行取负；末尾逐项验证 `M·M⁻¹=I`）。收尾
  `check_against` 自校验分解（`"image basis factorization"`）。
- `gcd_sweep`（对应 matreduc.h:70-122 的 gcd + column 记录）：复制局部行；
  最小绝对值主元；**负主元取正并记 `ops[mindex][mindex] = −1`**（注释：
  E6 involution-187 的分解只有带这个记录符号才成立）；消元用
  `div_euclid`（负余数会选出负主元、反转典范像基定向）；ops 同时作用于
  `a` 与 `col`（`apply_column_ops` = 上游 `column_apply` 的前 limit 列）。
- `transported(reflection)`（上游 `add_cross`）：`m_real' = m_real·s`、
  `lift_mat' = s·lift_mat`；像基路径相关（与新做阶梯约化差列号/列序），
  不变量 `(sL)(Ms) = s(1−θ)s = 1−θ'` 保持。只校验方阵性，**不**校验
  反射矩阵阶 = n，也不重跑 check_against。
- 阅读观察：`coordinates` 的 zip 截断（短 weight 静默按零）；`lift` 对过长
  坐标直接下标会 panic；`invert_integer_matrix` 的消元用**非受检**普通
  算术，与文件其余的全 checked 风格不对称；分配受检性也不对称
  （try_reserve_exact vs vec!/to_vec/collect）。

测试 4 个：带符号 gcd 扫描对 original3840186 的逐字锚定
（`[[8,-12],[4,-6]]` → pivot 2、image `[[0,4],[0,2]]`、columns
`[[-3,2],[-2,1]]`）；斜环面（θ=[[−7,12],[−4,7]]）与斜乘积的像基字面量；
零/满像边界（恒等 → rank 0，−I → 满秩、lift_mat = 2I）。`transported`
无测试。

## matreduc.rs：diagonalise 与精确求解

逐操作移植上游 `matreduc.cpp`（`diagonalise`/`gcd`/`has_solution`/
`find_solution`）+ `matrix::inverse_upper_triangular` + `arithmetic::exp_i`。
动机：欠定系统的**选定解**在下游可观测（τ/t 坐标奇偶性进入
`ext_block::same_sign`），故复现幺模操作序列与行列式符号簿记；算术全程
wrapping i32，镜像 C++ int（含溢出域）。

- `divide(a,b)`：正除数下取整（`a≥0` 直除，否则 `-1-((-1-a)/b)`，避开
  i32::MIN 取负）。
- `gcd(row, &mut flip, dest)`：最小绝对值主元；负主元取正时翻转 flip 并在
  记录矩阵置 −1；`divide` 消元；末尾列交换到 dest 也翻 flip。
- `diagonalise`：返回 `(row, col, diagonal)`，`row·m·col` 对角，对角项
  除首项外为正。**簿记细节（逐行核对）**：每列首个 gcd 的 flip 对
  `row_minus` 是**覆盖**赋值；内层循环交替行/列 gcd，flip 分别 ^= 进
  col_minus/row_minus；退出后再 `row_minus ^= flip`（从行 gcd break 退出时
  该 flip 计入两侧；从列 gcd break 退出时净效果抵消）；主元列未左对齐时
  用稳定排列 `pull_back_columns` 并异或置换符号；最后
  `row_minus != col_minus` 时 `diagonal[0]` 取负，`row_minus`/`col_minus`
  分别经第 0 行/列乘 −1 归一（注释口径不一致：一处说 "ensure det(row)=1"，
  测试注释说上游只强制 det(col)=1、det(row) 可为 −1——以测试为准）。
- `has_solution`（逆序整除/归零判定）、`find_solution`（None 取代上游异常；
  调用方应先 has_solution）、`in_left_image`/`in_right_image`
  （ext_block.cpp 的 in_L_image/in_R_image，分别用左/右因子变换后判定）。
- `inverse_upper_triangular`：非方阵/非单位对角报 `RepInvariantViolation`
  （两处 invariant 字面量），回代 wrapping。`exp_i`：偶数前提为上游
  assert（debug_assert），n%4==0 → 1 否则 −1；release 下奇输入落 −1 分支。
- panic 面：from_entries 形状、apply_to/right_prod 长度、transpose 方阵、
  row/column_apply 边界全部 assert——故 has_solution 等对长度不匹配的 b
  是 panic 而非 Err。

测试 6 个：11 用例重构（|det(row)|=1、det(col)=1、逐对角核对、首项外为
正）；find_solution 三形态；像判定一维例；**oracle_reference_cases**——
取自 C++ oracle 的逐字节锚定（[[0,5],[0,0]] → diagonal [−5] 与精确
row/col；[[−4]] → [−4]；6×6 秩亏且主元列未左对齐的完整 row/col 字面量与
解 `[6,14,5,−37,−421,345]`）；单位上三角逆两例 + 两拒绝；exp_i 五点。

## 接口关系与限制

两文件无直接调用；分歧表：i64 checked vs i32 wrapping、div_euclid vs
自定义 divide、ops 就地施加 vs 返回后 transpose+apply、符号记入 ops vs
显式 flip 簿记。real_projection 不复用 matreduc 的 gcd 是否刻意（隔离两种
溢出制度），字节无说明——复核备注。未测面：matreduc 的 wrapping 溢出域
（文档声明可观测）、in_*_image 的矩形/秩亏、空形状 diagonalise；
real_projection 的 transported/各错误分支。

## 来源与限制

精确读取身份见
[`2026-10-06-real-projection-matreduc.json`](snapshots/2026-10-06-real-projection-matreduc.json)：
绑定 Git base、两文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe
（无工具档案）以两文件完整字节起草（1200s 期限，exit 0，455.9s），维护者
对照源码逐条核对改写（含 diagonalise 符号簿记的逐行追踪复核）。本次知识
维护未执行 Atlas、Cargo、测试或 benchmark。
