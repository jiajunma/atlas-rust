---
title: 块图 SCC 与根表（domain_builtins.rs 3022–5005 + 7992–9606）——strong_components、ByLastCoordinate 序、对合校验与 RootTable
source: atlas-rust/atlas-core-domain-scc-root-table
ingestedAt: 2026-10-09T18:00:00Z
---

# 块图 SCC 与根表（domain_builtins.rs 3022–5005 + 7992–9606）

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包覆盖
`crates/atlas-core/src/domain_builtins.rs` 的 `strong_components`
（3022–5005 区域头部）与 7992–9606 区域：`ByLastCoordinate` 序、
矩阵辅助（`dot_row`/`dot_column`/`transpose`/`integer_matrix_product`/
`is_identity_square`）、对合构造器的包装侧公共校验、`RootTable::build`。
结构性阅读，不声称数学验收。

## `strong_components`（3022+）：块图的迭代 SCC

迭代式 Tarjan 形强连通分量：`rank`/`class_of`/`partition`/`induced`；
`active` 条目为 (顶点, 父位置, 下一边索引, 最小秩)；`nil=size`、
`infinity=size+1`。用于块图（BlockGraph 的 SCC 消费者）。

## 7992–9382：序、矩阵辅助与对合校验

- `ByLastCoordinate`：**坐标逆序**的字典序（`iter().rev()` 比较）——
  与 `root_compare`（rootdata.cpp:118-129）的"末坐标向前"一致。
- `dot_row`/`dot_column`/`transpose`；`integer_matrix_product`：**宽
  累积**（i128）——用户矩阵无界，其乘积只喂等值测试（下方的扭曲兼容
  检查）。
- `is_identity_square`。
- 对合构造器的包装侧公共校验：`mat` 保留 `Vec<Vec<_>>` 无法恢复的维数
  （特别是 `0xN`），故先检查值再适配成行；Atlas 把**矩阵行数**选为独立
  情形的期望秩。

## `RootTable::build`（9382+）

`prefer_coroots` 时先转置 Cartan 生成正（余）根再换回；`components` 分
分量；`express` 把单坐标向量表到**环境格基**；`length_flags` 分别给
根/余根的长标志。

## 边界与限制

- 这两个区域的大块内容（SCC 细节、RootTable 的消费者）只在头部精读；
  中心分类与轨道词见 [中心分类器包](atlas-core-center-classifier.md)。
- 上游行号引用是**实现方移植陈述**；兼容以 HPC 差分门为准。
- 字节数/哈希只标识本快照字节（git base `964f0033`）。
