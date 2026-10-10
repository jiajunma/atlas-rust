---
title: Twisted involution 的三阶段规范化
summary: canonicalize 依次优势化正实根与正虚根之和、选取共同正交生成元并保持残余复子系统正性；受限版本将残余生成元与 active 取交。
sources:
  - inner-class.md
kind: concept
createdAt: "2026-10-09T14:50:46.121Z"
updatedAt: "2026-10-10T00:35:16.549Z"
tags:
  - 对合
  - 规范化
  - 算法
aliases:
  - twisted-involution-的三阶段规范化
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
title: Twisted involution 的三阶段规范化
summary: canonicalize 将正实根和与正虚根和优势化，限制到共同正交的简单生成元，再使实际对合在残余复子系统中保持正性；受限版本同时遵守 active 集合。
sources:
  - inner-class.md
kind: concept
tags:
  - 扭曲对合
  - 规范化
  - 算法
aliases:
  - twisted-involution-的三阶段规范化
---

# Twisted involution 的三阶段规范化

`InnerClass::canonicalize` 采用三阶段 Atlas 算法，将 twisted involution 搬运到规范代表元，并返回按执行顺序排列的生成元序列。来源将其对应到上游 `InnerClass::canonicalize`（`innerclass.cpp:740-832`）。^[inner-class.md:63-68]

## 三阶段算法

第一阶段，使正实根之和与正虚根之和均成为优势的（dominant）。第二阶段，将操作范围限制到同时与这两个和正交的简单生成元。第三阶段，使实际对合在残余复子系统中保持正性（positivity）。第二阶段使用的是优势化后的两个根和。^[inner-class.md:63-72]

## 返回序列与作用方向

记 distinguished involution 为 $\delta$，输入的 Weyl 部分为 $\sigma$。对返回序列中的每个生成元 $s$，按执行顺序更新 $\sigma \leftarrow s\cdot\sigma\cdot\delta(s)$；依次执行这些扭曲共轭操作，即把输入搬运到规范代表元。$\delta(s)$ 表示 distinguished involution 在简单生成元上诱导的置换像，参见 [[Based involution 验证与生成元 twist]]。^[inner-class.md:49-51, inner-class.md:55-57, inner-class.md:63-68]

## 限定生成元的规范化

`canonicalize_with_generators` 将算法限制在 `active` 指定的简单生成元内。其第二阶段的残余子系统由 `active` 与“同时正交于两个优势根和的生成元”取交集得到。上游 `Rep_context::to_singular_canonical` 使用 singular generators 调用这一变体，来源标注的位置为 `repr.cpp:613-620`。^[inner-class.md:69-72]

## 与相关操作的区别

[[Twisted involution 的规范约化表达式]]描述另一种输出：`canonical_involution_expr` 给出 twisted involution 的 Weyl 部分的约化 twisted-involution 表达式，并在外部生成元编号下取字典序最小。`canonicalize` 返回的则是将输入搬运到规范代表元的生成元序列；两者的输出含义不同。^[inner-class.md:63-68, inner-class.md:76-85]

[[Twisted involution 枚举与共轭轨道分区]]中的 `twisted_conjugacy_classes` 给出确定性的 Weyl twisted-conjugacy 轨道，但其代表元并非 Atlas-canonical；该接口也不构造 Cartan fibers、实形式或 Cartan 偏序。因此，轨道枚举接口的代表元不能直接视为上述规范化算法的结果。^[inner-class.md:63-68, inner-class.md:89-96]

## 证据边界

来源材料属于对 `inner_class.rs` 的结构性阅读，记录的源码字节来自 dirty 工作区。上游位置转述自源码注释，未独立重读上游，行号可能随版本变化。来源未执行构建、测试或原版运行，因此不提供数学验收、性能或并行结论。^[inner-class.md:9-15, inner-class.md:105-114]

## Sources

- [inner-class.md](../../sources/inner-class.md) — Inner class 层：构造、验证门与 twisted 共轭枚举。
