---
title: 基于饱和像适应基的 KGB 去重指纹
summary: fingerprint 将 log_2pi 分子投影到 θ+I 饱和像的适应基并按分母取模，利用基变换的幺模性质保持判别信息，结合对合标识完成元素去重。
sources:
  - global-kgb.md
kind: concept
createdAt: "2026-10-09T14:49:05.847Z"
updatedAt: "2026-10-09T14:49:05.847Z"
tags:
  - 格理论
  - 元素去重
  - KGB图
aliases:
  - 基于饱和像适应基的-kgb-去重指纹
  - 基K去
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 基于饱和像适应基的 KGB 去重指纹

KGB 去重指纹由 `fingerprint` 计算：它将环面元素的 `log_2pi` 分子投影到对合 \(\theta\) 的 \(\theta+I\) 饱和像适应基，并对分母取余。该指纹用于 [[内类范围的 GlobalKgb 图]] 的基本纤维去重和 cross/Cayley 闭包构造。^[global-kgb.md:33-36, global-kgb.md:63-74]

## 指纹的计算

设 `log_2pi` 返回的有理向量为 \(n/d\)，适应基矩阵为 \(B\)，令 \(r=\texttt{diagonal.len()}\)。指纹保留前 \(r\) 个投影分量，可写为
\[
f_B(n/d)=
\left(
\left(\sum_j n_j B_{ji}\right)\operatorname{rem\_euclid}d
\right)_{i=0}^{r-1}.
\]
这里的基来自 \(\theta+I\) 的饱和像；实现只保留 `diagonal.len()` 个分量，并通过预算将格条目限制在 64 位范围，使 `i128` 累加不溢出。相关基构造见 [[承载可观测量的适配基（adapted_basis）]]。^[global-kgb.md:33-36]

环面元素本身采用“算术历史”表示，而非始终维持规范形：构造入口会约化坐标，`simple_reflect` 后却故意不再约化，因此可能出现负分子。`log_2pi` 返回原分子除以两倍原分母，并仅作 gcd 归一化；指纹计算随后对投影结果使用 `rem_euclid`。这一表示纪律与 [[全局环面元素的算术历史表示]] 直接相关。^[global-kgb.md:19-29, global-kgb.md:33-34]

## 换基为何不丢失信息

源码注释给出的无损性依据是：生成同一饱和像的两组基相差一个幺模左因子，而幺模因子在模 \(d\) 下可逆。因此，换用这样的基不会丢失投影余数所携带的信息；这里的依据是模 \(d\) 可逆性，而非不同基下指纹各分量逐项相同。^[global-kgb.md:33-36]

## 在构造流程中的作用

[[基本纤维与平方类播种]] 阶段先枚举平方类子集形成基本余权位移，再叠加纤维群的 lift。所有种子进入包 0，并以 `(identity_id, fingerprint)` 为键去重；若发生冲突，构造报出 `"fundamental fiber distinctness"`。因此，该阶段要求生成的基本纤维种子彼此可区分，冲突被视为不变量失败。^[global-kgb.md:63-66]

在 cross/Cayley 闭包阶段，新指纹只允许出现在当前正开启的新包中，否则触发 `"cross image inside closed packet"`。这一约束将去重与按包区间推进的 BFS 结合起来；同阶段还要求 cross 两端的 Cartan 类号一致，而 Cayley 链接仅为 `ImaginaryNoncompact` 元素建立，并原样克隆其环面部分。^[global-kgb.md:67-74]

## 证据与覆盖边界

来源记录了 SC A1、adjoint A1、SC B2 的逐字节打印匹配，以及 B2 的元素数、包大小、包字、cross 对合性和 Cayley 配对检查；未列出独立的指纹换基测试，且明确没有错误分支测试。半单秩 0 的情形因共享内类机制的 panic 而有意未测。^[global-kgb.md:94-105]

本页依据结构性源码阅读材料。来源未核对所引上游代码的字节，不声称数学验收；本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[global-kgb.md:9-15, global-kgb.md:109-113]

## Sources

- [global-kgb.md](../../sources/global-kgb.md) — 内类范围 KGB 图与 print_X 布局（global_kgb.rs）
