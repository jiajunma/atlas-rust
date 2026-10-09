---
title: KGB 图的混合自包含存储
summary: 图复制各对合位置的数据与余特征，使除 torus_factor 外的访问器均不依赖对合表，而精确有理环面因子计算仍需查询表。
sources:
  - kgb-graph-structure.md
kind: concept
createdAt: "2026-10-09T14:54:32.874Z"
updatedAt: "2026-10-09T22:34:37.241Z"
tags:
  - KGB
  - 数据布局
aliases:
  - kgb-图的混合自包含存储
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: KGB 图的混合自包含存储
summary: KgbGraph 复制各对合位置的数据与余特征，状态和链接采用平铺布局；只有计算精确有理环面因子的 torus_factor 访问器仍依赖对合表。
sources:
  - kgb-graph-structure.md
kind: concept
tags:
  - KGB
  - 存储设计
  - 有理算术
aliases:
  - kgb-图的混合自包含存储
provenanceState: extracted
---

# KGB 图的混合自包含存储

`KgbGraph` 为一个弱实形式保存一张 KGB 图，元素是该形式各对合（involution）之上的 Tits 元素。图采用混合自包含（HYBRID self-contained）存储：将每个对合位置的数据及余特征（cocharacter）复制进图，使除 `torus_factor` 外的所有访问器都不需要对合表。相关背景见 [[KGB 图与弱实形式]]。^[kgb-graph-structure.md:19-23, kgb-graph-structure.md:81-84]

## 状态与链接的平铺布局

每个元素在每个单生成元处携带一个状态、一条 cross 链接，以及可能存在的 Cayley 和 inverse-Cayley 链接。`statuses`、`cross`、`cayley`、`inverse_cayley` 均按 `x * rank + s` 平铺，以元素编号 `x` 和生成元编号 `s` 定位槽位。状态分类与下降规则见 [[KGB 生成元状态与下降判定]]。^[kgb-graph-structure.md:21-34, kgb-graph-structure.md:74-74]

`cross(x, s)` 返回 `Option<KgbId>`，越界时为 `None`。`cayley(x, s)` 返回 `Result<Option<KgbId>, _>`：`Ok(None)` 表示该生成元在此元素处不是非紧致虚根类型，因而没有 Cayley 链接；`Err` 仅来自下标检查。^[kgb-graph-structure.md:75-77]

`inverse_cayley(x, s)` 在生成元不是 real 时返回 `Ok(None)`。存在逆 Cayley 链接时，II 型只有一个前像，配对内容为 `(first, None)`；I 型有两个前像，可选值为 `Some((first, Some(second)))`，且保证 `first < second`。这些链接由元素编号标准化之后的升序后处理安装，详见 [[Cross、Cayley 与逆 Cayley 链接]]。^[kgb-graph-structure.md:78-80]

## 对合位置与 tau packet

`positions` 为每个排序位置记录 `(InvolutionId, involution 长度, CartanId)`；`first_of_tau` 是长度为 `positions.len()+1` 的累计计数。`tau_packet(position)` 返回对应 packet 的首元素与大小。^[kgb-graph-structure.md:67-70]

编号标准化先按对合长度、Weyl 长度及 `WeylElt::pieces` 字典序排列对合，再用计数排序将 BFS 发现的元素归入各 tau packet。packet 之间按对合的排序位置排列，packet 内保持 BFS 发现顺序。对合排序键是严格全序，因此 `sort_unstable` 的稳定性无关紧要；承载 packet 内顺序语义的是计数排序，参见 [[tau packet 与 KGB 元素编号标准化]]。^[kgb-graph-structure.md:60-67]

## 保留的表依赖与打印数据

`torus_factor` 是唯一仍需对合表的访问器，因为计算所需的 `theta` 随对合而异。来源将其精确有理数计算概述为 `(g_rho_check - lift(bits) + theta^T 作用) / 2`，这一访问器因而保留对逐对合数据的表依赖。^[kgb-graph-structure.md:81-84]

`base_grading` 对应上游 `KGB_base::base_grading`，用于 `var_print_KGB` 输出中的 `Base grading: [...]` 头部。^[kgb-graph-structure.md:85-86]

## 证据边界

本页依据对 `kgb_graph.rs` 的结构性阅读，所读字节记录于 `snapshots/2026-10-03-kgb-graph.json`，对应 dirty 工作区。来源未执行构建、测试或原版运行，不构成 KGB 枚举数学正确性、性能或兼容性的验收结论；上游引用行号来自源码注释，未经独立重读，可能随上游演进而漂移。^[kgb-graph-structure.md:9-15, kgb-graph-structure.md:90-98]

## Sources

- [KGB 图的结构与构造（每个弱实形式一张图）](../../sources/kgb-graph-structure.md)
