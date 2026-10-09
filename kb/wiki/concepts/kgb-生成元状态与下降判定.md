---
title: KGB 生成元状态与下降判定
summary: 生成元状态由根类型及 imaginary grading 分为四类；real 恒为 descent，imaginary 恒非 descent，complex 根据 cross 目标的 involution 长度是否更短判定。
sources:
  - kgb-graph-structure.md
kind: concept
createdAt: "2026-10-09T14:54:00.953Z"
updatedAt: "2026-10-09T14:54:00.953Z"
tags:
  - KGB
  - 根分类
  - 下降判定
aliases:
  - kgb-生成元状态与下降判定
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# KGB 生成元状态与下降判定

`KgbStatus` 描述一个单生成元在某个 KGB 元素处的状态。Atlas 的 KGB 集合是 $K$ 在旗簇 $G/B$ 上的轨道集合；本实现中，每张 `KgbGraph` 对应一个弱实形式，每个元素在每个单生成元处携带状态、cross 链接，以及可能的 Cayley 与 inverse-Cayley 链接。参见 [[KGB 图与弱实形式]]。^[kgb-graph-structure.md:19-23]

## 四种生成元状态

`KgbStatus` 是四值枚举。分类先通过 involution 表的 `simple_root_kind` 获得 `RootKind`，区分 Complex、Real 和 Imaginary；仅在 Imaginary 情形下，再通过 coset 的 `simple_grading_pregated` 区分紧致与非紧致。该过程对应 [[对合下的虚根、实根与复根分类]] 的根类型区分，并进一步细化虚根状态。^[kgb-graph-structure.md:27-31]

| 状态 | 含义 | 分类依据 |
|---|---|---|
| `Complex` | 复根 | `RootKind::Complex` |
| `ImaginaryCompact` | 紧致虚根 | Imaginary，且 grading 判为紧致 |
| `Real` | 实根 | `RootKind::Real` |
| `ImaginaryNoncompact` | 非紧致虚根 | Imaginary，且 grading 判为非紧致 |

以上四种状态及其两步分类依据均由状态分类逻辑给出。^[kgb-graph-structure.md:27-31]

## 下降判定

`is_descent` 按状态应用不同规则：`Real` 恒为下降，两种 Imaginary 状态恒不是下降；`Complex` 则比较 cross 链接两端元素的 **involution 长度**，仅当目标长度严格小于源长度时判为下降。这里比较的是 involution 长度。^[kgb-graph-structure.md:33-34]

因此，实根与虚根的下降判定只需读取状态，而复根还需读取 cross 目标及两端的 involution 长度。复根的规则可写为：令 $y=\operatorname{cross}(x,s)$，则生成元 $s$ 在 $x$ 处为下降，当且仅当 $\ell_{\mathrm{inv}}(y)<\ell_{\mathrm{inv}}(x)$。^[kgb-graph-structure.md:33-34]

## 状态与变换链接

非紧致虚根状态对应 Cayley 链接。构造时，其 Cayley 目标必须存在，且目标的 involution 长度恰比源增加一，否则触发 `"Cayley length step"` 不变量错误。查询 `cayley(x, s)` 时，`Ok(None)` 表示该处不是非紧致虚根，因而没有 Cayley 链接；`Err` 只来自下标检查。^[kgb-graph-structure.md:52-55, kgb-graph-structure.md:75-77]

实根状态对应 inverse-Cayley 查询：`Ok(None)` 表示该处不是实根；存在链接时，`(first, None)` 表示 II 型，`Some((first, Some(second)))` 表示 I 型，并满足 `first < second`。这些逆链接在元素编号标准化后按升序后处理安装。参见 [[Cross、Cayley 与逆 Cayley 链接]]。^[kgb-graph-structure.md:78-80]

## 构造与存储约束

状态分类与 cross、Cayley 目标计算发生在 [[分窗两相 BFS 构造]] 的并行纯计算阶段，此时 involution 表与 coset 只读；随后顺序执行 `intern`，按 `TitsElement` 去重并分配新编号。每个状态槽 `(x, s)` 只能写入一次，重复写入会报 `KgbInvariantViolation`。^[kgb-graph-structure.md:49-53]

`statuses` 与各类链接数组统一按 `x * rank + s` 平铺存储，将元素编号与单生成元编号定位到对应槽位。^[kgb-graph-structure.md:74-74]

## 证据边界

本页依据对 `kgb_graph.rs` 的结构性阅读，说明状态分类、下降规则与相关构造约束。来源未执行构建、测试或原版运行，不构成 KGB 枚举数学正确性、性能或兼容性的验证；数学正确性属于独立的 [[HPC 验收证据链]]。^[kgb-graph-structure.md:9-15, kgb-graph-structure.md:97-98]

## Sources

- [kgb-graph-structure.md](kgb-graph-structure.md) — KGB 图的结构与构造（每个弱实形式一张图）。
