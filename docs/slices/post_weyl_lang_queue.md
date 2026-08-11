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

**fixture 已备（2026-08-11，未跟踪文件，等实现切片）**：
`tests/fixtures/domain/coroot_queries.atlas(+_rejected)` 与
`root_numbering.atlas(+_rejected)`，均已按 harness 真实条件（无 basic.at
预载，source+quit）在 pinned oracle 上验证：accepted 0 错误，rejected
错误逐条确认。探针事实：
- `reducibility_points` 非空样例：SL(2,R)（rd1=sc-A1，ic=inner_class(rd1,
  [[1]])，rf1=real_form(ic1,1)=split）x=KGB(rf1,2) 上 nu=[3]/2→[2/3]、
  [2]/1→[1/2]、[5]/2→[2/5]、[4]/1→[1/4,3/4]；nu=[1]/2 与 SU(2,1) 平凡
  参数 → []。语义（repr.cpp:825-888）：实根 |num|≥d 才有点，
  fracs=RatNum(s,num)，s 按奇偶表步进 2d。
- `adjoint(Lie_type("A1.T1"),false)` → Runtime error `"Sub-lattice matrix
  should have size 2x2"`（Cartan_matrix(lt) 对带环面型只给半单尺寸）——
  好 rejected 案例。带环面 datum 用 `root_datum([[2,0]],[[1,0]],true)`。
- `coroot_radical` = 简单余根 ++ radical_basis（rank×rank 列矩阵，
  atlas-types.w:1691-1698）；sc-B2 上是单位阵。`root_coradical` 已 live。
- `two_rho_check` 的 [int]/bool 第二参数重载是 basic.at 脚本层定义，
  harness 不可用——rejected fixture 里 `two_rho_check(sc,[1,0])` 在
  无 basic.at 时是类型错误 `"found (RootDatum,[int]) while RootDatum
  was needed"`。

**print_X wrapper 事实（atlas-types.w:8999-9008，安装 :9124）**：签名
`(InnerClass->)`。`kgb::global_KGB kgb(G)`（全 square classes 的全局
Tits KGB，kgb.h:213-266 + kgb.cpp 部分，共 ~1280 行文件）+
`kgb_io::print_X` 布局（kgb_io.cpp，全文件 277 行）。crate 里停放的
global_tits.rs（551 行）是合成 real-form 的有理环面运输层，**不是**
global_KGB——print_X 需要新的 crate 切片（估 600+ 行），不能复用停放件。

### 5.5 twisted/deform 族 + print 三件套签名锚点

- `twisted_deform (Param->ParamPol)`（wrapper :8120，安装 :8573）
- `twisted_full_deform` 两重载：`(Param->KTypePol)`（:8229，安装 :8577）
  和 timed `(Param,int->|KTypePol)`（:8293，安装 :8581）
- `twisted_KL_sum_at_s` 两重载：`(Param->ParamPol)`（:8370，安装 :8586）
  和 external `(Param,mat->ParamPol)`（:8420，安装 :8589）
- `block_deform (Param,ParamPol,int->ParamPol,ParamPol)`（:8178，安装 :8574）
- `dual_KL_block (Param->[Param],int,mat,[vec])`（wrapper :7053，安装 :7517）
- `print_common_block (Param->)`（print_c_block_wrapper，安装 :7505）、
  `print_partial_block (Param->)`（:7507）、
  `print_partial_common_block (Param->)`（:7509）——均绑 common-block
  srm pool，最后做

**srm pool 语义锚点（2026-08-11 补查，repr.h/atlas-types.w）**：
- "srm pool" = `Rep_table`（repr.h:534）：common_block 池，键为
  `Reduced_param` 哈希（reduced_hash → place → block_list 迭代器 +
  块内元素号）。参数命中已存 Reduced_param 时，存储块与用户块同构，
  `block_modifier` 记录回变换。
- `block_modifier`（repr.h:493）：继承 `locator`（repr.h:485：
  `int_sys_nr` 积分系序号、`w` WeylElt（作用于基本 alcove 积分系）、
  `simp_int` 单积分根像、`simple_pi` 置换）+ `RatWeight shift`。
