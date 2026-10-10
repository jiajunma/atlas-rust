---
title: 根 involution 的 InnerClass 构造与 Weyl 因子
summary: 构造在调用方根枚举预算内验证根与余根运输，再左复合 Weyl 字得到保持基的对合，并按入口返回或丢弃 Weyl 因子。
sources:
  - inner-class.md
kind: concept
createdAt: "2026-10-09T14:50:42.364Z"
updatedAt: "2026-10-10T00:34:54.734Z"
tags:
  - 根理论
  - 构造验证
aliases:
  - 根-involution-的-innerclass-构造与-weyl-因子
  - 根I的I构W因
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 根 involution 的 InnerClass 构造与 Weyl 因子
summary: InnerClass 在调用方根枚举预算内验证根与余根作用，通过左复合 Weyl 字得到保持基的对合，并按入口返回 Weyl 因子或丢弃该词。
sources:
  - inner-class.md
kind: concept
tags:
  - 内类
  - 根数据
  - 构造验证
aliases:
  - 根-involution-的-innerclass-构造与-weyl-因子
---

# 根 involution 的 InnerClass 构造与 Weyl 因子

`InnerClass` 的根对合构造入口接受不要求保持基的根数据对合，验证其置换根系并传送余根，再通过左复合 Weyl 作用将其调整为保持基的 distinguished involution。不同入口决定是否返回输入相对于所得 distinguished involution 的 Weyl 因子。^[inner-class.md:29-40]

## 构造状态与预算

`InnerClass` 拥有经过验证的 `BasedRootDatum`、有限常根系 `RootSystem` 和 distinguished `RootInvolutionData`。这是根理论层面的部分实现：支持 twisted-conjugacy 轨道枚举，为 `CayleyCrossDecomposition` 提供上下文，并为 `RealFormLabels` 锚定来源身份；尚未构建 Atlas Cartan-class fibers，不持有实形式数据，也不包含构造 KGB 图所需的环面数据。参见 [[InnerClass 的根理论状态与实现边界]]。^[inner-class.md:19-25]

`InnerClass::new` 构建共享的根理论状态，根枚举预算由调用方明确提供。成功结果保证 distinguished lattice involution 置换该根系并传送余根，但不构成 Atlas 实形式兼容性的保证。^[inner-class.md:32-35]

## 从根对合得到保持基的对合

`InnerClass::from_root_involution` 对应上游 `inner_class(RootDatum,mat)` 入口的 `check_involution` 流程。它接受不带基的根数据上的任意对合，先验证其置换根系并传送余根，再由 `wrt_distinguished` 从经过反射的单根像读出 Weyl 字。将该词对应的作用**左复合**到输入对合后，得到保持基的对合；此入口随后丢弃该 Weyl 字，与上游 wrapper 一致。^[inner-class.md:36-40]

需要同时取得 Weyl 因子时，使用 `inner_class_with_twisted_involution(datum, involution, root_budget)`。该函数委托 `InnerClass::from_root_involution_with_factor`，构造输入对合定义的内类，并同时返回输入相对于所得 distinguished involution 的 Weyl 因子。^[inner-class.md:29-31]

## 保持基的验证与生成元 twist

`based_involution_twist` 要求对合置换本类根系、传送余根，并将每个单根映到单根。验证失败返回 `StructureError::InvalidBasedAutomorphism`；成功时返回诱导的单根置换。相关验证门见 [[Based involution 验证与生成元 twist]]。^[inner-class.md:44-48]

`generator_twist()` 给出 distinguished involution 对简单生成元的置换。若生成元 `s` 对应单根 $\alpha_s$，则 `twist[s]` 是 distinguished involution 将 $\alpha_s$ 映到的单根所对应的生成元。^[inner-class.md:49-51]

## 已有内类中的 Weyl 分解

对于已有的 `InnerClass`，`twisted_from_involution` 验证输入是否属于该内类，并返回分解 $\theta=w\cdot\delta$ 中的 Weyl 元素 $w$，其中 $\delta$ 是该内类的 distinguished involution。调用方须已完成 square 与 involutive 检查；成员判定失败由 `StructureError::InvalidBasedAutomorphism` 承载。来源还说明，权格矩阵相等蕴含 twist 比较。参见 [[InnerClass 成员判定与 twisted 分解]]。^[inner-class.md:53-59]

## 证据范围

本页依据 `inner_class.rs` 的结构性阅读，所读字节来自记录了 SHA-256 的 dirty 工作区快照。来源包不重述或扩展 inner class 自身的 HPC 正确性证据链；上游对应关系与行号转述自源码注释，未独立重读上游，可能随版本演进漂移。^[inner-class.md:9-15, inner-class.md:105-108]

来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。^[inner-class.md:114-114]

## Sources

- [inner-class.md](../../sources/inner-class.md) — Inner class 层：构造、验证门与 twisted 共轭枚举。
