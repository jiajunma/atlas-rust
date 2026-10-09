---
title: BuildAndDrop 领域内建验证策略
summary: validate 仅构造并丢弃 Subgroup，不计算轨道，同时确保非法根号、非 Cartan 矩阵和整数收窄错误在丢弃前得到诊断。
sources:
  - atlas-core-weyl-subgroup.md
kind: concept
createdAt: "2026-10-09T14:39:57.048Z"
updatedAt: "2026-10-09T20:47:26.094Z"
tags:
  - 领域内建
  - 无值求值
  - 输入校验
aliases:
  - buildanddrop-领域内建验证策略
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: BuildAndDrop 领域内建验证策略
summary: validate 仅构造并丢弃 Subgroup，不计算轨道；非法生成元及构造阶段的配对、窄化错误仍须在丢弃前得到诊断。
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

BuildAndDrop 是反射子群领域内建 `Weyl_orbit` 与 `Weyl_orbit_ws` 使用的验证策略：`validate` 仅通过 `Subgroup::new` 构造并校验子群，随后丢弃构造结果，不计算轨道。构造校验与轨道矩阵、见证词的实际求值因此具有明确边界。^[atlas-core-weyl-subgroup.md:44-52]

## 构造阶段的校验

内建接收三个参数，通过首参是否为 `RootDatum` 识别 dual 形式；来源列出的 `Weyl_orbit_ws` dual 参数顺序为 `vec, rd, gens`。生成元经 `check_W_subsystem` 收窄，再逐个通过 `internal_root_nbr` 检查有符号根号。生成元不限于单根，但配对必须合法。^[atlas-core-weyl-subgroup.md:25-33, atlas-core-weyl-subgroup.md:68-69]

配对矩阵采用 `i128` 中间精度，窄化失败时报 `"Integer value too big for Cartan pairing"`；`infer_lie_type` 识别失败时报 `"Matrix for root indices is not a Cartan matrix: \n  …"`。单生成元特例以双层方括号包围单个条目 `c` 的形式打印矩阵。相关算术与诊断边界见[[反射子群构造校验与安全整数运算]]。^[atlas-core-weyl-subgroup.md:29-33]

## 无值需求下的诊断顺序

BuildAndDrop 要求先校验、后丢弃：不需要计算结果时，非法生成元仍须得到诊断。源码测试锚点覆盖非法根号、非 Cartan 矩阵文本及 `i32` 收窄，要求这些错误在丢弃前匹配原版诊断；来源也明确指出原版先校验，再进入无值门。^[atlas-core-weyl-subgroup.md:63-69]

## 与实际求值的边界

实际求值时，`Weyl_orbit` 返回由轨道列组成的矩阵，每列具有校验过的格秩；`Weyl_orbit_ws` 则构造见证词，经 `build_weyl_context` 与逐次 `right_multiply_simple` 重建 Weyl 元素，再由 `weyl_elt_value` 固定其 canonical word。这些工作不属于仅构造子群的验证路径。见证的作用方向与事件拼接约定见[[ambient Weyl 见证词与作用方向]]。^[atlas-core-weyl-subgroup.md:44-52]

来源另列出向量大小不匹配时的错误 `"Wrong vector size for Weyl subgroup orbit"`，并明确实现不模拟上游读取短向量时的未定义行为。测试锚点也要求溢出与错秩成为安全错误，而不是产生回绕轨道；这些求值相关边界不能等同于验证路径已计算或检查了整个轨道。^[atlas-core-weyl-subgroup.md:52-54, atlas-core-weyl-subgroup.md:63-64]

## 证据范围

本说明依据 `crates/atlas-core/src/domain_builtins/weyl_subgroup.rs` 的结构性阅读，仅覆盖上述两个领域内建。测试锚点记录了校验顺序和错误行为的覆盖，但来源不以结构性阅读声称数学验收，也未将 BuildAndDrop 推广为所有领域内建的统一策略。^[atlas-core-weyl-subgroup.md:9-14, atlas-core-weyl-subgroup.md:56-72]

## Sources

- [atlas-core-weyl-subgroup.md](atlas-core-weyl-subgroup.md) — 反射子群轨道与 ambient Weyl 见证（domain_builtins/weyl_subgroup.rs）。
