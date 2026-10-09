---
title: 中心分类器与轨道词（domain_builtins.rs 6102–7992）——CenterClassifier 的中心陪集 tabulation 与 adjoint 轨道 BFS/词转换
source: atlas-rust/atlas-core-center-classifier
ingestedAt: 2026-10-09T17:40:00Z
---

# 中心分类器与轨道词（domain_builtins.rs 6102–7992）

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包覆盖
`crates/atlas-core/src/domain_builtins.rs` 的 `CenterClassifier`
（6090–6193）与 adjoint 轨道机器（6195–7992）：`AdjOrbitElem`/
`adjoint_orbit_bfs`/`basic_orbit_adjoint`/`vertex_orbit`/
`convert_to_words`/`simple_reflect_root_nbr`/`reflection_word`/
`word_act_root`/`word_act_weight`。结构性阅读，不声称数学验收。

## `CenterClassifier`（alcoves.cpp:793-870 的 `center_classifier`）

tabulate 基本权的子集和中**落在根格**者，按根格陪集（adjoint 坐标的
分数部）分组：

- 构造：对 Cartan 取 `(adjugate, det)`；2^rank 个子集各自求和，分数部
  `rem_euclid(det)` 分桶、整数部 `div_euclid(det)` 记入 `shift_of`；
  `table` = 排序后的 (分数部, 子集列表)；`class_of` = 每子集的陪集号。
  全部算术在逆 Cartan 的 adjugate/行列式表示上，**与上游 `C_denom`
  完全一致**。
- `shifts(fix, pos, neg)`：根格移位 `fw(fix+A)−fw(B)`（A⊆pos、B⊆neg）
  的 adjoint（单根）坐标——用陪集表 + 分数部借位（fix_entry ≤ entry
  则直减，否则 rts+1 且 entry -= fix_entry - det），只在
  `pos & subset == subset` 时输出。

## adjoint 轨道机器（6195+）

- `AdjOrbitElem`：`orbit_elem`（alcoves.cpp:437-445）的 adjoint 坐标
  轨道元素。
- `adjoint_orbit_bfs`：`basic_orbit`/`vertex_orbit` 的共享 BFS 核——
  新元素插在 `finish` 之后保持尾部**递减**，每完成一层反转为递增
  （alcoves.cpp:480-565）。
- `basic_orbit_adjoint`：`basic_orbit`（alcoves.cpp:526-565）——
  `cartan` 前 `i+1` 个生成元的 Levi 子商轨道（adjoint 坐标）。
- `vertex_orbit`（alcoves.cpp:458-518）：模变体，用于沿 label>1 的最终
  扩展。
- `convert_to_words`（alcoves.cpp:746-774）：经陪集树展开恒等——每步的
  反射词**左乘**到父段条目上（与 [子群包](atlas-core-weyl-subgroup.md)
  的见证序配对：词权作用约定一致）。

## 根号反射与词作用约定

- `simple_reflect_root_nbr`：单根 s 的反射作用在 RootNbr 上。
- `reflection_word`（RootSystem::reflection_word，rootdata.cpp:601-618）：
  沿**首个下降**降到单根，再逆序回溯得共轭词。
- `word_act_root`（RootSystem::permuted_root，rootdata.h:313-318）：Weyl
  词作用于根时**最后一个字母先作用**；`word_act_weight`
  （weyl.cpp:1071-1082）同约定。

## 边界与限制

- `strong_components`、ByLastCoordinate/RootTable、validate/print 各有包；
  根编号与 alcove 墙/标签见 [根编号包](atlas-core-root-numbering-alcove.md)。
- 上游行号引用是**实现方移植陈述**；兼容以 HPC 差分门为准。
- 字节数/哈希只标识本快照字节（git base `964f0033`）。
