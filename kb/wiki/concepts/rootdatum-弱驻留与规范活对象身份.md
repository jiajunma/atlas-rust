---
title: RootDatum 弱驻留与规范活对象身份
summary: 原版按完整 PreRootDatum 内容及 preference 弱驻留对象，以存活对象指针判等；重型对象可释放，但轻量驻留键仍保留。
sources:
  - weyl-context-identity-and-sharing.md
kind: concept
createdAt: "2026-10-09T15:16:07.609Z"
updatedAt: "2026-10-09T22:52:58.955Z"
tags:
  - 对象身份
  - 生命周期
  - 弱驻留
aliases:
  - rootdatum-弱驻留与规范活对象身份
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: RootDatum 弱驻留与规范活对象身份
summary: RootDatum 按完整构造内容及 preference 弱驻留，复用仍存活的规范身份；datum 身份、Weyl 群身份与结构相等承担不同职责。
sources:
  - weyl-context-identity-and-sharing.md
kind: concept
tags:
  - 对象身份
  - 生命周期
  - 弱驻留
aliases:
  - rootdatum-弱驻留与规范活对象身份
provenanceState: extracted
---

# RootDatum 弱驻留与规范活对象身份

RootDatum 的弱驻留（weak interning）按完整构造内容复用仍然存活的对象，形成规范活对象身份（canonical live identity）。在 original Atlas 中，相同内容的 RootDatum 构造之所以在语言层相等，是因为驻留后取得同一个 `shared_ptr`；相等运算本身比较指针身份，不重新比较结构。^[weyl-context-identity-and-sharing.md:164-172]

## 驻留键与生命周期

original Atlas 的驻留键包含完整 `PreRootDatum` 内容：simple roots、simple coroots 和 `prefer_coroots`。身份 `store` 只保存 `weak_ptr`；匹配对象仍存活时复用其 `shared_ptr`，对象已释放时则在原槽位重建。这保证了存活对象的规范身份，同时允许不可达的重型 datum 被释放。^[weyl-context-identity-and-sharing.md:164-170]

弱驻留不意味着整个索引自动清空。静态 `pool/hash` 仍保留轻量的 `PreRootDatum` 键，因此必须区分重型对象的生命周期与驻留索引的存储。^[weyl-context-identity-and-sharing.md:167-170]

每个 `W_elt_value` 强持有其 root datum，并保存对该 datum 的 Weyl 群的引用；因此，只要元素仍存活，其 datum 就不会失效。original 的 `inner_class_value::build` 还会立即调用 `srd->dual()`，并强持有 primal 与 dual 两个 datum。^[weyl-context-identity-and-sharing.md:81-85, weyl-context-identity-and-sharing.md:183-187]

## Datum 身份与 Weyl 群身份

RootDatum 的规范身份与 Weyl 群身份属于不同层次。每个活的 `root_datum_value` 含有初始为空的强 `shared_ptr<WeylGroup> W_ptr`，首次调用 `W()` 时才构造群对象。`dual()` 先通过同一弱驻留表取得 canonical dual：目标的 `W_ptr` 为空时，确保 source 的群已经建立，再将同一个群指针安装到目标；目标已预热时，绝不覆盖其原有群对象。^[weyl-context-identity-and-sharing.md:174-181]

Weyl 元素的二元 `=`、`!=` 和 `*` 在产生结果前检查 WeylGroup 地址；地址不同就抛出 `Weyl group mismatch`，且检查先于 `no_value` gate。因此，兼容性不仅涉及 datum 内容，也取决于规范 owner 的存活期及 `dual()` 的预热历史，不能仅由 Cartan 矩阵或根置换相等判断。参见 [[dual 预热历史与 Weyl 群兼容性]]。^[weyl-context-identity-and-sharing.md:183-187]

## Rust 中的身份实现

已落地的 Rust 修复为 `RootDatumHandle` 引入 `Arc<DatumWeylIdentity>`，通过进程级弱注册表按完整 datum 内容加 preference 驻留身份。身份对象含有仅在成功时发布的惰性 `DatumWeylKernel`（`RootSystem`）与 `AbstractWeylGroup`（`WeylInterface`）两个 cell；八个 handle 构造点均经过私有 `interned` 构造器。结构性的 RootDatum `Eq`/`Debug` 保持不变，因此不能将 Rust 的结构相等直接等同于抽象群身份相同。^[weyl-context-identity-and-sharing.md:249-261]

`dual(RootDatum)` 仅在 canonical target 尚未预热时安装 source 的抽象群，绝不覆盖已预热目标。Weyl 二元关系和乘法先比较抽象群的 `Arc` 身份，再在左侧坐标系重放右侧 external word；关系检查在 `no_value` 级别同样执行。相关语义见 [[Weyl 元素兼容性与跨坐标词重放]] 与 [[Weyl 元素的可失败关系与跨坐标运算]]。^[weyl-context-identity-and-sharing.md:252-256]

## 安全共享的约束

弱驻留本身不能解决所有缓存问题。`elliptic.at` 中的临时 WeylElt 可在迭代间释放，因此仅缓存 `Weak` context 可能无法命中；若把包含 handle 的完整 context 强持有回 handle，又会形成 `handle -> context -> handle` 强引用环。后续共享设计提案将坐标内核与抽象群拆为不反向持有 handle 的层，详见 [[Rust Weyl 内核与抽象群的无环所有权模型]]；该提案应与已落地的身份语义修复区分。^[weyl-context-identity-and-sharing.md:222-227, weyl-context-identity-and-sharing.md:263-278]

惰性初始化必须允许失败后重试，不能将失败的 `Diagnostic` 或首次调用的 `SourceSpan` 缓存进 cell；后续错误应定位到当前调用。自动内存管理也不能单独证明驻留键、共享范围或历史语义正确。参见 [[可失败重试的惰性身份初始化]]。^[weyl-context-identity-and-sharing.md:294-297]

## 验证范围

来源记录 AFTER-v5 job `3900050` 以 `COMPLETED 0:0` 完成，修复随后以生产提交 `690c2b92` 落地。cold_dual 完全字节相等，prewarmed_dual 的 stdout、退出码与有序错误摘要一致。验收记录为 `math_weyl_context_core_after_v5_acceptance_2026_10_06.json`，report SHA 前缀为 `3288480d…`；接受范围仍限于 A1 语义，不授予缓存、性能、内存、更高 rank 或更广数学 release。^[weyl-context-identity-and-sharing.md:63-74]

A1 fixture 在 rebind 后仍由 `wc_alias` 保持旧 RootDatum 存活，尚未验证“仅由保存的 WeylElt 维持 datum 生命周期”。Atlas 输出也不能证明 fresh-equal owner 使用独立 coordinate cell，这些性质需要额外 lifetime fixture 与 HPC-only `Weak`/work-count 守卫。截至来源的 2026-10-09 状态，sole-WeylElt lifetime 等后续见证仍为 provisional fixture，不能视为已经通过的覆盖。^[weyl-context-identity-and-sharing.md:313-330]

## Sources

- [weyl-context-identity-and-sharing.md](../../sources/weyl-context-identity-and-sharing.md) — Weyl 对象身份、dual 历史与安全共享边界。
