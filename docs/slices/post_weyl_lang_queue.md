# Post-Weyl 语言层队列（agent-30 释放 domain_builtins.rs 后按序执行）

串行纪律：语言层（typed.rs/domain_builtins.rs）同一时刻只跑一个执行者。
本队列按序做，每件走同一流程：实现 → 本地三门 → fixture+捕获 → HPC 差分 → 提交。

## 1. fiber_size 切到强实数据（B2 block_sizes 根因修复）

根因（已查明，2026-08-09）：上游 `fiberSize`（innerclass.cpp:603-614）是
**强实全纤维**按中心平方类划分的轨道类大小；Rust `fiber_size`
（domain_builtins.rs:4398）错用伴随纤维的弱划分计数。crate 强实机器已验证
正确（fiber_probe 重算 9 格全中 oracle `| 0, 0, 1 | / | 0, 0, 4 | /
| 1, 5, 12 |`；sc-B2 与 adjoint-C2 双侧逐 Cartan 吻合）。

补丁（domain_builtins.rs，~15 行）：
- `block_size_sum` 里 `fiber_size(cartan, internal, span)` 改为
  `context.strong.fiber_size(internal, id)`；对偶侧
  `fiber_size(dual_cartan, dual_internal, span)` 改为
  `dual.strong.fiber_size(dual_internal, *dual_id)`。
- `StrongRealClassification::fiber_size(form, cartan) -> Option<usize>`
  已存在（strong_real.rs:528），form 不出现时为 `Some(0)`；
  block_size_sum 已跳过不出现的 Cartan，直接 `u64::try_from` 转换。
- 旧 `fiber_size` helper（4398-4422）删除（grep 确认无其他调用点：
  当前仅 4464-4465 两处）。
- `InnerClassContext.strong` 字段在 domain_builtins.rs:150，构建在 :1686。

验证：
- `cargo run -p atlas-real-group --example fiber_probe` 已给出正确矩阵
  （探针未提交，在 crates/atlas-real-group/examples/fiber_probe.rs）。
- fixture 扩回 B2：旧版 `git show 8097c05^:tests/fixtures/domain/block_sizes.atlas`
  （多 rb/ib 两行 + `block_sizes(ib)` 一行）。扩回后重新捕获 + 差分。
- 顺手清 domain_builtins.rs 中 `_t_a` 计时残留（约 1678-1680 附近，
  下划线前缀 clippy 不报）。

## 2. alcove/FPP builtin 切片

## 1.5 Weyl_orbit 超尺寸输入语义分歧（已记录，未修）

`Weyl_orbit(rd,v)`/`Weyl_orbit_ws` 的 wrapper（atlas-types.w:1840-1888）
**不做尺寸检查**，直接调 `Weyl_orbit(rd->val, v->val)`（rootdata.cpp:1690-1876）。
v.size() != rank 时两侧确定性输出不同（A2 例：`Weyl_orbit(rd,[1,1,1])`
oracle 得 `| 1, -1, -1 | / | 0, 0, 0 |`，Rust 得 `| 1, -1, 0 | /
| 0, 1, -1 |`；`Weyl_orbit(rd,[1])` 也有分歧）。要修需逐行读 rootdata.cpp
basic_orbit/extend_orbit 对 v.size() 的假设（疑似按 v.size() 解释坐标基）。
weyl_orbit_rejected fixture 只覆盖真拒绝（类型不匹配），未覆盖此面。
agent-30 的 p11/p12 探针文件在 /tmp/weyl_probe（未跑出 .out，当时未验证）。



签名与 oracle 锚点（2026-08-09 本地 oracle 探针 /tmp/alcove_probe.atlas，
atlas-scripts 目录 + `< basic.at` + `< groups.at`）：

- `alcove_center (Param->Param)`：weyl::alcove_center（alcoves.cpp）；
  SU(2,1) trivial → `final parameter(x=5,lambda=[2,1]/1,nu=[2,1]/1)`。
- `alcove_root_vertex (RootDatum,ratvec->vec)`：
  A2 sc，`[0,0]`→`[ 0, 0 ]`；`[1,1]/3`→`[ 0, 0 ]`。
- `FPP_numers (RootDatum,ratvec->[vec])`：`[0,0]`→`[[ 0, 0 ],[ 2, 1 ]]`；
  `[1,0]/2`→`[[ 1, 0 ],[ 3, 1 ]]`。gamma 不在基本 alcove 时报 runtime_error
  （"Rational weight is not in fundamental alcove"）→ 可做 rejected case。
- `FPP_w_shifts (RootDatum,ratvec->[(WeylElt,[vec])])`：
  `[0,0]`→`[(<>,[[ 0, 0 ],[ 2, 1 ]])]`。

