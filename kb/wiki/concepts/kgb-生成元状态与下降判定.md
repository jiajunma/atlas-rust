---
title: KGB 生成元状态与下降判定
summary: 生成元分为复、紧虚、实和非紧虚四类；实根恒下降，虚根恒不下降，复根通过 cross 两端的对合长度判定下降。
sources:
  - kgb-graph-structure.md
kind: concept
createdAt: "2026-10-09T14:54:00.953Z"
updatedAt: "2026-10-09T20:56:58.889Z"
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
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: KGB 生成元状态与下降判定
summary: KGB 单生成元状态分为复、紧致虚、实和非紧致虚四类；实根恒下降、虚根恒不下降，复根按 cross 目标的对合长度是否缩短判定。
sources:
  - kgb-graph-structure.md
kind: concept
tags:
  - KGB
  - 根分类
aliases:
  - kgb-生成元状态与下降判定
---

# KGB 生成元状态与下降判定

`KgbStatus` 描述单生成元在某个 KGB 元素处的状态，共有复根、紧致虚根、实根和非紧致虚根四类。`is_descent` 据此判定下降：实根恒为下降，虚根恒非下降，复根则比较 cross 链接两端的对合（involution）长度。^[kgb-graph-structure.md:27-34]

在本实现中，每张 `KgbGraph` 对应一个弱实形式。每个元素在每个单生成元处携带状态、cross 链接，以及可能存在的 Cayley 与 inverse-Cayley 链接；背景见 [[KGB 图与弱实形式]]。^[kgb-graph-structure.md:19-23]

## 状态分类

分类分两步：先通过 involution 表的 `simple_root_kind` 获取 `RootKind`，区分 `Complex`、`Real` 和 `Imaginary`；仅对 `Imaginary` 情形，再通过 coset 的 `simple_grading_pregated` 区分紧致与非紧致，得到下表中的四种状态。^[kgb-graph-structure.md:27-31]

| `KgbStatus` | 含义 | 分类依据 |
|---|---|---|
| `Complex` | 复根 | `RootKind` 为 `Complex` |
| `ImaginaryCompact` | 紧致虚根 | `RootKind` 为 `Imaginary`，grading 判为紧致 |
| `Real` | 实根 | `RootKind` 为 `Real` |
| `ImaginaryNoncompact` | 非紧致虚根 | `RootKind` 为 `Imaginary`，grading 判为非紧致 |

## 下降规则

`is_descent` 对 `Real` 恒判为下降，对 `ImaginaryCompact` 和 `ImaginaryNoncompact` 恒判为非下降。对于 `Complex`，令 \(y=\operatorname{cross}(x,s)\)，则生成元 \(s\) 在元素 \(x\) 处为下降，当且仅当 \(\ell_{\mathrm{inv}}(y)<\ell_{\mathrm{inv}}(x)\)，其中 \(\ell_{\mathrm{inv}}\) 表示元素所属 involution 的长度。^[kgb-graph-structure.md:33-34]

## 状态与变换链接

非紧致虚根必须具有 Cayley 目标，且目标的 involution 长度恰比源增加一；这是构造时检查的 `"Cayley length step"` 不变量。查询 `cayley(x, s)` 时，`Ok(None)` 表示该处不是非紧致虚根，`Err` 只来自下标检查。^[kgb-graph-structure.md:52-55, kgb-graph-structure.md:75-77]

`inverse_cayley(x, s)` 返回 `Ok(None)` 表示该处不是实根。有逆链接时，单个前像 `(first, None)` 对应 II 型，两个前像 `Some((first, Some(second)))` 对应 I 型，并满足 `first < second`。逆链接在元素编号标准化后通过升序后处理安装，详见 [[Cross、Cayley 与逆 Cayley 链接]]。^[kgb-graph-structure.md:78-80]

## 构造与存储约束

状态分类与 cross、Cayley 目标计算发生在 [[分窗两相 BFS 构造]] 的第一相：以 64 个元素为一窗，使用 Rayon 并行执行纯计算，期间 involution 表与 coset 只读；第二相顺序执行 `intern`，按 `TitsElement` 去重并分配编号。每个状态槽 `(x, s)` 只能写入一次，重复写入会报 `KgbInvariantViolation`。^[kgb-graph-structure.md:49-53]

`statuses`、`cross`、`cayley` 和 `inverse_cayley` 数组统一按 `x * rank + s` 平铺，以元素编号和单生成元编号定位槽位。^[kgb-graph-structure.md:74-74]

## 证据边界

本页依据 `kgb_graph.rs` 的结构性阅读，所读字节来自来源记录的 dirty 工作区快照。来源未执行构建、测试或原版运行，不提供 KGB 枚举数学正确性、性能或兼容性结论；枚举正确性的证据属于独立的 HPC gate 链。^[kgb-graph-structure.md:9-15, kgb-graph-structure.md:97-98]

实现存在并行计算结构，但数学套件的计时运行强制使用 `RAYON_NUM_THREADS=1`，因此不能据此声称已有多核加速证据。^[kgb-graph-structure.md:49-56]

## Sources

- [kgb-graph-structure.md](../../sources/kgb-graph-structure.md) — KGB 图的结构与构造（每个弱实形式一张图）。
