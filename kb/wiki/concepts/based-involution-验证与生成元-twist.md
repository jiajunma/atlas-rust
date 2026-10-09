---
title: Based involution 验证与生成元 twist
summary: based_involution_twist 验证根置换、余根运输及单根到单根的映射，generator_twist 给出 distinguished 对合诱导的简单生成元置换。
sources:
  - inner-class.md
kind: concept
createdAt: "2026-10-09T14:50:44.971Z"
updatedAt: "2026-10-09T19:29:51.711Z"
tags:
  - 对合
  - 输入验证
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
---

# Based involution 验证与生成元 twist

`based_involution_twist` 验证一个对合是否保持当前内类的带基根数据，成功时返回其诱导的简单根置换。`generator_twist()` 则给出 distinguished involution 对简单生成元的置换，供 twisted 操作使用。^[inner-class.md:44-51]

## 验证条件与返回结果

`based_involution_twist` 要求对合同时满足三个条件：置换本类根系、正确传送余根，以及把每个简单根映到简单根。验证失败时返回 `StructureError::InvalidBasedAutomorphism`；成功时返回的简单根置换对应上游 `rootdata::twist`。该验证移植自上游 `check_based_root_datum_involution`。^[inner-class.md:44-48]

这些条件涉及 [[BasedRootDatum：带基根数据与构造不变量|带基根数据]] 中的根、余根和所选简单根系。`InnerClass::new` 的成功结果保证 distinguished lattice involution 置换根系并传送余根，但来源明确指出，这并非 Atlas 实形式兼容性的声明。^[inner-class.md:32-35, inner-class.md:44-48]

对于未必保持所选基的根数据对合，`InnerClass::from_root_involution` 先验证其置换根系并传送余根，再左复合由 `wrt_distinguished` 从反射后的简单根像中读出的 Weyl 字，使之成为带基根数据的对合。该入口随后丢弃这个 Weyl 字；相关流程见 [[从根数据对合构造内类]]。^[inner-class.md:36-40]

## 生成元 twist 的含义

设 distinguished involution 为 \(\delta\)，简单生成元 \(s\) 对应简单根 \(\alpha_s\)。`generator_twist()` 返回的置换满足：`twist[s]` 是对应于根 \(\delta(\alpha_s)\) 的简单生成元。该接口对应上游 `TwistedWeylGroup` 的 `weyl::Twist`。^[inner-class.md:49-51]

这一置换参与 twisted conjugation。在 `canonicalize` 中，返回的生成元按执行顺序排列；对每个 \(s\)，依次执行 \(\sigma \leftarrow s\cdot\sigma\cdot\delta(s)\)，即可将输入传送到规范代表元，其中 \(\delta(s)\) 是 distinguished involution 诱导的生成元像。相关算法见 [[Twisted involution 的三阶段规范化]]。^[inner-class.md:49-51, inner-class.md:63-68]

## 与内类成员判定的关系

`twisted_from_involution` 验证给定对合是否属于当前内类，并返回分解 \(\theta=w\cdot\delta\) 中的 Weyl 元素 \(w\)。调用方须预先检查矩阵为方阵且满足对合条件；该方法中的权格矩阵相等性蕴含 twist 比较，拒绝同样使用 `StructureError::InvalidBasedAutomorphism`。详见 [[InnerClass 成员判定与 twisted 分解]]。^[inner-class.md:55-59]

## 实现与证据边界

来源描述的 `InnerClass` 是根理论层的部分实现，只持有已验证的带基根数据、有限常根系和 distinguished involution 数据；它尚未构建 Atlas Cartan-class fibers，也不持有实形式数据或构造 KGB 图所需的环面数据。应在这一范围内理解上述验证，参见 [[InnerClass 的根理论状态与实现边界]]。^[inner-class.md:19-25]

本文依据结构性源码阅读材料。上游对应关系与行号转述自源码注释，未独立重读上游；该材料未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。^[inner-class.md:107-108, inner-class.md:114-114]

## Sources

- [inner-class.md](inner-class.md)：Inner class 层：构造、验证门与 twisted 共轭枚举。