wrapper 位置：atlas-types.w:1945（alcove_center）、:1994
（alcove_root_vertex）、:2122（FPP_numers）、:2159（FPP_w_shifts）；
安装 :2279-2290。fixture 构造 Param 用 `param(KGB(rf,n),lambda,nu)`
（参照 full_deform.atlas）。

## 3. ext_block builtin 注册（ext_kl 已落地 602fce6，预研 agent-34 已完）

三个 builtin 签名均为 `(Param,mat)`（wrapper 先 pop mat）：

- `extended_block (Param,mat->[Param],mat,mat,mat)`（atlas-types.w:7366-7431，
  安装 :7531）：test_standard → test_compatible → gamma-fix 检查
  （`"Involution does not fix infinitesimal character"`）→ 建全 block。
  **quirk 必须复制**：扩展用的是 `rc.inner_class().distinguished()` 而非用户
  的 delta（:7392），用户 delta 只做校验门。返回 [Param]（逐元素
  lambda_rho_z = (gamma-rho).integer_diff(gamma_lambda(z))，rc.sr_gamma）、
  types 矩阵（DescValue 0..31）、links0/links1（compact/nonparity→size 哨兵；
  否则 cross 或 Cayley，epsilon<0 时取 -1-link；links1 在 link_count==1 时
  为 size，否则 Cayleys.second 或 cross）。
- `raw_ext_KL (Param,mat->mat,[vec],vec)`（:8682-8728，安装 :9103）：delta 不
  固定 gamma 时**返回空三元组**（不报错）；否则用**用户的** delta 建扩展块，
  KL 表填满；mat 条目 `flip ? -inx : inx`（符号取反池索引）；[vec] 池系数
  向量（最低次在前，:6980-6989）；vec length_stops 长 parent_max_length+2，
  `stops[i]=eb.length_first(i)`。
- `partial_extended_KL_block (Param,mat->[Param],mat,[vec])`（:7445-7468，
  安装 :7533）：走 `ext_kl::ext_KL_matrix`（crate 已备，ext_kl.rs:1159）；
  `size=eb.element(entry_element+1)`；singular=与 gamma 配对为 0 的简单
  余根；凝聚→survivors→奇偶长度取负；params 用 `rc.sr(representative, gamma)`。
  delta 不固定 gamma 时上游**先往 stdout 打印** `Delta does not fix gamma=...`
  再抛 `"No valid extended block"`——差分会比对 stdout，需决定是否复制该打印。

**预研结论（agent-34）**：三者均不需要 common-block 切片——语言层已有全块
替代模式（partial_KL_block/KL_column/W_graph 用 build_block + x 坐标定位，
domain_builtins.rs:10764/:10349）。ext_kl.rs:36-43 标注的 deferred 边界
（从 StandardRepr 建扩展块）只挡 ext_param/star 和 pool swallow。crate 调用：
`ExtBlock::build`（ext_block.rs:676）、`ExtKlTable::new/fill_columns/
kl_pol_index/polys`（ext_kl.rs:433/668/495/473）、`ext_kl_matrix`
（ext_kl.rs:1159）；对偶侧配方见 ext_block.rs 测试 :1821/:1839（转置
delta + based_involution_twist）。`compatible_outer_twist`
（domain_builtins.rs:5921）已是 test_compatible 逐字移植，直接复用。

三个 soft flag：
1. `test_standard` helper 不存在，需新写：`"{descr}:\n  {param}\n  Parameter
   not standard"`（用 StandardRepr::is_standard + Param Display）。
2. 逐元素 lambda_rho：已验证的 partial_KL_block 对所有 survivor 用单个
   lambda_rho；extended_block oracle 是逐元素算——差分定夺。
3. 注册模式照抄 raw_KL（typed.rs:4958-4969 + domain_builtins.rs:10289），
   纯 domain_builtin（BuildAndDrop）；无需新值类型，输出全是既有 Prim。

fixture 建议：A1/A2(SU(2,1)) identity delta（crate 锚点 ext_block.rs:1983/
:2068）、SL(3,R) 翻转 distinguished（打印列约定，传参要转置 [[1,1],[0,-1]]）、
Sp4 12 元素非退化（agent-33 锚点）、raw_ext_KL 空三元组情形、
partial 在 singular/regular gamma 各一例；rejected：非 standard Param、
四种 compatible_outer_twist 措辞、extended_block 的 gamma-fix 拒绝、
partial 的 stdout 打印+异常情形。

**oracle 探针捕获（2026-08-11，/tmp/ext_*_probe.at，本地钉住 oracle 4d3e9449）**：

