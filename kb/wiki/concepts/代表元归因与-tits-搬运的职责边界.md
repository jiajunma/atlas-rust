---
title: 代表元归因与 Tits 搬运的职责边界
summary: 归因 helper 仅接受分类存储的代表元，一般扭对合需通过表驱动的 Tits cross actions 同步搬运环面因子，后续下降另依赖 inverse-Cayley 操作。
sources:
  - weak-real-form.md
kind: concept
createdAt: "2026-10-09T15:15:58.723Z"
updatedAt: "2026-10-09T21:13:18.214Z"
tags:
  - Tits作用
  - 代表元归因
  - 架构边界
aliases:
  - 代表元归因与-tits-搬运的职责边界
  - 代T搬
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 代表元归因与 Tits 搬运的职责边界
summary: 代表元归因仅接受 Cartan 分类已存储的代表元；一般 twisted involution 的归约须通过 table-backed Tits cross actions 同步搬运环面因子，后续最小环面下降仍依赖分离的 inverse-Cayley 操作。
sources:
  - weak-real-form.md
kind: concept
tags:
  - Tits作用
  - Cartan分类
  - 系统设计
---

# 代表元归因与 Tits 搬运的职责边界

`weak_real_form_at_representative` 是弱实形式的代表元级归因内核：输入 `twisted` 必须恰好是 `CartanClassification` 已存储的某个代表元。函数根据环面因子确定局部 grading，再通过该 Cartan 的标签取得全局弱实形式编号；将一般 twisted involution 搬运到代表元位置属于独立职责。^[weak-real-form.md:58-72, weak-real-form.md:80-82]

## 代表元归因的计算路径

归因对环面因子施加 dual fixed-point projection，采用 Atlas 的行向量、右乘约定，即 \(v\mapsto (v+v\theta)/2\)。投影值的整数配对使平方中心化；虚单根上的偶数配对标记 noncompact，由此确定 grading。^[weak-real-form.md:58-63]

整性检查先于虚根 grading 提取，并覆盖**每个单根**；遇到非整值配对时报 `InvalidStrongTorusFactor`。因此，即使实 Cartan 的虚根基为空，也不能绕过该检查。随后依次调用 `element_from_grading`、局部 `class_of` 和 `labels().label(local)`，完成[[弱实形式的局部到全局标签映射]]。^[weak-real-form.md:74-82]

局部类别来自 adjoint Cartan fiber 的 \(W_{im}\) 轨道划分，每条轨道对应该 Cartan involution 处的一个弱实形式。划分保存类表和确定性代表元，实形标签则保存在 `RealFormLabels` 中；详见[[弱实形式的伴随 Cartan 纤维轨道划分]]。^[weak-real-form.md:20-23]

## Tits 搬运与后续下降

一般 twisted involution 移到分类代表元时，必须通过 table-backed Tits cross actions **同步搬运 torus factor**。归因 helper 不会隐式构建或扩展所需表，其前置条件始终是 `twisted` 已处于分类存储的代表元位置。^[weak-real-form.md:65-67]

后续 `minimal_torus_part` 的下降还需要目前仍分离的 inverse-Cayley 操作。这一依赖与代表元级归因的计算路径分别存在，相关主题见[[最小环面部分的 grading 轨道搜索]]。^[weak-real-form.md:65-72]

## 来源一致性与验证顺序

`CartanClassification` 不保留 `InnerClass` 句柄，因此归因函数先从分类的第一个代表元重构归一化 distinguished involution，作为显式来源一致性（provenance）门；不匹配时报 `DatumMismatch`。随后检查 `twisted` 的 datum 与 distinguished-involution 分解，分解不匹配时报 `DistinguishedInvolutionMismatch`。^[weak-real-form.md:65-71]

验证顺序为：原始环面因子长度等于格秩，否则报 `RankMismatch`；检查 provenance；检查 `twisted` 的 datum；验证 \(w\cdot\delta=\theta\)；查找存储代表元；执行投影；检查整性；最后提取 grading 并查询标签。分解门通过 `compose_matrices` 检查 weight 与 coweight 双矩阵，代表元查找先于投影。更多细节见[[弱实形式归因的来源与整性门控]]。^[weak-real-form.md:74-82]

## 测试锚点与证据范围

来源记录的相关测试锚点包括：合成 A1 的紧、分裂 Cartan 配环面因子 0 得类 0，紧 Cartan 配 \(1/2\) 得类 1；秩与半整投影门控；外来同秩分类优先报 `DatumMismatch`；以及 A2 反射的半整投影诊断。来源同时指出，各 `ArithmeticOverflow` 转换分支尚未触及。^[weak-real-form.md:84-91]

本说明依据两次字节未变的结构性源码阅读。来源包未执行构建、测试或原版运行，不扩展既有 HPC 正确性证据链，也不提供数学验收、性能或并行结论；上游行号转述自源码注释，未独立重读上游。^[weak-real-form.md:9-16, weak-real-form.md:101-106]

## Sources

- [weak-real-form.md](weak-real-form.md) — 弱实形式划分：adjoint fiber 的 \(W_{im}\) 轨道。
