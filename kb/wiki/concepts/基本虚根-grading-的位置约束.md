---
title: 基本虚根 grading 的位置约束
summary: 在基本虚单根基上按实际位置匹配 datum 单根的目标紧致性约束，与上游前导段配对一致的条件仅有注释声明支持。
sources:
  - minimal-torus.md
kind: concept
createdAt: "2026-10-09T15:03:24.587Z"
updatedAt: "2026-10-09T21:02:37.470Z"
tags:
  - 虚根
  - 分级
  - 编号约定
aliases:
  - 基本虚根-grading-的位置约束
  - 基G的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 基本虚根 grading 的位置约束
summary: minimal_torus_part 在基本虚单根基中实际属于 datum-单根的位置施加目标约束；与上游前导段配对的一致性依赖排序不变量，仍属未经独立验收的注释声明。
sources:
  - minimal-torus.md
kind: concept
tags:
  - 虚根
  - grading
  - 基线对齐
---

# 基本虚根 grading 的位置约束

基本虚根 grading 的位置约束，是 `minimal_torus_part` 在基本纤维的虚单根基上建立的目标匹配条件：只在基根同时属于 datum-单根的位置施加约束，再从 grading 轨道中选取满足条件的最小环面部分，使其 datum-单根紧致性模式复现目标弱实形。^[minimal-torus.md:22-25, minimal-torus.md:63-76]

## 基与位的含义

算法先将强代表下降到基本纤维，并对环面部分作约化，再构造余权 \(\mathrm{coweight}=\mathrm{coch}+\mathrm{lift}(\mathrm{tp})\)，其中 `lift(tp)` 在环面部分的置位坐标加 1。目标模式建立在基本纤维 `CartanId(0)` 的虚单根基上；下降过程见 [[强代表下降到基本纤维]]。^[minimal-torus.md:53-64]

`start` 由各虚单根与 `coweight` 的偶配对确定，置位表示紧致；`constrained` 标记各位置的基根是否为 datum-单根；`target` 在受约束位置记录目标实形的非紧致性，并通过翻转适配 crate 的 `Grading` 存储约定。这些位的语义需要分别保留，参见 [[Grading 的位向量类型纪律]]。^[minimal-torus.md:63-67]

## 逐位置配对与上游顺序

源包记录了一处有意的移植差异：上游把目标 grading 切片的位与基本单虚根基中**前导的 datum-单根条目**配对，依赖根编号将 datum-单根排在最前。Rust crate 的虚根基采用自己的确定性根序，因此改为**逐位置配对**，在实际属于 datum-单根的位置建立对应约束。^[minimal-torus.md:68-71]

源码注释声明，上游前导段不变量成立时，特别是在平凡 distinguished 对合下，两种方式一致。源包未独立验证这一声明，因此不能将其表述为任意根序或任意 distinguished 对合下已经确认的等价性。^[minimal-torus.md:68-71, minimal-torus.md:95-98]

## 约束如何筛选轨道状态

轨道变换使用各基余根的奇偶向量 `m_alpha` 和基间配对的模二矩阵 `grading_shift`。搜索采用 LIFO 栈与 `BTreeSet<u64>` 去重；在每个置位的 grading 位上，以 `m_alpha[i]` 平移环面部分，并按 `grading_shift[i]` 翻转 grading。只有受约束位置与目标一致的状态才进入候选集合。^[minimal-torus.md:67-74]

最终选举按 `ModTwoVector` 的顺序取最小候选；对等维位向量，该顺序就是整数序。候选为空时，Rust 返回具名错误 `"minimal torus part candidates"`，对应上游的断言。完整搜索流程见 [[最小环面部分的 grading 轨道搜索]]。^[minimal-torus.md:72-76]

## 证据边界

现有四个测试锚点均为 rank 2、紧致内类，未覆盖非紧致 distinguished 情形。三个正例都满足 `coch == factor`，未通过可区分的断言刻画非平凡运输；其中第三个测试与第二个测试的第一组输入完全相同。因此，现有测试不能完整验证位置配对的一般一致性，参见 [[最小环面算法的测试覆盖边界]]。^[minimal-torus.md:78-98]

源包属于结构性源码阅读，不声称数学验收；上游位置仅转录自代码注释，未核对上游字节。本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[minimal-torus.md:9-13, minimal-torus.md:104-108]

## Sources

- [minimal-torus.md](../../sources/minimal-torus.md) — 合成实形的选定余特征与初始环面部分（minimal_torus.rs）
