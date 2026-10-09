---
title: 带基数预算的 Weyl 群作用枚举
summary: enumerate_actions 在显式基数预算内通过 CompactWeyl 枚举并用 Rayon 物化矩阵，来源中的排序与性能陈述未经本包测试或测量验证。
sources:
  - weyl-layer.md
kind: concept
createdAt: "2026-10-09T15:17:46.546Z"
updatedAt: "2026-10-09T21:14:41.140Z"
tags:
  - Weyl群
  - 资源预算
  - 并行枚举
  - 证据边界
aliases:
  - 带基数预算的-weyl-群作用枚举
  - 带W群
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 带基数预算的 Weyl 群作用枚举
summary: enumerate_actions 在显式基数预算内通过 CompactWeyl 枚举元素，再以 Rayon 物化矩阵作用；排序与性能说明仍受结构性阅读的证据范围限制。
sources:
  - weyl-layer.md
kind: concept
tags:
  - Weyl群
  - 资源预算
  - 枚举算法
---

# 带基数预算的 Weyl 群作用枚举

`WeylGroup::enumerate_actions(budget)` 在显式基数预算下枚举全部 canonical Weyl 群作用。完整枚举与单个 `WeylAction` 的构造刻意分离：构造作用无需枚举整个群，只有调用枚举接口才请求全部作用。该接口属于 [[Weyl 群的矩阵作用与词级元素双层结构]] 中的矩阵作用层。^[weyl-layer.md:19-26, weyl-layer.md:30-41]

## 预算与返回值

`budget` 限定枚举的基数。来源列出的 A2 测试锚点覆盖预算为 6 时成功、预算为 5 时返回 `ResourceLimitExceeded` 的边界；来源包未执行这些测试，因此这里记录的是已有测试的覆盖情况，而非本包验证结果。^[weyl-layer.md:59-62, weyl-layer.md:121-121]

返回的 `WeylAction` 包含 `datum: Arc<BasedRootDatum>`、`weight_matrix` 与 `coweight_matrix`，同时表示 character 与 cocharacter 两个全格上的作用。来源将枚举结果描述为按 character-lattice 作用矩阵的字典序排列。^[weyl-layer.md:19-21, weyl-layer.md:30-41]

作用的等值判断采用派生的逐字段比较，包含 datum 值。即使矩阵相同，datum 值不同的两个作用仍不相等；`Arc` 的 `PartialEq` 委派给内部值。具体语义见 [[WeylAction 的等值与 datum 身份语义]]。^[weyl-layer.md:45-47]

## 枚举流程与排序

实现先通过 `weyl_transducer::CompactWeyl` 的紧致表示枚举元素，再使用 Rayon `par_iter` 对各元素调用 `compose_fast`，物化矩阵作用。来源将结果的字典序归因于 CompactWeyl 的输出顺序与 Rayon 保序 `collect`，同时明确指出没有测试断言验证这一排序性质。相关表示见 [[Weyl 群的紧凑 Transducer 表示]]。^[weyl-layer.md:52-55]

文件中仍保留私有的 `insert_action`，它是使用 `VecDeque` 的 BFS 去重助手，但文件内没有调用点。来源将其视为疑似旧矩阵 BFS 路径的遗留代码；所记录的当前枚举路径使用 CompactWeyl。^[weyl-layer.md:52-57]

## 算术约束

矩阵计算还具有算术前置条件。`compose_matrices` 使用 `i64` 累加，再通过 `sum as i32` 无检查截断；源码注释以 Weyl 矩阵条目受 Cartan 界约束解释这一选择，但论证未形式化。枚举热循环使用的 `compose_fast` 是 `pub(crate)` 接口，完全无检查，前置条件违约存在 panic 风险。^[weyl-layer.md:48-51]

## 性能与证据边界

源码注释给出 E6 紧致枚举约 50 ms、矩阵 BFS 约 1.1 s 的比较。这些数字是来源转述，并非来源包的独立测量；本包不提供性能或并行结论。^[weyl-layer.md:52-55, weyl-layer.md:121-121]

来源属于结构性阅读，未执行构建、测试或原版运行，也不构成数学验收。Weyl 层正确性由其独立的 HPC 证据链负责，包括 capacity gate 与 Weyl owner 语义线；本页不扩展该证据范围。`weyl_transducer.rs` 的枚举实现属于后续来源包，因此本页仅说明已记录的调用路径。^[weyl-layer.md:9-12, weyl-layer.md:120-121]

## Sources

- [weyl-layer.md](../../sources/weyl-layer.md)：Weyl 群层：矩阵作用与词级元素的双层结构。
