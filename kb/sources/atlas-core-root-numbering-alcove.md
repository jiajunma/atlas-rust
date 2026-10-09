---
title: 根编号与 alcove 机器（domain_builtins.rs 5005–5924）——RootNumbering 的 RootNbr 序与 alcove 墙/标签/词
source: atlas-rust/atlas-core-root-numbering-alcove
ingestedAt: 2026-10-09T17:00:00Z
---

# 根编号与 alcove 机器（domain_builtins.rs 5005–5924）

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包覆盖
`crates/atlas-core/src/domain_builtins.rs` 的 `RootNumbering` 实现
（5005–5092）与 alcove 机器（5094–5924）：`wall_set`/`root_components`/
`labels_for_component`/`sorted_by_label`/`from_fundamental_alcove`/
`fundamental_alcove_wall_count`/精确 Cartan 逆。结构性阅读，不声称数学
验收。

## `RootNumbering`：RootNbr 序

- `new(root_system, prefer_coroots)`：正根按 `(level, root_compare)` 排序
  ——level 是坐标和；`root_compare`（rootdata.cpp:118-129）从**最后一个
  坐标向前**比。`prefer_coroots` 选坐标系（单余根坐标 vs 单根坐标）。
- 编号：正根占 `[npos, total)`；负根经 `rootMinus`（rootdata.h:264-265）：
  正根 p 的负 = RootNbr `npos-1-p`。`npos` 是正根数。
- `signed(nbr) = nbr - npos`（`convert_to_signed_root_index`，
  atlas-types.w:1478-1485）；`is_negative(nbr) = nbr < npos`。
- 坐标→正根位的索引表（BTreeMap 按坐标）；每个负根必有正对应对。

## alcove 机器

- `wall_set`（weyl::wall_set，alcoves.cpp:112-138）：由 `gamma` 的小
  dominant 位移所达 alcove 的墙余根；`integrals` 收在 gamma 上取整的墙
  （上游 `on_wall_coroots`）；过滤器只保留**余根不能被减去**的根——
  上游 `min_coroots_for` 成员（rootdata.h:154-157：α∨−β∨ 不是余根）。
- `root_components`（rootdata::components，rootdata.cpp:1514-1537）：根
  子集在非正交下的连通分量，每个分量内 RootNbr 升序。**原版在新根触及
  分量时就追加它，故最终分量顺序按最大 RootNbr，而非最小**——这个顺序
  在 FPP 乘积向量及其 Weyl 见证中可观察（FPP 乘积顺序课）。
- `labels_for_component`（alcoves.cpp:141-154）：一个墙分量的余根间
  **唯一原始整数关系**，取正；环境余根坐标与上游单余根坐标携带同样的
  线性关系，故核由环境表计算。`gcd`/`lcm` 辅助。
- `sorted_by_label`（weyl::sorted_by_label，alcoves.cpp:157-184）：全部墙
  按分量标签**降序**，并列按 RootNbr 序。
- `from_fundamental_alcove`（alcoves.cpp:186-236）：其 alcove 有给定墙集
  的 Weyl 词——每个分量留出一个单位标签墙，其余经 `to_positive_system`
  （rootdata.cpp:1329-1347）移到单根系，逆序步骤经最终单值索引得词。
- `fundamental_alcove_wall_count`：基本 alcove = 单余根 + 每个不可约分量
  一条最低余根（RootSystem::fundamental_alcove_walls，
  rootdata.cpp:474-481），大小 = 秩 + 分量数；**只有大小可观察**
  （"Too few walls" 检查）。
- 精确 Cartan 逆为 (分子, 分母)（分数自由消元）。

## 边界与限制

- `strong_components`、CenterClassifier、ByLastCoordinate、RootTable 各待
  分包；Weyl 子群轨道用本编号见 [子群包](atlas-core-weyl-subgroup.md)。
- 上游行号引用是**实现方移植陈述**；编号/alcove 兼容以 HPC 差分门为准。
- 字节数/哈希只标识本快照字节（git base `964f0033`）。
