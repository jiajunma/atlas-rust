---
title: dual 预热历史与 Weyl 群兼容性
summary: canonical dual 仅在目标仍冷时接收源抽象群身份，预热目标不被覆盖，因此构造历史会影响 Weyl 兼容性。
sources:
  - atlas-core-domain-values.md
  - weyl-context-identity-and-sharing.md
kind: concept
createdAt: "2026-10-09T15:16:17.578Z"
updatedAt: "2026-10-10T00:19:25.519Z"
tags:
  - Weyl群
  - 对偶
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
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: dual 预热历史与 Weyl 群兼容性
summary: canonical dual 仅在目标抽象群尚未初始化时接收源群身份；独立预热的目标不会被覆盖，因此 Weyl 兼容性取决于构造历史与对象生命周期。
sources:
  - atlas-core-domain-values.md
  - weyl-context-identity-and-sharing.md
kind: concept
tags:
  - Weyl群
  - 对偶构造
  - 身份管理
aliases:
  - dual-预热历史与-weyl-群兼容性
---

# dual 预热历史与 Weyl 群兼容性

Weyl 元素的兼容性由抽象 Weyl 群的身份决定：Rust 使用群对象的 `Arc` 指针同一性，而不是根数据句柄的结构等值或局部坐标缓存的身份。`dual()` 是否共享群身份取决于 canonical dual 目标的预热状态，因此构造历史与对象生命周期会影响后续 `=`、`!=` 和乘法是否被接受。^[atlas-core-domain-values.md:53-56, weyl-context-identity-and-sharing.md:174-187]

## canonical dual 的冷共享规则

原版 Atlas 的每个活根数据对象都有初始为空的 `W_ptr`，首次调用 `W()` 时构造 Weyl 群。`dual()` 先从弱驻留表取得 canonical dual：若目标的 `W_ptr` 为空，就确保源群已经建立，再把同一个群指针装入目标；若目标已经预热，则保留其现有群指针。因此，独立预热形成的不同群身份不会因后续 `dual()` 调用而合并。^[weyl-context-identity-and-sharing.md:174-181]

Rust 的 `share_group_into_if_cold` 实现同样的规则：仅在目标 canonical dual 仍冷时安装源的抽象群身份，预热目标不被覆盖。这里的“冷”指抽象群尚未初始化，不意味着根数据尚未构造。并发共享发生竞争时，仅一个候选被发布，另一个被丢弃。^[atlas-core-domain-values.md:30-33, weyl-context-identity-and-sharing.md:249-256]

共享抽象群并不合并局部坐标。`DatumWeylIdentity` 分别持有惰性的 `DatumWeylKernel` 与 `AbstractWeylGroup`：前者缓存 owner 局部坐标中的枚举根系，后者提供 canonical-word 接口。两个单元都不回指 handle，避免所有权环，参见 [[Rust Weyl 内核与抽象群的无环所有权模型]]。^[atlas-core-domain-values.md:24-29]

## 弱驻留与生命周期

原版按完整 `PreRootDatum` 内容驻留根数据，包括 simple roots、simple coroots 和 `prefer_coroots`。驻留表只保存弱指针：相同对象仍存活时复用其规范身份，释放后则可在原槽位重建。轻量 key 仍保留在索引中，因此弱驻留不等于整个索引自动清空。原版 RootDatum 的语言等值比较使用这一规范活对象的指针身份。^[weyl-context-identity-and-sharing.md:164-172]

Rust 的 `DATUM_WEYL_IDENTITIES` 按完整内容弱驻留 Weyl 身份。等值 owner 存活时，新构造复用其身份单元；所有强引用消失后，槽位过期，后来等值构造重新开始。表超过 4096 项时才扫描死槽；唯一入口 `RootDatumHandle::interned` 保证等值活数据共享身份。相关机制见 [[RootDatum 弱驻留与规范活对象身份]]。^[atlas-core-domain-values.md:34-41]

根数据的结构等值仍与 Weyl 兼容性分离。`RootDatumHandle::PartialEq` 忽略身份缓存，只比较 `datum`、`lie_type`、`isogeny` 和 `prefers_coroots`；这一结构比较不能替代抽象群身份检查。^[atlas-core-domain-values.md:41-43, atlas-core-domain-values.md:53-56]

身份初始化采用 `WeylIdentityCell<T>` 的 `OnceLock` 与互斥锁，只发布成功结果；失败不占用单元，后续调用可以重试。双重检查与锁防止并发重复初始化，毒化或二次初始化报告 `StructureError::RepInvariantViolation`。失败诊断和首次调用的 `SourceSpan` 不应缓存，错误应定位到当前调用，参见 [[可失败重试的惰性身份初始化]]。^[atlas-core-domain-values.md:21-23, weyl-context-identity-and-sharing.md:294-297]

## 二元关系与跨坐标运算

