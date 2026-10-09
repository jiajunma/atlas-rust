---
title: 语言层 Weyl 元素的紧凑表示迁移边界
summary: 语言层 Weyl 元素仍持有根置换、逆置换及规范词向量，迁往已有 CompactWeyl 固定数组表示须独立验证词序、根作用、来源与错误路径，A1 身份验收不证明性能或内存收益。
sources:
  - weyl-context-identity-and-sharing.md
kind: concept
createdAt: "2026-10-09T21:14:06.184Z"
updatedAt: "2026-10-09T21:14:06.184Z"
tags:
  - weyl
  - rust
  - representation
  - performance
aliases:
  - 语言层-weyl-元素的紧凑表示迁移边界
  - 语W元
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# 语言层 Weyl 元素的紧凑表示迁移边界

语言层 Weyl 元素的紧凑表示迁移，是把 Rust `W_elt` 的元素存储改为已有紧凑表示的后续优化方向。它独立于已落地的 owner/kernel 身份语义修复，可能影响规范词、根作用、构造来源与错误路径，必须单独验证；A1 身份语义捕获不能证明这种迁移的速度或内存收益。^[weyl-context-identity-and-sharing.md:238-245]

## 当前表示与迁移目标

original Atlas 的 `WeylElt` 使用固定 `RANK_MAX` 长度的 `unsigned char` 数组，没有元素自身的堆分配。来源记录的 Rust 语言层 `WeylElement` 则保存根置换及其逆置换两个 `Vec`，以及长度；外层 `WeylEltValue` 还保存规范词 `Vec`。仓库已有内部 `CompactWeyl` 和 `[u8; 32]` transducer 元素，但语言层 `W_elt` 尚未使用它们。相关表示见 [[WeylElt 的固定数组与容量边界]]、[[WeylElement 的置换表示与长度下降不变量]] 与 [[Weyl 群的紧凑 Transducer 表示]]。^[weyl-context-identity-and-sharing.md:238-242]

紧凑表示迁移涉及的不只是存储布局，还包括 [[Weyl 元素的规范词]]、root action、provenance 和错误路径。来源明确要求将其与本轮 owner/kernel 修复分开处理，不能把身份兼容性修复的证据直接用于元素表示替换。^[weyl-context-identity-and-sharing.md:243-245]

## 必须保留的身份与运算语义

original Atlas 的 `W_elt_value` 强持有所属 root datum，并引用该 datum 的 Weyl group。二元 `=`、`!=`、`*` 在产生结果之前比较 WeylGroup 地址；地址不同就抛出 `Weyl group mismatch`，而且检查先于 `no_value` gate。因此，兼容性包含 canonical owner 的存活期和 `dual()` 的预热历史，不能仅由 Cartan 矩阵、结构相等的根数据或相同根置换决定。^[weyl-context-identity-and-sharing.md:183-187]

`dual()` 仅在 canonical target 的 Weyl group 尚未构造时共享 source 的 group；若 target 已预热，则保留其既有 group。这使内容相关的两个 datum 可能因构造历史而具有不同的 Weyl group identity。紧凑元素表示仍须保留这一可观察区别。^[weyl-context-identity-and-sharing.md:174-181]

已落地的 Rust 修复用 abstract-group 的 `Arc` identity 检查二元运算兼容性，并在左侧坐标系重放右侧 external word；关系检查在 `no_value` 级别仍执行。上游乘法结果保留左操作数的 owner。因此，迁移不能以紧凑编码相同替代身份检查，也不能绕过跨坐标系的运算约定。^[weyl-context-identity-and-sharing.md:80-85, weyl-context-identity-and-sharing.md:249-261]

## 与上下文共享优化的区别

上下文重复构造是另一项性能问题：`elliptic.at` 在库加载时，为 G2、F4、E6、E7、E8 的五个 datum 共执行 59 次 `W_elt(rd,w).matrix.char_poly`，仅保留特征多项式向量，临时 WeylElt 可在迭代间释放。来源据此说明，仅缓存弱 context 可能无法命中，而把完整 context 强放回 handle 又会产生 `handle -> context -> handle` 强引用环。这些问题属于上下文生命周期与共享设计，不能与元素紧凑表示的收益混为一谈。^[weyl-context-identity-and-sharing.md:222-227]

内存收益也不能预先认定为单调改善。多个 WeylElt 同时存活时，共享 RootSystem 应减少重复对象；但对于短命元素，owner 强持有 kernel 可能使 peak RSS 不变甚至略升。后续报告需要同时记录 live owner/kernel/build 数、分配和 peak RSS，不能只凭 `Arc`／`Weak` 的设计判断效果。^[weyl-context-identity-and-sharing.md:332-335]

## 验证范围与后续边界

截至来源的 2026-10-09 更新，owner/kernel 修复已通过 AFTER-v5 job `3900050`，并以生产提交 `690c2b92` 落地；接受范围仍限于 A1 语义，不授予缓存、性能、内存、更高 rank 或更广数学 release。这一状态不代表紧凑表示迁移已经完成。^[weyl-context-identity-and-sharing.md:63-74, weyl-context-identity-and-sharing.md:238-245]

A1 的跨 dual 乘法仅检验 \(s_0s_0=1\)，无法发现错误的生成元重编号或直接复合 foreign root permutation。既有 rebind 用例还保留 `wc_alias`，因此没有证明“仅由保存的 WeylElt 维持 datum 生命周期”。这些限制直接影响新表示能否被现有测试充分约束。^[weyl-context-identity-and-sharing.md:313-318]

来源规划的后续语义门包括 G2 非对称 interface-order 见证、B2/C2、两个乘法操作数顺序、inner-class dual construction、`no_value` relations，以及 sole-WeylElt lifetime。相关 G2 区别见 [[Canonical dual 的转置根数据与 G2 预热见证]]；这些后续 fixture 在来源更新时仍为 provisional，不能写成已验证结果。^[weyl-context-identity-and-sharing.md:320-330]

性能验证须在相应语义门通过后开展，并采用同节点、交替顺序、fresh-process 的 time/CPU/RSS A/B。来源中的 `59 -> 至多 5` 仅是上下文构造次数假设，既不是已测加速，也不是元素紧凑表示迁移的收益结论。^[weyl-context-identity-and-sharing.md:320-324]

## Sources

- [weyl-context-identity-and-sharing.md](../../sources/weyl-context-identity-and-sharing.md) — Weyl 对象身份、dual 历史与安全共享边界。
