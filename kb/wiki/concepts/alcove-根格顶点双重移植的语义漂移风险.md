---
title: Alcove 根格顶点双重移植的语义漂移风险
summary: 逐行对账确认两份 root_vertex_simple 核心算法等价，但错误、分配、bracket 处理及溢出纪律不同：领域版截断收窄并用普通算术，alcove.rs 版使用受检运算；尚无溢出输入见证。
sources:
  - atlas-core-domain-seams.md
kind: concept
createdAt: "2026-10-09T20:33:35.559Z"
updatedAt: "2026-10-10T01:53:10.838Z"
tags:
  - Alcove
  - 移植一致性
  - 整数溢出
aliases:
  - alcove-根格顶点双重移植的语义漂移风险
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Alcove 根格顶点双重移植的语义漂移风险

`root_vertex_simple` 有两份对应同一上游函数的 Rust 移植，分别位于 `atlas-core` 的 `domain_builtins.rs:6002` 和 `atlas-real-group/alcove.rs:643`。2026-10-10 的逐行对账确认，两份实现的核心算法逐步等价，但错误通道、预算纪律、`bracket` 失败处理和溢出纪律存在四处差异。其中，溢出处理差异是潜在的健壮性缺口，尚无输入证明它会产生错误结果。^[atlas-core-domain-seams.md:63-87]

## 共同算法与对齐依据

算法通过 `labels_for_component` 获取墙分量标签，将第一个标签为 1 的墙作为“最低余根”墙剔除，由其余墙生成有限部分；两侧各用自己的精确有理逆助手（domain 侧 `inverse_cartan`，alcove 侧 `rational_inverse`）求转置子 Cartan 矩阵的逆。两份实现都将 `bracket(gen[j], gen[i])` 放在矩阵的 `[i][j]` 位置，转置构造逐元相同。相关概念见 [[Alcove 墙标签与标签排序]]与 [[Alcove 算法中的精确有理线性代数]]。^[atlas-core-domain-seams.md:63-77]

搜索先以 `numer·floors` 尝试未移位解；若分子坐标不能全部被 `denom` 整除，则依次针对其余标签为 1 的墙加一列重试。全部失败时，`domain_builtins.rs` 版本报 `no root lattice vertex found for alcove component`；成功时，将系数乘以根向量并累加到结果。^[atlas-core-domain-seams.md:66-69]

两份实现的尝试组织方式不同：`alcove.rs` 预生成 `attempts` 列表，`domain_builtins.rs` 则先执行 `try_vertex(None)`，再逐列执行 `Some(column)`。逐行对账确认，其尝试顺序与数值、整性判据以及系数乘根坐标的累加数学均相同。^[atlas-core-domain-seams.md:73-77]

## 四处实现差异

### 错误通道

`domain_builtins.rs` 版本使用 `Result<_, String>`，`alcove.rs` 版本使用 `StructureError`。两者的错误表示方式不同，后者所属的错误体系见 [[StructureError 统一错误分类学]]。^[atlas-core-domain-seams.md:78-80]

### 预算纪律

来源将 `domain_builtins.rs` 版本记为无对应预算纪律，而 `alcove.rs` 版本使用 `try_reserve_exact`。这一分配处理差异独立于核心数学步骤的等价性。^[atlas-core-domain-seams.md:73-80]

### 配对失败处理

`domain_builtins.rs` 通过 `unwrap_or(0)` 将 `bracket` 失败静默转为零，`alcove.rs` 则通过 `?` 传播错误。不过，两侧调用点都只传入同一分量的已枚举根，来源据此将该失败情形记为实际不可达。因此，处理方式不同并不意味着已发现可触发的错误。^[atlas-core-domain-seams.md:79-81]

### 溢出纪律

`domain_builtins.rs` 使用 `coefficient as i32` 截断窄化，并以普通 `*`、`+` 累加；来源指出，这存在溢出回绕而产生错误顶点的风险。`alcove.rs` 则使用 `checked_mul`、`checked_add`，并通过逐坐标 `i32::try_from` 执行收窄。^[atlas-core-domain-seams.md:82-84]

这项差异是逐行对账新增的观察。系数来自子 Cartan 逆乘以小整数墙取值，目前尚未展示能触发溢出的输入；若溢出可达，正确行为应是报错而非回绕。来源将其列为需要构造输入的后续 HPC 探针候选，而非已证实的错误结果或修复授权。^[atlas-core-domain-seams.md:78-87]

## 调用范围与证据边界

`domain_builtins.rs` 版本的唯一调用方是 `"alcove_root_vertex"` 派发臂，来源标注位置为 `13275/13315`。这一调用范围限定了该版本在本次阅读中的使用入口。^[atlas-core-domain-seams.md:88-88]

两份实现的注释分别引用 `alcoves.cpp:345-408` 和 `alcoves.cpp:347-412`。这些上游位置均转述自 Rust 源码注释，未独立重读上游，行号可能随版本漂移。因此，两份 Rust 实现的算法对齐不等同于独立核对上游后的兼容性验收。^[atlas-core-domain-seams.md:63-77, atlas-core-domain-seams.md:124-125]

材料属于对当前工作区字节的结构性阅读，不声称语言或数学验收。两份移植的并存及其差异是阅读观察，不构成缺陷判定或修复授权；尤其应区分已确认的实现差异、当前调用下不可达的失败路径，以及仍待输入验证的溢出风险。^[atlas-core-domain-seams.md:19-19, atlas-core-domain-seams.md:79-87, atlas-core-domain-seams.md:124-127]

## Sources

- [atlas-core-domain-seams.md](../../sources/atlas-core-domain-seams.md) — 领域层接缝：值提取器、alcove 助手与 Weyl 词/生成元校验（domain_builtins.rs 三处缝隙）。
