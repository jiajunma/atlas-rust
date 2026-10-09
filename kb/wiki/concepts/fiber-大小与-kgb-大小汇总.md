---
title: fiber 大小与 KGB 大小汇总
summary: 分类预计算各实形式的 KGB 大小与全局总量；当实形式不属于指定 Cartan 类时，fiber_size 返回 Some(0)，使跨 Cartan 求和保持正确。
sources:
  - strong-real.md
kind: concept
createdAt: "2026-10-09T15:12:44.597Z"
updatedAt: "2026-10-09T19:36:50.780Z"
tags:
  - KGB
  - 纤维计数
  - 接口语义
aliases:
  - fiber-大小与-kgb-大小汇总
  - F大K大
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# fiber 大小与 KGB 大小汇总

`StrongRealClassification` 在已完成的 [[Cartan 分类构造与共享分区|Cartan 分类]]之上构建强实层，提供逐 Cartan 的 fiber 大小、逐实形式的 KGB 大小及全局 KGB 大小查询。相关字段包括 `per_cartan`、`local_of_form`、`kgb_sizes` 和 `global_kgb_size`。^[strong-real.md:19-22, strong-real.md:49-58]

## 大小查询与汇总语义

`fiber_size(form, cartan)` 查询指定实形式在指定 Cartan 类上的 fiber 大小。当该实形式不属于该 Cartan 类时，返回 `Some(0)`，使遍历全部 Cartan 类求和时仍保持正确：不包含该实形式的 Cartan 类贡献零。^[strong-real.md:56-58]

`kgb_size(form)` 返回预计算的指定实形式 KGB 大小；`global_kgb_size()` 返回全局大小，来源将其描述为“所有强实形式的总和”。逐 Cartan 的强实数据则通过 `strong_real_data(cartan)` 访问。^[strong-real.md:56-58]

## fiber 轨道与计数不变量

一个强实形式代表元对应某个平方类的 fiber group 中的一个 $W_{im}$ 轨道，并位于一个弱实形式之上。逐 Cartan 的 [[Cartan 类的强实层 StrongRealData|StrongRealData]] 保存平方类集合、逐平方类的 fiber 轨道大小及强代表元，提供 `fiber_size(local)`、`fiber_orbit_count(square)`、`orbit_elements(square, orbit)` 和 `weak_real_of_orbit` 等访问器。^[strong-real.md:19-22, strong-real.md:34-47]

轨道编号与轨道大小具有不同的依赖关系：`fiber_orbit` 编号依赖消元时选取的具体解，但轨道大小不依赖这一选择，因为 `ker(toAdjoint)` 的平移与作用交换。平方类的换基同样只会置换标签，不改变 partition 结构及各项大小；参见 [[平方类编号与换基不变量]]。^[strong-real.md:26-37]

## 构造与资源边界

`StrongRealClassification::build(classification, max_fiber_elements)` 逐 Cartan 计算强实数据：取得 grading 的 adjoint fiber 与 ambient fiber，构造 fiber map 的像坐标，再使用 `ModTwoSubquotient` 求平方商。当 fiber 维数超过 `MAX_MASK_BITS` 时，构造报 `StrongRealResourceLimit`。相关说明见 [[强实分类的构造与资源边界]]。^[strong-real.md:19-20, strong-real.md:51-54]

## 证据范围

上述接口与计数语义来自对 `strong_real.rs` 的结构性阅读。来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；强实分类的正确性属于其独立的 [[HPC 验收证据链]]，包括 Cartan/seed gate 等。^[strong-real.md:9-15, strong-real.md:77-77]

## Sources

- [strong-real.md](strong-real.md)：强实形式分类：平方类编号与 fiber 轨道。
