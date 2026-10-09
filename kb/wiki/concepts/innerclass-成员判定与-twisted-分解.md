---
title: InnerClass 成员判定与 twisted 分解
summary: twisted_from_involution 在调用方已检查平方与对合性的前提下验证内类归属，返回 θ=w·δ 中的 Weyl 元素，失败时报 InvalidBasedAutomorphism。
sources:
  - inner-class.md
kind: concept
createdAt: "2026-10-09T14:50:54.028Z"
updatedAt: "2026-10-09T19:29:47.001Z"
tags:
  - 内类
  - 成员判定
  - 扭曲对合
aliases:
  - innerclass-成员判定与-twisted-分解
  - I成T分
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# InnerClass 成员判定与 twisted 分解

`InnerClass::twisted_from_involution` 验证给定对合是否属于当前 `InnerClass`，并返回分解 \(\theta = w\cdot\delta\) 中的 Weyl 元素 \(w\)。其中 \(\theta\) 是输入对合，\(\delta\) 是当前内类的 distinguished involution（特选对合）。该方法同时完成内类成员判定与相对于特选对合的 Weyl 分解。^[inner-class.md:53-59]

## 上下文与调用前提

`InnerClass` 持有经过验证的 `BasedRootDatum`、有限常根系 `RootSystem` 和特选 `RootInvolutionData`。成员判定检查的是输入是否属于这一具体上下文所确定的内类，相关状态范围见 [[InnerClass 的根理论状态与实现边界]]。^[inner-class.md:19-25, inner-class.md:55-59]

调用方须事先完成方阵性（square）与对合性（involutive）检查；`twisted_from_involution` 在这些前提下验证内类归属。来源将其标为上游 `twisted_from_involution` 的移植，对应源码注释中的 `atlas-types.w:3844-3851`。^[inner-class.md:53-59]

## 判定依据与失败行为

判定中的权格矩阵相等性（weight-matrix equality）蕴含 twist 比较。成功时返回满足 \(\theta = w\cdot\delta\) 的 Weyl 元素 \(w\)；拒绝输入时使用 `StructureError::InvalidBasedAutomorphism`。^[inner-class.md:55-59]

相关接口 `generator_twist()` 给出特选对合对简单生成元的置换：`twist[s]` 对应的简单根是 \(\alpha_s\) 在特选对合下的像。[[Based involution 验证与生成元 twist]] 说明这一置换及其带基根数据验证条件。^[inner-class.md:44-51]

## 与构造入口及后续操作的关系

`InnerClass::from_root_involution` 是构造入口：它接受未带基根数据上的任意对合，验证其置换根系并传送余根，再左复合由 `wrt_distinguished` 得到的 Weyl 词，使其成为带基根数据的对合；该入口随后丢弃这个词。`inner_class_with_twisted_involution` 则同时返回构造出的内类及相对于所得特选对合的 Weyl 因子。相关内容见 [[根 involution 的 InnerClass 构造与 Weyl 因子]]。^[inner-class.md:29-40]

成员判定在已有内类中验证输入归属，并返回相对于该内类特选对合的 Weyl 部分。后续 `canonical_involution_expr` 要求输入 `weyl` 是本内类某个 twisted involution 的 Weyl 部分；这一调用方契约保证每步降低 twisted length，是循环终止性的依据。该操作输出外部生成元编号下字典序最小的约化 twisted-involution 表达式，见 [[Twisted involution 的规范约化表达式]]。^[inner-class.md:55-59, inner-class.md:76-85]

## 能力与证据边界

这里的归属与分解属于根理论层。来源描述的 `InnerClass` 是部分实现，尚未构建 Atlas Cartan-class fibers，不持有实形式数据，也不包含构造 KGB 图所需的环面数据；其构造验证成功不构成 Atlas 实形式兼容性的声明。^[inner-class.md:19-25, inner-class.md:32-35]

来源属于结构性源码阅读，未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。文中的上游位置转述自源码注释，未独立重读上游，行号可能随版本演进而变化。^[inner-class.md:9-15, inner-class.md:103-114]

## Sources

- [inner-class.md](inner-class.md) — Inner class 层：构造、验证门与 twisted 共轭枚举。
