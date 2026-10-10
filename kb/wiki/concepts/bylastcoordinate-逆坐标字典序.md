---
title: ByLastCoordinate 逆坐标字典序
summary: ByLastCoordinate 使用 iter().rev() 从末坐标向前进行字典序比较，与上游 root_compare 一致的说法属于实现方移植陈述。
sources:
  - atlas-core-domain-scc-root-table.md
kind: concept
createdAt: "2026-10-09T14:29:18.003Z"
updatedAt: "2026-10-10T00:17:32.413Z"
tags:
  - 排序
  - 根系
aliases:
  - bylastcoordinate-逆坐标字典序
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: ByLastCoordinate 逆坐标字典序
summary: ByLastCoordinate 通过 iter().rev() 从末坐标向前作字典序比较；与上游 root_compare 的对应属于移植陈述，兼容性以 HPC 差分门为准。
sources:
  - atlas-core-domain-scc-root-table.md
kind: concept
tags:
  - 排序
  - 根数据
  - 兼容性
aliases:
  - bylastcoordinate-逆坐标字典序
provenanceState: extracted
---

# ByLastCoordinate 逆坐标字典序

`ByLastCoordinate` 采用**从末坐标向前比较的字典序**，通过 `iter().rev()` 逆序遍历坐标。“逆坐标”描述的是坐标参与比较的先后顺序，而非将比较结果反转。^[atlas-core-domain-scc-root-table.md:24-25]

## 实现位置与上游对应

该比较规则位于 `crates/atlas-core/src/domain_builtins.rs`，来源将其归入 7992–9382 区域的序、矩阵辅助与对合校验部分。来源指出，这一顺序与 C++ 上游 `rootdata.cpp:118–129` 中 `root_compare` 的“末坐标向前”规则一致。^[atlas-core-domain-scc-root-table.md:9-13, atlas-core-domain-scc-root-table.md:22-25]

同一源码阅读包还覆盖 [[RootTable 的根与余根构建]]：`RootTable::build` 按分量处理，通过 `express` 将单坐标向量表达为环境格基坐标，并分别记录根与余根的长标志。^[atlas-core-domain-scc-root-table.md:34-38]

## 证据边界

本页依据结构性阅读记录，不构成数学验收。来源中的上游行号引用属于实现方的移植陈述；上述顺序对应不能单独作为兼容性验收结论，兼容性仍以 [[HPC 验收证据链|HPC 差分门]] 为准。^[atlas-core-domain-scc-root-table.md:9-14, atlas-core-domain-scc-root-table.md:44-45]

来源注明快照的 Git base 为 `964f0033`，字节数与哈希仅标识快照字节。SCC 细节及 RootTable 消费者等大块内容只在头部精读，因此本页不据此扩展实现覆盖或验证范围。^[atlas-core-domain-scc-root-table.md:40-45]

## Sources

- [atlas-core-domain-scc-root-table.md](../../sources/atlas-core-domain-scc-root-table.md) — 块图 SCC 与根表（domain_builtins.rs 3022–5005 + 7992–9606）。
