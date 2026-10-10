---
title: Rust Weyl 内核与抽象群的无环所有权模型
summary: 分离环境坐标 RootSystem 与抽象 Weyl 群身份，并禁止两者反向持有 handle；身份修复已落地，后续共享与性能方案仍需独立验证。
sources:
  - weyl-context-identity-and-sharing.md
kind: concept
createdAt: "2026-10-09T15:17:00.639Z"
updatedAt: "2026-10-10T00:55:47.500Z"
tags:
  - rust
  - 所有权
  - Weyl
aliases:
  - rust-weyl-内核与抽象群的无环所有权模型
  - RW内
confidence: 1
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Rust Weyl 内核与抽象群的无环所有权模型
summary: 分离 datum 坐标内核与抽象 Weyl 群身份，避免强引用环并保留 dual 历史及跨坐标运算语义；核心身份修复已落地，进一步共享与性能收益仍需验证。
sources:
  - weyl-context-identity-and-sharing.md
kind: concept
tags:
  - Rust
  - 所有权
  - 上下文共享
aliases:
  - rust-weyl-内核与抽象群的无环所有权模型
---

# Rust Weyl 内核与抽象群的无环所有权模型

Rust Weyl 所有权模型区分 **datum 环境坐标下的计算内核**与**决定元素兼容性的抽象群身份**。核心身份修复已通过 AFTER-v5 的 A1 限定语义验收，并以生产提交 `690c2b92` 落地；来源另列的进一步共享方案仍属提案，不能据此宣称缓存、性能、内存或更高 rank 已获验证。^[weyl-context-identity-and-sharing.md:63-74, weyl-context-identity-and-sharing.md:249-265]

## 设计动机

修复前，每次 `build_weyl_context` 都重新构造 `RootSystem` 与 `WeylInterface`，再连同完整 handle 放入新的 `Arc<WeylEltContext>`。同一长期存活 datum 上的短命 Weyl 元素因此反复构建上下文。只弱缓存 context，可能在临时元素释放后失去缓存；将完整 context 强存回 handle，则会形成 `handle -> context -> handle` 强引用环。^[weyl-context-identity-and-sharing.md:195-206, weyl-context-identity-and-sharing.md:222-227]

共享范围同时影响可观察语义。Original Atlas 通过弱驻留获得 canonical datum 的活对象身份；`dual()` 仅在目标群尚未初始化时共享源群，绝不覆盖已预热目标的群。Weyl 元素的 `=`、`!=`、`*` 在产生结果前检查群地址，不同则抛出 `Weyl group mismatch`，且检查先于 `no_value` gate。因此，结构相等或根排列相同不足以判定兼容性，必须保留 [[dual 预热历史与 Weyl 群兼容性]]。^[weyl-context-identity-and-sharing.md:164-187]

## 已落地的身份机制

`RootDatumHandle` 携带 `Arc<DatumWeylIdentity>`，其中为 `DatumWeylKernel`（`RootSystem`）与 `AbstractWeylGroup`（`WeylInterface`）分别设置仅成功发布的惰性 cell。进程级弱注册表按完整 datum 内容加 preference 驻留 identity，八个 handle 构造点统一经过私有 `interned` 构造器；结构性 RootDatum `Eq`/`Debug` 保持不变。该实现最初记录为候选，随后通过 AFTER-v5 并落地。^[weyl-context-identity-and-sharing.md:249-261]

`dual(RootDatum)` 只向仍未初始化的 canonical target 安装源 group，已预热的 target 保留原身份。二元 `=`、`!=`、`*` 先比较 abstract-group 的 `Arc` identity，再将右侧 external word 在左侧坐标系中重放；关系检查在 `no_value` 级别同样执行。参见 [[Weyl 元素的可失败关系与跨坐标运算]]。^[weyl-context-identity-and-sharing.md:252-256]

## 无环共享设计

来源将进一步共享方案明确列为后续提案：拆分两个无反向引用的不可变层，先修复语义，再分别验证缓存与性能。第一层由 datum owner 通过仅成功发布的惰性 `Arc<DatumWeylKernel>` 持有环境坐标下的 `RootSystem`。handle clone、语言 alias 与 `root_datum(WeylElt)` 往返共享该内核；真正新建的 quotient、dual、derived、integral、folded 或 explicit datum 获得新 cell。这些细化规则不能整体视为已验证的生产共享契约。^[weyl-context-identity-and-sharing.md:263-270]

