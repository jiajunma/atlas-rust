勘察完毕。以下是结构化侦察报告。

# ext_block 层实现侦察报告（只读勘察，未修改任何文件）

## 1. 上游内建定义位置与签名

所有 wrapper 均在 `sources/interpreter/atlas-types.w`（HEAD 4d3e9449，总行数 9151）。

| 内建 | wrapper 位置 | 安装签名 | 数学层入口 |
|---|---|---|---|
| `default_extended` | atlas-types.w:7313-7334 | `(Param,mat->vec,vec,vec,vec)`（:7528） | `ext_block::default_extend` ext_block.cpp:2352；组件在 `set_default_extended` ext_block.cpp:222-233 |
| `extend` | （LieType 族，atlas-types.w:425 安装 `extend_Lie_type_wrapper`） | — | Rust 已完成的是 LieType 的 extend（domain_builtins.rs:6155），与 ext_block 无关 |
| `extended_block` | atlas-types.w:7366-7386 + 构造块 :7391-7431 | `(Param,mat->[Param],mat,mat,mat)`（:7531） | `blocks::common_block` 全块 + `common_block::extended_block(delta)` blocks.cpp:1305-1310 → `ext_block::ext_block` ext_block.cpp:618-668 |
| `finalize_extended` | atlas-types.w:8514-8537 | `(Param,mat->ParamPol)`（:8595） | `ext_block::extended_finalise` ext_block.cpp:2598-2721 |
| `partial_extended_KL_block` | atlas-types.w:7445-7468 | `(Param,mat->[Param],mat,[vec])`（:7533） | `ext_kl::ext_KL_matrix` ext_kl.cpp:939-1020 |
| `dual_KL_block` | atlas-types.w:7053-7090（survivors :7095-7106，枚举 :7117-7136） | `(Param->[Param],int,mat,[vec])`（:7517） | `lookup_full_block` repr.cpp:1773-1794 + `Bare_block::dual` blocks.h:238 + `kl::KL_table`；**不涉及 ext_block** |
| `K_type_pol_extended` | atlas-types.w:8487-8500 | `(Param,mat->KTypePol)`（:8593） | `ext_block::extended_restrict_to_K` ext_block.cpp:2435-2547 |
| `scale_extended` | atlas-types.w:8449-8472 | `(Param,mat,rat->Param,bool)`（:8591） | `ext_block::scaled_extended_finalise` ext_block.cpp:2736-2807 |
| `raw_ext_KL` | atlas-types.w:8682-8728 | `(Param,mat->mat,[vec],vec)`（:9103） | common_block 全块 + `ext_block::ext_block` + `ext_kl::KL_table::fill_columns` ext_kl.cpp:429 |
| `shift_flip` | atlas-types.w:7341-7359 | `(Param,mat,ratvec->bool)`（:7530） | `ext_block::shifted_default_extension` ext_block.h:353-361 + `is_default` ext_block.h:363 + `same_sign` ext_block.cpp:936；核心是 `ext_param` + `context::shift_flip` ext_block.cpp:324-336 |
| `twisted_deform` | atlas-types.w:8120-8150 | `(Param->ParamPol)`（:8573） | `Rep_table::twisted_deformation_terms` repr.cpp:2426-2520（依赖 `block.extended_block(bm, poly_hash)` + `contributions(ext_block,...)` repr.cpp:1901-1931） |
| `twisted_full_deform` | atlas-types.w:8229-8251（timed 版 :8293-8330） | `(Param->KTypePol)` / `(Param,int->\|KTypePol)`（:8577,:8581） | `ext_block::extended_finalise` + `Rep_table::twisted_deformation` repr.cpp:2552-2653 |
| `twisted_KL_sum_at_s` | atlas-types.w:8370-8382（external 版 :8420-8431） | `(Param->ParamPol)` / `(Param,mat->ParamPol)`（:8586,:8589） | `Rep_table::twisted_KL_column_at_s` repr.cpp:2371-2423 / 自由函数 repr.cpp:2354-2367 + `twisted_KL_sum` repr.cpp:2304-2350 |

