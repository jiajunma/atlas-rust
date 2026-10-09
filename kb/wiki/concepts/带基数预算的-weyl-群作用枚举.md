---
title: 带基数预算的 Weyl 群作用枚举
summary: enumerate_actions 通过 CompactWeyl 枚举并并行物化矩阵，使用显式基数预算；来源描述了字典序输出，但未提供排序测试或独立性能验证。
sources:
  - weyl-layer.md
kind: concept
createdAt: "2026-10-09T15:17:46.546Z"
updatedAt: "2026-10-09T15:17:46.546Z"
tags:
  - Weyl群
  - 枚举算法
  - 资源预算
aliases:
  - 带基数预算的-weyl-群作用枚举
  - 带W群
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 带基数预算的 Weyl 群作用枚举

`WeylGroup::enumerate_actions(budget)` 在显式基数预算下枚举全部 canonical Weyl 群作用。枚举与作用构造刻意分离：单个 `WeylAction` 的构造不要求枚举整个群，完整枚举则通过独立接口请求。该接口属于 [[Weyl 群的矩阵作用与词级元素双层结构]] 中的矩阵作用层。^[weyl-layer.md:19-26, weyl-layer.md:30-41]

## 预算与返回结果

`budget` 限定枚举允许的基数。已有测试锚点覆盖 A2：预算为 6 时成功，预算为 5 时返回 `ResourceLimitExceeded`。这一锚点说明了该案例的预算边界；来源包本身没有执行这些测试。^[weyl-layer.md:59-62, weyl-layer.md:121-121]

返回的每个 `WeylAction` 包含 `datum: Arc<BasedRootDatum>`、`weight_matrix` 和 `coweight_matrix`，同时描述 character 与 cocharacter 两个全格上的作用。来源将结果顺序描述为 character-lattice 作用矩阵的字典序。^[weyl-layer.md:19-21, weyl-layer.md:30-41]

作用值的等值判断需要保留 datum 条件：实际实现使用派生的逐字段比较，包含 datum 值；即使矩阵相同，datum 值不同的两个作用仍不相等。`Arc` 的 `PartialEq` 委派给内部值，因此这里比较的是 datum 值，而不是分配地址。相关语义见 [[WeylAction 的等值与 datum 身份语义]]。^[weyl-layer.md:45-47]

## 枚举流程与排序

当前实现先通过 `weyl_transducer::CompactWeyl` 的紧致表示枚举元素，再以 Rayon `par_iter` 对各元素调用 `compose_fast`，物化矩阵作用。结果的字典序依赖 CompactWeyl 的输出顺序与 Rayon 保序 `collect`；来源特别指出，没有测试断言验证该排序性质。相关表示见 [[Weyl 群的紧凑 Transducer 表示]]。^[weyl-layer.md:52-55]

文件内还保留私有 `insert_action`，它是使用 `VecDeque` 的 BFS 去重助手，但没有调用点。来源将其标记为疑似旧矩阵 BFS 路径的遗留代码，当前枚举流程使用 CompactWeyl。^[weyl-layer.md:52-57]

## 算术约束与证据边界

基数预算之外，矩阵物化仍依赖算术实现的前置条件。`compose_matrices` 使用 i64 累加后以 `sum as i32` 无检查截断；源码注释以 Weyl 矩阵条目受 Cartan 界约束解释其合理性，但该论证未形式化。枚举热循环使用的 `compose_fast` 是完全无检查的 crate 内部接口，前置条件违约存在 panic 风险。^[weyl-layer.md:48-51]

源码注释给出 E6 紧致枚举约 50 ms、矩阵 BFS 约 1.1 s 的比较，但这只是来源转述，不能视为本来源包的性能测量。该包仅提供结构性阅读证据，没有执行构建、测试或原版运行，也不提供数学验收、性能或并行结论；Weyl 层的正确性另属其 [[HPC 验收证据链]]。^[weyl-layer.md:9-12, weyl-layer.md:52-55, weyl-layer.md:121-121]

## Sources

- [weyl-layer.md](weyl-layer.md)：Weyl 群层：矩阵作用与词级元素的双层结构。
