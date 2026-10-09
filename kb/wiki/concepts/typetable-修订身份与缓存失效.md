---
title: TypeTable 修订身份与缓存失效
summary: revision 通过克隆共享、突变更换的 Arc<()> 提供非语义快照身份，供缓存读者识别修订并防止地址复用，同时保持 Send+Sync。
sources:
  - atlas-core-types.md
kind: concept
createdAt: "2026-10-09T14:38:15.589Z"
updatedAt: "2026-10-09T14:38:15.589Z"
tags:
  - 缓存
  - 修订管理
  - Rust
aliases:
  - typetable-修订身份与缓存失效
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# TypeTable 修订身份与缓存失效

`TypeTable` 通过 `revision: Arc<()>` 表示非语义的快照身份：克隆共享该身份，发生突变时则替换身份。缓存读者可以据此识别表的修订变化，避免把旧快照的缓存结果用于更新后的表。^[atlas-core-types.md:39-47]

## 修订身份与类型身份

`TypeTable` 同时保存绑定序列 `bindings`、活跃名字映射 `active`、构造器信息 `constructors` 和修订身份 `revision`。修订身份描述表的快照，不承担类型的语义等值判定；后者由 `equivalent` 处理，并以递归名作为递归比较的终止边界。相关概念见 [[TypeTable 的稳定身份与活跃绑定]] 与 [[Type 类型表示与语义等值]]。^[atlas-core-types.md:33-47]

## 缓存失效机制

克隆共享 `revision`，而突变会生成新的修订身份。缓存读者持有该 `Arc`，使旧身份在仍被引用时保持存活，从而阻止地址复用；`Arc` 的选择也保留了 `TypeTable` 的 `Send + Sync` 性质。这里需要保留的是可比较的快照身份，而非具有类型语义的修订内容。^[atlas-core-types.md:45-47]

表的修改入口包括 `add`、`update`、`add_constructor`、`forget` 和 `add_simple`。其中，`forget` 只移除活跃名字，已存值仍保留旧类型；`add_simple` 则按名字、定义和 arity 去重，并搬移字段元数据。因此，理解修订变化时，需要同时区分活跃名字的变化与保留定义的存在。^[atlas-core-types.md:48-53]

## 使用上下文与证据边界

该修订身份设计用于跨命令缓存，来源将其标为已验收设计，并指向命令 AFTER R2、`tests/reference/hpc/` 的命令链验收记录及 `docs/HANDOFF.md` 索引。`TypedContext` 是该表的消费方，相关会话概念见 [[TypedContext 会话状态与启动初始化]]。^[atlas-core-types.md:45-47, atlas-core-types.md:86-89]

来源列出 4 个 `revision_tests` 测试，但本包整体属于结构性阅读，不声称完成语言验收。上游行号仅代表实现方的移植陈述，类型行为兼容仍以 HPC 语言语料门为准；不能将修订身份的缓存验收扩大为整个类型系统的兼容性结论。^[atlas-core-types.md:9-18, atlas-core-types.md:86-91]

## Sources

- [atlas-core-types.md](atlas-core-types.md)
