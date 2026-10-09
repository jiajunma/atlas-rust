---
title: dual 预热历史与 Weyl 群兼容性
summary: canonical dual 仅在目标仍冷时接收源抽象群身份，预热目标不被覆盖，因此 Weyl 兼容性受构造历史影响。
sources:
  - atlas-core-domain-values.md
  - weyl-context-identity-and-sharing.md
kind: concept
createdAt: "2026-10-09T15:16:17.578Z"
updatedAt: "2026-10-09T22:16:57.882Z"
tags:
  - Weyl群
  - 对偶构造
  - 身份管理
aliases:
  - dual-预热历史与-weyl-群兼容性
  - D预W群
confidence: 1
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: dual 预热历史与 Weyl 群兼容性
summary: canonical dual 仅在目标抽象群尚未初始化时共享源身份；独立预热的目标不会被覆盖，因此 Weyl 二元关系与乘积的兼容性受构造历史和对象生命周期影响。
sources:
  - atlas-core-domain-values.md
  - weyl-context-identity-and-sharing.md
kind: concept
tags:
  - Weyl群
  - 对偶
  - 惰性初始化
  - 对象身份
aliases:
  - dual-预热历史与-weyl-群兼容性
  - D预W群
provenanceState: extracted
---

# dual 预热历史与 Weyl 群兼容性

Weyl 元素的兼容性由抽象 Weyl 群的 `Arc` 身份决定，而不是根数据句柄的结构相等或局部坐标缓存相同。`dual()` 是否共享这一身份，取决于 canonical dual 目标是否已经初始化抽象群；因此，构造历史与对象存活期会影响后续 `=`、`!=` 和乘积是否被接受。^[atlas-core-domain-values.md:53-56, weyl-context-identity-and-sharing.md:174-187]

## canonical dual 的预热规则

原版 Atlas 的每个活根数据对象都有初始为空的 `W_ptr`，首次调用 `W()` 时才构造 Weyl 群。`dual()` 先取得弱驻留表中的 canonical dual：若目标 `W_ptr` 为空，就确保源群已经建立，并将同一个群指针装入目标；若目标已经预热，则保留其原有指针。独立预热形成的不同群身份不会因后续 `dual()` 调用而合并。^[weyl-context-identity-and-sharing.md:174-181]

Rust 的 `share_group_into_if_cold` 实现这一规则：仅在目标仍冷时装入源的抽象群身份，已预热目标永不被覆盖。并发共享发生竞争时，仅一个候选被发布，另一个被丢弃。这里的“冷”指目标的抽象群尚未初始化，而不是根数据对象尚未构造。^[atlas-core-domain-values.md:30-33, weyl-context-identity-and-sharing.md:249-256]

共享针对抽象群，局部坐标仍属于各自的 datum。`DatumWeylIdentity` 分别保存惰性的 `DatumWeylKernel` 和 `AbstractWeylGroup`：前者缓存 owner 的枚举根系，后者提供 canonical-word 接口。两个单元都不回指 handle，避免形成强引用环，参见 [[Rust Weyl 内核与抽象群的无环所有权模型]]。^[atlas-core-domain-values.md:24-29]

## 弱驻留与生命周期

原版按完整 `PreRootDatum` 内容驻留根数据，包括 simple roots、simple coroots 和 `prefer_coroots`。表中保存弱指针：相同对象仍存活时复用其规范身份；对象释放后，后续构造可以重新建立对象。轻量 key 仍可能留在索引中，因此不能将弱驻留解释为整个索引自动清空。原版 RootDatum 的语言等值比较使用这一规范活对象的指针身份。^[weyl-context-identity-and-sharing.md:164-172]

Rust 的 `DATUM_WEYL_IDENTITIES` 同样按完整内容弱驻留身份。等值 owner 活着时，新构造复用其身份单元；所有强引用消失后，槽位过期，后来等值构造重新开始。表超过 4096 项时才扫描死槽；唯一构造入口 `RootDatumHandle::interned` 保证等值活数据共享 Weyl 身份，相关机制见 [[RootDatum 弱驻留与规范活对象身份]]。^[atlas-core-domain-values.md:34-41]

根数据结构等值与 Weyl 群兼容性保持分离。Rust 的 `RootDatumHandle::PartialEq` 忽略 Weyl 身份缓存，只比较 `datum`、`lie_type`、`isogeny` 和 `prefers_coroots`；因此，不能用句柄结构相等替代群身份检查。^[atlas-core-domain-values.md:41-43, atlas-core-domain-values.md:53-56]

身份初始化使用 `WeylIdentityCell<T>` 的 `OnceLock` 与互斥锁，仅发布成功结果；失败不会占用单元，后续调用可重试。双重检查与锁避免并发重复初始化，毒化或二次初始化报告 `StructureError::RepInvariantViolation`。失败诊断和首次调用的源码位置不能缓存进单元，错误应定位到当前调用，参见 [[可失败重试的惰性身份初始化]]。^[atlas-core-domain-values.md:21-23, weyl-context-identity-and-sharing.md:294-297]

