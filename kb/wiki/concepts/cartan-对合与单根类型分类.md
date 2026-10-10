---
title: Cartan 对合与单根类型分类
summary: 原型 CartanInvolution 校验 M²=I 及单根像属于根系，紧性标志是已由 Grading 取代的未校验断言，simple_real_rank 不等同于一般实秩。
sources:
  - lib-root.md
kind: concept
createdAt: "2026-10-09T14:58:57.263Z"
updatedAt: "2026-10-10T00:41:56.599Z"
tags:
  - 对合
  - 根分类
aliases:
  - cartan-对合与单根类型分类
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Cartan 对合与单根类型分类
summary: A1 原型层的 CartanInvolution 校验矩阵对合与单根像的根系成员关系；单根按对合像分类，虚根紧性依赖未经校验且已被 Grading 取代的调用方标志。
sources:
  - lib-root.md
kind: concept
tags:
  - Cartan对合
  - 根类型
  - 原型限制
aliases:
  - cartan-对合与单根类型分类
---

# Cartan 对合与单根类型分类

`CartanInvolution` 与 `RealReductiveGroup` 位于 `atlas-real-group` 的 [[A1 迁移原型层与对偶格类型设计|A1 迁移原型层]]，均为 `pub(crate)`，实现注释标明待替换（pending replacement）。前者校验对合条件，后者通过 `classify_simple_root` 根据单根的对合像进行分类。^[lib-root.md:30-49]

## 对合验证与紧性标志

`CartanInvolution` 检查矩阵满足 \(M^2=I\)，并检查单根的像属于根系。其 `compact_imaginary` 标志是未经校验的调用方断言，因此这些检查并未验证虚根紧性。来源明确标明该标志已被 `Grading` 取代，相关概念见 [[Grading 的位向量类型纪律]]。^[lib-root.md:45-47]

原型 `RootDatum::roots()` 以正负单根播种，通过 FIFO BFS 构造反射闭包，并按坐标字典序输出。计算采用 `i128` 中间精度再收窄至 `i32`，溢出时报 `ArithmeticOverflow`；第 4097 个互异向量触发 `RootSystemTooLarge`。此原型 `RootDatum` 与正式模块中的 [[BasedRootDatum：带基根数据与构造不变量|BasedRootDatum]] 是不同类型。^[lib-root.md:39-41, lib-root.md:57-58]

## 单根类型分类

`RootType` 提供四种分类：`CompactImaginary`（紧虚根）、`NoncompactImaginary`（非紧虚根）、`Real`（实根）和 `Complex`（复根）。`classify_simple_root` 根据单根的像等于原根、等于负根或属于其他情况进行分类。^[lib-root.md:45-49]

对于单根 \(\alpha\)，若对合固定 \(\alpha\)，则为虚根，再由紧性标志区分紧虚根与非紧虚根；若像为 \(-\alpha\)，则为实根；其余情况为复根。来源列出的固定根测试验证了按标志判为非紧虚根的情形。相关背景见 [[对合下的虚根、实根与复根分类]]。^[lib-root.md:45-49, lib-root.md:53-54]

`RealReductiveGroup` 的 `simple_real_rank` 有明确的范围限制：其含义刻意窄于一般 real rank，不能将两者等同。^[lib-root.md:48-49]

## 测试与证据边界

来源列出三个测试锚点：A1 反射取负、固定根按标志判为非紧虚根，以及 `i32::MAX` 坐标配对恰报 `ArithmeticOverflow` 而非回绕。这些是文件中记录的测试内容，详见 [[原型层的测试锚点与证据边界]]。^[lib-root.md:51-54]

本页依据 crate 门面与原型层的结构性阅读，不覆盖其他模块的完整实现，也不构成数学验收。来源材料经过维护者对照源码逐条核对；该次知识维护未执行 Atlas、Cargo、测试或 benchmark，因此测试描述不代表该次维护获得的执行结果。^[lib-root.md:9-14, lib-root.md:55-58, lib-root.md:62-66]

## Sources

- [lib-root.md](../../sources/lib-root.md)：crate 根：60 模块组织、52 条再导出与 A1 原型层（lib.rs）。
