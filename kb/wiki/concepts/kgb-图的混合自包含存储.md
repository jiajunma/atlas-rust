---
title: KGB 图的混合自包含存储
summary: 图复制各对合位置的数据和余特征，只有计算精确有理环面因子的 torus_factor 访问器仍依赖对合表。
sources:
  - kgb-graph-structure.md
kind: concept
createdAt: "2026-10-09T14:54:32.874Z"
updatedAt: "2026-10-09T19:31:57.097Z"
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
---

# KGB 图的混合自包含存储

`KgbGraph` 为一个弱实形式保存一张 KGB 图，元素是该形式各 involution 之上的 Tits 元素。图采用混合自包含（HYBRID self-contained）存储：将每个 involution 位置的数据及余特征（cocharacter）复制进图，使除 `torus_factor` 外的所有访问器都不再需要 involution 表。相关背景见 [[KGB 图与弱实形式]]。^[kgb-graph-structure.md:19-23, kgb-graph-structure.md:81-84]

## 状态与链接的平铺布局

每个元素在每个单生成元处携带状态、cross 链接，以及可能存在的 Cayley 和 inverse-Cayley 链接。`statuses`、`cross`、`cayley`、`inverse_cayley` 四组存储均按 `x * rank + s` 平铺，以元素编号 `x` 和生成元编号 `s` 定位槽位。状态语义见 [[KGB 生成元状态与下降判定]]。^[kgb-graph-structure.md:21-23, kgb-graph-structure.md:74-74]

`cross(x, s)` 返回 `Option<KgbId>`，越界时为 `None`。`cayley(x, s)` 返回 `Result<Option<KgbId>, _>`：`Ok(None)` 表示该生成元在此元素处不是非紧致 imaginary，因而没有 Cayley 链接；`Err` 仅来自下标检查。^[kgb-graph-structure.md:75-77]

`inverse_cayley(x, s)` 在生成元不是 real 时返回 `Ok(None)`。存在链接时，单前像 `(first, None)` 对应 II 型，双前像 `Some((first, Some(second)))` 对应 I 型，并保证 `first < second`。逆 Cayley 链接由元素编号标准化后的升序后处理安装，参见 [[Cross、Cayley 与逆 Cayley 链接]]。^[kgb-graph-structure.md:78-80]

## involution 位置与 tau packet

`positions` 为每个排序位置记录 `(InvolutionId, involution 长度, CartanId)`；`first_of_tau` 是长度为 `positions.len()+1` 的累计计数。`tau_packet(position)` 返回对应 packet 的首元素与大小。^[kgb-graph-structure.md:67-70]

这些位置由编号标准化确定：先按 involution 长度、Weyl 长度及 `WeylElt::pieces` 字典序排列 involution，再以计数排序将 BFS 发现的元素归入各 tau packet。packet 之间按 involution 的排序位置排列，packet 内保持 BFS 发现顺序。相关细节见 [[tau packet 与 KGB 元素编号标准化]]。^[kgb-graph-structure.md:60-67]

## 保留的表依赖

`torus_factor` 是唯一仍需 involution 表的访问器，因为计算所需的 `theta` 随 involution 而异。来源将其精确有理数计算概述为 `(g_rho_check - lift(bits) + theta^T 作用) / 2`；因此，图内复制的数据足以支持其余访问器，但这一计算仍需读取表中的逐 involution 数据。^[kgb-graph-structure.md:81-84]

图还提供 `base_grading`，对应上游 `KGB_base::base_grading`，用于 `var_print_KGB` 输出中的 `Base grading: [...]` 头部。参见 [[KGB 打印与实形一致性]]。^[kgb-graph-structure.md:85-86]

## 证据边界

上述布局与接口说明来自对 `kgb_graph.rs` 的结构性阅读，阅读快照对应 dirty 工作区。来源未执行构建、测试或原版运行，不构成 KGB 枚举数学正确性、性能或兼容性的验收结论；上游引用行号来自源码注释，未经独立重读。数学正确性的验收应另见 [[HPC 验收证据链]]。^[kgb-graph-structure.md:9-15, kgb-graph-structure.md:93-98]

## Sources

- [KGB 图的结构与构造（每个弱实形式一张图）](kgb-graph-structure.md)
