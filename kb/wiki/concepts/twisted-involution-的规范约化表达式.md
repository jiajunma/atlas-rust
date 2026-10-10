---
title: Twisted involution 的规范约化表达式
summary: canonical_involution_expr 按外部生成元序选择首个下降，以 s 编码 cross、!s 编码扭曲共轭，生成字典序最小约化表达式，终止性依赖输入契约。
sources:
  - inner-class.md
kind: concept
createdAt: "2026-10-09T14:51:08.217Z"
updatedAt: "2026-10-10T00:35:16.823Z"
tags:
  - 对合
  - Weyl群
  - 算法
aliases:
  - twisted-involution-的规范约化表达式
  - TI的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Twisted involution 的规范约化表达式
summary: canonical_involution_expr 按外部生成元编号选择首个下降，以 s 编码 cross、!s 编码扭曲共轭，生成字典序最小的约化 twisted-involution 表达式；终止性依赖调用方保证的输入契约。
sources:
  - inner-class.md
kind: concept
tags:
  - 扭曲对合
  - 约化表达式
  - 生成元编号
---

# Twisted involution 的规范约化表达式

`canonical_involution_expr` 为 twisted involution 的 Weyl 部分生成约化 twisted-involution 表达式，结果在**外部生成元编号**下字典序最小。该方法移植自上游 `TwistedWeylGroup::canonical_involution_expr`。^[inner-class.md:74-85]

## 输入契约

在分解 $\theta=w\cdot\delta$ 中，$\delta$ 是当前 inner class 的 distinguished involution，$w$ 是 Weyl 元素。调用方必须保证传入的 `weyl` 确实是本 inner class 某个 twisted involution 的 Weyl 部分；成员判定及相应分解见 [[InnerClass 成员判定与 twisted 分解]]。^[inner-class.md:55-59, inner-class.md:84-85]

## 下降选择与终止性

算法每一步按外部生成元编号递增的顺序选择第一个下降生成元（descent），遵循上游的 **external-least** 规则，而不是内部重编号顺序。随后通过 `hasTwistedCommutation` 区分该步采用 cross 还是 twisted conjugation（扭曲共轭）。^[inner-class.md:81-85]

满足输入契约时，每一步都会降低 twisted length；循环终止性依赖这一性质。因此，输入属于本 inner class 的 twisted-involution Weyl 部分是调用前提。^[inner-class.md:84-85]

## 带符号条目编码

输出的每一步用一个带符号条目表示：普通条目 `s` 表示 cross，即左乘 `s`；按位取反条目 `!s` 表示由 `s` 进行 twisted conjugation。这里的 `!s` 是按位取反编码，不能当作普通负号。上游将两类操作打包进一个 `int`，打印代码也按相同约定解码。^[inner-class.md:76-81]

## 与三阶段规范化的关系

`canonical_involution_expr` 的输出是规范约化表达式。另一个方法 `canonicalize` 则将输入搬运到规范代表元：先使正实根之和与正虚根之和均为 dominant，再限制到与两个和都正交的简单生成元，最后使实际 involution 在残余复根子系统中保持正性。详见 [[Twisted involution 的三阶段规范化]]。^[inner-class.md:63-68, inner-class.md:76-85]

`canonicalize` 返回的生成元按执行顺序使用：对每个 `s`，依次施加 $\sigma\mapsto s\cdot\sigma\cdot\delta(s)$。其中 $\delta(s)$ 是 distinguished involution 诱导的简单生成元置换，参见 [[Based involution 验证与生成元 twist]]。^[inner-class.md:49-51, inner-class.md:66-68]

## 证据范围

本页依据 `inner_class.rs` 的结构性阅读材料，对应快照记录的是 dirty 工作区字节。来源中的上游文件及行号转述自源码注释，未独立重读上游，可能随版本演进而漂移；来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。^[inner-class.md:9-15, inner-class.md:103-114]

## Sources

- [inner-class.md](../../sources/inner-class.md) — Inner class 层：构造、验证门与 twisted 共轭枚举。
