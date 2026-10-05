---
title: 弱实形 W_im 轨道划分与扭对合表（weak_real_form.rs / involution_table.rs）
source: atlas-rust/weak-real-form-involution-table
ingestedAt: 2026-10-06T08:40:00Z
---

# 弱实形 W_im 轨道划分与扭对合表（weak_real_form.rs / involution_table.rs）

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `weak_real_form.rs`（836 行）与 `involution_table.rs`（840 行）。
两文件互不导入，但都实现了同一道分解闸门（`compose_matrices(w·δ) == θ`
→ `DistinguishedInvolutionMismatch`）。本包是结构性阅读，不声称数学验收；
上游引用（innerclass.cpp、involutions.cpp、atlas-types.w）仅转录自注释。

## weak_real_form.rs：伴随纤维的 W_im 轨道

`WeakRealFormId` 的编号 = 各轨道最小元按典范坐标整数序升序赋号；类 0 =
恒等元轨道（quasisplit 规范化）。注释称 stage-(d) 审计（SEED_X0_DESIGN.md）
已证明该编号与上游内部 `RealFormNbr` 一致、不在 adapter 暂缓范围内；仅
解释器的外部 `FormNumberMap` 顺序需要 adapter 置换（注释转述）。

- `WeakRealFormPartition::build(grading, max_elements)`：维度 > 63 拒
  （`MAX_MASK_BITS`：u64 掩码的移位安全界）。逐虚根收集 base 位、
  列掩码（`alpha_columns[i]`：哪些伴随基方向的 shift 使根 i 非紧）与
  平移掩码（`m_alpha_masks[i]`：伴随 m_α 的坐标）。`max_elements` 是对
  `2^dimension` 的唯一预算旋钮。
- `walk_mask_orbits`（pub(crate)，调用方自带错误构造器以归属阶段）：u128
  计算的 `1 << dimension` 与 `widened(max_elements)` 比较（故意不饱和，
  防止 dimension==64 借 usize::MAX 漏过）；按掩码升序播种、LIFO 栈游走，
  转移规则 = FiberAction：`base[i] XOR parity(mask ∧ alpha_columns[i])`
  非紧则平移 `m_alpha_masks[i]`。类号按升序最小掩码赋号，首次出现即代表
  元。生成元对合性（`<α, α∨⟩ = 2` mod 2）仅 debug_assert。类表是
  `Vec<u32>`，`CLASS_SENTINEL = u32::MAX` 保留——`seeded_class` 守卫在
  单连通积（无伴随 m_α，每掩码自成一类）下原则上可达。
- `weak_real_form_at_representative`（上游 `real_form_of` 的代表元级内核，
  atlas-types.w:3878-3894 / innerclass.cpp:1305-1355）闸门序列：① 原始
  因子长度 = 格秩；② **出处闸门**：当场重建基本扭对合（恒等 Weyl 作用）
  并与 `classification.cartan_classes()[0]` 的代表比较（分类不持有内类
  句柄，故显式重建；注释称未来可用保留令牌省此开销）；③ twisted 的
  datum；④ `w·δ == θ` 分解（weight+coweight 双矩阵）；⑤ 代表元查找；
  ⑥ `project_torus_factor`：`v → (v + vθ)/2`，**行向量/右乘约定**；
  ⑦ 整性闸门先于虚 grading 提取——对**每个单根**（非仅虚根）要求整值
  配对，使实 Cartan（虚基为空）无法绕过（`InvalidStrongTorusFactor`）；
  ⑧ 偶配对标记非紧虚根；⑨ grading → `element_from_grading` → 局部
  `class_of` → `labels().label(local)` 得全局编号。约束：`twisted` 必须
  恰好是存储的代表元；搬运一般扭对合的环面因子需要表驱动的 Tits cross
  作用（本函数不建表）。

测试 11 个：合成 A1 锚点（紧/分裂 Cartan 配 0 → 类 0；紧配 1/2 → 类 1）；
秩与半整投影闸门；外来同秩分类先报 `DatumMismatch`；A2 反射的半整投影
（Display 文案钉死）；A2 恒等 2 类、B2 恒等 3 类（代表元精确钉定）、SC
A1 平凡作用 2 个单元类、A2 图扭转 1 类；外来元素/欠预算拒绝；rank-33 由
预算拒绝而 rank-64 由 mask-bits 拒绝（诚实的预算门而非秩上限）；
`seeded_class` 哨兵守卫直测。