- SU(2,1) 构造：`simply_connected(Lie_type("A2"),true)` + 单位对合的
  inner_class + `real_form(ic,1)`（打印 'su(2,1)'）；标准参数
  `param(KGB(rf,0),[0,0],[0,0]/1)`（归一化为 lambda=[1,1]/1,nu=[0,0]/1）。
  extended_block(p,id) 的 types 矩阵与 crate 锚点逐字节一致
  （[[2,2],[2,9],[9,2],[0,3],[3,0],[1,1]]）；raw_ext_KL pool [[],[1]]、
  stops [0,3,5,6]；partial 在 gamma=0（全 singular）凝聚为 1 survivor。
- SL(3,R) 构造：`inner_class(rd,[[0,1],[1,0]])`，只有 1 个 real form
  （'sl(3,R)'），distinguished 即图自同构 [[0,1],[1,0]]。
  extended_block(p,[[0,1],[1,0]]) → 2 元素，types |26|,|27|（与 crate
  锚点一致）。传 [[1,1],[0,-1]] 在此 inner class 下报
  `"Matrix maps simple root 0 to non-root"`（新措辞，来自 test_compatible
  的 based-root-datum 校验；agent-33 的 [[1,1],[0,-1]] 锚点属于
  groups.at trivial(SL(3,R)) 的另一种 inner class 构造，勿混用）。
- gamma 不固定（p2=param(KGB(rf,0),[0,0],[1,0]/1)，nu 归一化 [1,-1]/2，
  delta=[[0,1],[1,0]]）三种行为逐字节确认：
  - raw_ext_KL → `Value: (The 0x0 matrix,[],[ ])`（空三元组，无错）
  - extended_block → Runtime error `"Involution does not fix infinitesimal
    character"`
  - partial_extended_KL_block → **stdout 先打印** `Delta does not fix
    gamma=[3,1]/2.`（gamma 经归一化，不是 nu 原样）**再抛** Runtime error
    `"No valid extended block"`——Rust wrapper 必须复制该 stdout 行
  - partial(p2, id) → 1 survivor，pool [[],[1]]

## 4. print 族（顺序见 agent31 侦察报告 §6）

print_gradings → print_real_Weyl + print_blockstabilizer → print_X →
print_common_block（最后，绑 srm pool）。print_strong_real 已存在
（typed.rs:5275），勿重复。

**print_gradings wrapper 事实（atlas-types.w:4260-4300，安装 :9108）**：
签名 `(CartanClass,RealForm->)`（wrapper 先 pop rf 再 cc）。错误：
`"Inner class mismatch between real form and Cartan class"`、
`"Cartan class not defined for this real form"`。用
fiber().weakReal() 分块 + realFormLabels 翻译分块号→real form，
simpleImaginary 根的子系 Cartan 矩阵过 DynkinDiagram 取 Bourbaki
编号排列 sigma。oracle 布局（探针 /tmp/gradings_probe{,2}.at，B2/A2）：
`Imaginary root system is of type B2, with simple roots 4,5.`（空时
`Imaginary root system is empty.`），随后每个 weak-real 元素一行
grading `[01]`（空时 `[]`）。注意根号 4,5 是 RootNbr（全根系编号），
不是简单根下标。多个 real form 共享 Cartan 类时每类只打对应 part。

**shift_flip 依赖修正（2026-08-11 查证）**：队列旧注"最便宜 ~50 行"
不成立。wrapper（atlas-types.w:7341-7362，`(Param,mat,ratvec->bool)`，
安装 :7530）需要 per-parameter 扩展：
`ext_block::shifted_default_extension(ctxt,p,gamma)` +
`ext_block::is_default` + `Ext_rep_context`——crate 侧 ext_block.rs 只有
块级 ExtBlock + StarOracle::same_sign 缝（ext_block.rs:1356-1378），
没有 per-parameter default_extend/shifted_default_extension。故
shift_flip 归属 ext_param+star 大切片（~1000-1200 行），不是独立小件。
两个 gamma 检查的措辞不同：`"Involution does not fix rational weight"`
（对 ratvec 参数）vs `"...infinitesimal character"`（对 p 自带 gamma）。

## 5. 其余（REMAINING_BUILTINS.md）

root_expression/root_index/root_permutation（oracle 根编号阻塞）、
root_ladder_bottoms/coroot_ladder_bottoms、shift_flip（见 §4 依赖修正，
绑 ext_param+star）、twisted_KL_sum_at_s、twisted_deform 族、
block_deform/KL_block/dual_KL_block（common-block srm pool 语义）。

