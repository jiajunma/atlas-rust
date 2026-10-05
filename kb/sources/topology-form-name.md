---
title: 对偶分量群平凡性与实形命名（topology.rs / form_name.rs）
source: atlas-rust/topology-form-name
ingestedAt: 2026-10-06T02:20:00Z
---

# 对偶分量群平凡性与实形命名（topology.rs / form_name.rs）

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `crates/atlas-real-group/src/` 的 `topology.rs`（400 行）与
`form_name.rs`（362 行）。两者共同服务 presentation 层（见
weyl-size-presentation 包），但在所给字节内互不导入。本包是结构性阅读，
不声称这两层的数学验收；上游引用（topology.cpp:165-192、tori.cpp:162-176、
io/output.cpp:556-782 等）仅转录自代码注释，本包未核对上游字节。

## topology.rs：对偶分量群的平凡性与秩

实形式的连通性 = 其最分裂 Cartan 的对偶分量群（`dual_component_group_basis`，
topology.cpp:165-192）。设定：格基 `B` 的列 = 简单余根 + 余权格 radical 的一
组基；`i_sw = B^t·θ·B^{-t}` 是对合转运到单连通覆盖权格上的矩阵（上游
`theta.transposed().on_basis(basis).transposed()`）。对偶分量群 = 限制映射
`dualPi0(θ) -> dualPi0(i_sw)` 的核（由 `B_z^t mod 2` 诱导，`B_z` 把 B 的
radical 列清零）；实形式连通当且仅当核消失（诱导映射单射）。

构件：

- `dual_pi0`（私有）：上游 `tori::dualPi0` 子商（tori.cpp:162-176）：
  `ker_{F2}(θ+1)` 模「饱和 +1 特征格的 mod-2 像」。
- `CorootRestriction`（私有，`impl ModTwoAmbientMap`）：环境 mod-2 空间上的
  限制映射 `x -> B_z^t x mod 2`（逐输出列奇偶累加）。
- `dual_component_group_trivial`（`pub(crate)`）：形状检查 → radical（无根
  datum 取全格）→ 组基 B（先简单余根列、后 radical 列，分量经
  `i32::try_from`）→ 有理计算 `i_sw`（`invert_rational` + 两次
  `rational_product`，逐项 `integral_entry` 整性检查——上游 `on_basis`
  整除精确）→ 两侧 `dual_pi0` → `validate_induced_map_to`（诱导映射须可
  下降）→ 逐源基代表映射进目标子空间，返回「像秩 == 源维数」。
- `dual_component_group_rank`（`pub`）：同一管线，末行改返回
  `源维数 − 像秩`（即 `dualComponentReps` 基的大小；平凡恰为 0）。
  **阅读观察**：两函数整段管线复制，仅末行不同——存在同步漂移风险（已
  记录）。

测试锚点（5 个）：sc A1 split（B=[[1]]，限制为恒等，连通）；adjoint A1
split（B=[[2]] 在 mod 2 消失，PGL(2,R) 两分量）；sc A1 compact（θ=+1，
分子为零）；adjoint B2 compact；GL(2) 样式 split（环面坐标被清零，秩一
对偶分量群）。未覆盖：`dual_component_group_rank` 无直接测试；无根 datum
分支无测试。

## form_name.rs：实形式的李代数命名

上游 `output::printType` 及辅助 `print_real_form_name`/`printComplexType`/
`split` 的忠实移植（io/output.cpp:556-782）。输入 grading 是
`specialGrading` 划分重载（datum 单根上的位集，1 = 非紧虚根——见
real-form-labels-order 包的 `special_grading_key`），经拉回
`pulled[k] = grading[perm[k]]` 适配到 Bourbaki 序（permutations.cpp:66-76），
像上游 `gr >>= rank` 循环一样逐因子消费。

- `split(name, n, m)`：`m == 0` → `(n)`；否则 `(n-minority, minority)`
  弱递减（`su(1,2)` 打印为 `su(2,1)`）。
- `complex_name`（`'C'` 条目覆盖两个同构因子，传入第一个）：A→sl(n+1,C)、
  B→so(2n+1,C)、C→sp(2n,C)、D→so(2n,C)、E→e{rank}(C)、F→f4(C)、G→g2(C)、
  T→gl(1,C)。
- `factor_name(bits, letter, rank, ic)`：`m` = 最低置位 + 1（平凡为 0）。
  规则表（要点）：A1 平凡 su(2)/否则 sl(2,R)；A 更大：紧 `su` 分裂；
  不等秩 A 奇秩且平凡 → `sl((n)/2,H)`（注释：两个条件缺一不可）；B：
  `so(2n+1, 2m)`；C：`m == rank` → `sp(2n,R)` 否则 `sp(n,m)`；D：等秩分支
  `so*` 系列（`[1,0]`/`[0,1]` 由 rank%4 与 m==rank 的相等性决定），不等秩
  D4 的 `so(7,1)`/`so(5,3)`（任何非零 grading 都是 so(5,3)）；E：平凡且
  （紧或 rank>6）→ 裸 `e{rank}`，E6/E7 的 m 分派表，E8 经 `0xCC` 掩码
  （e7.su(2) 的非紧位 {2,3,6,7}）；F/G；T：紧 `u(1)`、否则 `gl(1,R)`。
  折叠小写字母 `'f'`/`'g'` 仅出现于某些对偶实形的对偶侧命名，本移植不覆盖。
- `form_type_name(layout, grading)`：perm 位置 ≥ 128 拒绝（u128 位集宽度）；
  逐字母：`'C'` 消费两个因子与两段切片；环面因子不消费 grading 位；否则
  经 perm 拉回切片位后调 `factor_name`。

测试锚点（7 个）：A1 紧/分裂名；A2 的内类字母分派（含弱递减打印与
「不等秩 A2 只有 sl(3,R)」）；A3 不等秩的 sl(2,H)/sl(4,R)；Complex 对的
sl(2,C)；B2 的三名（bit 0 = datum 序中前长根 → so(3,2)）；D4 不等秩的
so(7,1)/so(5,3)；环面字母的 u(1)/gl(1,R)。

## 接口关系与限制

两文件互不导入；共享 `StructureError`（注意两条导入路径写法不同：
`crate::StructureError` vs `crate::error::StructureError`——同一类型的
再导出）。对外可见面：`dual_component_group_rank`（pub）与
`form_type_name`（pub）。限制：不做数学验收；`factor_name` 的 `bits: u32`
假定单因子切片 ≤ 32 位（无防护，阅读观察）；topology 不校验 θ 是否为对合
（调用方保证）；E/F/G 与 `so*` 分支无测试锚点。

## 来源与限制

精确读取身份见
[`2026-10-06-topology-form-name.json`](snapshots/2026-10-06-topology-form-name.json)：
绑定 Git base、两文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe
（无工具档案）以两文件完整字节起草（exit 0，264.2s），维护者对照源码逐条
核对改写。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。
