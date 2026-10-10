---
title: Weyl 属主生命周期的可观察性与内部验证
summary: 保留 datum 别名的 rebind 测试不能证明 WeylElt 独自维持属主生命周期，语言输出也不能证明坐标 cell 的分离，须分别补充独占元素夹具及 Weak／构建计数守卫。
sources:
  - weyl-context-identity-and-sharing.md
kind: concept
createdAt: "2026-10-10T01:45:33.432Z"
updatedAt: "2026-10-10T01:45:33.432Z"
tags:
  - 对象生命周期
  - 所有权
  - 测试设计
aliases:
  - weyl-属主生命周期的可观察性与内部验证
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Weyl 属主生命周期的可观察性与内部验证

Weyl 属主生命周期涉及两个验证层面：语言输出可以检验元素兼容性、错误及属主返回等可观察行为；内部验证则需要确认对象是否仍存活、坐标内核是否独立以及构造是否重复。现有 A1 回归不能单独证明“仅由保存的 WeylElt 维持 datum 生命周期”，也不能从输出推断内部坐标 cell 的共享方式。^[weyl-context-identity-and-sharing.md:313-318]

## 属主身份与生命周期

original Atlas 按完整 `PreRootDatum` 内容及 `prefer_coroots` 对根数据进行弱驻留：相同对象仍存活时复用其 `shared_ptr`，对象释放后则在原槽位重建。RootDatum 的语言层等值比较使用这一规范活对象的指针身份。弱驻留不会永久保留不可达的重型 datum，但静态索引仍保留轻量 key，详见 [[RootDatum 弱驻留与规范活对象身份]]。^[weyl-context-identity-and-sharing.md:164-172]

每个 `W_elt_value` 强持有其 root datum，并保存对该 datum 的 Weyl group 的引用，因此元素存活期间 datum 不会失效。二元 `=`、`!=` 和 `*` 在产生结果前比较 WeylGroup 地址，不同则抛出 `Weyl group mismatch`；检查先于 `no_value` gate。兼容性因此同时受规范属主存活期和 `dual()` 预热历史影响，不能仅由根数据结构或根置换决定。^[weyl-context-identity-and-sharing.md:183-187]

`dual()` 通过同一弱驻留表取得规范目标：目标尚未建立 Weyl group 时共享源的 group；目标已预热时绝不覆盖。对象是否仍然存活，因而关系到后续构造复用哪个规范对象及其既有 group 状态，参见 [[dual 预热历史与 Weyl 群兼容性]]。^[weyl-context-identity-and-sharing.md:164-181]

## 可观察测试的覆盖与盲区

A1 fixture 覆盖 same owner、alias、fresh-equal datum、两种 warm-source/cold-target dual 方向，以及重新绑定变量后旧值的寿命；观测接口包括 `word`、`length`、`root_permutation`、`root_datum`、关系与乘法。另一组覆盖预热目标不兼容、owner/dual mismatch、非法生成元及错误后的恢复标记。^[weyl-context-identity-and-sharing.md:301-307]

其中，重新绑定后的场景仍由 `wc_alias` 保持旧 RootDatum 存活。因此，即使保存的 WeylElt 继续正常工作，也无法据此证明它自身足以维持 datum 生命周期。同样，Atlas 输出不能证明 fresh-equal owner 使用独立坐标 cell；这些是现有 fixture 的证据边界。^[weyl-context-identity-and-sharing.md:313-318]

需要补充的验证包括独立的 sole-WeylElt lifetime fixture，以及 HPC-only 的 `Weak` 和 work-count 单元守卫。前者针对仅剩元素持有属主的场景，后两者用于约束语言输出无法直接证明的生命周期和内部构造行为。^[weyl-context-identity-and-sharing.md:313-318, weyl-context-identity-and-sharing.md:326-330]

## Rust 所有权设计与内部验证

已落地的 Rust 修复让 `RootDatumHandle` 携带 `Arc<DatumWeylIdentity>`，其中包含仅成功时发布的惰性 `DatumWeylKernel` 与 `AbstractWeylGroup` cell，并通过进程级弱注册表驻留身份。关系与乘法先比较 abstract-group 的 `Arc` 身份，再在左侧坐标系重放右侧 external word；结构性的 RootDatum `Eq`/`Debug` 保持不变。该修复通过了 A1 限定范围的 AFTER-v5 验收。^[weyl-context-identity-and-sharing.md:249-261]

后续共享设计要求内核层不反向持有 handle，以避免 `handle -> context -> handle` 强引用环；`WeylEltContext` 则持有 handle、坐标内核和抽象群。这里必须区分数学兼容性与坐标存储：兼容性依据抽象群身份，坐标内核不同的兼容元素仍须通过词重放完成运算。来源将这一部分列为后续共享设计提案，不能全部视为已验收实现，参见 [[Rust Weyl 内核与抽象群的无环所有权模型]]。^[weyl-context-identity-and-sharing.md:263-284]

自动内存管理本身不能证明缓存 key、共享范围或历史语义正确。惰性 cell 也不能缓存失败的 `Diagnostic` 或首次调用的 `SourceSpan`：构造失败后必须允许重试，并将错误定位到当前调用，参见 [[可失败重试的惰性身份初始化]]。^[weyl-context-identity-and-sharing.md:294-297]

## 内类构造的生命周期要求

original Atlas 的 `inner_class_value::build` 会立即调用 `srd->dual()`，并由内类属主强持有 primal 与 dual 两个 datum。因此，只修复显式 `dual(RootDatum)` 路径不足以覆盖全部历史语义；内类构造也需要经过相同的规范 dual/identity 路径，并保留等效的目标生命周期。Rust 是否在 context 中增存 dual handle，需要原版支持的历史测试与无环所有权测试共同确定，参见 [[对偶内类与 Weyl 身份共享]]。^[weyl-context-identity-and-sharing.md:286-292]

## 验收范围

截至来源的 2026-10-09 更新，AFTER-v5 已完成并落地，但接受范围仍限于 A1 语义，不授予缓存、性能、内存或更高 rank 的验收。sole-WeylElt lifetime 等后续见证已起草为 provisional fixture，尚需逐个接入 gate；不能把已起草测试当作通过证据。^[weyl-context-identity-and-sharing.md:63-74, weyl-context-identity-and-sharing.md:326-330]

后续流程要求先完成扩展语义 gate，再加入 one-build work-count 测试，最后进行同节点、交替顺序、fresh-process 的 time/CPU/RSS A/B。内存报告还需提供 live owner/kernel/build 数、分配和 peak RSS：共享可减少同时存活元素的重复对象，但属主强持有内核也可能使短命元素场景的峰值内存不降反升，参见 [[Weyl 内核强缓存的驻留内存权衡]]。^[weyl-context-identity-and-sharing.md:320-335]

## Sources

- [weyl-context-identity-and-sharing.md](../../sources/weyl-context-identity-and-sharing.md) — Weyl 对象身份、dual 历史与安全共享边界。