第二层通过按 exact pre-root identity 弱驻留的 canonical cell 持有 `Arc<AbstractWeylGroup>`，保存 `WeylInterface` 与可观察的 canonical-word ordering。`dual()` 只在 canonical target 尚未初始化时共享该身份，两个已发布的 identity 永不合并。^[weyl-context-identity-and-sharing.md:271-274]

提议的 `WeylEltContext` 包含 `{handle, Arc<DatumWeylKernel>, Arc<AbstractWeylGroup>}`。两个共享层均不持有 handle，因此不会形成返回 handle 的强引用环；`Arc` 管理活对象，`Weak` 允许驻留槽失效，`OnceLock` 只发布完整成功结果。^[weyl-context-identity-and-sharing.md:275-278]

兼容性由 `AbstractWeylGroup` 的 `Arc` identity 决定，而非 structural handle 或 coordinate-kernel identity。即使两个值兼容，只要坐标内核不同，也不能直接比较或复合内部 root permutation；应在左值 `RootSystem` 中重放右值的 external generator word，结果归左 owner。二元关系必须采用可失败的 domain relation，离开不可失败的通用 `PartialEq` 路径。参见 [[Weyl 元素兼容性与跨坐标词重放]]。^[weyl-context-identity-and-sharing.md:279-284]

## 生命周期与失败处理

Original Atlas 的 `W_elt_value` 强持有 root datum，保证元素存活期间 datum 不失效；inner-class 构造则立即调用 `srd->dual()`，并强持有 primal 与 dual。来源指出，仅修复显式 `dual(RootDatum)` 不足以覆盖全部构造历史：inner-class 路径也需要保持 canonical dual 身份与相应生命周期。是否在 Rust context 中增存 dual handle，仍需 original-backed history 与无环所有权测试共同确定，参见 [[对偶内类与 Weyl 身份共享]]。^[weyl-context-identity-and-sharing.md:81-85, weyl-context-identity-and-sharing.md:286-292]

惰性 cell 不得缓存失败的 `Diagnostic` 或首次调用的 `SourceSpan`：构造失败后必须允许重试，并将错误定位到当前调用。现有 `FallibleOnce` 的 poison 文本专用于 lazy real-form，不能未经修改直接复用。此约束对应 [[可失败重试的惰性身份初始化]]；自动内存管理本身不能证明缓存键、共享范围或历史语义正确。^[weyl-context-identity-and-sharing.md:294-297]

## 验证范围与未决问题

来源记录 AFTER-v5 job `3900050` 为 `COMPLETED 0:0`：cold_dual 完全字节相等，prewarmed_dual 在 stdout、退出码与有序 error summary 上一致。验收证据文件为 `math_weyl_context_core_after_v5_acceptance_2026_10_06.json`，report SHA 前缀为 `3288480d…`。接受范围仅为 A1 限定语义，不授予 cache、performance、memory、rank 或更广数学 release。^[weyl-context-identity-and-sharing.md:63-69]

A1 跨 dual 乘法只涉及 \(s_0s_0=1\)，不能发现 generator renumbering 错误或直接复合 foreign root permutation 的问题。现有 rebind fixture 仍有 alias 保持旧 datum 存活，也未证明仅靠保存的 WeylElt 就能维持 datum 生命周期；Atlas 输出同样无法证明 fresh-equal owner 使用独立 coordinate cell。后两项需要额外 lifetime fixture 与 HPC-only `Weak`/work-count 守卫。^[weyl-context-identity-and-sharing.md:313-318]

后续 [[Weyl 语义回归的递进验证门禁]] 包括 G2 非对称 interface-order、B2/C2、两个乘法操作数顺序、inner-class dual 与 `no_value` relations。截至来源的 2026-10-09 更新，G2 gate 已冻结和彩排，因隧道中断尚待提交，其余后续见证已起草为 provisional fixture。语义 gate 全部通过后，才进入 one-build work-count 测试及同节点、交替顺序、fresh-process 的 time/CPU/RSS A/B；`59 -> 至多 5` 只是调用层工作量假设。^[weyl-context-identity-and-sharing.md:95-100, weyl-context-identity-and-sharing.md:320-330]

内存收益不能从无环设计直接推出。多个 WeylElt 同时存活时，共享 RootSystem 应减少重复对象；短命元素场景中，owner 强持有 kernel 却可能使 peak RSS 不变甚至略升。后续评估需同时报告 live owner/kernel/build 数、分配与 peak RSS，不能只引用 `Arc`/`Weak` 设计。^[weyl-context-identity-and-sharing.md:332-335]

## Sources

- [weyl-context-identity-and-sharing.md](../../sources/weyl-context-identity-and-sharing.md) — Weyl 对象身份、dual 历史与安全共享边界。
