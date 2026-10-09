---
title: Weyl 上下文共享的性能与内存证据边界
summary: 重复上下文构造和采样热点支持优化调查，但构建次数假设与探针计时不证明加速；共享可能延长内核存活，需独立测量构建数、分配及 time/CPU/RSS。
sources:
  - weyl-context-identity-and-sharing.md
kind: concept
createdAt: "2026-10-09T15:17:23.199Z"
updatedAt: "2026-10-09T15:17:23.199Z"
tags:
  - 性能分析
  - 缓存
  - 内存测量
aliases:
  - weyl-上下文共享的性能与内存证据边界
confidence: 1
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Weyl 上下文共享的性能与内存证据边界

Weyl 上下文共享旨在减少同一根数据上重复构造 `RootSystem` 和 `WeylInterface` 的工作，但共享范围同时影响对象身份、dual 预热历史与二元运算的可观察语义。必须区分性能线索、语义修复验收和实际性能测量：截至来源的 2026-10-09 更新，修复已通过 A1 限定语义验收并落地，尚未获得缓存、性能、内存或更高 rank 的验收。^[weyl-context-identity-and-sharing.md:63-74, weyl-context-identity-and-sharing.md:195-212]

## 性能线索及其含义

已接受的 rank-one、包含库加载的 profile（job `3868803`）中，`build_weyl_context` 相关栈约占 unitarity 样本的 14.52–14.58 个百分点、AV-ann 样本的 11.76–12.83 个百分点。这是采样归因，不能解释为独立 kernel 的运行时间，也不能与嵌套的 root-ladder 百分比相加。^[weyl-context-identity-and-sharing.md:214-220]

冻结的 `elliptic.at` 在加载时为 G2、F4、E6、E7、E8 各绑定一个 adjoint datum，分别对 3、9、5、12、30 个 word 调用 `W_elt(rd,w).matrix.char_poly`，共 59 次。调用只保留特征多项式向量，临时 WeylElt 可以在迭代间释放；因此，仅弱引用缓存完整 context 可能无法命中，而让 handle 强持有包含自身的完整 context 又会形成 `handle -> context -> handle` 强引用环。^[weyl-context-identity-and-sharing.md:222-227]

历史诊断 probe 在这五类 datum 上分别记录了 3、9、5、12、30 次 `weyl.context` 构造，聚合计时约为 G2 0.10 ms、F4 3.06 ms、E6 4.24 ms、E7 33.36 ms、E8 345.76 ms。这些记录将重复构造与调用方对应起来，但带探针的聚合计时不能替代无探针 A/B，也不能作为候选实现的实际加速比。^[weyl-context-identity-and-sharing.md:229-232]

已接受的历史端到端结果是 Rust 比 original 慢约 9.15971 倍（rank-one unitarity）和 7.53406 倍（finite AV-ann）。这些结果包含加载与解释器开销，不能归因成 Weyl kernel 可获得的加速空间。同样，v8 捕获中四次数学调用仅耗时 0.00–0.01 秒，而总作业约 345 秒、其中构建占 281.53 秒，不能据此计算 Rust/original 加速比。^[weyl-context-identity-and-sharing.md:159-160, weyl-context-identity-and-sharing.md:234-236]

## 共享必须先满足语义约束

original Atlas 通过弱驻留提供根数据的规范活对象身份，每个活 datum 再惰性强持有 WeylGroup。`dual()` 只在 canonical target 尚未预热时共享 source 的 WeylGroup；若 target 已预热，则保留其原有身份。Weyl 元素的 `=`、`!=`、`*` 在产生结果前检查 WeylGroup 地址，且检查先于 `no_value` gate。因此，兼容性不能仅由 Cartan 矩阵、根数据结构相等或根置换相同决定，参见 [[dual 预热历史与 Weyl 群兼容性]]。^[weyl-context-identity-and-sharing.md:164-187]