### 5.1 root_ladder_bottoms/coroot_ladder_bottoms（小件，可独立切片）

wrapper atlas-types.w:1569-1599（安装 :2241-2244），签名
`(RootDatum,int->[int])`。语义：`min_roots_for(α)`/`min_coroots_for(α)`
= 使 β-α 不是根的 β 集合（含 α 自身），返回有符号根号列表。
语言层已有 internal_root_index/convert_to_signed_root_index 等价物
（domain_builtins.rs:2901/:6562/:8597，walls 切片落地）；crate 缺
`min_roots_for`/`min_coroots_for`（root_system.rs 预计算，小块新增）。
oracle 探针（/tmp/ladder_probe.at，B2）：`root_ladder_bottoms(rb,0)` →
`[-4,-3,-1,0,1,2]`；越界报 `"Illegal root index 4"`。

### 5.3 小件 sweep（臂已存在，注册翻转 + 1 个新臂；语言层空出后即可做）

2026-08-11 查证：以下 skip 占位的臂已全部存在并与 live 兄弟共享，
只需把 domain_builtin_skip 翻成 domain_builtin（照 08-06 sweep 模式），
外加 fixture/差分：
- `poscoroots`（共享 posroots 臂，domain_builtins.rs:8165）
- `simple_coroots`（共享 simple_roots 臂，:8178）
- `two_rho_check`（共享 two_rho 臂，:9212；上游 two_rho_check 推的是
  dual_twoRho()，atlas-types.w:1415-1420，注意区分）
- `coroot_radical`（共享 root_coradical 臂，:8205；= 简单余根矩阵 +
  radical_basis，atlas-types.w:1691-1706）
- `mod_central_torus_info`（共享 derived_info 臂，:8982）
- `adjoint (LieType,bool->RootDatum)`（共享 simply_connected 臂，:8085；
  上游 wrapper atlas-types.w:1346-1360 纯组合：Cartan 转置 + 零对角补 1）
- `semisimple_rank (RootDatum->int)`：从未注册，需新注册 + 小臂
  （crate root_datum.rs:94 已有 semisimple_rank()；上游 wrapper
  atlas-types.w:1397-1400，安装 :2222）
适合派给轻量 subagent；fixture 走既有流程，可与下一语言切片合并差分。
追加：`reducibility_points (Param->[rat])` 也是近翻转——crate
rep_context.rs:1156-1159 已移植（repr.cpp:825-925），只差注册 + 薄
wrapper（atlas-types.w:6561-6568，安装 :7500-7501）。

### 5.4 deform/KL 族部分签名（臂存在但 partial，需真实现，非翻转）

08-06 sweep 记录"arms were partial"的精确签名地图（typed.rs skip 行 →
共享臂位置）：
- `KL_sum_at_s_to_height (Param,int)` → 共享 KL_sum_at_s 臂（:10516）
- `dual_KL` → 共享 raw_KL 臂（:10608）
- `W_cells` → 共享 W_graph 臂（:11269）
- `null_module` → 共享 null_K_module 臂（:12002）
- `first_term`/`last_term (Split,KType)` 与 `(Split,Param)` → 臂 :12021
  （KType/Param 两分支都在，缺口在 Split-int 定稿语义）
- `K_type_pol (Split,Param)` → 臂 :12074
- `truncate_above_height (KTypePol,int)`/`(ParamPol,int)` → 臂 :12310
- `KL_block` 第二重载 → 臂 :10774（common-block/PolP 缺口）
缺口标签（REMAINING 2026-08-06）：Split 定稿 + common-block srm pool +
PolP；实现切片按签名逐个补齐并用差分定夺。

### 5.2 affine_orbit_ws/basic_orbit_ws（中等 crate 切片）

wrapper atlas-types.w:2014-2060（安装 :2241 附近；basic_orbit_ws
`(RootDatum,[int],int->[W_elt])`，affine_orbit_ws `(RootDatum,ratvec->[W_elt])`）。
需要 weyl.cpp 的三个未移植函数：`affine_orbit_ws`、
`complete_affine_component`、`finite_subquotient`（crate weyl.rs 目前只有
WeylAction/WeylGroup 枚举，无轨道生成器）。basic_orbit_ws 的校验：
`"Index too large for given list of root numbers"`、锐角检查
`"Roots {a} and {b} have acute angle."`；to_affine_orbit 判定（final 的
余根是否依赖 stab 的余根 + stab 与 final 所在 Dynkin 分量求交）。
affine_orbit_ws 尺寸检查 `"Rank and rational weight size mismatch {r}:{s}"`。
W_elt 渲染已存在（Weyl_orbit 切片）。
