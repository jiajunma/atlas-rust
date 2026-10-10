---
title: Cayley/Cross 剥离预算的逻辑步计数
summary: 预算在找到下降后、执行前检查，复根步骤虽复合两次反射仍只计一步，因此无需剥离的输入可在零预算下通过。
sources:
  - cayley-cross.md
kind: concept
createdAt: "2026-10-10T02:31:48.149Z"
updatedAt: "2026-10-10T02:31:48.149Z"
tags:
  - 资源预算
  - Cayley变换
  - 算法边界
aliases:
  - cayleycross-剥离预算的逻辑步计数
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Cayley/Cross 剥离预算的逻辑步计数

Cayley/Cross 剥离预算以**逻辑剥离步**为计数单位，而非反射次数。在[[扭曲对合的 Cayley/Cross 分解]]中，`max_peeling_steps` 限制 peeling 循环的步数：Real 分支施加一次反射，Complex 分支复合两次反射，但二者都只消耗一个预算步。^[cayley-cross.md:35-43]

## 计数单位与分支行为

[[Cayley/Cross 的下降剥离算法]]按生成器下标升序寻找第一个 descent，判据是 `δ(θ(α_g))` 的简单坐标全不大于零。找到 descent 后，算法按根类型选择一次逻辑步的操作。^[cayley-cross.md:35-43]

| 根类型 | 记录的字母 | 当前作用的更新 | 预算计数 |
| --- | --- | --- | --- |
| Real | Cayley 字母 | `s_g ∘ current` | 1 步 |
| Complex | Cross 字母 | `s_g ∘ current ∘ s_{twist[g]}` | 1 步 |
| Imaginary | 不进入合法步进 | 返回 `"descent kind"` 不变量错误 | 不适用 |

其中，`twist[g]` 是 distinguished involution `δ` 将第 `g` 个简单根映到的生成器下标。Complex 分支虽然包含左右两次反射，仍被视为一次剥离操作；代码注释则声明 imaginary 根不可能是 descent。^[cayley-cross.md:32-43]

## 预算检查的位置与零预算语义

预算检查发生在**找到 descent 之后、执行步进之前**。当 `steps == max_peeling_steps` 时，构造返回 `CayleyCrossResourceLimit { resource: "peeling steps" }`，不再执行该步。因此，预算为零并不自动拒绝输入：无需剥离的输入仍可通过，需要执行剥离的输入才会触发限制。^[cayley-cross.md:35-40]

没有 descent 时，算法还要求当前作用为单位；若当前作用非单位，则返回 `"peeling termination"` 不变量错误。这一区分使“预算不足”与“未找到应有的下降步骤”分别对应资源错误和不变量错误。^[cayley-cross.md:35-39]

## 与终止性及输出的关系

构造在进入剥离循环前检查 datum 来源，并在 weight 与 coweight 两侧验证存储对合恰为 `w∘δ`。这一前置门通过强制 `w⁻¹ = δwδ` 支撑终止性论证；文档另声明每次剥离使长度减少 1 或 2。长度降幅和预算计数并非同一量，预算始终按上述逻辑步计数。^[cayley-cross.md:28-43, cayley-cross.md:95-96]

剥离后还要逆序重放字母、收集并变换 Cayley 根，随后进行正交性检查、长根化、取正与排序，最后验证重放结果等于输入。不同移植实现的 Cayley 根与 cross word 并不唯一，因此跨实现比较应采用重放不变量或标签级结果，不能要求原始分解部分一致。^[cayley-cross.md:17-24, cayley-cross.md:44-53]

## 测试与证据边界

来源列出的测试锚点包括预算为零的负路径，并精确匹配对应错误；另有 identity 空分解、A1×A1、A2、B2，以及 A2/B2 扭曲对合全枚举的分解与重放检查。不过，六种 `CayleyCrossInvariantViolation` 不变量错误均没有专门负测试，来源也未单独列出 Complex 分支“两次反射只计一步”的预算边界测试。^[cayley-cross.md:55-59, cayley-cross.md:97-99]

这些结论来自结构性源码阅读，不构成数学正确性验收。每步长度下降、长根计数递增及 imaginary 根不可能成为 descent 等终止性相关声明均为如实转述；本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[cayley-cross.md:95-96, cayley-cross.md:107-111]

## Sources

- [cayley-cross.md](../../sources/cayley-cross.md) — Cayley/Cross 分解与整对合分类（cayley_cross.rs / involution_classification.rs）。