另外 `block_deform`（batch 7 同族）：wrapper atlas-types.w:8178-8204，签名 `(Param,ParamPol,int->ParamPol,ParamPol)`（:8574），核心是 `Rep_table::block_deformation_to_height` repr.cpp:2027-2124。

## 2. 上游数学层模块盘点（要移植的代码量）

- `gkmod/ext_block.h`（391 行）+ `ext_block.cpp`（2814 行）：
  - `DescValue` 34 值枚举 + 12 个判定谓词（ext_block.h:38-93；实现 ext_block.cpp:60-210，其中 `link_count` :165-206）
  - `extended_type`（纯 Block_base 上的类型识别，无线性代数）ext_block.cpp:343-503
  - `ext_block::ext_block(common_block, bm, delta, pol_hash)` 构造 ext_block.cpp:618-668（不动点计算 `transformed_twisted` :597-616，`complete_construction` :696-856，`induced` 置换 :670-693）
  - `tune_signs` ext_block.cpp:1707-1876（符号调优，约 170 行）
  - `star` ext_block.cpp:990-1706（**全模块最大单函数，约 717 行**：按 orbit 长 1/2/3 对扩展参数做 Cayley/cross 分类；依赖 `fixed_conjugate_simple` :525-552、`context::to_simple_shift` :307、`context::shift_flip` :324、`z_align`、`same_sign` :936）
  - `complex_cross` ext_block.cpp:858-935
  - `ext_param` 结构及其构造（ext_block.h:293-343；实现 :2297-2410，含 `def_ext` :2330、`restrict`/`restrict_K` :2396-2410）
  - `context`（扩展块上下文，存 integral datum/SubSystem/delta 置换/lambda_shifts/l_shifts）ext_block.h:241-289；构造 ext_block.cpp:267-296
  - 三个 finalise 族函数（:2435/:2598/:2736，合计约 370 行，结构高度相似，都内嵌 `star` 调用和队列展开）
- `gkmod/ext_kl.h`（203 行）+ `ext_kl.cpp`（1021 行）：`descent_table`（:20-118）、`KL_table`（构造 :120，`product_comp` :204-247，`get_Mp` :364-407，`has_direct_recursion` :408，`fill_columns` :429-456，`extract_M` :457-607，`do_new_recursion` :608-841），`ext_KL_matrix` :939-1020。多项式池 `ext_KL_hash_Table = HashTable<IntPolEntry, ext_kl::KLIndex>` 定义在 `sources/Atlas.h:478`，`KLIndex` 带符号位（raw_ext_KL 的 `inx.second ? -inx.first : inx.first`，atlas-types.w:8713-8714）。
- `gkmod/repr.cpp` twisted 段：`contributions(ext_block 版)` :1901-1931、`twisted_KL_sum` :2304-2350、`twisted_KL_column_at_s` 两版 :2354-2423、`twisted_deformation_terms` :2426-2520、`twisted_deformation` :2552-2653；非 twisted 的 `block_deformation_to_height` :2027-2124（对偶块 + 上三角逆矩阵 + Split_integer 系数）。
- `gkmod/blocks.cpp`：`common_block::extended_block` 两版 :1305-1358（缓存按 `(w, shift)` 排序的 `ext_block_data` 数组 :1312-1325）；`common_block::swallow` 中 ext 块迁移目前被 `#if 0` 关闭（:1442-1469），**不用移植**。
- `gkmod/kl.cpp`（1061 行）普通 KL 表已在 Rust 侧完成（kl_table.rs 992 行）。

## 3. Rust 侧现状与缺口

已有（crates/atlas-real-group/src）：

