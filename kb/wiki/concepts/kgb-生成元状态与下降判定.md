---
title: KGB 生成元状态与下降判定
summary: 生成元分为复、紧虚、实和非紧虚四类；实根恒下降、虚根恒不下降，复根依据 cross 目标的对合长度是否更短判定。
sources:
  - kgb-graph-structure.md
kind: concept
createdAt: "2026-10-09T14:54:00.953Z"
updatedAt: "2026-10-10T00:37:38.637Z"
tags:
  - KGB
  - 根分类
aliases:
  - kgb-生成元状态与下降判定
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: KGB 生成元状态与下降判定
summary: KGB 单生成元状态分为复、紧致虚、实和非紧致虚四类；实根恒下降，虚根恒不下降，复根依据 cross 目标的对合长度是否缩短判定。
sources:
  - kgb-graph-structure.md
kind: concept
tags:
  - KGB
  - 根分类
  - 下降判定
aliases:
  - kgb-生成元状态与下降判定
---

# KGB 生成元状态与下降判定

`KgbStatus` 表示单生成元在某个 KGB 元素处的状态，包含 `Complex`、`ImaginaryCompact`、`Real` 和 `ImaginaryNoncompact` 四类。`is_descent` 根据这一状态判定下降：实根恒为下降，虚根恒非下降，复根则比较 cross 链接两端的对合（involution）长度。^[kgb-graph-structure.md:27-34]

本实现中，每张 `KgbGraph` 对应一个弱实形式。图中每个元素在每个单生成元处携带状态、一条 cross 链接，以及可能存在的 Cayley 与 inverse-Cayley 链接，背景见 [[KGB 图与弱实形式]]。^[kgb-graph-structure.md:19-23]

## 状态分类

分类分为两步：先由 involution 表的 `simple_root_kind` 返回 `RootKind`，区分 `Complex`、`Real` 和 `Imaginary`；仅对 `Imaginary` 情形，再通过 coset 的 `simple_grading_pregated` 区分紧致与非紧致。四种最终状态及其下降规则如下。^[kgb-graph-structure.md:27-34]

| `KgbStatus` | 含义 | `is_descent` 判定 |
|---|---|---|
| `Complex` | 复根 | cross 目标的对合长度严格更短时为下降 |
| `ImaginaryCompact` | 紧致虚根 | 恒为非下降 |
| `Real` | 实根 | 恒为下降 |
| `ImaginaryNoncompact` | 非紧致虚根 | 恒为非下降 |

## 复根的长度判据

对于 `Complex` 状态，令 \(y=\operatorname{cross}(x,s)\)，并用 \(\ell_{\mathrm{inv}}(x)\) 表示元素 \(x\) 所属对合的长度，则下降条件为 \(\ell_{\mathrm{inv}}(y)<\ell_{\mathrm{inv}}(x)\)。比较对象是两端的对合长度。^[kgb-graph-structure.md:33-34]

元素编号标准化另有排序键：“对合长度、Weyl 长度、`WeylElt::pieces` 字典序”。该排序包含两种长度，但下降判据只使用上述对合长度比较；编号过程见 [[tau packet 与 KGB 元素编号标准化]]。^[kgb-graph-structure.md:33-34, kgb-graph-structure.md:60-70]

## 状态与变换链接

非紧致虚根必须具有 Cayley 目标，且目标的对合长度恰比源增加一；构造过程以 `"Cayley length step"` 不变量检查这一条件。`cayley(x, s)` 返回 `Result<Option<KgbId>, _>`：`Ok(None)` 表示该处不是非紧致虚根，`Err` 只来自下标检查。^[kgb-graph-structure.md:52-55, kgb-graph-structure.md:75-77]

`inverse_cayley(x, s)` 返回 `Ok(None)` 表示该处不是实根。有逆链接时，单个前像对应 II 型，两个前像对应 I 型，且 I 型满足 `first < second`。逆链接在元素编号标准化后通过升序后处理安装，详见 [[Cross、Cayley 与逆 Cayley 链接]]。^[kgb-graph-structure.md:78-80]

## 构造与存储约束

状态分类与 cross、Cayley 目标计算发生在 [[分窗两相 BFS 构造]] 的第一相：每窗包含 64 个元素，使用 Rayon 执行纯计算，期间 involution 表与 coset 只读；第二相顺序执行 `intern`，按 `TitsElement` 去重并分配新编号。状态槽遵循 write-once 约束，同一 `(x, s)` 写入两次会报 `KgbInvariantViolation`。^[kgb-graph-structure.md:49-53]

`statuses`、`cross`、`cayley` 和 `inverse_cayley` 均按 `x * rank + s` 平铺存储，以元素编号和单生成元编号定位槽位。^[kgb-graph-structure.md:74-74]

## 证据边界

本页依据来源对 `kgb_graph.rs` 的结构性阅读，所读字节属于来源记录的 dirty 工作区快照。来源未执行构建、测试或原版运行，不提供 KGB 枚举数学正确性、性能或兼容性结论；数学正确性属于独立的 [[HPC 验收证据链]]。^[kgb-graph-structure.md:9-15, kgb-graph-structure.md:97-98]

实现具有并行计算结构，但数学套件的计时运行强制使用 `RAYON_NUM_THREADS=1`，因此该结构本身不构成多核加速证据。^[kgb-graph-structure.md:49-56]

## Sources

- [kgb-graph-structure.md](../../sources/kgb-graph-structure.md) — KGB 图的结构与构造（每个弱实形式一张图）。
