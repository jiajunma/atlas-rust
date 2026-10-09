---
title: TypeTable 修订身份与缓存失效
summary: revision 使用克隆共享、突变更换的 Arc<()> 标识非语义快照，缓存读者持有该身份以防地址复用，并保持 TypeTable 的 Send+Sync。
sources:
  - atlas-core-types.md
kind: concept
createdAt: "2026-10-09T14:38:15.589Z"
updatedAt: "2026-10-09T20:46:01.114Z"
tags:
  - 缓存
  - 修订身份
  - Rust
aliases:
  - typetable-修订身份与缓存失效
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: TypeTable 修订身份与缓存失效
summary: TypeTable 以克隆共享、突变更换的 Arc<()> 表示非语义快照身份；缓存读者持有该身份以识别修订并防止地址复用，同时保持 Send+Sync。
sources:
  - atlas-core-types.md
kind: concept
tags:
  - 缓存
  - 修订管理
  - Rust
aliases:
  - typetable-修订身份与缓存失效
---

# TypeTable 修订身份与缓存失效

`TypeTable` 使用 `revision: Arc<()>` 表示**非语义的快照身份**：克隆表时共享身份，表发生突变时更换身份。缓存读者持有这一身份，用于区分表的修订，并防止旧身份的地址被复用。来源将其列为跨命令缓存设计。^[atlas-core-types.md:39-47]

## 修订身份与类型身份

`TypeTable` 保存绑定序列 `bindings`、活跃名字映射 `active`、构造器信息 `constructors` 和修订身份 `revision`。保留定义与活跃名字具有不同生命周期：`forget` 只摘除活跃名字，已存值仍保留旧类型；字段与标签查询则在全部保留定义中进行。相关说明见 [[TypeTable 的稳定身份与活跃绑定]]。^[atlas-core-types.md:41-53]

修订身份描述表的快照，不承担类型的语义等值判定。类型等值由 `equivalent` 处理：先校验两侧构造器应用，再进行结构递归，并以递归名作为终止边界。参见 [[Type 类型表示与语义等值]]。^[atlas-core-types.md:33-35, atlas-core-types.md:45-47]

## 缓存失效机制

克隆共享 `revision`，使克隆后的表保留相同的快照身份；突变更换 `revision`，为缓存读者提供识别修订变化的依据。缓存读者继续持有旧 `Arc`，可阻止其身份地址被复用；采用 `Arc` 也保持了 `TypeTable` 的 `Send + Sync` 性质。^[atlas-core-types.md:45-47]

表的修改入口包括 `add`、`update`、`add_constructor`、`forget` 和 `add_simple`。其中，`add`／`update` 支持递归定义所需的两阶段安装：先注册全组名字，再解析各个右侧定义；`add_simple` 按名字、定义与 arity 去重，并搬移字段元数据。理解修订变化时，需要同时考虑定义和元数据，而不只关注活跃名字。^[atlas-core-types.md:48-53]

## 使用上下文与证据边界

来源将 `revision` 身份设计描述为已验收的跨命令缓存设计，并指向命令 AFTER R2、`tests/reference/hpc/` 中的命令链验收记录及 `docs/HANDOFF.md` 索引。来源也指出 `TypedContext` 消费该表，但本包未展开具体缓存条目的实现。^[atlas-core-types.md:45-47, atlas-core-types.md:86-89]

来源列出 4 个 `revision_tests` 测试；整份材料属于结构性阅读，不声称语言验收。上游行号是实现方的移植陈述，类型行为兼容仍以 HPC 语言语料门为准，因此不能将此处记载的缓存设计验收扩大为整个类型系统的兼容性结论。^[atlas-core-types.md:9-18, atlas-core-types.md:86-91]

## Sources

- [atlas-core-types.md](../../sources/atlas-core-types.md)：类型模型（types.rs + types/）——Type 面、TypeTable 与二阶机器。
