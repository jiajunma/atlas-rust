---
title: TypeTable 修订身份与缓存失效
summary: revision 使用克隆共享、突变更换的 Arc<()> 标识非语义快照，缓存读者持有该身份以防地址复用，并保持 TypeTable 的 Send+Sync。
sources:
  - atlas-core-types.md
kind: concept
createdAt: "2026-10-09T14:38:15.589Z"
updatedAt: "2026-10-10T00:25:19.466Z"
tags:
  - 类型系统
  - 缓存
  - Rust
aliases:
  - typetable-修订身份与缓存失效
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: TypeTable 修订身份与缓存失效
summary: TypeTable 使用克隆共享、突变更换的 Arc<()> 表示非语义快照身份，供缓存识别修订变化；缓存读者持有该身份以防地址复用，并保持 TypeTable 的 Send+Sync。
sources:
  - atlas-core-types.md
kind: concept
tags:
  - 缓存
  - Rust设计
aliases:
  - typetable-修订身份与缓存失效
---

# TypeTable 修订身份与缓存失效

`TypeTable` 通过 `revision: Arc<()>` 表示**非语义的快照身份**：克隆时共享身份，突变时更换身份。缓存读者持有这一身份，以阻止其地址被复用；采用 `Arc` 同时保持 `TypeTable` 的 `Send + Sync` 性质。^[atlas-core-types.md:39-47]

## 修订身份与类型语义

`TypeTable` 包含绑定序列 `bindings`、活跃名字映射 `active`、构造器信息 `constructors` 和修订身份 `revision`。绑定与活跃名字具有不同生命周期：`forget` 只摘除活跃名字，已存值仍保留旧类型；字段与标签元数据查询则遍历全部保留定义。相关背景见 [[TypeTable 的稳定身份与活跃绑定]]。^[atlas-core-types.md:41-53]

修订身份标识表的快照，不承担类型语义等值判断。类型等值由 `equivalent` 处理：先校验两侧构造器应用，再进行结构递归，并以递归名作为终止边界。参见 [[Type 类型表示与语义等值]]。^[atlas-core-types.md:33-35, atlas-core-types.md:45-47]

## 缓存失效依据

克隆共享 `revision`，因此克隆操作本身保留原有快照身份；突变更换 `revision`，为缓存识别表的修订变化提供依据。缓存读者继续持有旧身份，可以阻止旧身份的地址在其使用期间被复用。来源明确记录了这一身份管理约定，但未展开具体缓存条目的匹配、清除或重算算法。^[atlas-core-types.md:45-47]

表的修改接口包括 `add`、`update`、`add_constructor`、`forget` 和 `add_simple`。括号形式的 `set_type` 采用两阶段处理以支持递归：先注册全组名字，再解析各个右侧定义；`add_simple` 则按名字、定义与 arity 去重，并搬移字段元数据。^[atlas-core-types.md:48-50]

## 使用上下文与证据边界

来源将 `revision` 身份设计描述为已验收的跨命令缓存设计，并指向命令 AFTER R2、`tests/reference/hpc/` 中的命令链验收记录及 `docs/HANDOFF.md` 索引。[[TypedContext 会话状态与启动初始化|TypedContext]] 是该表的消费方；本来源未提供具体缓存调用路径。^[atlas-core-types.md:45-47, atlas-core-types.md:86-89]

材料列出 4 个 `revision_tests` 测试，但整份来源属于结构性阅读，不声称语言验收。上游行号属于实现方的移植陈述，类型行为兼容仍以 HPC 语言语料门为准；缓存设计的验收记载不能扩大为整个类型系统的兼容性结论。^[atlas-core-types.md:9-18, atlas-core-types.md:86-91]

## Sources

- [atlas-core-types.md](../../sources/atlas-core-types.md)：类型模型（types.rs + types/）——Type 面、TypeTable 与二阶机器。