- `rep_context.rs`（2776 行）：`StandardRepr`（:111）、`RepContext`（:559），含 `finals_for`(:1261)、`reducibility_points`(:1159)、`deformation_terms`(:1459)、`scale`(:1699)、`made_dominant`(:2646)、`deform_readjust`(:2692)、`sr_gamma`(:1413)、`torus_part`(:1074)、`real_unique`(:1102)、`gamma_lambda`(:1140)、`sr_k_of_standard`(:1552) 等。
- `block.rs`（913 行）：`BlockGraph`（:115）+ `BlockDescent`（:35），cross/Cayley/inverse_cayley/length/Bruhat Hasse 齐全。
- `kl_table.rs`/`kl_support.rs`/`kl_polynomial.rs`：普通 KL 表、`KlHashTable`、`KlPol`（含 `evaluate_at_minus_one` :107、`quotient_by_1_plus_q` :148）。
- `kgb_graph.rs:775`：`KgbGraph::twisted(id, table, delta, twist)`（上游 `KGB::twisted`，ext_block 不动点计算的关键件）**已存在**。
- `domain_builtins.rs`：`default_extended`（:9277-9482，内含 test_compatible、gamma 固定检查、`find_solution` 解 `(1-θ)τ=(δ-1)λ_ρ` 与 `(θᵀ+1)t=(δ-1)l`）、`deform`（:9995，每 final 重建 rf×对偶 quasisplit 块）、`KL_block`（:8524，已实现但被 REMAINING 标记为 revert-to-skip：common-block/PolP 缺口）、`partial_block`(:8684)、`partial_KL_block`(:8833)、`full_deform`(:8802)。
- `find_solution`（matreduc 移植）在 domain_builtins.rs:2504 附近已有。
- `twisted_involution.rs`、`involution_table.rs`（含 `twisted_involution()` :60）、`global_tits.rs`（`GlobalTitsElement`，pub(crate)）。

**缺口（ext_block 层核心，全部不存在）**：

1. `ext_block::ext_block` 结构本身：`DescValue` 34 值枚举 + 谓词、`extended_type` 分类、不动点 Bitmap（`transformed_twisted` 需 srm 的 shift/transform，即 block_modifier 语义）、`complete_construction`、`tune_signs`、fold 后 DynkinDiagram（`folded(orbits)`）、`epsilon` 符号查询。
2. `ext_param` + `ext_block::context`（integral datum/SubSystem 已在 Rust 有 root_system/subsystem？——未确认 SubSystem 等价物，`is_very_complex`/`to_simple_shift`/`shift_flip` 也没有）；`star` 的 717 行是最大单块工作。
3. `ext_kl::descent_table` + `ext_kl::KL_table`（约 1000 行，结构镜像已有 kl_table.rs，但递归式不同：`get_Mp`/`do_new_recursion`/`extract_M`，且 `KLIndex` 带符号）。
4. Rep_table 层：`lookup_full_block`/`block_modifier`（`w`、`shift`、`simple_pi`、`simp_int`）/`alcove_hash`/`deformation_unit`/多项式共享池——Rust 目前**没有 srm pool**，`deform`/`KL_block` 都是现造 `BlockGraph`（rf × 对偶 quasisplit）再用 `common_block_members`（domain_builtins.rs:2427-2502，按 cross/Cayley/parity 闭包过滤）近似 common block。
5. `is_fixed`/`is_delta_fixed`（wrapper 门槛，repr.cpp，Rust 完全没有，但实现很小）。
6. `Bare_block::dual`（blocks.cpp）+ `dual_map`——Rust 有 `dual.rs`（dual_datum/dual inner class），但 `dual_KL_block` 需要的是**同一 common block 的元素级对偶**（`last-x` 编号翻转，见 atlas-types.w:7125-7132），与普通 `raw_dual_KL` 的 Block 重建路径不同；这一点上游其实很简单（编号翻转 + 填表），可独立做。

## 4. 依赖关系