已落地的 Rust 修复引入 `DatumWeylIdentity`，分别容纳 success-only lazy 的 `DatumWeylKernel` 与 `AbstractWeylGroup`，并按完整 datum 内容及 preference 弱驻留身份。二元关系和乘法先比较 abstract-group 的 `Arc` 身份，再在左侧坐标系重放右侧 external word。这些机制落实了身份与跨坐标运算规则，但其 A1 语义验收不能自动转化为缓存或性能结论，参见 [[Weyl 元素的可失败关系与跨坐标运算]]。^[weyl-context-identity-and-sharing.md:249-261, weyl-context-identity-and-sharing.md:63-69]

后续共享设计提出，将环境坐标 kernel 与抽象群分成无反向引用的不可变层，使 `WeylEltContext` 持有 handle 和两个 `Arc`，而两个 kernel 均不持有 handle。该设计旨在避免强引用环；来源明确将后续共享方案与本轮已实现的语义修复区分开，不能将完整提案视为已验证实现，参见 [[Rust Weyl 内核与抽象群的无环所有权模型]]。^[weyl-context-identity-and-sharing.md:263-284]

## A1 验收的覆盖上限

AFTER-v5 job `3900050` 以 `COMPLETED 0:0` 完成：cold_dual 完全字节相等，prewarmed_dual 的 stdout、退出码和有序 error summary 一致，验证树随后以生产提交 `690c2b92` 落地。这一结果只接受 A1 限定语义，不授予缓存、性能、内存、rank 或更广数学 release。^[weyl-context-identity-and-sharing.md:63-74]

A1 的跨 dual 乘法仅验证 \(s_0s_0=1\)，无法发现错误的生成元重编号或直接复合外部坐标根置换的问题。现有 rebind 情形仍有 `wc_alias` 保持旧 RootDatum 存活，也没有证明仅由 WeylElt 维持 datum 生命周期。Atlas 输出只能约束可观察语义，不能证明 fresh-equal owner 使用独立 coordinate cell；这些问题需要额外 lifetime fixture 和 HPC-only 的 `Weak`／work-count 守卫。^[weyl-context-identity-and-sharing.md:313-318]

后续验证顺序是先覆盖 G2 非对称 interface-order、B2/C2、两个乘法操作数顺序、inner-class dual construction 与 `no_value` relations，再加入 one-build work-count 测试，最后进行同节点、交替顺序、fresh-process 的 time／CPU／RSS A/B。来源更新时，G2 capture pair 已冻结和彩排，等待提交，其他后续见证仍为 provisional fixture；`59 -> 至多 5` 只是调用方工作次数假设，并非已测加速。^[weyl-context-identity-and-sharing.md:320-330]

## 内存收益与表示优化的边界

共享 RootSystem 在多个 WeylElt 同时存活时应减少重复对象，但 `elliptic.at` 中的值寿命很短，owner 强持有 kernel 可能使 peak RSS 不变甚至略升。因此，内存收益不能预报为单调节省；后续报告必须同时给出存活 owner／kernel 数、构建次数、分配情况和 peak RSS，不能仅以 `Arc`／`Weak` 的设计作为证明。^[weyl-context-identity-and-sharing.md:332-335]

弱驻留也不意味着整个索引自动释放。original 的身份 store 只保存 `weak_ptr`，不可达的重型 datum 可以释放，但静态 `pool/hash` 仍保留轻量 `PreRootDatum` key。描述生命周期与内存行为时，应区分重型对象回收和驻留索引残留，参见 [[RootDatum 弱驻留与规范活对象身份]]。^[weyl-context-identity-and-sharing.md:164-172]

元素表示是另一个必须独立测量的优化方向：original `WeylElt` 使用固定 `RANK_MAX` 的 `unsigned char` 数组，而来源描述的 Rust 语言层 `WeylElement` 保存 root permutation、inverse 两个 `Vec` 和 length，`WeylEltValue` 另存 canonical word `Vec`。已有内部 `CompactWeyl` 与 `[u8; 32]` transducer element 尚未用于语言 `W_elt`；迁移可能影响 canonical word、根作用、provenance 和错误路径，不能与 owner/kernel 修复合并论证，更不能从 A1 身份捕获推断速度或内存收益。^[weyl-context-identity-and-sharing.md:238-245]

## Sources

- [weyl-context-identity-and-sharing.md](../../sources/weyl-context-identity-and-sharing.md) — Weyl 对象身份、dual 历史与安全共享边界。
