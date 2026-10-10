---
title: Weyl 内核强缓存的驻留内存权衡
summary: 共享内核可能减少同时存活元素的重复对象，但长期属主保留短命元素所需内核可能使峰值 RSS 不降反升，须联合测量存活对象、构建次数、分配与 RSS。
sources:
  - weyl-context-identity-and-sharing.md
kind: concept
createdAt: "2026-10-10T01:45:26.881Z"
updatedAt: "2026-10-10T01:45:26.881Z"
tags:
  - 缓存设计
  - 内存管理
  - 性能测量
aliases:
  - weyl-内核强缓存的驻留内存权衡
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Weyl 内核强缓存的驻留内存权衡

Weyl 内核强缓存的核心权衡，是在减少重复构造的同时延长内核驻留时间。多个 WeylElt 同时存活时，共享 `RootSystem` 应减少重复对象；但对于元素短命、datum owner 长期存活的工作负载，由 owner 强持有内核可能令峰值 RSS 不变甚至略升。因此，共享设计本身不能证明内存节约。^[weyl-context-identity-and-sharing.md:332-335]

## 短命元素与缓存寿命

冻结的 `elliptic.at` 在库加载时，为 G2、F4、E6、E7、E8 各绑定一个 adjoint datum，分别执行 3、9、5、12、30 次 `W_elt(rd,w).matrix.char_poly`。这 59 次调用只保留特征多项式向量，临时 WeylElt 可以在迭代间释放。若缓存仅弱引用随元素释放的 context，就可能无法跨调用命中；而将完整 context 强引用回存到 handle，又会形成 `handle -> context -> handle` 强引用环。^[weyl-context-identity-and-sharing.md:222-227]

历史诊断 probe 对上述五类 datum 恰好记录了相同次数的 `weyl.context` 构造，支持重复构造与调用方之间的对应关系。不过，这些带探针的聚合计时不能作为候选方案的实际加速比，也不能替代无探针的 A/B 测量。^[weyl-context-identity-and-sharing.md:229-232]

## 强持有与弱驻留的职责

original Atlas 使用两层生命周期管理：完整 `PreRootDatum` 内容对应的身份表只保存 `weak_ptr`，活对象可复用，已释放对象可在原槽位重建；每个活 datum 则强持有一个惰性构造的 `shared_ptr<WeylGroup>`。这种安排允许不可达的重型 datum 释放，但静态 `pool/hash` 仍保留轻量 key，因此不能说整个驻留索引会自动清空。参见 [[RootDatum 弱驻留与规范活对象身份]]。^[weyl-context-identity-and-sharing.md:164-181]

来源提出的后续共享设计将坐标内核 `DatumWeylKernel` 与抽象群 `AbstractWeylGroup` 拆开，由 `WeylEltContext` 持有 handle 和两层对象的 `Arc`，两层对象均不反向持有 handle。这样可避免强引用环，同时用 `Weak` 允许驻留槽失效，并只发布完整成功的初始化结果。该设计方向属于 [[Rust Weyl 内核与抽象群的无环所有权模型]]，不能整体视为本轮已经验收的缓存方案。^[weyl-context-identity-and-sharing.md:263-278]

共享范围还受可观察语义约束：`dual()` 只在 canonical target 尚未预热时共享群身份，已经预热的目标不能被覆盖；Weyl 元素兼容性依赖群身份及预热历史。因而不能仅按 Cartan 矩阵合并缓存，并假定行为不变。参见 [[dual 预热历史与 Weyl 群兼容性]]。^[weyl-context-identity-and-sharing.md:174-187, weyl-context-identity-and-sharing.md:208-212]

## 如何验证内存收益

内存报告必须同时记录存活的 owner、kernel 数量、构建次数、分配情况和峰值 RSS。这些指标用于区分减少重复对象与延长对象存活时间的影响，不能仅凭采用 `Arc`、`Weak` 就宣称内存改善。^[weyl-context-identity-and-sharing.md:332-335]

验证顺序应先完成 G2、B2/C2、两个乘法操作数顺序、inner-class dual construction 和 `no_value` relations 等语义门，再加入单次构建的 work-count 测试，最后进行同节点、交替顺序、独立新进程的 time/CPU/RSS A/B。将 59 次构造降至至多 5 次只是调用方层面的工作量假设，不是已经测得的加速结果。^[weyl-context-identity-and-sharing.md:320-324]

元素表示本身也是独立的内存变量：original `WeylElt` 使用固定长度字节数组，而来源所述 Rust 语言层元素保存根置换、逆置换两个 `Vec`，并另存规范词 `Vec`。迁移至紧凑表示可能影响规范词、根作用、provenance 和错误路径，须独立测量，不能与 owner/kernel 修复混为同一项收益。^[weyl-context-identity-and-sharing.md:238-245]

## 当前证据边界

AFTER-v5 job `3900050` 已完成，修复以提交 `690c2b92` 落地，但接受范围仍限于 A1 语义，不授予缓存、性能、内存、rank 或更广数学验收。强缓存是否降低驻留内存，仍须由后续测量回答；相关范围见 [[Weyl 上下文共享的性能与内存证据边界]]。^[weyl-context-identity-and-sharing.md:63-74]

## Sources

- [weyl-context-identity-and-sharing.md](../../sources/weyl-context-identity-and-sharing.md) — Weyl 对象身份、dual 历史与安全共享边界。