- **可独立做（不依赖 ext_block 核心）**：
  - `dual_KL_block`：只要 `lookup_full_block` 等价物 + 普通 KL 表 + survivors 过滤（atlas-types.w:7053-7136）。但注意它走 `lookup_full_block`，而 Rust 的 `KL_block` 正是卡在 common-block/srm pool 语义上（REMAINING 里 "KL_block (common-block/PolP gaps) revert"）——所以严格说它依赖"common block 语义修正"，不依赖 ext_block。
  - `shift_flip`：只需 `ext_param` 的 default_extend 组件（Rust 已有其公式）+ `same_sign`（ext_block.cpp:936-988，约 50 行，比较两 ext_param 的 τ/t 分量）——**不需要** ext_block 构造、不需要 `star`。是最便宜的 ext 族入口。
- **必须有 ext_block 核心结构（构造 + DescValue 分类）**：`extended_block`、`raw_ext_KL`、`partial_extended_KL_block`（还要 ext_kl::KL_table + condense ext_block.cpp:2809 模板/:2015-2048）。
- **必须有 ext_param + `star`**：`finalize_extended`、`scale_extended`、`K_type_pol_extended`、`twisted_full_deform`（经 `extended_finalise` + `scaled_extended_finalise`）、`twisted_deform`（经 `twisted_deformation`，内部调 `scaled_extended_finalise`）。
- **`twisted_KL_sum_at_s`**：两个重载里 `(Param,mat)` 外部 delta 版（repr.cpp:2354-2367）只需 common_block 全块 + `extended_block(delta)` + `twisted_KL_sum`（:2304-2350，~50 行）+ ext_kl 表填到 y+1；`(Param)` 内 delta 版（:2371-2423）走 Rep_table 缓存路径，可先做外部版的等价逻辑再复用。
- **common-block srm pool 的组织**（上游）：`Rep_table`（repr.h:534-652）持 `HashTable<Reduced_param,...> reduced_hash` + `place` 池；`lookup_full_block`（repr.cpp:1773-1794）把 sr 做成 dominant → `StandardReprMod::mod_reduce` → `Reduced_param::reduce` 得 `block_modifier bm`（含 WeylElt `w`、RatWeight `shift`、`simple_pi`、`simp_int`，repr.h:493-520）→ 未命中或不满则 `add_block`（可 swallow 旧 partial 块）。`block_deformation_to_height`（:2027-2124）在此之上：对偶 Bare_block → KL 表（`plug_hole` 剔除超高度列）→ survivors（`block.survives(elt,singular)`）→ 上三角 Q_mat（q=-1 赋值）→ `inverse_upper_triangular` → 按奇偶长度差和 orientation_number 差累计 Split_integer 系数（`times_1_s`/`times_s`）。`KL_block` 的 survivors condensation 在 atlas-types.w:6912-6948（`finals_for(x,singular)` + 奇偶长度差符号的 `Pol` 加减），Rust 已在 KL_block arm 中按同样公式实现，差在块来源。
- Rust 的可行捷径：上游 `common_block(ctxt,srm,start)` 直接构造（blocks.cpp，ext_KL_matrix :950-952 与 raw_ext_KL atlas-types.w:8693-8696 都用这条**不走 Rep_table**），即用 `StandardReprMod::mod_reduce` + integral 子系统直接建块、bm 取平凡（`block_modifier bm(B)`）。`extended_block`、`raw_ext_KL`、`partial_extended_KL_block`、`twisted_KL_sum_at_s(Param,mat)` 四件全部可以走这条捷径，**绕过 srm pool**；只有 `twisted_deform`/`twisted_full_deform`/`block_deform`/`KL_block`/`dual_KL_block`/`print_common_block` 真正需要 `lookup`/`lookup_full_block` 的 bm 语义。

## 5. 复杂度评估与建议切片顺序（从小到大）

