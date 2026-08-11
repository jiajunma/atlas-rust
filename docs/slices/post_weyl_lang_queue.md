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

## 3. ext_block builtin 注册（agent-33 ext_kl 验收后）

`extended_block (Param,mat->[Param],mat,mat,mat)`（wrapper
atlas-types.w:7366-7431，纯格式转换）+ agent-33 交付后的
`raw_ext_KL`（:8682-8728）/`partial_extended_KL_block`（:7445-7468）。
crate 侧 ext_block.rs（28e6109）+ ext_kl.rs（agent-33 在途）已备。
注意 oracle 的 delta 约定：打印列作为像，传参行作为像（要转置），
且必须是 distinguished（SL(3,R) 的 distinguished 打印 [[1,0],[1,-1]]，
传参 [[1,1],[0,-1]]）。

## 4. print 族（顺序见 agent31 侦察报告 §6）

print_gradings → print_real_Weyl + print_blockstabilizer → print_X →
print_common_block（最后，绑 srm pool）。print_strong_real 已存在
（typed.rs:5275），勿重复。

## 5. 其余（REMAINING_BUILTINS.md）

root_expression/root_index/root_permutation（oracle 根编号阻塞）、
root_ladder_bottoms/coroot_ladder_bottoms、shift_flip（最便宜的 ext 族，
只需 same_sign ~50 行）、twisted_KL_sum_at_s、twisted_deform 族、
block_deform/KL_block/dual_KL_block（common-block srm pool 语义）。
