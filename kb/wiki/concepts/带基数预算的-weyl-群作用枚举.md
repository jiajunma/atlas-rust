---
title: 带基数预算的 Weyl 群作用枚举
summary: enumerate_actions 在显式基数预算内使用 CompactWeyl 枚举并以 Rayon 物化矩阵，来源所述字典序及性能未获本包测试验证。
sources:
  - weyl-layer.md
kind: concept
createdAt: "2026-10-09T15:17:46.546Z"
updatedAt: "2026-10-09T19:38:24.993Z"
tags:
  - Weyl群
  - 资源预算
  - 枚举算法
aliases:
  - 带基数预算的-weyl-群作用枚举
  - 带W群
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 带基数预算的 Weyl 群作用枚举

`WeylGroup::enumerate_actions(budget)` 在显式基数预算下枚举全部 canonical Weyl 群作用。完整枚举与单个作用的构造刻意分离：构造 `WeylAction` 无需枚举整个群，完整枚举则由独立接口请求。该接口属于 [[Weyl 群的矩阵作用与词级元素双层结构]] 中的矩阵作用层。^[weyl-layer.md:19-26, weyl-layer.md:30-41]

## 预算与返回结果

`budget` 是枚举的显式基数预算。来源列出的 A2 测试锚点覆盖预算边界：预算为 6 时成功，为 5 时返回 `ResourceLimitExceeded`。来源包本身未执行这些测试，因此这里记录的是测试覆盖情况。^[weyl-layer.md:39-41, weyl-layer.md:59-62, weyl-layer.md:121-121]

返回的 `WeylAction` 包含 `datum: Arc<BasedRootDatum>`、`weight_matrix` 和 `coweight_matrix`，同时表示 character 与 cocharacter 两个全格上的作用。来源将结果顺序描述为 character-lattice 作用矩阵的字典序。^[weyl-layer.md:19-21, weyl-layer.md:30-41]

作用的等值判断采用派生的逐字段比较，包含 datum 值：即使矩阵相同，datum 值不同的两个作用仍不相等。`Arc` 的 `PartialEq` 委派给内部值，因此此处比较的是 datum 值，而非分配地址。参见 [[WeylAction 的等值与 datum 身份语义]]。^[weyl-layer.md:45-47]

## 枚举流程与排序

实现先通过 `weyl_transducer::CompactWeyl` 的紧致表示枚举元素，再使用 Rayon `par_iter` 对各元素调用 `compose_fast`，物化矩阵作用。来源将字典序归因于 CompactWeyl 的输出顺序与 Rayon 保序 `collect`，同时明确指出没有测试断言验证这一排序性质。相关表示见 [[Weyl 群的紧凑 Transducer 表示]]。^[weyl-layer.md:52-55]

文件中还保留私有的 `insert_action`，它是使用 `VecDeque` 的 BFS 去重助手，但文件内没有调用点。来源将其标记为疑似旧矩阵 BFS 路径的遗留代码；当前记录的枚举路径使用 CompactWeyl。^[weyl-layer.md:52-57]

## 算术约束

矩阵运算仍有独立于基数预算的算术前置条件。`compose_matrices` 使用 `i64` 累加，再以 `sum as i32` 无检查截断；源码注释以 Weyl 矩阵条目受 Cartan 界约束解释这一选择，但论证未形式化。枚举热循环使用的 `compose_fast` 是 `pub(crate)` 接口，完全无检查，前置条件违约存在 panic 风险。^[weyl-layer.md:48-51]

## 性能与证据边界

源码注释给出 E6 紧致枚举约 50 ms、矩阵 BFS 约 1.1 s 的比较。这些数字只是来源转述，不是来源包的独立性能测量，也不能据此认定并行性能已获验证。^[weyl-layer.md:52-55, weyl-layer.md:121-121]

来源包属于结构性阅读，未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。Weyl 层正确性属于其独立的 [[HPC 验收证据链]]，包括 capacity gate 与 Weyl owner 语义线；本页不扩展这一证据范围。^[weyl-layer.md:9-12, weyl-layer.md:121-121]

## Sources

- [weyl-layer.md](weyl-layer.md)：Weyl 群层：矩阵作用与词级元素的双层结构。
