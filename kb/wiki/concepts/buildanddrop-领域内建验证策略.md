---
title: BuildAndDrop 领域内建验证策略
summary: validate 仅构造并丢弃 Subgroup，不计算轨道，但在丢弃前完成非法根号、非 Cartan 矩阵及整数收窄检查。
sources:
  - atlas-core-weyl-subgroup.md
kind: concept
createdAt: "2026-10-09T14:39:57.048Z"
updatedAt: "2026-10-10T00:26:46.852Z"
tags:
  - 领域内建
  - 输入校验
  - 求值策略
aliases:
  - buildanddrop-领域内建验证策略
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: BuildAndDrop 领域内建验证策略
summary: validate 仅构造并丢弃 Subgroup，不计算轨道；构造阶段的非法生成元、Cartan 配对及整数窄化错误仍须得到诊断。
sources:
  - atlas-core-weyl-subgroup.md
kind: concept
tags:
  - 领域内建
  - 输入校验
  - 无值求值
aliases:
  - buildanddrop-领域内建验证策略
provenanceState: extracted
---

# BuildAndDrop 领域内建验证策略

BuildAndDrop 是反射子群领域内建 `Weyl_orbit` 与 `Weyl_orbit_ws` 使用的验证策略：`validate` 仅调用 `Subgroup::new` 构造并校验子群，随后丢弃结果，不计算轨道。轨道矩阵与见证词由实际求值路径生成。^[atlas-core-weyl-subgroup.md:10-11, atlas-core-weyl-subgroup.md:44-52]

## 构造阶段的校验

内建接收三个参数，通过首参是否为 `RootDatum` 识别 dual 形式；来源列出的 `Weyl_orbit_ws` dual 参数顺序为 `vec, rd, gens`。生成元经 `check_W_subsystem` 先收窄，再逐个通过 `internal_root_nbr` 检查有符号根号。生成元不限于单根，可以是任意有符号根号，但配对必须合法。^[atlas-core-weyl-subgroup.md:27-33, atlas-core-weyl-subgroup.md:68-69]

配对矩阵采用 `i128` 中间精度，窄化失败时报 `"Integer value too big for Cartan pairing"`；`infer_lie_type` 失败时报 `"Matrix for root indices is not a Cartan matrix: \n  …"`。单生成元特例的矩阵文本以双层方括号包围条目 `c`。相关算术边界见 [[反射子群构造校验与安全整数运算]]。^[atlas-core-weyl-subgroup.md:29-33]

## 无值需求下的诊断顺序

BuildAndDrop 保留“先校验、后丢弃”的顺序：不需要返回值时，非法生成元仍须得到诊断。来源指出原版同样先校验，再进入无值门；本文件的测试锚点覆盖非法根号、非 Cartan 矩阵文本及 `i32` 收窄，要求在丢弃前匹配原版诊断。^[atlas-core-weyl-subgroup.md:63-69]

## 与实际求值的边界

实际求值时，`Weyl_orbit` 将轨道列组成矩阵，列的格秩经过校验；`Weyl_orbit_ws` 生成见证词，经 `build_weyl_context` 和逐次 `right_multiply_simple` 重建 Weyl 元素，再由 `weyl_elt_value` 冻结规范词（canonical word）。这些工作不属于仅构造子群的验证路径。见证的作用约定见 [[ambient Weyl 见证词与作用方向]]。^[atlas-core-weyl-subgroup.md:44-52]

来源还记录了向量大小不匹配时的错误 `"Wrong vector size for Weyl subgroup orbit"`，并明确实现不模拟上游读取短向量时的未定义行为。测试锚点要求溢出与错秩成为安全错误，而非产生回绕轨道；这些约束不改变 `validate` 不计算轨道的边界。^[atlas-core-weyl-subgroup.md:52-54, atlas-core-weyl-subgroup.md:63-64]

## 证据范围

本页依据对 `crates/atlas-core/src/domain_builtins/weyl_subgroup.rs` 的结构性阅读，范围限于该文件中的两个领域内建。来源记录了测试锚点，但明确不以结构性阅读声称数学验收；本页也不将 BuildAndDrop 推广为所有领域内建的统一策略。相关测试背景见 [[反射子群轨道与见证的独立验证]]。^[atlas-core-weyl-subgroup.md:9-14, atlas-core-weyl-subgroup.md:56-72]

## Sources

- [atlas-core-weyl-subgroup.md](../../sources/atlas-core-weyl-subgroup.md) — 反射子群轨道与 ambient Weyl 见证（domain_builtins/weyl_subgroup.rs）。
