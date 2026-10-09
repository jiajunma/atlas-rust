---
title: 基本虚根 grading 的位置约束
summary: 算法在基本虚单根基上构造紧致性起始位与 datum-单根目标约束，按位置匹配 crate 的根序；与上游前导段配对的一致性依赖其排序不变量，尚属注释声明。
sources:
  - minimal-torus.md
kind: concept
createdAt: "2026-10-09T15:03:24.587Z"
updatedAt: "2026-10-09T15:03:24.587Z"
tags:
  - 虚根
  - Grading
  - 基线对齐
aliases:
  - 基本虚根-grading-的位置约束
  - 基G的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 基本虚根 grading 的位置约束

基本虚根 grading 的位置约束，是 `minimal_torus_part` 在基本纤维的虚单根基上逐位置建立的匹配条件：只在基根同时属于 datum-单根的位置约束目标模式，再从 grading 轨道中选取满足条件的环面部分。该流程用于使所选环面部分的 datum-单根紧致性模式复现目标弱实形。^[minimal-torus.md:22-25, minimal-torus.md:63-76]

## 基与位的含义

算法先将强代表下降到基本纤维，并由约化后的环面部分构造余权
\( \mathrm{coweight}=\mathrm{coch}+\mathrm{lift}(\mathrm{tp}) \)，其中 `lift(tp)` 在环面部分的置位坐标加 1。随后在基本纤维 `CartanId(0)` 的虚单根基上建立 grading 数据。相关下降过程见 [[强代表下降到基本纤维]]。^[minimal-torus.md:53-64]

`start` 记录各虚单根与 `coweight` 的配对偶性，置位表示紧致；`constrained` 标记该位置的基根是否为 datum-单根；`target` 则在受约束位置取目标实形的非紧致性信息，并通过翻转适配 crate 的 `Grading` 存储约定。因此，解读这些位时必须保留各自的语义，不能直接把所有置位都理解成同一种紧致性标记。参见 [[Grading 的位向量类型纪律]]。^[minimal-torus.md:63-67]

## 逐位置配对与上游前导段约定

源包记录了一处有意的翻译差异：上游把目标 grading 切片的位与基本单虚根基中**前导的 datum-单根条目**配对，依赖其根编号将 datum-单根排在最前。Rust crate 的虚根基采用自己的确定性根序，因此改为**逐位置配对**，在实际属于 datum-单根的位置施加对应约束。^[minimal-torus.md:68-71]

源码注释声明，当上游的前导段不变量成立时，特别是在平凡 distinguished 对合下，两种配对方式一致。该一致性在源包中属于注释声明，未被独立验收，不应推广为任意根序或任意 distinguished 对合下已经验证的等价性。^[minimal-torus.md:68-71, minimal-torus.md:95-98]

## 约束在轨道搜索中的作用

轨道搜索以 LIFO 栈遍历状态，并用 `BTreeSet<u64>` 去重。每个置位的 grading 位都可触发一步变换：以对应基余根的奇偶向量 `m_alpha[i]` 平移环面部分，再按基间配对模二矩阵的 `grading_shift[i]` 翻转 grading。受约束位置与目标一致的状态才进入候选集合。^[minimal-torus.md:67-74]

算法最终按 `ModTwoVector` 的顺序选取最小候选；对等维位向量，这就是整数序。若候选为空，Rust 实现返回具名错误 `"minimal torus part candidates"`，而上游此处使用断言。位置约束由此直接决定哪些轨道状态可以参与最终选举，详见 [[最小环面部分的 grading 轨道搜索]]。^[minimal-torus.md:72-76]

## 证据边界

现有四个测试锚点均为 rank 2、紧致内类，未覆盖非紧致 distinguished 情形；三个正例都满足 `coch == factor`，也没有通过可区分的断言刻画非平凡运输。因此，这些测试不能为上述位置配对的一般一致性提供完整验证。源包本身仅报告结构性阅读，不声称数学验收，也未核对所引用的上游源码字节。^[minimal-torus.md:10-13, minimal-torus.md:78-98]

## Sources

- [minimal-torus.md](minimal-torus.md) — 合成实形的选定余特征与初始环面部分（minimal_torus.rs）
