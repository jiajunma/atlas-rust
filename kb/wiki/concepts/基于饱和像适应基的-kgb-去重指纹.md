---
title: 基于饱和像适应基的 KGB 去重指纹
summary: fingerprint 将 log_2pi 分子投影到 θ+I 饱和像的适应基并对分母取模，与对合编号共同构成去重键；无损性论据来自源码注释。
sources:
  - global-kgb.md
kind: concept
createdAt: "2026-10-09T14:49:05.847Z"
updatedAt: "2026-10-09T22:30:28.707Z"
tags:
  - KGB
  - 整数格
  - 去重
aliases:
  - 基于饱和像适应基的-kgb-去重指纹
  - 基K去
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 基于饱和像适应基的 KGB 去重指纹
summary: fingerprint 将环面元素的 log_2pi 分子投影到 θ+I 饱和像的适应基并对分母取余，用于基本纤维去重及按包推进的闭包构造；换基无损性的依据来自源码注释。
sources:
  - global-kgb.md
kind: concept
tags:
  - KGB
  - 整数格
  - 去重
aliases:
  - 基于饱和像适应基的-kgb-去重指纹
---

# 基于饱和像适应基的 KGB 去重指纹

`fingerprint` 是 `GlobalKgb` 构造中的去重指纹：将环面元素的 `log_2pi` 分子投影到 \(\theta+I\) 饱和像的适应基，对分母取余，并仅保留 `diagonal.len()` 个分量。它用于基本纤维种子的互异性检查，以及按 tau 包推进的 cross/Cayley 闭包构造。^[global-kgb.md:33-36, global-kgb.md:63-74]

## 计算方式

设 `log_2pi` 返回有理向量 \(n/d\)，适应基矩阵为 \(B\)，令 \(r=\texttt{diagonal.len()}\)。实现将分子与适应基各列配对，再以 `rem_euclid` 对分母取余，可写为 \(f_B(n/d)_i=(\sum_j n_jB_{ji})\operatorname{rem\_euclid}d\)，其中 \(0\le i<r\)。相关基构造见 [[承载可观测量的适配基（adapted_basis）]]。^[global-kgb.md:33-36]

来源指出，预算将格条目限制在 64 位范围，因此此处使用的 `i128` 累加不会溢出；这一说明适用于指纹投影的计算边界。^[global-kgb.md:33-36]

## 与环面表示的关系

环面元素保留算术历史：构造入口将坐标约化，但 `simple_reflect` 后故意不再约化，因此可出现负分子。`log_2pi` 返回原分子除以两倍原分母，并仅作 gcd 归一化；指纹计算随后对投影结果取欧几里得余数。两层表示的关系见 [[全局环面元素的算术历史表示]]。^[global-kgb.md:19-29, global-kgb.md:33-34]

## 换基无损性的依据

源码注释给出的依据是：生成同一饱和像的两组基相差幺模左因子，该因子在模 \(d\) 下可逆。因此，换基保留投影余数的判别信息；这里的无损性不要求不同基下的指纹分量逐项相同。该依据属于源码注释中的论证。^[global-kgb.md:33-36]

## 构造中的去重约束

在 [[基本纤维与平方类播种]] 阶段，平方类子集给出基本余权位移，再叠加纤维群的 lift。全部种子进入包 0，以 `(identity_id, fingerprint)` 为键去重；冲突触发 `"fundamental fiber distinctness"`，表示基本纤维种子互异性检查失败。^[global-kgb.md:63-66]

在 cross/Cayley 闭包阶段，新指纹只允许落入当前正在开启的新包，否则触发 `"cross image inside closed packet"`。同阶段还要求 cross 两端的 Cartan 类号一致；Cayley 链接仅对 `ImaginaryNoncompact` 元素建立，并原样克隆环面部分。完整流程见 [[GlobalKgb 的分阶段广度优先构造]]。^[global-kgb.md:67-74]

## 证据与覆盖边界

来源列出 SC A1、adjoint A1、SC B2 的逐字节打印测试，以及 B2 元素数、包大小、包字、cross 对合性和 Cayley 配对的结构检查。列出的四个测试中没有独立的指纹换基测试，也没有错误分支测试；半单秩 0 因共享内类机制在空生成元集合上 panic 而有意未测。参见 [[GlobalKgb 的回归测试与证据边界]]。^[global-kgb.md:94-105]

本页依据结构性源码阅读材料。来源中的上游行号转录自代码注释，未核对上游字节，不声称数学验收；本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[global-kgb.md:9-15, global-kgb.md:109-113]

## Sources

- [global-kgb.md](../../sources/global-kgb.md) — 内类范围 KGB 图与 print_X 布局（global_kgb.rs）。
