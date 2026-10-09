---
title: Based involution 验证与生成元 twist
summary: based_involution_twist 要求 involution 置换根系、正确传输余根且将简单根映为简单根；generator_twist 给出 distinguished involution 诱导的简单生成元置换。
sources:
  - inner-class.md
kind: concept
createdAt: "2026-10-09T14:50:44.971Z"
updatedAt: "2026-10-09T14:50:44.971Z"
tags:
  - 验证
  - 根数据
  - 生成元置换
aliases:
  - based-involution-验证与生成元-twist
  - BI验T
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Based involution 验证与生成元 twist

Based involution 验证用于检查一个对合是否保持当前内类的带基根数据，并提取其对简单根的置换。`based_involution_twist` 要求对合置换本类根系、正确传送余根，并把每个简单根映到简单根；成功时返回诱导的简单根置换，拒绝时使用 `StructureError::InvalidBasedAutomorphism`。^[inner-class.md:44-48]

## 验证条件与构造边界

验证同时涉及根、余根和所选简单根系。保持根系及余根的条件，也出现在 `InnerClass::new` 的构造保证中；而 `based_involution_twist` 进一步明确要求保持简单根集合。相关对象与构造不变量可参见 [[BasedRootDatum：带基根数据与构造不变量]]。^[inner-class.md:32-35, inner-class.md:44-48]

对于未必保持所选基的根数据对合，`InnerClass::from_root_involution` 先验证其置换根系并传送余根，再左复合由 `wrt_distinguished` 从反射后的简单根像中读出的 Weyl 字，使之成为带基根数据的对合。该入口随后丢弃这个 Weyl 字；相关流程见 [[从根数据对合构造内类]]。^[inner-class.md:36-40]

## 生成元 twist 的含义

`generator_twist()` 给出 distinguished involution 对简单生成元的置换：若生成元 \(s\) 对应简单根 \(\alpha_s\)，则 `twist[s]` 是对应于 \(\alpha_s\) 的 distinguished image 的那个生成元。源码注释将这一接口对应到上游 `TwistedWeylGroup` 的 `weyl::Twist`，而验证函数返回的简单根置换对应上游 `rootdata::twist`。^[inner-class.md:44-51]

这一置换参与 twisted conjugation 的生成元操作。在 `canonicalize` 中，返回的生成元按执行顺序排列；对每个 \(s\)，依次执行 \(\sigma \leftarrow s\cdot\sigma\cdot\delta(s)\)，即可将输入传送到规范代表元，其中 \(\delta(s)\) 表示 distinguished involution 诱导的生成元像。相关算法见 [[Twisted involution 的三阶段规范化]]。^[inner-class.md:49-51, inner-class.md:63-68]

## 与内类成员判定的关系

`twisted_from_involution` 处理的是对合是否属于当前内类的问题，并返回分解 \(\theta=w\cdot\delta\) 中的 Weyl 元素 \(w\)，其中 \(\delta\) 是 distinguished involution。调用方须预先检查矩阵为方阵且满足对合条件；方法中的 weight-matrix equality 蕴含 twist 比较，拒绝同样由 `StructureError::InvalidBasedAutomorphism` 承载。详见 [[InnerClass 成员判定与 twisted 分解]]。^[inner-class.md:55-59]

## 证据与适用范围

这些验证属于 `InnerClass` 的根理论层。来源描述的实现只持有已验证的带基根数据、有限常根系和 distinguished involution 数据，尚不构建 Cartan-class fibers，也不持有实形式数据或构造 KGB 图所需的环面数据。因此，根与余根作用验证成功并不构成 Atlas 实形式兼容性的声明。^[inner-class.md:19-25, inner-class.md:32-35]

本文依据结构性源码阅读材料；其中上游对应关系与行号来自源码注释，未独立重读上游。该材料未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。^[inner-class.md:107-108, inner-class.md:114-114]

## Sources

- [inner-class.md](inner-class.md)：Inner class 层：构造、验证门与 twisted 共轭枚举。
