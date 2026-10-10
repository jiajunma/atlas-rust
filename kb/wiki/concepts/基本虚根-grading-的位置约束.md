---
title: 基本虚根 grading 的位置约束
summary: 在基本虚单根基的实际位置匹配 datum 单根的目标紧致性约束，与上游前导段配对的一致性仅有条件性的注释声明支持。
sources:
  - minimal-torus.md
kind: concept
createdAt: "2026-10-09T15:03:24.587Z"
updatedAt: "2026-10-10T00:43:34.363Z"
tags:
  - grading
  - 根编号
  - 兼容性
aliases:
  - 基本虚根-grading-的位置约束
  - 基G的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 基本虚根 grading 的位置约束
summary: minimal_torus_part 按基本虚单根基的实际位置施加 datum-单根约束；与上游前导段配对的一致性依赖排序不变量，仅有源码注释声明支持。
sources:
  - minimal-torus.md
kind: concept
tags:
  - grading
  - 根编号
  - 兼容性
---

# 基本虚根 grading 的位置约束

基本虚根 grading 的位置约束，是 `minimal_torus_part` 在基本纤维的虚单根基上建立的目标匹配条件。算法仅在基根属于 datum-单根的位置施加约束，从 grading 轨道中选出符合目标模式的最小环面部分，使其 datum-单根紧致性模式复现目标弱实形。^[minimal-torus.md:22-25, minimal-torus.md:63-76]

## 基与位的含义

算法先将[[强代表下降到基本纤维]]，约化环面部分，再构造余权 $\mathrm{coweight}=\mathrm{coch}+\mathrm{lift}(\mathrm{tp})$，其中 `lift(tp)` 在环面部分的置位坐标加 1。目标模式建立在基本纤维 `CartanId(0)` 的虚单根基上。^[minimal-torus.md:53-64]

`start` 由各虚单根与 `coweight` 的配对是否为偶数确定，置位表示紧致；`constrained` 标记该位置的基根是否为 datum-单根；`target` 在受约束位置记录目标实形的**非紧致性**，通过翻转适配 crate 的 `Grading` 存储约定。解释这些位时须保留各自的紧致性语义，相关背景见[[Grading 的位向量类型纪律]]。^[minimal-torus.md:63-67]

## 逐位置配对与上游顺序

源包记录了一处有意的移植差异：上游把目标 grading 切片的位与基本虚单根基中**前导的 datum-单根条目**配对，依赖根编号将 datum-单根排在最前。Rust crate 的虚根基采用自己的确定性根序，因此改为**逐位置配对**，在实际属于 datum-单根的位置建立对应约束。^[minimal-torus.md:68-71]

源码注释声明，上游前导段不变量成立时，特别是在平凡 distinguished 对合下，两种方式一致。这是有条件的注释声明；源包未独立验证，不能将其视为任意根序或任意 distinguished 对合下的等价性验收。^[minimal-torus.md:68-71, minimal-torus.md:95-98]

## 轨道中的约束筛选

轨道变换使用各基余根的奇偶向量 `m_alpha`，以及基间配对的模二矩阵 `grading_shift`。搜索采用 LIFO 栈与 `BTreeSet<u64>` 去重；在每个置位的 grading 位上，以 `m_alpha[i]` 平移环面部分，并按 `grading_shift[i]` 翻转 grading。只有受约束位置与目标一致的状态才进入候选集合。^[minimal-torus.md:67-74]

最终按 `ModTwoVector` 顺序选取最小候选；对等维位向量，该顺序就是整数序。候选为空时，Rust 返回具名错误 `"minimal torus part candidates"`，对应上游的断言。完整流程见[[最小环面部分的 grading 轨道搜索]]。^[minimal-torus.md:72-76]

## 证据边界

现有四个测试锚点均为 rank 2、紧致内类，未覆盖非紧致 distinguished 情形。三个正例均满足 `coch == factor`，未通过可区分的断言刻画非平凡运输；第三个测试与第二个测试的第一组输入完全相同，其独立价值仅在注释叙述中。测试范围详见[[最小环面算法的测试覆盖边界]]。^[minimal-torus.md:78-98]

源包属于结构性源码阅读，不声称数学验收；上游引用及逐位置配对的一致性说明来自代码注释，未核对上游字节。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[minimal-torus.md:9-13, minimal-torus.md:95-98, minimal-torus.md:104-108]

## Sources

- [minimal-torus.md](../../sources/minimal-torus.md) — 合成实形的选定余特征与初始环面部分（minimal_torus.rs）。
