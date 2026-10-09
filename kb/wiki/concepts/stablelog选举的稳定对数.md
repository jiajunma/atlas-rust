---
title: stable_log：选举的稳定对数
summary: stable_log 对输入及固定格的 adapted-basis 坐标取非负 mod 1，要求尾部坐标为整数，并选举恰好位于 ξᵀ 的 +1 特征空间的代表元。
sources:
  - real-form-seed.md
kind: concept
createdAt: "2026-10-09T15:05:38.808Z"
updatedAt: "2026-10-09T22:43:19.493Z"
tags:
  - 稳定对数
  - 精确算术
aliases:
  - stablelog选举的稳定对数
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: stable_log：选举的稳定对数
summary: 通过逐坐标及固定格的 adapted-basis 坐标模 1 归约，选取精确位于 ξᵀ 的 +1 特征空间的稳定对数；前提是归约后尾部 adapted-basis 坐标为整数。
sources:
  - real-form-seed.md
kind: concept
tags:
  - KGB种子
  - 稳定对数
  - 精确有理运算
aliases:
  - stablelog选举的稳定对数
provenanceState: extracted
---

# stable_log：选举的稳定对数

`stable_log` 为 KGB 种子 \(x_0\) 的构造选取一个精确位于 \(\xi^T\) 的 \(+1\) 特征空间中的稳定对数。其 adapted-basis 代表元选择承载可观测量：它固定 `g_rho_check`，进而固定每一个下游 `torus_factor` 的有理数值。^[real-form-seed.md:19-31]

## 计算过程

来源将上游 `stable_log` 定位于 `y_values.cpp:155-166`，计算过程依次包含以下四步。^[real-form-seed.md:26-28]

1. 将输入逐坐标模 \(1\) 归约，取非负剩余。
2. 取 \(\xi+1\) 的前 \(d\) 个 adapted-basis 坐标。
3. 将这些坐标再次模 \(1\) 归约，即模整个不动格（fixed lattice）。
4. 转换回原坐标。

输出恰好落在 \(+1\) 特征空间。两次模 \(1\) 归约分别作用于输入的原坐标和所选的不动格坐标；代表元选择与 [[承载可观测量的适配基（adapted_basis）]] 相关。^[real-form-seed.md:21-31]

## 输入前置条件

实现检查的前置条件是：逐坐标归约后，输入的尾部 adapted-basis 坐标必须为整数。这等价于输入模余特征格 \(X_*\) 同余于一个被对合精确固定的向量。`some_coch` 输入在结构上满足这一条件，一般的 squares 则不一定满足。^[real-form-seed.md:29-31]

## 在种子构造中的作用

KGB 种子构造分两步落地：先实现 `stable_log` 与 [[基本余权的精确构造]] 所需的精确有理数机制，再实现 `RealFormSeed` 构建器。由于选定代表元决定下游环面因子的具体有理数值，这一选择属于构造的可观测行为。^[real-form-seed.md:19-22]

`RealFormSeed` 在同一共享表的编号下，组合选举的 base-grading offset、精确的 square-class cocharacter，以及在 fundamental involution 处归约的种子元素。其字段私有，并保持 `grading_offset == grading_of_simples(cocharacter)`；相关约束见 [[RealFormSeed 的封装与构造不变量]]。^[real-form-seed.md:49-54]

## 证据范围

本页依据对 `real_form_seed.rs` 的结构性阅读。来源记录的字节来自 dirty 工作区，见其引用的 `2026-10-03-real-form-seed.json` 快照；种子构造的正确性另属 fundamental-lattice、seed gate 等 HPC 证据链，本来源包不重述或扩展这些结论。^[real-form-seed.md:9-15]

上游位置由源码注释转述，未独立重读上游，行号可能随版本演进而漂移。本来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。^[real-form-seed.md:69-79]

## Sources

- [real-form-seed.md](../../sources/real-form-seed.md) — KGB 种子 x0：stable_log、基本余权与 RealFormSeed。
