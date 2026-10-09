---
title: BuildAndDrop 领域内建验证策略
summary: validate 仅构造并丢弃 Subgroup，不计算轨道，同时确保非法根号、非 Cartan 矩阵及整数窄化问题在丢弃前得到诊断。
sources:
  - atlas-core-weyl-subgroup.md
kind: concept
createdAt: "2026-10-09T14:39:57.048Z"
updatedAt: "2026-10-09T14:39:57.048Z"
tags:
  - 领域内建
  - 验证策略
  - 错误诊断
aliases:
  - buildanddrop-领域内建验证策略
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# BuildAndDrop 领域内建验证策略

BuildAndDrop 是反射子群领域内建 `Weyl_orbit` 与 `Weyl_orbit_ws` 使用的验证策略：`validate` 仅调用 `Subgroup::new` 构造并校验子群，随后丢弃构造结果，不计算轨道。它将构造阶段的合法性检查与轨道、见证词的求值分开。^[atlas-core-weyl-subgroup.md:44-54]

## 构造阶段的校验

`Subgroup::new` 接收三个参数，并通过首参是否为 `RootDatum` 识别 dual 形式；`Weyl_orbit_ws` 的 dual 参数顺序为 `vec, rd, gens`。生成元通过 `internal_root_nbr` 逐个转换和检查，配对矩阵使用 `i128` 中间精度，并由 `infer_lie_type` 检查其 Cartan 矩阵合法性。生成元可以是任意有符号根号，不限于单根，但配对必须合法。^[atlas-core-weyl-subgroup.md:25-33, atlas-core-weyl-subgroup.md:68-69]

构造失败会保留明确诊断：Cartan 配对窄化失败时报 `"Integer value too big for Cartan pairing"`；矩阵类型识别失败时报 `"Matrix for root indices is not a Cartan matrix: \n  …"`，单生成元特例采用 `[[c]]` 格式。这些检查属于[[反射子群构造校验与安全整数运算]]的构造边界。^[atlas-core-weyl-subgroup.md:29-33]

## 无值需求下的行为

BuildAndDrop 的关键约束是先校验、后丢弃：不需要返回计算值，并不意味着可以跳过非法生成元检查。源码测试锚点明确覆盖非法根号、非 Cartan 矩阵文本及 `i32` 收窄，要求它们在丢弃前给出与原版匹配的诊断；来源也指出原版采用先校验、后进入无值门的顺序。^[atlas-core-weyl-subgroup.md:63-69]

实际求值则承担额外工作：`Weyl_orbit` 返回由轨道列组成的矩阵，`Weyl_orbit_ws` 构造 ambient Weyl 见证词，并重建 Weyl 元素、冻结其 canonical word。这些工作不属于仅调用 `Subgroup::new` 的验证路径；相关作用方向见[[ambient Weyl 见证词与作用方向]]。^[atlas-core-weyl-subgroup.md:44-52]

## 证据边界

本策略的说明来自 `domain_builtins/weyl_subgroup.rs` 的结构性阅读，覆盖上述两个领域内建。测试锚点提供了校验与丢弃顺序的源码依据，但来源明确不以结构性阅读声称数学验收，也不能据此断言所有领域内建均采用相同策略。^[atlas-core-weyl-subgroup.md:9-14, atlas-core-weyl-subgroup.md:56-64, atlas-core-weyl-subgroup.md:70-72]

## Sources

- [atlas-core-weyl-subgroup.md](atlas-core-weyl-subgroup.md)
