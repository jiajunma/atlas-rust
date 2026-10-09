---
title: Weyl 上下文共享的性能与内存证据边界
summary: 重复构造、采样热点及 59 次调用提供优化线索，但不能证明加速或内存节约，后续须独立测量构建次数、分配及 time/CPU/RSS。
sources:
  - weyl-context-identity-and-sharing.md
kind: concept
createdAt: "2026-10-09T15:17:23.199Z"
updatedAt: "2026-10-09T22:53:45.032Z"
tags:
  - 性能分析
  - 内存管理
  - 证据范围
aliases:
  - weyl-上下文共享的性能与内存证据边界
confidence: 1
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Weyl 上下文共享的性能与内存证据边界
summary: 重复构造与采样热点支持优化调查，但不证明加速；A1 身份语义修复已落地，缓存、性能与内存收益仍需独立验证。
sources:
  - weyl-context-identity-and-sharing.md
kind: concept
tags:
  - 性能分析
  - 缓存
  - 内存测量
aliases:
  - weyl-上下文共享的性能与内存证据边界
---

# Weyl 上下文共享的性能与内存证据边界

Weyl 上下文共享旨在减少同一根数据上重复构造 `RootSystem` 和 `WeylInterface` 的工作，但共享范围也影响对象身份、dual 预热历史和二元运算的可观察行为。截至来源的 2026-10-09 更新，身份语义修复已通过 A1 限定验收并落地；这一结果不授予缓存、性能、内存或更高 rank 的验收。^[weyl-context-identity-and-sharing.md:63-74, weyl-context-identity-and-sharing.md:195-212]

## 性能线索

已接受的 rank-one、包含库加载的 profile（job `3868803`）中，`build_weyl_context` 相关栈约占 unitarity 样本的 14.52–14.58 个百分点、AV-ann 样本的 11.76–12.83 个百分点。这是采样归因，不是独立 kernel 的运行时间，也不能与嵌套的 root-ladder 百分比相加。^[weyl-context-identity-and-sharing.md:214-220]

冻结的 `elliptic.at` 在库加载时为 G2、F4、E6、E7、E8 各绑定一个 adjoint datum，分别对 3、9、5、12、30 个 word 调用 `W_elt(rd,w).matrix.char_poly`，合计 59 次。调用只保留特征多项式向量，临时 WeylElt 可以在迭代间释放。因此，仅用 `Weak` 缓存完整 context 可能无法命中；而让 handle 强持有包含自身的完整 context，会形成 `handle -> context -> handle` 强引用环。^[weyl-context-identity-and-sharing.md:222-227]

历史诊断 probe 在上述五类 datum 上分别记录了 3、9、5、12、30 次 `weyl.context` 构造，聚合计时约为 G2 0.10 ms、F4 3.06 ms、E6 4.24 ms、E7 33.36 ms、E8 345.76 ms。这些数据将重复构造与调用方对应起来，但带探针的聚合计时不能替代无探针 A/B，也不能视为候选实现的加速比。^[weyl-context-identity-and-sharing.md:229-232]

已接受的历史端到端结果是 Rust 比 original 慢约 9.15971 倍（rank-one unitarity）和 7.53406 倍（finite AV-ann）。这些结果包含加载与解释器开销，不能归因成 Weyl kernel 的可得加速比。另一次 v8 捕获的四次数学调用仅耗时 0.00–0.01 秒，而总作业约 345 秒，其中构建占 281.53 秒，同样不支持 Rust/original 加速比结论。^[weyl-context-identity-and-sharing.md:159-160, weyl-context-identity-and-sharing.md:234-236]

## 共享的语义前提

original Atlas 通过弱驻留提供根数据的规范活对象身份，每个活 datum 再惰性强持有 WeylGroup。`dual()` 只在 canonical target 尚未预热时共享 source 的 WeylGroup；若 target 已预热，则保留其原有身份。Weyl 元素的 `=`、`!=`、`*` 在产生结果前检查 WeylGroup 地址，且检查先于 `no_value` gate。因此，兼容性不能仅由 Cartan 矩阵、根数据结构相等或根置换相同决定，详见 [[dual 预热历史与 Weyl 群兼容性]]。^[weyl-context-identity-and-sharing.md:164-187]

已落地的 Rust 修复引入 `DatumWeylIdentity`，包含 success-only lazy 的 `DatumWeylKernel` 与 `AbstractWeylGroup` 两个 cell，并按完整 datum 内容及 preference 弱驻留身份。二元关系与乘法先比较 abstract-group 的 `Arc` 身份，再在左侧坐标系重放右侧 external word；关系检查在 `no_value` 级别同样执行。相关机制见 [[Weyl 元素的可失败关系与跨坐标运算]]，其实现状态不能直接证明性能收益。^[weyl-context-identity-and-sharing.md:249-261, weyl-context-identity-and-sharing.md:63-69]