- `print_c_block_wrapper`（:6668-6695）：`rt().lookup_full_block(val,
  init, bm)`；输出头 `Parameter defines element N of the following
  common block,\nas transformed by <w0.w1...>`，bm.simple_pi 非单位时
  追加 `, simple reflections permuted (i->j,...)`，再 `:` 换行；
  `block.shift(bm.shift)` 后用 `block.singular(bm,gamma)` 打印，打完
  shift 回去。
- `print_pc_block_wrapper`（:6713-6739）：`rt().lookup(val,init,bm)`
  （partial）+ bruhatOrder().poset().below(init)；位图满且非末元素时
  打 `Elements <= N of following block\n`，否则打
  `Subset {n,...,init} in the following common block:\n`，再 shift+打印。
- `common_block_wrapper`/`partial_common_block_wrapper`（:6748/:6786，
  已 live）：survivors = `block.survives(z, singular(bm,gamma))`，
  回变换 `rc.sr(block.representative(z), bm, gamma)`。
- **Rust 现状**：block/partial_block 用 `common_block_members`（按
  gamma-lambda mod 余特征格的反射闭包）仿真 lookup_full_block，
  差分已验；但仿真**不产生 bm 显示数据**（w 字、simple_pi、shift），
  print 两件套恰恰要打这些——实现切片必须补一个最小 bm 等价物
  （变换记录），或另寻差分等价的显示路径。差分定夺。
相关已 live 兄弟：full_deform 两安装行（:8575/:8578）、KL_sum_at_s
（:8583）、block/partial_block（common_block_wrapper 系）。

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

**twisted 族 crate 缺口清单（2026-08-11 补查）**：
- `twisted_deform`（:8120-8150）：核心是 `Rep_table::
  twisted_deformation_terms`（repr.cpp:2426-2520，~95 行）——依赖
  ext_block 版 `contributions`（repr.cpp:1901-1931，~30 行，用
  first_descent_among/descent_type/is_like_compact/has_double_image/
  Cayleys/some_scent/epsilon + "October surprise" 长度变 2 符号）、
  ExtKlTable（已有）、pool 在 q=-1 求值、orientation_number（语言层
  orientation_nr 已 live，repr.cpp:455-493 已移植）、exp_i。
  **crate 缺**：ext contributions；其余齐。
- `twisted_full_deform`（:8229+）：`Rep_table::twisted_deformation`
  （repr.cpp:2552-2653）= scale-0 + reducibility 递归（对照语言层
  full_deformation_terms :2050 模板）+ `ext_block::
  scaled_extended_finalise`（ext_block.cpp:2736-2807，~70 行）+
  alcove_center（已 live）。**crate 缺**：scaled_extended_finalise。
- `twisted_KL_sum_at_s` 两重载（:8370/:8420）：`twisted_KL_column_at_s`
  （repr.cpp:~2300-2424）。
- `block_deform`（:8178-8204）：`Rep_table::block_deformation_to_height`
  （repr.cpp:2027-2124，~100 行）+ reducibility_points 下滑（crate
  已有 reducibility_points/scale）。
- is_delta_fixed 检查（两 wrapper 都有）：错误文本
  `"Parameter not fixed by inner class involution"`；test_final 文本
  `"Twisted deformation requires final parameter"`。
- 语言层仿真模板已验证：full_deformation_terms（domain_builtins.rs:
  2050-2110）按 reducibility point 逐点建块 + crate
  deformation_terms，无需真 Rep_table 池。twisted 族可同构仿真。

### 5.7 根编号族六个 builtin（wrapper 锚点 + 探针数据 + 开放问题）

wrapper 全在 atlas-types.w:1487-1560/:2604-2618，安装 :2230-2239/:2649：
- `root_index (RootDatum,vec->int)`：`find_index(d_roots, r)` 线性查找
  （rootdata.h:523-524）+ convert_to_signed_root_index（:1478-1485）。
- `coroot_index (RootDatum,vec->int)`：同构（rootdata.h:536-537）。
- `root_expression (RootDatum,int->vec)`：`root_expr(alpha)` 简单坐标
  （rootdata.h:221，internal_root_index 校验）。
- `coroot_expression (RootDatum,int->vec)`：同构（rootdata.h:222）。
- `root_involution (RootDatum,int->vec)`：
  `simple_root_permutation(rt_abs(alpha))`——任意正根号的反射排列。
