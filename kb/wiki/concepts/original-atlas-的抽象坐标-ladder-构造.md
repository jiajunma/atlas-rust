---
title: Original Atlas 的抽象坐标 ladder 构造
summary: 原版先在抽象简单根坐标中建立 ladder，再通过 Weyl 反射置换扩展到全部根，环境格嵌入的大坐标不参与该阶段减法。
sources:
  - root-ladder-overflow-repair.md
kind: concept
createdAt: "2026-10-09T15:10:41.042Z"
updatedAt: "2026-10-10T00:50:46.192Z"
tags:
  - 根系
  - baseline
aliases:
  - original-atlas-的抽象坐标-ladder-构造
  - OA的L构
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Original Atlas 的抽象坐标 ladder 构造
summary: Original Atlas 在抽象简单根坐标中建立 simple-root ladder，再通过 Weyl 反射置换传送到全部正负根，使环境格嵌入的大坐标不参与该阶段减法。
sources:
  - root-ladder-overflow-repair.md
kind: concept
tags:
  - 根系
  - 算法
  - 上游兼容
aliases:
  - original-atlas-的抽象坐标-ladder-构造
provenanceState: extracted
---

# Original Atlas 的抽象坐标 ladder 构造

Original Atlas 先用抽象简单根坐标 `Byte_vector` 建立 simple-root ladder，再利用 Weyl reflection permutation（Weyl 反射置换）将表传送到所有正根和负根。含环面因子的环境格嵌入所产生的大坐标不进入这一阶段的减法。来源对应冻结 commit `7e1b958c7aa9456769cc9cf09ac1542814b4800a`，源码位置为 `sources/structure/rootdata.cpp:238-317`。^[root-ladder-overflow-repair.md:57-62]

## 数学对象与成员关系

对完整存储的有限根集 $R\subseteq\mathbb Z^d$ 和 $\alpha\in R$，ladder bottom 集定义为 $B_\alpha=\{\beta\in R\mid\beta-\alpha\notin R\}$，即收集减去 $\alpha$ 后不再属于根集的根 $\beta$。参见 [[Root ladder bottom 集与固定宽度成员查询]]。^[root-ladder-overflow-repair.md:41-49]

构造必须保持精确的根／余根成员关系。Original Atlas 与 Rust 可以使用不同的坐标和存储策略，但固定宽度表示不能改变成员查询的结果。^[root-ladder-overflow-repair.md:64-66]

## 两阶段构造与坐标边界

原版首先在抽象简单根坐标中建立简单根对应的 ladder，然后借助 Weyl 反射置换传送已有表，覆盖全部正负根。抽象坐标阶段不读取含 torus factor 的环境格嵌入坐标，因此该嵌入中的大坐标不会影响这一阶段的减法。^[root-ladder-overflow-repair.md:59-62]

Rust 则逐对相减环境格坐标来进行成员查询。来源要求两者保持相同的可观测成员关系，并不要求将 Rust 整体改写成 C++ 的数据结构。^[root-ladder-overflow-repair.md:64-66]

## 与 Rust 坐标溢出修复的关系

Rust 原有错误是在构造 root/coroot ladder bottom 表时，将环境格坐标差的 `i32` 溢出视为整个 `RootSystem` 构造失败。原版的抽象坐标构造不经过这一环境格大坐标减法路径。^[root-ladder-overflow-repair.md:13-15, root-ladder-overflow-repair.md:57-62]

对于完整存储为 `i32` 坐标的根或余根集合，如果数学整数中的精确差有任一坐标超出 `i32` 范围，该差就不可能等于任何已存向量。因此，这一成员查询应返回 `false`，相应的 $\beta$ 应进入 bottom 集。这个论证不允许 wrapping 或 saturating 算术，也不能推广至一般向量减法、反射、root combination、seed negation 或构造输入验证。^[root-ladder-overflow-repair.md:46-53]

[[Rust ladder 成员查询的选择性溢出处理|Rust 的局部修复]]仅调整 `build_ladder_bottoms` 中的两次成员查询：减法成功时保持原有 root 二分查找或 coroot map 查找；仅将 `StructureError::ArithmeticOverflow` 解释为不属于集合；分配失败及其他错误继续传播。root 与 coroot 查询独立执行，root 溢出不会跳过 coroot 查询；底层减法、反射、`combine_roots`、数据布局、排序和 public API 均不修改。^[root-ladder-overflow-repair.md:74-83]

## 兼容性证据与限制

[[A1 加中心环面边界 fixture 与历史 oracle]] 包含 11 个 A1+torus case，覆盖 root/coroot 交换、两种编号、阈值下方、阈值上方及 `i32::MAX`，并保留 recovery marker `719`。原版 oracle 来自历史 capture job `3868832`，其[捕获记录](../../tests/reference/hpc/math_weyl_context_capture_2026_09_30.json)绑定 case、oracle binary/source 和 raw stream；AFTER 作业没有重新运行原版 oracle。^[root-ladder-overflow-repair.md:92-98, root-ladder-overflow-repair.md:124-130]

来源前段将 AFTER-v3 的限定接受范围写为不含 acceptance-index 登记，后段则记载已追加条目 `0003-a1-torus-root-coroot-ladder-boundary`，状态为 `accepted`／`math_pass`，并绑定报告、独立 review 和 canonical 源文件清单。这两处记载应保留区分，详见 [[Root ladder 修复的限定接受与账本状态歧义]]。^[root-ladder-overflow-repair.md:30-37, root-ladder-overflow-repair.md:115-123]

相关证据仅覆盖 A1+中心环面坐标边界 fixture 及其回归范围，不能推广为一般 root system 正确性、更高 rank、KLV、unitarity、Hodge、associated cycle 或 AV-ann 的证明。Rust 完整测试套件是回归守卫，而非 oracle 证明；来源也不支持性能或内存结论。^[root-ladder-overflow-repair.md:124-130, root-ladder-overflow-repair.md:141-143]

## Sources

- [root-ladder-overflow-repair.md — Root ladder 固定宽度坐标溢出修复](../../sources/root-ladder-overflow-repair.md)
