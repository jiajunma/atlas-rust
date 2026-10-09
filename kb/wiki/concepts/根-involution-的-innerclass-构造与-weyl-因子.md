---
title: 根 involution 的 InnerClass 构造与 Weyl 因子
summary: 构造入口在调用方根枚举预算内验证根与余根的传输，并可通过左复合 Weyl word 将任意合格的根 involution 转为保持基的 involution；不同入口保留或丢弃所得 Weyl 因子。
sources:
  - inner-class.md
kind: concept
createdAt: "2026-10-09T14:50:42.364Z"
updatedAt: "2026-10-09T14:50:42.364Z"
tags:
  - 内部类
  - 构造算法
  - Weyl群
aliases:
  - 根-involution-的-innerclass-构造与-weyl-因子
  - 根I的I构W因
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 根 involution 的 InnerClass 构造与 Weyl 因子

`InnerClass` 的根对合构造入口接受 unbased root datum 上的任意 involution，验证它对根与余根的作用，再通过 Weyl 作用将其调整为 based datum 的 involution。构造接口可以同时返回相对于所得 distinguished involution 的 Weyl 因子，也可以仅返回内类并丢弃构造过程中得到的 Weyl word。^[inner-class.md:29-40]

## 根理论状态与构造边界

这里的 `InnerClass` 是有意保留的部分实现：它拥有一个经过验证的 `BasedRootDatum`、有限常根系 `RootSystem` 和 distinguished `RootInvolutionData`。这些状态支持根理论层面的 twisted-conjugacy 轨道枚举，为 `CayleyCrossDecomposition` 提供 distinguished-involution 上下文，并为 `RealFormLabels` 锚定来源身份。它尚未构建 Atlas Cartan-class fibers，不持有 real-form data，也不含构造 KGB graph 所需的 torus data。参见 [[InnerClass 的根理论状态与实现边界]]。^[inner-class.md:19-25]

`InnerClass::new` 构建共享的根理论状态，根枚举预算由调用方明确提供。构造成功说明 distinguished lattice involution 置换该根系，并相应传送余根；这一保证不等同于 Atlas real-form 兼容性。^[inner-class.md:32-35]

## 从任意根对合到 distinguished involution

`InnerClass::from_root_involution` 对应上游 `inner_class(RootDatum,mat)` 入口的处理方式。它先验证输入 involution 置换根系并传送余根，再由 `wrt_distinguished` 从经过反射的单根像读出 Weyl word，并将该词对应的作用左复合到输入 involution，使之成为 based datum 的 involution。此入口随后丢弃该 Weyl word，与上游 wrapper 的行为一致。^[inner-class.md:36-40]

需要保留 Weyl 因子时，可使用 `inner_class_with_twisted_involution(datum, involution, root_budget)`。它委托 `InnerClass::from_root_involution_with_factor`，返回由输入 involution 定义的 inner class，以及输入相对于所得 distinguished involution 的 Weyl 因子。^[inner-class.md:29-31]

## Based 验证与生成元 twist

`based_involution_twist` 要求 involution 不仅置换本类根系并传送余根，还必须把每个单根映到单根。验证失败返回 `StructureError::InvalidBasedAutomorphism`；成功时返回诱导的单根置换。这一验证门见 [[Based involution 验证与生成元 twist]]。^[inner-class.md:44-48]

`generator_twist()` 给出 distinguished involution 对简单生成元的置换：若生成元 `s` 对应单根 \(\alpha_s\)，则 `twist[s]` 是 distinguished involution 将 \(\alpha_s\) 映到的单根所对应的生成元。^[inner-class.md:49-51]

## 已有内类中的 Weyl 因子

对于已有 `InnerClass`，`twisted_from_involution` 用于验证输入对合是否属于该内类，并返回分解
\[
\theta = w\cdot\delta
\]
中的 Weyl 元素 \(w\)，其中 \(\delta\) 是该内类的 distinguished involution。调用方须已完成 square 与 involutive 检查；该方法负责内类成员判定，拒绝时使用 `StructureError::InvalidBasedAutomorphism`。源码说明 weight-matrix equality 蕴含 twist 比较。参见 [[InnerClass 成员判定与 twisted 分解]]。^[inner-class.md:53-59]

## 证据范围

本页依据的是对 `inner_class.rs` 的结构性阅读，所读字节来自记录了 SHA-256 的 dirty 工作区快照。来源包未执行构建、测试或原版运行，因此这些说明不构成数学验收、性能或并行结论；其中上游位置均转述自源码注释，未独立重读上游。^[inner-class.md:9-15, inner-class.md:105-108, inner-class.md:114-114]

## Sources

- [inner-class.md](inner-class.md) — Inner class 层：构造、验证门与 twisted 共轭枚举。
