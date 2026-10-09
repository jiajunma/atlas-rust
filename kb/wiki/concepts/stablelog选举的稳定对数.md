---
title: stable_log：选举的稳定对数
summary: 通过非负 mod 1 归约与 adapted-basis 坐标选举，构造恰好位于 ξᵀ 的 +1 特征空间的对数；要求归约后尾部 adapted-basis 坐标为整数。
sources:
  - real-form-seed.md
kind: concept
createdAt: "2026-10-09T15:05:38.808Z"
updatedAt: "2026-10-09T15:05:38.808Z"
tags:
  - 稳定对数
  - 精确有理数
  - 格理论
aliases:
  - stablelog选举的稳定对数
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# stable_log：选举的稳定对数

`stable_log` 为 KGB 种子 \(x_0\) 的构造选取一个精确落在 \(+1\) 特征空间中的 \(\xi^T\)-稳定对数。这里的代表元选举承载可观测量：adapted-basis 代表元固定 `g_rho_check`，进而固定每一个下游 `torus_factor` 的有理数值。^[real-form-seed.md:19-31]

## 计算过程

源材料转述的上游 `stable_log` 算法使用 \(\xi+1\) 的 adapted basis，按以下顺序归约和重建坐标。^[real-form-seed.md:26-28]

1. 对输入的每个坐标模 \(1\) 归约，取非负剩余。
2. 取 \(\xi+1\) 的前 \(d\) 个 adapted-basis 坐标。
3. 对这些坐标再次模 \(1\) 归约，即模整个 fixed lattice（不动格）。
4. 转换回原坐标。

输出恰好位于 \(+1\) 特征空间。第二次模 \(1\) 归约是在不动格坐标中进行的，是代表元选举的一部分。相关基选择可参见 [[承载可观测量的适配基（adapted_basis）]]。^[real-form-seed.md:26-31]

## 输入前置条件

算法检查的前置条件是：逐坐标归约后，输入的尾部 adapted-basis 坐标必须为整数。这等价于输入模余特征格 \(X_*\) 同余于一个被对合精确固定的向量。`some_coch` 输入在结构上满足这一条件，一般的 squares 则不一定满足。^[real-form-seed.md:29-31]

## 在种子构造中的作用

种子构造分两步落地：先实现 `stable_log` 与 [[基本余权的精确构造]] 所需的精确有理数机制，再实现 `RealFormSeed` 构建器。由于所选对数代表元影响下游环面因子的具体数值，这一选择属于构造的可观测行为。^[real-form-seed.md:19-22]

`RealFormSeed` 将选举的 base-grading offset、精确的 square-class cocharacter，以及在 fundamental involution 处归约的种子元素组合在同一共享表的编号下。其封装与构造约束见 [[RealFormSeed 的封装与构造不变量]]。^[real-form-seed.md:47-54]

## 证据范围

本说明依据对 `real_form_seed.rs` 的结构性阅读；所读字节来自 dirty 工作区，并由来源包中的快照记录。种子构造的正确性另属 fundamental-lattice、seed gate 等 HPC 证据链，本来源包不重述或扩展这些结论。^[real-form-seed.md:9-15]

上游 `y_values.cpp:155-166` 的定位来自源码注释转述，未独立重读上游，行号可能随版本变化。本来源包未执行构建、测试或原版运行，因此不提供数学验收、性能或并行结论。^[real-form-seed.md:26-31, real-form-seed.md:72-79]

## Sources

- [real-form-seed.md](real-form-seed.md) — KGB 种子 x0：stable_log、基本余权与 RealFormSeed。
