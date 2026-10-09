---
title: 弱实形式的伴随 Cartan 纤维轨道划分
summary: WeakRealFormPartition 将伴随 Cartan fiber 划分为 W_im 轨道并保存类表与确定性代表元，实形标签及强实层由其他结构承担。
sources:
  - weak-real-form.md
kind: concept
createdAt: "2026-10-09T15:15:25.818Z"
updatedAt: "2026-10-09T22:52:19.024Z"
tags:
  - 弱实形式
  - Cartan纤维
  - 轨道划分
aliases:
  - 弱实形式的伴随-cartan-纤维轨道划分
  - 弱C纤
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 弱实形式的伴随 Cartan 纤维轨道划分
summary: WeakRealFormPartition 将伴随 Cartan 纤维划分为 W_im 轨道，以典范坐标最小元确定代表元与编号，并通过 Cartan 标签衔接全局弱实形式编号。
sources:
  - weak-real-form.md
kind: concept
tags:
  - 弱实形式
  - Cartan纤维
  - 轨道划分
aliases:
  - 弱实形式的伴随-cartan-纤维轨道划分
  - 弱C纤
provenanceState: extracted
---

# 弱实形式的伴随 Cartan 纤维轨道划分

`WeakRealFormPartition` 将一个伴随 Cartan 纤维（adjoint Cartan fiber）划分为 $W_{im}$ 轨道，每个轨道对应内类在该 Cartan 对合处的一个弱实形式。划分保存类表及每类的确定性代表元；实形标签由 `RealFormLabels` 保存，平方类与强实层由其他结构承担，参见[[强实形式与 fiber 轨道]]。^[weak-real-form.md:20-23]

## 编号与代表元

`WeakRealFormId` 按各轨道在典范坐标整数序下的最小元升序分配，该最小元同时作为轨道代表元。类 0 是单位元素所在的轨道，对应 quasisplit 规范化；由基本 Cartan（fundamental Cartan）的划分产生的编号，兼任 crate 的全局实形编号。^[weak-real-form.md:27-32, weak-real-form.md:49-54]

来源记录的 stage-(d) 排序审计确认，该编号与上游内部 `RealFormNbr` 一致：双方使用相同的升序轨道播种、低主元 RREF 子商基和坐标提取方式。只有解释器外部的 `FormNumberMap` 顺序需要适配器置换，详见[[WeakRealFormId 的确定性编号与上游对齐]]。^[weak-real-form.md:27-32]

## 掩码轨道枚举

`walk_mask_orbits` 以整数掩码表示纤维坐标，按掩码升序播种，使用 LIFO 栈遍历轨道。生成元 `i` 的转移遵循 `FiberAction`：若 `base[i] XOR parity(mask ∧ alpha_columns[i])` 判定为非紧，则以 `m_alpha_masks[i]` 作平移。每个轨道首次出现的掩码就是其最小代表元，类号据此依次分配，参见[[基于位掩码的 W_im 轨道枚举]]。^[weak-real-form.md:37-43]

类表使用 `Vec<u32>`，保留 `u32::MAX` 作为 `CLASS_SENTINEL`。生成元对合性依赖 $\langle\alpha,\alpha^\vee\rangle=2$ 的模二关系，实现仅用 `debug_assert` 检查这一性质。^[weak-real-form.md:40-43]

## 构建与资源边界

`WeakRealFormPartition::build(grading, max_elements)` 对已校验 grading 表背后的伴随纤维构建划分。调用方通过 `max_elements` 限制枚举规模 $2^{\mathrm{dimension}}$；掩码使用 `u64`，另设 `MAX_MASK_BITS = 63`，确保所需移位保持在范围内。^[weak-real-form.md:34-35, weak-real-form.md:49-51]

枚举规模以 `u128` 计算，再与加宽后的 `max_elements` 比较。这里故意不使用饱和计算，防止 `dimension == 64` 时借 `usize::MAX` 上限漏过规模检查。^[weak-real-form.md:37-39]

类号具有独立的容量守卫：新类序号无法装入 `u32`，或等于保留哨兵时，`seeded_class` 返回 `limit_error("classes", …)`。在没有伴随 `m_alpha`、每个掩码自成一类的单连通积情形下，该守卫原则上可达。^[weak-real-form.md:43-45]

## 查询接口

`class_count` 返回类数，`classes` 提供升序 `ExactSizeIterator`；`class_of` 与 `class_of_mask` 查询类归属，典范坐标掩码查询支撑语言级 `fiber_partition`。`class_representative` 返回该类的典范坐标最小元，`quasisplit_class` 返回类 0，`adjoint_fiber` 提供底层伴随纤维。^[weak-real-form.md:49-54]

## 从局部轨道到全局编号

[[代表元级弱实形式归因]]内核 `weak_real_form_at_representative` 按 Atlas 的行向量右乘约定，先施加对偶不动点投影 $v\mapsto(v+v\theta)/2$。投影值的整数配对使平方中心化，偶数配对标记非紧单虚根；所得 grading 确定局部伴随纤维轨道，再由该 Cartan 的标签映射到基本 Cartan 的弱实形式编号。^[weak-real-form.md:58-63]

该内核要求 `twisted` 恰为分类中存储的某个代表元。将一般 twisted involution 移到代表元时，还须通过基于表的 Tits cross actions 同时搬运环面因子；此 helper 不会隐式构建或扩展该表。后续 `minimal_torus_part` 下降还需要目前仍分离的 inverse-Cayley 操作，参见[[代表元归因与 Tits 搬运的职责边界]]。^[weak-real-form.md:65-72]

归因依次检查原始因子长度、分类来源、`twisted` 的 datum、$w\delta=\theta$ 分解及代表元归属，然后进行投影。整性门先于虚根 grading 提取，对每个单根要求整值配对，失败时报 `InvalidStrongTorusFactor`；因此虚基为空的实 Cartan 也不能绕过检查。随后通过 grading、`element_from_grading`、局部 `class_of` 和 `labels().label(local)` 得到全局编号。^[weak-real-form.md:74-82]

## 测试锚点与证据边界

来源列出的划分测试包括 A2 恒等对合的 2 类、B2 恒等对合的 3 类及精确代表元、单连通 A1 平凡作用的 2 个单元类，以及 A2 图扭转的 1 类。另有外来元素、预算不足和 `seeded_class` 哨兵守卫测试；rank-33 因预算被拒绝，rank-64 因掩码位数限制被拒绝，体现两种不同的资源边界。^[weak-real-form.md:84-89]

尚未覆盖的分支包括 `class_of_mask` 越界、各 `ArithmeticOverflow` 转换分支，以及 `max_elements == 2^dimension` 的显式边界断言，详见[[弱实形式划分与归因的测试覆盖边界]]。^[weak-real-form.md:89-91]

来源属于两次结构性源码阅读，两次所读字节的 SHA-256 相同。该来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；正确性仍属于独立的 Cartan／seed gate 等 HPC 证据链。上游行号转述自源码注释，未独立重读上游，可能随版本演进漂移。^[weak-real-form.md:9-16, weak-real-form.md:101-106]

## Sources

- [weak-real-form.md](../../sources/weak-real-form.md) — 弱实形式划分：adjoint fiber 的 W_im 轨道。