## involution_table.rs：扭对合表（KGB stage b）

每条 `InvolutionRecord` = WeylElement + TwistedInvolution + mod-2 去重子空间
（`negative_coweight_eigenspace` → `reduce_basis_mod_two`；其有序基服务
Tits 阶段的 inverse-Cayley 修复）+ `(1+θ)ρ`（存为 `(2ρ + θ·2ρ)/2`，奇坐标
报 `"theta rho parity"`）+ 双长度 + `(1−θ)X*` 像基对。**除像基对外所有
字段入表时从 θ 重新典范推导；像基对在轨道典范对合处播种、沿 cross BFS
搬运**（路径依赖，y_lift 符号依赖它——B2 x=4 测试锚定：搬运值
`lift_mat=[[2],[-2]]` vs 现场重算 `[[-2],[2]]`，`assert_ne`）。

- `new`：预算扭转移位（InvalidBasedAutomorphism 两分支）、预建全部单反射
  的元素与作用（BFS 边不得每次重建反射）、从正性切片累加 `two_rho`。
- `add_cartan`：幂等（重加返回既有切片）；种子 = 类代表 +
  `CayleyCrossDecomposition`（每类一次，绝不在条目级）算
  `(W_length + #Cayley)/2`（奇数报 `"length parity"`）；外序 BFS：邻居 =
  `s_g · current · s_{twist(g)}`（词级两次 multiply，作用级两次 compose），
  去重键 = 前向根置换（stage (a) 钉定的完整相等键）；新长度
  `stepped_length`（Weyl 长度差恰 ±2 → 对合长度 ∓1，否则
  `"twisted length step"`；差 0 已被去重命中消费）；投影用**普通生成元
  s 而非 twist(s)** 的矩阵搬运（δ 已并入 θ）；每访问节点推一条
  cross_links（此后 cross 是 O(1) 直查）；闭轨后大小须恰为类的
  `twisted_involution_count`（`"orbit size"`）。条目上限是包含式
  （`len == max_involutions` 即拒）。
- `push_record`：搬运投影先 `check_against` 本记录新鲜推导的 θ 再采用
  （边数学对账）。种子插入 `index_by_permutation` **无碰撞检查**
  （BTreeMap::insert 同键静默覆盖——依赖不同 Cartan 轨道键不重叠的调用
  纪律，阅读观察）。
- 查询：`lookup`（置换键）；`cayley` 在目标 Cartan 未加入时返回
  `Ok(None)`（stage-(e) 契约：先加入该形的上闭 Cartan 集，此后 None 即
  调用方不变量违例）；`cross` 查存储链接；`simple_root_kind` 一条访问器
  覆盖上游三个 is_*_simple；`cartan_of` 扫描轨道切片。

测试 7 个：A1 分裂两单元轨道 + 幂等 + 逐字段锚定；B2 全表与分类对账且
两次建表逐切片相等（可复现性）；B2 每条记录的 θ 典范性（像 =
weyl.image(δ.image(root))、(W+Cayley)/2 公式、(2ρ+θ·2ρ)/2）；Cayley 边在
其 Cartan 加入前后（None → Some）；扭转 A2 轨道大小 [1,3]；B2 投影搬运
锚点；预算/越界守卫。

## 接口关系与限制

两文件共享 `try_capacity`、`compose_matrices`（weak_real_form 顶部导入，
involution_table 在 `gate_twisted` 内局部导入）与同一闸门逻辑。文档层互指：
weak_real_form 的「表驱动 Tits cross 搬运」指向 InvolutionTable 的
cross/cayley；两处的 inverse-Cayley 注释都指向后续 Tits 层。未触分支：
四个不变量错误字面量（length parity / orbit size / theta rho parity /
twisted length step）、`AllocationFailed`、class_of_mask 越界等。

## 来源与限制

精确读取身份见
[`2026-10-06-weak-real-form-involution-table.json`](snapshots/2026-10-06-weak-real-form-involution-table.json)：
绑定 Git base、两文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe
（无工具档案）以两文件完整字节起草（1600s 期限，exit 0，658.1s），维护者
对照源码逐条核对改写。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。