1. **`shift_flip`（最小）**：复用 default_extended 的组件 + 移植 `same_sign`（~50 行）+ `is_default`。无新结构。估 100-200 行 Rust。
2. **`extended_block`（结构切片）**：`DescValue` 枚举+谓词（~150 行）+ `extended_type`（~160 行）+ 不动点计算（`KgbGraph::twisted` 已有，但 `transformed_twisted` 需要 srm shift/transform 语义；走 bm=平凡的捷径时只需 `kgb.twisted(x,delta)` + `StandardReprMod::build` + `block.lookup`）+ `complete_construction`（~160 行）+ `tune_signs`（~170 行，**最微妙**，符号二次/辫关系检查 check_quadratic/check_braid :2140-2245 可作 debug 断言先跳过）+ folded Dynkin（dynkin.rs 已有 DynkinDiagram，需加 `folded(orbits)`）。估 600-900 行。产出即解锁 `extended_block` builtin（wrapper 输出 4 元组，atlas-types.w:7391-7431 纯格式转换）。
3. **`ext_kl::KL_table`（第二大块）**：镜像 kl_table.rs 但递归不同，~800-1000 行移植；descent_table（~100 行）。解锁 `raw_ext_KL`（矩阵+池+length_stops 导出，wrapper 很薄）。
4. **`partial_extended_KL_block`**：`ext_KL_matrix`（:939-1020，~80 行）+ `condense`（~35 行模板）+ survivors 重编号/符号翻转。依赖 2+3。
5. **`ext_param + context + star`（最大单块）**：context（~60 行）+ ext_param（~150 行）+ `fixed_conjugate_simple`（~30 行）+ `star`（717 行，三种 orbit 长度全 case）+ `z_align`。估 1000-1200 行。
6. **finalise 三函数 + `finalize_extended`/`scale_extended`/`K_type_pol_extended`**：三个函数结构几乎相同（队列 + restart 循环），合 ~370 行上游，但全依赖 5。三个 wrapper 都很薄。
7. **`dual_KL_block`**：独立小块（~150 行），但建议与 KL_block 的 common-block 修正同批做。
8. **`twisted_KL_sum_at_s`**：`twisted_KL_sum` ~50 行 + contributions(ext_block 版)（repr.cpp:1901-1931，~30 行）→ 依赖 2+3。
9. **`twisted_deform`/`twisted_full_deform`**：`twisted_deformation_terms`（~95 行，镜像已有 deformation_terms）+ `twisted_deformation`（~100 行递归 + alcove 缓存；Rust 侧 full_deform 没有 Rep_table 缓存，可同样做成无缓存递归，但要接受性能差异）→ 依赖 2+3+5+6。
10. **`block_deform`**：`block_deformation_to_height`（~100 行）+ `Bare_block::dual` + 上三角逆 + `plug_hole`（kl::KL_table 的洞剔除，Rust kl_table.rs 无此法）→ 依赖 srm pool 语义，建议最后。

上游移植总量估计：ext_block.cpp 有效约 2200 行（去掉 #if 0 和 debug）+ ext_kl.cpp 约 900 行 + repr.cpp twisted 段约 350 行 ≈ 3400 行 C++；对应 Rust 新代码估计 3000-4500 行，建议新 crate 模块 `ext_block.rs` / `ext_kl.rs`（放 atlas-real-group），语言层继续走 domain_builtins.rs 注册。

## 6. print 族锚点与难度