- `root_permutation (WeylElt->vec)`：`permuted_root` 全体正根 +
  rootMinus 负根镜像，返回长 numRoots 的**内部 RootNbr（无符号）**向量。

探针捕获（/tmp/rootnum_probe.at、/tmp/rootidx_probe.at，oracle 4d3e9449）：
- root_expression B2 prefer_coroots=true：序 [1,0],[0,1],[1,2],[1,1]
  （余根键排序，[1,2] 先于 [1,1]）；G2：[1,0],[0,1],[3,1],[3,2],[1,1],[2,1]。
- B2 prefer_coroots=false（fixture 约定）：posroots 列 [1,0],[0,1],[1,1],
  [1,2]（根坐标 level 序）——与 prefer_coroots=true 的序不同，两边都已
  与 Rust 已验证行为一致（rootdata.cpp:164-167 对偶 Cartan 语义）。
- root_involution(rb,2) → `[ 5, 6, 7, 3, 4, 0, 1, 2 ]`（长 numRoots=8，
  内部 RootNbr）；root_involution(rb,3) → `[ 7, 4, 2, 6, 1, 5, 3, 0 ]`。

~~开放问题~~ **已解（2026-08-11 追加探针 /tmp/probe_rootidx2.at）**：
无 bug，vec 坐标基就是 datum 各自的原生格基，之前的"反直觉"全部
来自 sc/ad 两种 datum 的基不同：
- **adjoint datum**（`adjoint(Lie_type("B2"))`）：根 = 简单根坐标，
  posroots `[1,0],[0,1],[1,1],[1,2]`，root_index 依次 0/1/2/3；
  余根 = 基本余权重坐标（Cartan 的列），poscoroots
  `[2,-1],[-2,2],[2,0],[0,1]`，coroot_index([0,1])=3，
  `([1,0])=([1,1])=([1,2])=4`（未命中哨兵）。
- **simply_connected datum**（`simply_connected(Lie_type("B2"),true)`）：
  根 = 基本权重坐标（Cartan 的行，C=[[2,-2],[-1,2]]），posroots
  `[2,-2],[-1,2],[0,2],[1,0]`，root_index([1,0])=3，
  `([0,1])=([1,1])=([2,1])=4`；余根 = 简单余根坐标，poscoroots
  `[1,0],[0,1],[1,1],[2,1]`，coroot_index 依次 0/1/2/3（[2,1]→3）。
- **符号编号**：正根 0..numPos-1（= posroots 显示序 = 内部 d_roots
  序）；负根 -1..-numPos，`-k` = posroot `k-1` 的负（rootMinus
  `numRoots-1-i`）；`root(rd,-1)`/`coroot(rd,-1)` 均已验证。
- **未命中哨兵**：find_index miss → internal=numRoots →
  signed=numPosRoots（B2 上即 4），无错误。
- **越界错误**：`root(sc,4)` → Runtime error `Illegal root index 4`
  （internal_root_index 校验，atlas-types.w:1428-1439）。
- 之前探针的 root_index([1,0])=2 来自另一种 datum 构造（Cartan 约定
  不同），与本次自洽数据不矛盾。fixture 应同时覆盖 sc 与 ad 两种 datum，
  并含负根、miss、越界错误三态。

### 5.6 根编号族解封（2026-08-11 查证，原"阻塞"状态过期）

REMAINING 2026-08-04 两条"Root-index builtins limit"已过期。证据：
1. `RootNumbering`（domain_builtins.rs:2809-2880）已完整移植 oracle 的
   （level, root_compare 末坐标向前）排序（rootdata.cpp:118-129），
   prefer_coroots 时用余根坐标做键（rootdata.cpp:164-167），
   rootMinus 约定 RootNbr `npos-1-p`（rootdata.h:264-265）。
2. B2 posroots/poscoroots 探针 fixture verified_hpc（差分 3516408）：
   正根顺序 `[1,0],[0,1],[2,1],[1,1]` 与 oracle 逐字节一致。
