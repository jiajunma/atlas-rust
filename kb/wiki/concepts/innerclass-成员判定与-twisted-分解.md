---
title: InnerClass 成员判定与 twisted 分解
summary: twisted_from_involution 在调用方已完成 square 与 involutive 检查的前提下，验证输入属于当前 inner class，并返回分解 θ = w·δ 中的 Weyl 元素 w，失败时报 InvalidBasedAutomorphism。
sources:
  - inner-class.md
kind: concept
createdAt: "2026-10-09T14:50:54.028Z"
updatedAt: "2026-10-09T14:50:54.028Z"
tags:
  - 内部类
  - 成员判定
  - Weyl群
aliases:
  - innerclass-成员判定与-twisted-分解
  - I成T分
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# InnerClass 成员判定与 twisted 分解

`InnerClass::twisted_from_involution` 用于验证给定对合是否属于当前 `InnerClass`，并返回分解
\[
\theta = w\cdot\delta
\]
中的 Weyl 元素 \(w\)。这里 \(\theta\) 是待判定的对合，\(\delta\) 是当前内类的 distinguished involution（特选对合）。因此，该方法同时提供成员判定与相对于特选对合的 Weyl 分解。^[inner-class.md:53-59]

## 上下文与调用前提

`InnerClass` 持有经过验证的 `BasedRootDatum`、有限常根系 `RootSystem` 和 distinguished `RootInvolutionData`。成员判定依赖这一具体上下文，检查的是输入是否落在**当前内类**中。有关这些状态的范围，可参见 [[InnerClass 的根理论状态与实现边界]]。^[inner-class.md:19-25, inner-class.md:55-59]

调用方必须事先完成矩阵为方阵及其为对合的检查；`twisted_from_involution` 在此基础上验证内类归属。该方法移植自上游 `twisted_from_involution`，来源包标注的对应位置为 `atlas-types.w:3844-3851`。^[inner-class.md:53-59]

## 判定依据与失败行为

判定中的权格矩阵相等性（weight-matrix equality）蕴含 twist 比较；拒绝输入时使用 `StructureError::InvalidBasedAutomorphism`。成功返回值是上述等式中的 Weyl 元素 \(w\)，其含义由当前内类的 \(\delta\) 确定。^[inner-class.md:55-59]

相关的 `generator_twist()` 给出 distinguished involution 对简单生成元的置换：`twist[s]` 对应的简单根是 \(\alpha_s\) 在 distinguished involution 下的像。[[Based involution 验证与生成元 twist]] 进一步说明这一置换与带基根数据验证之间的关系。^[inner-class.md:44-51]

## 与构造及后续操作的关系

[[从根数据对合构造内类]] 处理的是构造入口：`InnerClass::from_root_involution` 接受未带基根数据上的任意对合，验证其置换根系并传送余根，再左复合由 `wrt_distinguished` 得到的 Weyl 词，使其成为带基根数据的对合；该入口随后丢弃这个 Weyl 词。成员判定则在已有内类中验证归属，并返回相对于其 distinguished involution 的 Weyl 因子。^[inner-class.md:36-40, inner-class.md:55-59]

后续的 `canonical_involution_expr` 要求输入 `weyl` 是本内类某个 twisted involution 的 Weyl 部分；其终止性依赖这一调用方契约，因为每步都降低 twisted length。它输出外部生成元编号下字典序最小的约化 twisted-involution 表达式，详见 [[Twisted involution 的规范约化表达式]]。^[inner-class.md:74-85]

## 能力与证据边界

这里的归属与分解属于根理论层。来源描述的 `InnerClass` 是部分实现，尚未构建 Atlas Cartan-class fibers，不持有 real-form data，也不含构造 KGB 图所需的 torus data；其构造验证成功不构成 Atlas real-form 兼容性的声明。^[inner-class.md:19-25, inner-class.md:32-35]

来源包基于结构性源码阅读，未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。文中上游位置转述自源码注释，未独立重读上游，行号可能随版本变化。^[inner-class.md:9-15, inner-class.md:103-114]

## Sources

- [inner-class.md](inner-class.md) — Inner class 层：构造、验证门与 twisted 共轭枚举。
