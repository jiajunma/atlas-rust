---
title: 补全候选的会话级顺序索引（frames/completions.rs）
source: atlas-rust/atlas-core-completions
ingestedAt: 2026-10-10T09:20:00Z
---

# 补全候选的会话级顺序索引

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包覆盖
`crates/atlas-core/src/frames/completions.rs`（112 行，全部）——
`frames` 的私有子模块，为补全查询维护会话拥有的名字顺序索引。对应
[阅读快照](snapshots/2026-10-10-atlas-core-completions.json)。
结构性阅读，不声称语言验收。

## 模型与纪律

模块自述的契约：原版 `buffer.w` 只在**查询时**遍历其 intern 表；本实现
把名字只保留一份，索引与有序条目共享同一个 `Rc<str>` 分配，并按需惰性
提供借用切片 API。这里不捕获任何全局缓存或运行时表；只有命令发表
（publication）改变可见性。

- 数据结构 `CompletionCandidates`：`names: Vec<(Rc<str>, bool)>`
  （名字 + active 位，**intern 顺序**）、`indices: BTreeMap<Rc<str>,
  usize>`（去重反查）、`snapshot: OnceCell<Vec<String>>`（惰性快照）。
- `intern`：已有名字直接回下标；新名字追加为 **inactive**。注释明确：
  非 active/局部名字不改变既有快照——`intern` 永不使快照失效。
- `set_active`：只有 active 位**真的变化**才 `snapshot.take()`；重复
  设置同一状态不失效（测试用指针相等钉住这一点）。
- `snapshot`：`get_or_init` 时过滤 active 名字，保持 intern 顺序——
  即首次词法使用顺序，而不是首次成功定义顺序（与历史 completion 语义
  笔记一致：lexer 在首次出现时 intern，顺序与可见性无关）。
- `replace`（遗留公开 API）：整体重置后按调用方顺序重建，并用
  `OnceCell::from` **绕过索引**直接保存所给快照——保留调用方顺序，
  甚至保留重复名字（测试锚：`["b","a","b"]` 原样返回）。
- 可见性 revival：名字失效后再激活，回到其 intern 原位（测试锚：
  early/late 交替）。

## 测试锚点与证据归属

三个单测：`no_snapshot_until_queried`（惰性 + 指针稳定性 + 失效仅发生在
真实状态变化）、`visibility_and_revival_order`（顺序语义与 `Rc` 共享
计数为 2）、`legacy_snapshot_replacement`（replace 的重置与保序）。
惰性快照机制的行为验收属于 HPC 侧 completion AFTER 门（job 3868661，
限定范围已接受）与后续 command AFTER R2（job 3868782，过载视图修订）；
本包只记录机制本身，不重述或扩展那些证据。

## 边界声明

本包是对当前工作区字节的结构性阅读（git base 与哈希见快照）；原版
`buffer.w` 的行为描述转述自模块注释与历史笔记，未在本次独立重读上游。