来源将后续共享设计与本轮语义修复明确区分：提案让 `WeylEltContext` 持有 handle、环境坐标 kernel 和抽象群的 `Arc`，而底层对象不反向持有 handle，以避免强引用环。完整提案仍需独立验证，不能整体表述为已验收实现，参见 [[Rust Weyl 内核与抽象群的无环所有权模型]]。^[weyl-context-identity-and-sharing.md:263-284]

惰性初始化也必须保留失败语义：不能将失败的 `Diagnostic` 或首次调用的 `SourceSpan` 缓存进 cell；构造失败后，下次调用必须重试，并将错误定位到当前调用。自动内存管理本身不证明缓存键、共享范围或历史语义正确。^[weyl-context-identity-and-sharing.md:294-297]

## A1 验收与后续验证顺序

来源记录 AFTER-v5 job `3900050` 以 `COMPLETED 0:0` 完成，验收证据文件为 `math_weyl_context_core_after_v5_acceptance_2026_10_06.json`，report SHA 前缀为 `3288480d…`。其中 cold_dual 完全字节相等，prewarmed_dual 的 stdout、退出码和有序 error summary 一致；验证树随后以生产提交 `690c2b92` 落地。接受范围仍限于 A1 语义，不授予缓存、性能、内存、rank 或更广数学 release。^[weyl-context-identity-and-sharing.md:63-74]

A1 的跨 dual 乘法仅为 \(s_0s_0=1\)，不能发现错误的生成元重编号或直接复合外部坐标根置换的问题。现有 rebind 情形仍由 `wc_alias` 保持旧 RootDatum 存活，未证明“仅由保存的 WeylElt 维持 datum 生命周期”。Atlas 输出也不能证明 fresh-equal owner 使用独立 coordinate cell；这些问题需要额外 lifetime fixture 和 HPC-only 的 `Weak`／work-count 守卫。^[weyl-context-identity-and-sharing.md:313-318]

后续顺序是先覆盖 G2 非对称 interface-order、B2/C2、两个乘法操作数顺序、inner-class dual construction 和 `no_value` relations，再加入 one-build work-count 测试，最后进行同节点、交替顺序、fresh-process 的 time／CPU／RSS A/B。`59 -> 至多 5` 只是调用方工作次数假设，并非已测加速，参见 [[Weyl 语义回归的递进验证门禁]]。^[weyl-context-identity-and-sharing.md:320-324]

截至来源更新，G2 capture pair 已冻结和彩排，因隧道中断等待提交；包括 sole-WeylElt lifetime 在内的后续见证仍为 provisional fixture。G2 预登记还指出，`adjoint(G2,false)` 并非 canonical dual，不能形成真正的目标预热见证，需要显式转置内容的后续 fixture；这些属于捕获前预期，不能作为已观察结果。^[weyl-context-identity-and-sharing.md:86-100, weyl-context-identity-and-sharing.md:326-330]

## 内存收益的边界

多个 WeylElt 同时存活时，共享 RootSystem 应减少重复对象；但 `elliptic.at` 中的值寿命很短，owner 强持有 kernel 可能使 peak RSS 不变甚至略升。因此，内存收益不能预报为单调节省。后续报告必须同时给出存活 owner／kernel 数、构建次数、分配情况和 peak RSS，不能仅凭 `Arc`／`Weak` 设计推断内存节约。^[weyl-context-identity-and-sharing.md:332-335]

弱驻留也不意味着整个索引自动释放。original 的身份 store 只保存 `weak_ptr`，不可达的重型 datum 可以释放，但静态 `pool/hash` 仍保留轻量 `PreRootDatum` key。重型对象回收与驻留索引残留应分别描述，参见 [[RootDatum 弱驻留与规范活对象身份]]。^[weyl-context-identity-and-sharing.md:164-172]

## 元素表示是独立优化方向

original `WeylElt` 使用固定 `RANK_MAX` 的 `unsigned char` 数组，没有元素自身的堆分配；来源描述的 Rust 语言层 `WeylElement` 保存 root permutation、inverse 两个 `Vec` 和 length，`WeylEltValue` 另存 canonical word `Vec`。仓库已有内部 `CompactWeyl` 与 `[u8; 32]` transducer element，但语言 `W_elt` 尚未使用。迁移可能影响 canonical word、根作用、provenance 和错误路径，必须独立测量，不能与 owner/kernel 修复混成同一补丁，也不能从 A1 身份捕获推断速度或内存收益。^[weyl-context-identity-and-sharing.md:238-245]

## Sources

- [weyl-context-identity-and-sharing.md](../../sources/weyl-context-identity-and-sharing.md) — Weyl 对象身份、dual 历史与安全共享边界。
