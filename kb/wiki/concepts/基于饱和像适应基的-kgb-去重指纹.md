---
title: 基于饱和像适应基的 KGB 去重指纹
summary: fingerprint 将 log_2pi 分子投影到 θ+I 饱和像的适应基并对分母取模，与对合编号共同去重；无损性依据来自源码注释。
sources:
  - global-kgb.md
kind: concept
createdAt: "2026-10-09T14:49:05.847Z"
updatedAt: "2026-10-10T00:33:29.382Z"
tags:
  - kgb
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
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
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

`fingerprint` 是 `GlobalKgb` 构造中的去重指纹：将环面元素的 `log_2pi` 分子投影到 $\theta+I$ 饱和像的适应基，对分母取欧几里得余数，并仅保留 `diagonal.len()` 个分量。它参与基本纤维种子的互异性检查及按 tau 包推进的 cross/Cayley 闭包构造。^[global-kgb.md:33-36, global-kgb.md:63-74]

## 计算方式与算术边界

设 `log_2pi` 返回有理向量 $n/d$，适应基矩阵为 $B$，保留的分量数为 $r=\texttt{diagonal.len()}$。将分子与适应基各列配对、再对分母取余的计算可写为 $f_B(n/d)_i=\operatorname{rem\_euclid}(\sum_j n_jB_{ji},d)$，其中 $0\le i<r$。相关基构造参见 [[承载可观测量的适配基（adapted_basis）]]。^[global-kgb.md:33-36]

来源指出，预算将格条目限制在 64 位范围，因此指纹计算中的 `i128` 累加不会溢出。这是该投影计算的算术边界说明。^[global-kgb.md:33-36]

## 与环面表示的关系

`GlobalTorusElement` 以分子向量和分母表示 $\exp(i\pi\,n/d)$，坐标按模 $2\mathbb Z^{\mathrm{rank}}$ 理解。构造入口约化坐标，但 `simple_reflect` 后故意不再约化，因此可能保留负分子。`log_2pi` 返回分子除以两倍原分母，并仅作 gcd 归一化；指纹随后对投影结果执行 `rem_euclid`。参见 [[全局环面元素的算术历史表示]]。^[global-kgb.md:19-29, global-kgb.md:33-34]

## 换基无损性的依据

源码注释给出的论据是：生成同一饱和像的两组基相差幺模左因子，该因子在模 $d$ 下可逆。因此，换基保留余数所携带的判别信息；这里的无损性不意味着不同基下的指纹分量逐项相同。这一依据属于源码注释中的论证。^[global-kgb.md:33-36]

## 构造中的去重约束

在 [[基本纤维与平方类播种]] 阶段，平方类子集给出基本余权位移，再经 `add_torus_part` 叠加纤维群的 lift。全部种子进入包 0，以 `(identity_id, fingerprint)` 为键去重；若发生冲突，则报 `"fundamental fiber distinctness"`，表示基本纤维种子的互异性检查失败。^[global-kgb.md:63-66]

在 cross/Cayley 闭包阶段，新指纹只允许落入当前正在开启的新包，否则触发 `"cross image inside closed packet"`。同阶段还检查 cross 两端的 Cartan 类号一致；Cayley 链接仅对 `ImaginaryNoncompact` 元素建立，其环面部分原样克隆。完整流程参见 [[GlobalKgb 的分阶段广度优先构造]]。^[global-kgb.md:67-74]

## 测试与证据边界

来源列出四个测试：SC A1、adjoint A1、SC B2 的逐字节打印匹配，以及 B2 的元素数、包大小、包字、cross 对合性和 Cayley 配对检查。该清单未列出独立的指纹换基测试，且明确没有错误分支测试。半单秩 0 因共享内类机制在空生成元集合上 panic 而有意未测。参见 [[GlobalKgb 的回归测试与证据边界]]。^[global-kgb.md:94-105]

本页依据结构性源码阅读材料。来源中的上游行号仅转录自代码注释，未核对上游字节，不声称数学验收；该次知识维护也未执行 Atlas、Cargo、测试或 benchmark，因此上述测试描述不代表本次执行结果。^[global-kgb.md:9-15, global-kgb.md:109-113]

## Sources

- [global-kgb.md](../../sources/global-kgb.md) — 内类范围 KGB 图与 print_X 布局（global_kgb.rs）。
