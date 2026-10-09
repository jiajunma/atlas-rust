---
title: fiber 大小与 KGB 大小汇总
summary: StrongRealClassification 预计算各 form 的 KGB 大小及全局总量；fiber_size(form, cartan) 在 form 不属于指定 Cartan 时返回 Some(0)，使跨全部 Cartan 的求和保持正确。
sources:
  - strong-real.md
kind: concept
createdAt: "2026-10-09T15:12:44.597Z"
updatedAt: "2026-10-09T15:12:44.597Z"
tags:
  - KGB
  - 计数
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
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# fiber 大小与 KGB 大小汇总

`StrongRealClassification` 在已完成的 [[Cartan 分类构造与共享分区|Cartan 分类]]之上构建强实层，提供按实形式与 Cartan 类查询 fiber 大小、按实形式查询 KGB 大小，以及查询全局 KGB 大小的接口。其汇总相关字段包括 `per_cartan`、`local_of_form`、`kgb_sizes` 和 `global_kgb_size`。^[strong-real.md:19-22, strong-real.md:49-58]

## fiber 大小与轨道数据

一个强实形式代表元对应某个平方类的 fiber group 中的一个 $W_{im}$ 轨道，并位于一个弱实形式之上。逐 Cartan 的 `StrongRealData` 保存平方类集合、各平方类的 fiber 轨道大小及强代表元，提供 `fiber_size(local)`、`orbit_elements(square, orbit)` 和 `weak_real_of_orbit` 等访问器。相关结构见 [[Cartan 类的强实层 StrongRealData]]与 [[强实形式与 fiber 轨道]]。^[strong-real.md:19-22, strong-real.md:34-47]

轨道编号与轨道大小需要区分：`fiber_orbit` 的编号依赖消元时选取的具体解，但轨道的 class size 不依赖该选择，因为 `ker(toAdjoint)` 的平移与作用交换。同样，平方类换基只会置换标签，不改变 partition 结构及各项大小；参见 [[平方类编号与换基不变量]]。^[strong-real.md:26-37]

## 按 Cartan 汇总与全局查询

`StrongRealClassification::fiber_size(form, cartan)` 查询指定实形式在指定 Cartan 类上的 fiber 大小。当该实形式不属于该 Cartan 类时，接口返回 `Some(0)`，因此遍历全部 Cartan 类求和时，该项贡献为零，汇总仍保持正确。^[strong-real.md:56-58]

`kgb_size(form)` 返回预计算的指定实形式 KGB 大小；`global_kgb_size()` 返回全局汇总值，来源将其描述为“所有强实形式的总和”。这两类查询与逐 Cartan 的 `fiber_size` 共同构成大小查询接口。^[strong-real.md:56-58]

## 构造与资源边界

`StrongRealClassification::build(classification, max_fiber_elements)` 逐 Cartan 构造强实数据：取得 grading 的 adjoint fiber 与 ambient fiber，构造 fiber map 的像坐标，并使用 `ModTwoSubquotient` 求平方商。当 fiber 维数超过 `MAX_MASK_BITS` 时，构造返回 `StrongRealResourceLimit`。相关边界见 [[强实分类的构造与资源边界]]。^[strong-real.md:19-20, strong-real.md:51-54]

## 证据范围

上述说明来自对 `strong_real.rs` 的结构性阅读，所读字节属于来源快照记录的 dirty 工作区。该来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；分类正确性仍属于其独立的 [[HPC 验收证据链]]。^[strong-real.md:9-15, strong-real.md:77-77]

## Sources

- [strong-real.md](strong-real.md)：强实形式分类：平方类编号与 fiber 轨道。
