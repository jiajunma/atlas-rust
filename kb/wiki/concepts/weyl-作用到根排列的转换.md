---
title: Weyl 作用到根排列的转换
summary: action_permutation 先检查 datum 一致性，再逐根施加 Weyl 作用并反查 RootId 形成排列；源码注释以对偶反射一致性解释为何不另行复查余根运输。
sources:
  - root-system.md
kind: concept
createdAt: "2026-10-09T15:12:15.666Z"
updatedAt: "2026-10-09T15:12:15.666Z"
tags:
  - Weyl群
  - 根系
  - 置换
aliases:
  - weyl-作用到根排列的转换
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Weyl 作用到根排列的转换

`RootSystem::action_permutation` 将 `WeylAction` 转换为根集合上的排列：先检查作用与根系的 datum 一致性，再逐根施加作用，并在根表中反查所得根的索引。排列所依据的是根表的稳定顺序。^[root-system.md:78-85]

## 根编号与转换过程

`RootSystem` 从 `BasedRootDatum` 枚举普通根系，按环境坐标字典序升序存储根。`roots`、`coroots` 与 `simple_coordinates` 三张表索引对齐，一个 `RootId` 同时标识其中的对应记录。最终根序由闭包 `BTreeMap` 的键序决定，与 BFS 的发现顺序无关；因此，理解排列索引时应区分根表顺序与生成器顺序。参见 [[RootId 与根系索引对齐]]。^[root-system.md:18-24, root-system.md:55-62, root-system.md:78-82]

转换的入口检查是 datum 一致性。通过检查后，`action_permutation` 对每个根施加作用，再反查其像；根表提供的 `id_of` 使用二分查找。这里的顺序依据是环境坐标字典序，不能用根表的前后两半来判定正负根；正性由预计算表给出。^[root-system.md:78-85]

## 余根运输的依据

该转换不重新核查余根运输。源码注释给出的理由是：`WeylAction` 是简单反射之词，其余权生成器与根系闭包构造使用的对偶反射相同。闭包枚举通过 `reflect_weight` 与 `reflect_coweight` 同时生成根和余根候选。相关作用结构见 [[WeylAction 的对偶全格作用]]，词的作用顺序见 [[Weyl 词对根与权的作用顺序]]。^[root-system.md:55-62, root-system.md:82-85]

## 测试与证据边界

来源列出的测试锚点包含 A2 全部 6 个 Weyl 作用的运输一致性；另有 A2 根字典序、根与余根配对坐标等测试锚点。这些记录说明了测试覆盖的具体范围，但来源未见直接触发 `DatumMismatch`、`InvalidRootAutomorphism` 或 `AllocationFailed` 的错误路径测试。^[root-system.md:114-120, root-system.md:130-131]

本页依据的是结构性源码阅读，不代表根系层的数学验收。来源中的上游对应关系与 HPC 捕获编号仅转录自代码注释，未核对上游字节；本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。参见 [[HPC 验收证据链]]。^[root-system.md:9-14, root-system.md:124-125, root-system.md:135-139]

## Sources

- [root-system.md](root-system.md) — 普通根系的确定性枚举：RootSystem、RootId 与梯子底表
