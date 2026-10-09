---
title: KGB 种子代表元的可观测影响
summary: stable_log 选定的 adapted-basis 代表元固定 g_rho_check，进而固定下游 torus_factor 的有理数值。
sources:
  - real-form-seed.md
kind: concept
createdAt: "2026-10-09T15:05:53.287Z"
updatedAt: "2026-10-09T22:43:28.382Z"
tags:
  - KGB
  - 代表元选择
aliases:
  - kgb-种子代表元的可观测影响
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: KGB 种子代表元的可观测影响
summary: stable_log 选定的 adapted-basis 代表元固定 g_rho_check，进而固定下游每个 torus_factor 的精确有理数值。
sources:
  - real-form-seed.md
kind: concept
tags:
  - KGB
  - 代表元选择
aliases:
  - kgb-种子代表元的可观测影响
provenanceState: extracted
---

# KGB 种子代表元的可观测影响

KGB 种子 $x_0$ 的代表元选举承载可观测量：`stable_log` 通过 adapted basis 选定的代表元固定 `g_rho_check`，进而固定每一个下游 `torus_factor` 的精确有理数值。这一选择是种子构造中影响下游数值的具体约定。^[real-form-seed.md:19-22]

## 稳定对数如何选定代表元

[[stable_log：选举的稳定对数|stable_log]] 构造选定的 $\xi^T$-稳定对数：先将输入逐坐标模 $1$ 归约为非负剩余，再取 $\xi+1$ 的前 $d$ 个 adapted-basis 坐标；将这些坐标再次模 $1$，即模整个不动格归约，最后转换回原坐标。输出恰好位于 $+1$ 特征空间。上述[[承载可观测量的适配基（adapted_basis）|适配基]]代表元选择固定了后续使用的有理数值。^[real-form-seed.md:19-30]

该过程检查的前置条件是：归约后输入的尾部 adapted-basis 坐标必须为整数，等价于输入模 $X_*$ 同余于一个严格不动的向量。`some_coch` 输入在结构上满足这一条件，一般的 squares 则不一定满足。^[real-form-seed.md:29-31]

## 精确坐标与种子封装

种子构造分为两步：先实现 `stable_log` 与[[基本余权的精确构造|基本余权]]的精确有理数机制，再实现 `RealFormSeed` 构建器。基本余权以完整格秩坐标表示，满足 $\varpi_i^\vee=\sum_j(C^{-1})_{ji}\alpha_j^\vee$ 及 $\langle\alpha_s,\varpi_i^\vee\rangle=\delta_{si}$，且 radical 分量为零。实现通过精确有理高斯消元求 $C^{-1}$，再将各列按实际单余根展开，而非按恒等填充的坐标轴展开。^[real-form-seed.md:19-22, real-form-seed.md:33-40]

[[RealFormSeed 的封装与构造不变量|RealFormSeed]] 封装选定的 base-grading offset、精确的 square-class cocharacter，以及在 fundamental involution 处归约的种子元素。三者使用同一共享表的编号，调用方为每个 inner class 保持一个仅追加的表。字段保持私有，以避免公开组装出不匹配的三元组；构造不变量为 `grading_offset == grading_of_simples(cocharacter)`，种子验证采用上游的 x0-compacts 断言。^[real-form-seed.md:49-54]

## 构造与自定义路径的约束

[[RealFormSeed 的构建门控与自定义种子|构建门控]]先检查表的 inner class 相等，再检查 classification 的 fundamental class 是否归一化为指定 datum 与 delta 上的恒等 twisted involution，逐一比较 Weyl action 与根 involution 矩阵；随后处理强层计数一致性降级与 form id 越界检查。^[real-form-seed.md:56-59]

`custom` 路径接受显式的 `(cocharacter, torus part)` 对。cocharacter 与简单根的配对须为整数，torus part 须复现该 form 的 compact pattern。^[real-form-seed.md:59-62]

## 证据边界

本页依据对 `real_form_seed.rs` 的结构性阅读，所读源码字节来自 dirty 工作区，并记录在 `snapshots/2026-10-03-real-form-seed.json` 中。种子构造的正确性属于其自身的 HPC 证据链，包括 fundamental-lattice 与 seed gate 等；来源包不重述或扩展这些证据。^[real-form-seed.md:9-15]

来源包未执行构建、测试或原版运行，不包含数学验收、性能或并行结论。上游行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[real-form-seed.md:72-79]

## Sources

- [real-form-seed.md](../../sources/real-form-seed.md) — KGB 种子 x0：stable_log、基本余权与 RealFormSeed。
