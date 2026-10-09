---
title: Cross、Cayley 与逆 Cayley 链接
summary: 链接按 x * rank + s 平铺；Cayley 适用于非紧虚根，逆 Cayley 适用于实根，并区分 II 型单前像与 I 型升序双前像。
sources:
  - kgb-graph-structure.md
kind: concept
createdAt: "2026-10-09T14:54:31.295Z"
updatedAt: "2026-10-09T20:57:28.993Z"
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
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Cross、Cayley 与逆 Cayley 链接
summary: KGB 链接按元素与单生成元平铺存储；Cayley 适用于非紧虚根，逆 Cayley 适用于实根，并区分 II 型单前像与 I 型升序双前像。
sources:
  - kgb-graph-structure.md
kind: concept
tags:
  - KGB
  - Cayley变换
---

# Cross、Cayley 与逆 Cayley 链接

在 [[KGB 图与弱实形式|KGB 图]]中，一张 `KgbGraph` 对应一个弱实形式，其元素是该形式各 involution 之上的 Tits 元素。每个元素在每个单生成元处携带一个状态、一条 cross 链接，以及可能存在的 Cayley 与 inverse-Cayley（逆 Cayley）链接。^[kgb-graph-structure.md:19-23]

## 状态与接口语义

生成元状态分为 `Complex`、`ImaginaryCompact`、`Real` 和 `ImaginaryNoncompact`。分类先由 involution 表的 `simple_root_kind` 确定根属于 complex、real 还是 imaginary，再通过 coset 的 `simple_grading_pregated` 将 imaginary 区分为紧致与非紧致。^[kgb-graph-structure.md:27-31]

### Cross 链接

`cross(x, s)` 返回 `Option<KgbId>`，下标越界时返回 `None`。对于 complex 生成元，[[KGB 生成元状态与下降判定|下降判定]]比较 cross 链接两端元素的 involution 长度：目标更短时为 descent。real 生成元恒为 descent，imaginary 生成元恒不是 descent。^[kgb-graph-structure.md:33-34, kgb-graph-structure.md:74-77]

### Cayley 链接

`cayley(x, s)` 返回 `Result<Option<KgbId>, _>`。Cayley 链接适用于非紧致 imaginary 状态；`Ok(None)` 表示该生成元在当前元素处不是非紧致 imaginary，因而没有 Cayley 链接。`Err` 只来自下标检查。^[kgb-graph-structure.md:75-77]

构造时，非紧致 imaginary 的 Cayley 目标必须存在，且目标的 involution 长度恰比源多一；这一构造不变量以 `"Cayley length step"` 标识。^[kgb-graph-structure.md:52-55]

### 逆 Cayley 链接

`inverse_cayley(x, s)` 在生成元不是 real 时返回 `Ok(None)`。存在链接时，前像对中的 `(first, None)` 表示 II 型，只有一个前像；`Some((first, Some(second)))` 表示 I 型，具有两个前像，并满足 `first < second`。逆 Cayley 链接由元素编号标准化之后的升序后处理安装。^[kgb-graph-structure.md:78-80]

## 构造与编号顺序

状态、cross 目标和 Cayley 目标在[[分窗两相 BFS 构造]]中计算。每窗包含 64 个元素：第一相使用 Rayon 对窗内元素进行纯计算，此时表与 coset 只读；第二相顺序执行 `intern`，按 `TitsElement` 去重并分配新编号。每个 `(x, s)` 的状态槽只能写入一次，重复写入会触发 `KgbInvariantViolation`。^[kgb-graph-structure.md:49-53]

BFS 后，involution 按“involution 长度、Weyl 长度、`WeylElt::pieces` 字典序”排序，再通过计数排序完成[[tau packet 与 KGB 元素编号标准化|元素编号标准化]]。不同 tau packet 按排序后的 involution 位置排列，每个 packet 内保留 BFS 发现顺序。排序键是严格全序，所需的稳定性由计数排序承载；逆 Cayley 链接的升序后处理在编号标准化之后进行。^[kgb-graph-structure.md:60-70, kgb-graph-structure.md:78-80]

## 存储布局

`statuses`、`cross`、`cayley` 和 `inverse_cayley` 均按 `x * rank + s` 平铺存储。图采用[[KGB 图的混合自包含存储|混合自包含存储]]，将每个 involution 位置的数据和 cocharacter 复制进图，因此这些链接访问器不需要外部 involution 表；唯一仍需要该表的访问器是 `torus_factor`。^[kgb-graph-structure.md:74-84]

## 证据边界

上述说明来自对 `kgb_graph.rs` 的结构性阅读，描述接口、存储和构造不变量，不构成 KGB 枚举数学正确性、性能或兼容性的验证。源材料未执行构建、测试或原版运行；枚举正确性的证据属于独立的 HPC gate 链。材料引用的上游行号来自源码注释，未经独立重读。^[kgb-graph-structure.md:9-15, kgb-graph-structure.md:90-98]

两相 BFS 的并行结构也不等于多核加速证据：源材料指出，数学套件的计时运行强制使用 `RAYON_NUM_THREADS=1`。^[kgb-graph-structure.md:49-56]

## Sources

- [kgb-graph-structure.md](../../sources/kgb-graph-structure.md) — KGB 图的结构与构造（每个弱实形式一张图）。