3. walls fixture（差分 3533446）在 B2 上验证用户可见根号。
故 root_expression/coroot_expression/root_index/coroot_index/
root_permutation/root_involution 可以做了——直接用既有 RootNumbering。
**保留 caveat**：已验证的根号案例集中在简单根和 B2；非简单正根、
G2/F4/E 型的排序细节（[2,1] 先于 [1,1] 这类非纯 level 序）由首次
差分定夺，fixture 要覆盖这些型。

### 5.2 affine_orbit_ws/basic_orbit_ws（~~crate 切片~~ 已降级为语言层切片）

wrapper atlas-types.w:2014-2060（安装 :2241 附近；basic_orbit_ws
`(RootDatum,[int],int->[W_elt])`，affine_orbit_ws `(RootDatum,ratvec->[W_elt])`）。
**2026-08-11 复查**：alcove/FPP 切片（53581d8）已把几乎全部机制搬进
domain_builtins.rs helper 层——wall_set(:2915)、extend_orbit_words(:3574)、
basic_orbit_adjoint(:4070)、vertex_orbit(:4095)、reflection_word(:4167)、
list_roots_and_labels(:4258)、components(:6370，签名是 Cartan 矩阵版，
C++ rootdata::components 是 (rd,S) 根子集版，需薄适配)。
**不再需要 crate 切片**；缺的只是三个薄顶层：
`extend_affine_component`（alcoves.cpp:665-697，组合
list_roots_and_labels + extend_orbit_words + vertex_orbit）、
`finite_subquotient`(:699-711)/`complete_affine_component`(:713-722)、
`affine_orbit_ws`(:725-738，wall_set + components + 逐分量
extend_affine_component)。
basic_orbit_ws 的校验：`"Index too large for given list of root numbers"`、
锐角检查 `"Roots {a} and {b} have acute angle."`（atlas-types.w:2068-2090，
内部号转符号号输出）；to_affine_orbit 判定（:2100-2116，分量内余根
Cartan 子阵 kernel 维数>0，stab 与 final 分量求交）。
affine_orbit_ws 尺寸检查 `"Rank and rational weight size mismatch {r}:{s}"`。
W_elt 渲染已存在（Weyl_orbit 切片）。
**另**：root_ladder_bottoms/coroot_ladder_bottoms 需要 RootSystem 加
`min_roots_for`/`min_coroots_for`（rootdata.h:154-157 语义，wrapper
:1569-1597，RootNbrSet → 符号根号 [int] 输出）——这是本族唯一需要
碰 crate 结构体的部分，量很小，可与上述合并为一个语言层切片。

**fixture 已备（2026-08-11，commit 6de7f27）**：
`tests/fixtures/domain/orbit_ws.atlas(+_rejected)`，harness 条件（无
basic.at）oracle 验证通过。探针事实：
- `basic_orbit_ws(rd, v, stab_rank)` 约定：v 前 stab_rank 项是 stab 墙，
  v[stab_rank] 是 final 根——`v.size()<=stab_rank` 才报
  `"Index too large for given list of root numbers"`（`[0],1` 也报）。
- 有限分支样例：B2 `[0,1],1` → `[<>,<1>,<0.1>,<1.0.1>]`；
  `[0,2],1` → `[<>,<1.0.1>]`；G2 `[0,1],1` → 6 元。
- 仿射分支（final 余根依赖 stab 余根）：B2 `[0,1,-3],2`、`[0,1,-4],2`、
  A1 `[0,-1],1`、G2 `[0,1,-6],2` 均 → `[<>]`（stab 满秩时轨道平凡，
  属正常）；fixture 以此覆盖 to_affine_orbit=true 路径。
- 锐角拒绝：B2 根对 (1,2)/(0,3)/(0,0)/(0,-2) 全报
  `"Roots {a} and {b} have acute angle."`（符号号输出）。
- ladder 样例：B2 root_ladder_bottoms(0) → `[-4,-3,-1,0,1,2]`，
  coroot_ladder_bottoms(0) → `[-4,-1,0,1]`；G2 root_ladder_bottoms(5)
  → `[-6,-5,-4,-3,-2,-1,0,5]`。
- affine_orbit_ws：B2 `[0,0]/1` → `[<>]`、`[1,0]/2` → `[<>,<1.0.1>]`、
  `[1,1]/3` → 4 元；尺寸错报 `"Rank and rational weight size
  mismatch 2:3"`。