| 内建 | wrapper | 核心依赖 | Rust 缺口 | 难度 |
|---|---|---|---|---|
| `print_X` | atlas-types.w:8999-9008，签名 `(InnerClass->)`（:9124） | `kgb::global_KGB`（kgb.h:236-266，构造 kgb.cpp:190-288：遍历所有 Cartan orbit + 所有 square class 的 fundamental fiber）+ `kgb_io::print_X`（kgb_io.cpp:150-159，输出 `\exp(i\pi\check\rho)` 偏移 + 元素列表） | Rust 无 `GlobalTitsGroup`（structure/tits.h:125+）、无 global KGB；有 `GlobalTitsElement`（global_tits.rs，pub(crate)）。体量：GlobalTitsGroup + global_KGB 生成 + 打印，**中等偏大**（预估 600+ 行），但不含 KL/ext，独立性强 |
| `print_gradings` | atlas-types.w:4260-4291（打印循环 :4304-4330），签名 `(CartanClass,RealForm->)`（:9108） | fiber `weakReal()` 划分 + `simpleImaginary()` + Dynkin Bourbaki 置换（atlas-types.w:4296-4299）+ **oracle 根编号**（输出 simple root 编号列表 si[sigma[i]]，:4310-4313） | Rust 有 cartan_fiber.rs/grading.rs/dynkin.rs；缺 oracle 根编号映射（已知遗留问题，REMAINING 中 root_expression/root_index 同因）。**如果不追求 simple-root 编号行的字节一致则小；要全字节一致则被根编号阻塞** |
| `print_real_Weyl` | atlas-types.w:8831-8847，签名 `(RealForm,CartanClass->)`（:9110） | `output::printRealWeyl`（output.cpp:445-466）→ `realweyl::RealWeyl`（io/realweyl.h:36-181，构造需 CartanClass + 两个 adjoint fiber 元 + RootDatum + WeylGroup）+ `RealWeylGenerators`（:183-242）+ `realweyl_io::printRealWeyl`（realweyl_io.cpp，200 行；realweyl.cpp 283 行） | Rust 无 RealWeyl。**中等**（~500 行移植），依赖 Cartan 纤维件（已有 cartan_fiber.rs/adjoint_fiber.rs），不含块/KL，独立性较好 |
| `print_blockstabilizer` | atlas-types.w:8920-8929，签名 `(Block,CartanClass->)`（:9117） | `output::printBlockStabilizer`（output.cpp:361-379）→ 同 RealWeyl + `realweyl_io::printBlockStabilizer` | 与 print_real_Weyl 共享 RealWeyl 移植，做了 print_real_Weyl 后**增量很小**；但需要 Block 值持有 rf/dual_rf（Rust 的 Block 值已有） |
| `print_common_block` | atlas-types.w:6668-6695，签名 `(Param->)`（:7505） | `lookup_full_block`（srm pool + block_modifier 的 w/shift/simple_pi 打印头部）+ `common_block::shift`（blocks.cpp:1328-1342）+ `common_block::print_to`（block_io.cpp:48-110，involution 表达式 + singular 标记） | Rust 有普通 `print_block`（typed.rs:4685）；缺 srm pool/bm 语义与 common-block 打印格式。**难度最高的 print**，建议与 KL_block/common-block 修正同批 |

**print 族建议顺序**：`print_gradings`（若接受根编号问题先冻结 A2 类小例）→ `print_real_Weyl` + `print_blockstabilizer`（一批，共享 RealWeyl）→ `print_X`（独立大块，GlobalTitsGroup + global KGB）→ `print_common_block`（最后，与 srm pool 批次绑定）。

## 关键风险记录

- `tune_signs`（ext_block.cpp:1707-1876）是 ext_block 构造里最容易出 silent 错误的部分；上游有 `check_quadratic`/`check_braid`（:2140-2245）可作验证闸门，建议移植为 debug 断言。
- `star` 内 `z_align`、`same_sign` 的符号语义（October surprise 的额外 flip，ext_block.cpp:136-139、2490/2503/2519-2522 注释）必须逐行对照，wrapper 层的 `E.flip(has_october_surprise(type))` "undo extra flip" 模式在三个 finalise 函数中重复出现。
- Rust 侧 `KL_block`/`dual_KL_block` 的共同根因是 common-block 语义（lookup_full_block + bm），先修这个比单补 builtin 更值。
- `ext_KL_hash_Table` 在 `Atlas.h:478`；`KLIndex` 符号位约定（raw_ext_KL atlas-types.w:8713-8714、twisted_deformation_terms repr.cpp:2486-2488）与 Rust `KlHashTable`（kl_polynomial.rs:191）不同，移植时需扩展。

（以上行号均对上游 HEAD 4d3e9449 与本地工作树 c5afc4f 核验。）