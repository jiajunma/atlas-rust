---
title: 数学测试与基准库结构（tests/math、tests/fixtures、tests/reference）
source: atlas-rust/tests-math-library
ingestedAt: 2026-10-10T10:30:00Z
---

# 数学测试与基准库结构

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包记录
`tests/math/`、`tests/fixtures/` 与 `tests/reference/` 的目录结构、
JSON schema 与证据纪律。对应
[阅读快照](snapshots/2026-10-10-tests-math-library.json)。注意取证时刻的
工作区状态：`tests/math/catalog.json`、`tests/math/generics/catalog.json`
与 `tests/math/README.md` 有 owner 未提交修改，`tests/math/rank6/catalog.json`
尚未被跟踪——本包记录的是这些**脏字节**的结构，不代表已提交内容；
owner 的 README 是逐弧线的运行日志，其自述规则是"最新的
HANDOFF/回执优先于旧记录"。结构性阅读，不声称任何数学验收；
库规模是清单（inventory），不是正确率。

## 顶层三个 JSON

- `catalog.json`（schema `atlas-math-catalog-v1`，20269 字节）：
  8 个群（classical：A2/B2/C2/D4 为 small；exceptional：G2 small、
  F4 medium、E6/E7 large）× 8 个操作（`root_data`、`kgb`、`klv`、
  `unitarity`、`hodge`、`fpp`、`annihilator_variety`、
  `cycle_foundation`），外加 1 个拒绝模板（`fpp_rank_rejected`，
  秩与有理权尺寸不匹配诊断）、3 个语言操作（`generic_pair`、
  `generic_identity`、`basic_load`——不依赖 basic.at 的最小泛型
  构造）与 13 个 `additional_grids`（grid 级 groups×operations 覆盖
  扩展）。
- `benchmarks.json`（`atlas-math-benchmarks-v1`）：3 个 KGB 计时用例
  （E7、D6、D8），每引擎 300 秒超时、4 轮；scope 条款：新鲜进程、
  交替顺序、同节点；每轮完整输出必须一致，首次不一致即停；按引擎做
  分钟窗标定，不得把短用例垫成分钟级负载。
- `baseline.json`：双仓库钉——oracle 为 `7e1b958c`（当前 pin），Rust
  侧记录 `05625c5d`（main 分支的历史基线字节，是基准建立时的存档点，
  不是当前 HEAD）。

## templates/（31 个模板）

按操作的参数化 `.atlas` 模板：`root_data`、`kgb`、`klv`、`unitarity`、
`hodge`、`fpp`、`annihilator_variety`、`cycle_foundation`，加上
`real_form_*` 逐实形变体（metadata/kgb/klv/unitarity/hodge/fpp/
annihilator/cycle）、`partial_kl_*` 历史/包含/拒绝/回归四件、
`hodge_nontrivial`/`hodge_trace`、`fpp_d4_wall`、
`cartan_subsystems`、`cycle_product_ranks`、`real_forms_inventory`、
`basic_load`、`generic_pair`/`generic_identity` 与拒绝模板
`fpp_rank_rejected`、`partial_kl_nonstandard_rejected`。

## generics/（语言探针库）

`catalog.json`（schema `atlas-generic-language-probes-v1`，91816 字节）
登记 273 个用例；目录现有 363 个 `.atlas` 源文件与 52 个 `.oracle.*`
逐字节金标。快照时该目录另有 owner 未提交修改与若干未跟踪新文件。
金标纪律见 session 包：fixture 经 `include_str!` 内嵌，与 `.oracle.*`
逐字节比对——金标只能来自原版完整流，绝不由 Rust 输出充当。

## progressive/（渐进秩门，22 项）

rank-1 渐进验证 fixture：`unitarity_forms_rank1.json` 与
`unitarity_rank1_parameters`（+expect）、`hodge_rank1_{control,padding,
specialization,trace}`、`av_ann_rank1_finite_cycle`（+expect）、
`load_av_ann_rank1`、`load_unitarity_rank1`、`loading_rank1.expect.json`、
`cartan_rank1/rank2` 与 `cartan_classes_rank1`（+拒绝件）、
`full_deform_rank1_{control,split}`（+原版 stdout 金标）、
`type_equivalence_language.json`。`.expect.json` 与 `.oracle.stdout`
是两种期望载体：前者为结构化断言，后者为原版逐字节流。

## rank6/（秩 6 清单）

`catalog.json`（schema `atlas-rank6-inventory-v1`，快照时未跟踪）：
`rank_bound`、`simple_types`、`unequal_inner_types`、
`intermediate_quotients`、`numberings` 五键；配套
`blocks/parameters/forms/klv_scale` 四个目录清单与
`inventory/kgb/klv_block/parameters/cartan_permutations/f4_class_trace`
等 fixture，外加 `cartan-regressions.patch`。秩 6 的复杂群以实根数据
秩 12 计（AGENTS.md 的既定约定）。

## tests/fixtures/ 与 tests/reference/

`tests/fixtures/` 现有 353 个 `.atlas` 与 1 个 `.at`——语言语料库；
可执行三元组（fixture/event/meta）的精确计数与路径敏感清单以
HANDOFF 的冻结迁移清单为准。`tests/reference/` 存 oracle 元数据与
HPC 证据；`tests/reference/hpc/math_acceptance_index_2026_10_01.json`
是 append-only 验收账本种子：每条验收追加哈希链记录，绑定确切源码、
报告、独立审查、选用例 ID、断言与限制；每个已发布后缀须有永久前缀
检查点，否则只是草稿。

## 证据纪律（来自 README 与根 AGENTS.md）

- 原版可执行文件是语言 oracle；金标永远来自原版完整流（含拒绝与恢复
  标记），fresh process 捕获。
- 清单规模（README 自述：405 个 Atlas 输入、32 个目录、798 条 pre-index
  HPC JSON 记录）是覆盖清单，不是正确性百分比。
- harness 失败（checker 自检、标签错配等）不是数学失败，也不能充当
  数学证据；不可变失败阶段保留冻结。
- 本包不复制账本内容；验收状态以账本与 HANDOFF 为准。
