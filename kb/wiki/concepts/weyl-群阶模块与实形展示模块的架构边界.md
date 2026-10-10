---
title: Weyl 群阶模块与实形展示模块的架构边界
summary: 两模块互不导入，仅共享 StructureError；所读两文件无法确认 CartanClassification 内部是否调用 Weyl 群阶入口。
sources:
  - weyl-size-presentation.md
kind: concept
createdAt: "2026-10-09T15:18:55.758Z"
updatedAt: "2026-10-10T00:57:18.310Z"
tags:
  - 架构边界
  - 源码证据
aliases:
  - weyl-群阶模块与实形展示模块的架构边界
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Weyl 群阶模块与实形展示模块的架构边界
summary: weyl_size.rs 与 presentation.rs 互不导入，仅共享 StructureError；所读文件无法确认 CartanClassification 内部是否调用群阶入口。
sources:
  - weyl-size-presentation.md
kind: concept
tags:
  - Rust设计
  - 模块边界
  - 证据范围
---

# Weyl 群阶模块与实形展示模块的架构边界

`crates/atlas-real-group/src/` 中的 `weyl_size.rs` 负责由 Cartan 矩阵识别 Weyl 群阶，`presentation.rs` 负责构建实形名称与状态信息。两文件互不导入，共享类型仅为 `StructureError`；模块注释描述了各自用途，但不足以确定完整调用链。^[weyl-size-presentation.md:10-13, weyl-size-presentation.md:17-21, weyl-size-presentation.md:45-50, weyl-size-presentation.md:64-68]

## 群阶计算的职责

`weyl_size.rs` 的唯一入口是 crate 内可见的 `weyl_order_of_cartan`。模块注释将其用途关联到按需 twisted 共轭划分中的轨道大小核对，所用阶商公式为 $\lvert W\rvert/(\lvert W_{\mathrm{im}}\rvert\cdot\lvert W_{\mathrm{re}}\rvert\cdot\lvert W_{\mathrm{cx}}\rvert)$。这一用途只需要群阶，因此无需区分同阶的 B/C 型取向；识别过程可采用连通分量拆分与分支形状分析。详见 [[基于 Cartan 矩阵识别的 Weyl 群阶计算]]。^[weyl-size-presentation.md:17-21]

计算先逐行检查矩阵是否方形，再按非零非对角链接以 BFS 划分连通分量，最后将各分量的阶相乘。零对角行列对应的环面因子贡献阶 1。群阶使用精确 `Integer` 算术，因为分量乘积在 crate 的动态秩范围内可超出 `u128`。^[weyl-size-presentation.md:21-25]

## 实形展示的职责

`presentation.rs` 构建具有五个公开字段的 `RealFormPresentation`：Lie 代数名，以及 `IsCompact`、`IsSplit`、`IsQuasisplit`、`IsConnected` 四个状态位。compact/split 通过将 most-split Cartan 对合 `ms_tau` 与正负单位矩阵逐元素比较判定；quasisplit 由 external 编号是否等于 `quasisplit_external()` 判定；connected 则委托 `topology::dual_component_group_trivial` 判断对偶分量群是否平凡。参见 [[实形展示状态的构建与判定]] 与 [[对偶分量群与实形式连通性]]。^[weyl-size-presentation.md:45-50]

`build_presentations` 按 external 编号顺序逐形计算。compact/split 的扫描先于 `ms_tau` 的秩检查，后者仅检查行数；失败时结果整体丢弃，无可观察副作用。错误统一为 `LayoutInvariantViolation`，包含 `"external form number"`、`"most split Cartan"`、`"most split involution rank"` 和 `"special grading"` 四种 reason 字符串，其中 `"most split Cartan"` 由两处共用。详见 [[实形展示的编号顺序与不变量错误处理]]。^[weyl-size-presentation.md:52-56]

## 共享错误类型与调用边界

两模块分别通过 `crate::StructureError` 和 `crate::error::StructureError` 导入错误类型；这两条路径对应同一类型的再导出，并不构成两模块之间的导入依赖。相关错误体系见 [[StructureError 统一错误分类学]]。^[weyl-size-presentation.md:64-66]

两模块的架构关联仅见于模块注释：一侧描述 task #9 的阶商校验，另一侧描述展示状态位。`CartanClassification` 内部是否调用 `weyl_order_of_cartan`，在所读两份文件中不可见，因此不能据此确认群阶入口在分类流程中的实际调用位置。^[weyl-size-presentation.md:66-68]

## 验证与证据范围

本页依据两文件的结构性阅读，不构成数学或正确性验收。上游位置 `cartanclass.cpp:1046-1064`、`realredgp.cpp:68-80` 和 `atlas-types.w:3566-3575` 仅转录自代码注释，来源未核对上游字节。^[weyl-size-presentation.md:9-13, weyl-size-presentation.md:70-72]

群阶模块的测试锚点涵盖部分经典型、例外型、带环面因子的直积及空系统；展示模块的三个测试锚点涉及单连通 A1、伴随 A1 和单连通 B2。覆盖缺口包括群阶模块的所有错误分支、E7/E8、B4/C4 非 F4 分支，以及展示模块的非恒等对合（非 split 内类）、quasisplit 但非 split 的形和四类不变量分支。^[weyl-size-presentation.md:40-41, weyl-size-presentation.md:58-60, weyl-size-presentation.md:73-76]

来源列出的快照绑定 Git base、两文件字节 SHA-256 与草案调用记录，维护者已对照源码逐条核对改写。该次知识维护未执行 Atlas、Cargo、测试或 benchmark；所列测试锚点不代表该次执行结果。^[weyl-size-presentation.md:80-84]

## Sources

- [weyl-size-presentation.md](../../sources/weyl-size-presentation.md) — Weyl 群阶识别与实形展示层（`weyl_size.rs` / `presentation.rs`）。