`weyl_group_compatible` 使用 `Arc::ptr_eq(group)`。二元 `=`、`!=` 和乘积必须在无值门之前检查兼容性，不兼容时报 `Weyl group mismatch`；即使调用方丢弃结果，也不能把不兼容的关系运算求值为普通布尔值。参见 [[Weyl 元素的可失败关系与跨坐标运算]]。^[atlas-core-domain-values.md:89-91, weyl-context-identity-and-sharing.md:249-256]

兼容元素仍可能属于不同局部坐标系统。等值检查先确认群身份，再把右元素的 canonical 外生成元词在左系统重放后比较；乘法也在左侧坐标中重放右词，结果保留左操作数的 owner。外来根置换不得直接比较或复合，详见 [[Weyl 元素兼容性与跨坐标词重放]]。^[atlas-core-domain-values.md:80-82, weyl-context-identity-and-sharing.md:249-256, weyl-context-identity-and-sharing.md:279-284]

`WeylEltContext` 的内部生成元重编号固定上游 canonical-word 的选择。`WeylEltValue` 在构造时计算并冻结 canonical 既约词，`Display` 与 `word` 只读取该词，参见 [[Weyl 元素值的规范词冻结]]。^[atlas-core-domain-values.md:53-58, atlas-core-domain-values.md:92-92]

## 预热必须命中真正的 canonical dual

上游 `PreRootDatum::dualise` 交换 roots/coroots 并翻转 preference；新 RootDatum 从转置 Cartan 矩阵重新编号。相比之下，`DualTag` 元数据 dual 保持原序。两条路径的编号语义应区分，参见 [[Canonical dual 与 DualTag 的根编号差异]]。^[weyl-context-identity-and-sharing.md:75-85]

来源对 G2 的捕获前登记指出，`adjoint(G2,false)` 不是所需 canonical dual，无法占用其冷共享槽位。真正的 G2 预热见证需要显式构造转置内容，即以单位根矩阵、余根矩阵 \(\begin{pmatrix}2&-3\\-1&2\end{pmatrix}\) 和 `false` preference 构造根数据。B2/C2 的情况不同：C2 的固定 Cartan 矩阵正是 B2 的转置，因此 `dual(SC(B2,true))` 与 `adjoint(C2,false)` 内容一致。这些是源码预期，不能当作已完成的高秩捕获结论。^[weyl-context-identity-and-sharing.md:86-100]

原版内类构造也会立即调用 `srd->dual()`，并强持有 primal 与 dual。因此，显式 `dual(RootDatum)` 之外，还需验证内类构造引起的身份共享与目标生命周期。来源将 inner-class-dual 列为后续 provisional fixture，未报告其已通过验收。^[weyl-context-identity-and-sharing.md:81-85, weyl-context-identity-and-sharing.md:95-100]

## 修复历史与证据边界

修复前，Rust 以结构句柄和内部元素判断关系及乘法兼容性，无法表达原版的群指针身份。A1 原版捕获观察到两类差异：冷 canonical dual 在原版中比较相等且允许乘法，Rust 却返回不等并拒绝乘法；独立预热的不兼容 owner 在原版中拒绝关系运算，Rust 却返回布尔值。来源明确将这些行为保留为修复前的历史对照。^[weyl-context-identity-and-sharing.md:140-151, weyl-context-identity-and-sharing.md:189-212]

来源记录 AFTER-v5 job `3900050` 以 `COMPLETED 0:0` 结束，修复以生产提交 `690c2b92` 落地；证据文件为 `tests/reference/hpc/math_weyl_context_core_after_v5_acceptance_2026_10_06.json`。cold-dual 输出完全字节相等，prewarmed-dual 的 stdout、退出码和有序错误摘要一致。该记录的接受范围仅为 A1 限定语义，不授予缓存、性能、内存或更高秩验收。^[weyl-context-identity-and-sharing.md:63-74]

A1 跨 dual 乘法只检查 \(s_0s_0=1\)，不足以发现生成元重编号错误或直接复合外来根置换的问题。其生命周期用例仍有 alias 保持旧 datum 存活，也未证明仅靠保存的 WeylElt 维持 datum 生命周期。G2、B2/C2、两种操作数顺序、内类 dual、无值关系与 sole-WeylElt lifetime 仍需逐项验证，参见 [[Weyl 语义回归的递进验证门禁]]。^[weyl-context-identity-and-sharing.md:313-330]

共享设计本身不能证明性能或内存收益。构建次数需要独立 work-count 测试，速度与资源使用需要后续 A/B 测量；owner 持有 kernel 可能减少同时存活元素的重复对象，也可能使短命元素场景的峰值 RSS 不变甚至上升。^[weyl-context-identity-and-sharing.md:320-335]

## Sources

- [atlas-core-domain-values.md](../../sources/atlas-core-domain-values.md) — 领域值与 Weyl 身份。
- [weyl-context-identity-and-sharing.md](../../sources/weyl-context-identity-and-sharing.md) — Weyl 对象身份、dual 历史与安全共享边界。
