---
title: Cross、Cayley 与逆 Cayley 链接
summary: 链接按元素与生成元平铺存储：Cayley 适用于非紧致 imaginary，逆 Cayley 适用于 real，后者区分单前像 II 型与双前像 I 型，并在编号标准化后按升序安装。
sources:
  - kgb-graph-structure.md
kind: concept
createdAt: "2026-10-09T14:54:31.295Z"
updatedAt: "2026-10-09T14:54:31.295Z"
tags:
  - KGB
  - Cayley变换
  - 数据布局
aliases:
  - crosscayley-与逆-cayley-链接
  - C与C链
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Cross、Cayley 与逆 Cayley 链接

在 [[KGB 图与弱实形式|KGB 图]]中，一张 `KgbGraph` 对应一个弱实形式，元素是该形式各 involution 之上的 Tits 元素。每个元素在每个单生成元处携带一个状态、一条 cross 链接，以及可能存在的 Cayley 与 inverse-Cayley（逆 Cayley）链接。^[kgb-graph-structure.md:19-23]

## 状态与链接语义

生成元状态分为 `Complex`、`ImaginaryCompact`、`Real` 和 `ImaginaryNoncompact`。分类先通过 involution 表确定根属于 complex、real 还是 imaginary，再用 coset 的 grading 将 imaginary 区分为紧致与非紧致；这些状态决定 Cayley 与逆 Cayley 链接是否适用。^[kgb-graph-structure.md:27-31, kgb-graph-structure.md:75-80]

### Cross 链接

`cross(x, s)` 返回 `Option<KgbId>`，下标越界时返回 `None`。对于 complex 生成元，[[KGB 生成元状态与下降判定|下降判定]]比较 cross 链接两端元素的 involution 长度：目标更短时为 descent。real 生成元恒为 descent，imaginary 生成元恒不是 descent。^[kgb-graph-structure.md:33-34, kgb-graph-structure.md:74-77]

### Cayley 链接

`cayley(x, s)` 返回 `Result<Option<KgbId>, _>`。Cayley 链接适用于非紧致 imaginary 状态；`Ok(None)` 表示该生成元在当前元素处不是非紧致 imaginary，因而没有 Cayley 链接。`Err` 只来自下标检查。^[kgb-graph-structure.md:75-77]

构造时，非紧致 imaginary 状态的 Cayley 目标必须存在，且目标的 involution 长度必须恰比源多一。这是显式检查的不变量，长度条件对应 `"Cayley length step"`。^[kgb-graph-structure.md:52-55]

### 逆 Cayley 链接

`inverse_cayley(x, s)` 在生成元不是 real 时返回 `Ok(None)`。存在逆 Cayley 链接时，`Some((first, None))` 表示 II 型，只有一个目标；`Some((first, Some(second)))` 表示 I 型，有两个目标且满足 `first < second`。逆 Cayley 链接由元素编号标准化之后的升序后处理安装。^[kgb-graph-structure.md:78-80]

## 构造与存储

状态、cross 目标和 Cayley 目标在[[分窗两相 BFS 构造]]中计算：每窗包含 64 个元素，第一相使用 Rayon 并行进行纯计算，此时表与 coset 只读；第二相顺序执行 `intern`，按 `TitsElement` 去重并分配新编号。每个 `(x, s)` 的状态槽只能写入一次，重复写入会触发 `KgbInvariantViolation`。^[kgb-graph-structure.md:49-53]

BFS 发现的元素随后按 involution 排序位置分组，并通过计数排序完成[[tau packet 与 KGB 元素编号标准化|编号标准化]]；每个 tau packet 内保留 BFS 发现顺序。逆 Cayley 的两个目标之间的编号顺序由标准化后的后处理确定。^[kgb-graph-structure.md:60-70, kgb-graph-structure.md:78-80]

`statuses`、`cross`、`cayley` 和 `inverse_cayley` 均使用 `x * rank + s` 的平铺布局。图采用[[KGB 图的混合自包含存储|混合自包含存储]]，这些链接访问器不需要外部 involution 表；仍需该表的访问器是 `torus_factor`。^[kgb-graph-structure.md:74-84]

## 证据边界

上述说明来自对 `kgb_graph.rs` 的结构性阅读，描述接口、存储和构造不变量，不构成 KGB 枚举数学正确性、性能或兼容性的验证。源材料未执行构建、测试或原版运行；相关正确性结论属于独立的 [[HPC 验收证据链]]。^[kgb-graph-structure.md:9-15, kgb-graph-structure.md:97-98]

## Sources

- [kgb-graph-structure.md](../../sources/kgb-graph-structure.md)