## 兼容检查与跨坐标运算

`weyl_group_compatible` 使用 `Arc::ptr_eq(group)` 判定兼容。二元 `=`、`!=` 与乘积在无值门之前执行兼容检查，不兼容时报 `Weyl group mismatch`；即使调用方丢弃结果，也不能将不兼容关系求值为普通布尔值。相关错误语义见 [[Weyl 元素的可失败关系与跨坐标运算]]。^[atlas-core-domain-values.md:89-91, weyl-context-identity-and-sharing.md:249-256]

共享抽象群的元素仍可能使用不同的局部坐标。比较时，先检查群身份，再将右元素的 canonical 外生成元词在左侧系统中重放；乘积也采用这一方向，并保留左操作数的 owner。外来根置换不得直接比较或复合，详见 [[Weyl 元素兼容性与跨坐标词重放]]。^[atlas-core-domain-values.md:80-82, atlas-core-domain-values.md:123-124, weyl-context-identity-and-sharing.md:279-284]

`WeylEltContext` 中的内部生成元重编号固定上游 canonical-word 的选择。`WeylEltValue` 在构造时计算并冻结 canonical 既约词，`Display` 与 `word` 只读取该词，不重新选择表示。^[atlas-core-domain-values.md:53-58, atlas-core-domain-values.md:92-92]

## canonical dual 的内容必须匹配

预热见证必须构造出真正的 canonical dual 内容。上游 `PreRootDatum::dualise` 交换 roots/coroots 并翻转 preference；新 RootDatum 从转置 Cartan 矩阵重新编号。仅有相同 Lie 类型或使用 `adjoint` 构造器，并不足以保证命中 canonical dual。^[weyl-context-identity-and-sharing.md:75-94]

来源对 G2 的登记预期指出：`adjoint(G2,false)` 不是所需 canonical dual，不能占用其冷共享槽位；真正的预热见证需要显式构造转置内容。B2/C2 则不同，C2 的固定 Cartan 矩阵正是 B2 的转置，因此 `dual(SC(B2,true))` 与 `adjoint(C2,false)` 内容一致。这些是捕获前的源码预期，不是已完成的高秩验证结论，参见 [[Canonical dual 的转置根数据与 G2 预热见证]]。^[weyl-context-identity-and-sharing.md:86-100]

原版内类构造也会立即调用 `srd->dual()`，并强持有 primal 与 dual。因而，验证预热历史不能只覆盖显式 `dual(RootDatum)`；内类构造引起的身份共享和目标生命周期也需要独立见证。来源将 inner-class-dual 列为后续 provisional fixture，未将其报告为已验收结果。^[weyl-context-identity-and-sharing.md:81-85, weyl-context-identity-and-sharing.md:95-100]

## 修复历史与证据范围

修复前，Rust 以结构句柄和内部元素判断关系及乘法兼容性，无法表达原版群指针身份。A1 原版捕获观察到两类相反差异：冷 canonical dual 在原版中允许比较和乘法，Rust 却拒绝乘法；独立预热的不兼容 owner 在原版中拒绝关系运算，Rust 却返回布尔值。来源明确将这些行为保留为历史对照，两类差异已由落地修复消除。^[weyl-context-identity-and-sharing.md:140-151, weyl-context-identity-and-sharing.md:189-212]

来源记录 AFTER-v5 job `3900050` 以 `COMPLETED 0:0` 结束，修复以生产提交 `690c2b92` 落地。其验收记录为 `tests/reference/hpc/math_weyl_context_core_after_v5_acceptance_2026_10_06.json`：cold-dual 输出完全字节相等，prewarmed-dual 的 stdout、退出码与有序错误摘要一致。接受范围仅为 A1 限定语义，不授予缓存、性能、内存或更高秩验收。^[weyl-context-identity-and-sharing.md:63-74]

A1 跨 dual 乘积只检验 \(s_0s_0=1\)，不足以发现生成元重编号错误或直接复合外来根置换的问题；其生命周期用例仍有 alias 保持旧 datum 存活，也未证明仅由 WeylElt 维持 datum 生命周期。G2、B2/C2、反向操作数、内类 dual、无值关系和 sole-WeylElt lifetime 仍需后续逐项验证。缓存构建次数、速度与内存收益必须另行测量，参见 [[Weyl 语义回归的递进验证门禁]] 和 [[Weyl 上下文共享的性能与内存证据边界]]。^[weyl-context-identity-and-sharing.md:313-335]

## Sources

- [atlas-core-domain-values.md](../../sources/atlas-core-domain-values.md) — 领域值与 Weyl 身份。
- [weyl-context-identity-and-sharing.md](../../sources/weyl-context-identity-and-sharing.md) — Weyl 对象身份、dual 历史与安全共享边界。
