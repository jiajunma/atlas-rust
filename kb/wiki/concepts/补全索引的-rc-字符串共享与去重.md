---
title: 补全索引的 Rc 字符串共享与去重
summary: 有序名字列表与 BTreeMap 反查索引共享同一 Rc<str> 分配，intern 对已有名字返回原下标，新名字以 inactive 状态追加。
sources:
  - atlas-core-completions.md
kind: concept
createdAt: "2026-10-09T20:30:42.604Z"
updatedAt: "2026-10-09T22:13:06.585Z"
tags:
  - 补全
  - Rust
  - 内存共享
aliases:
  - 补全索引的-rc-字符串共享与去重
  - 补R字
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 补全索引的 Rc 字符串共享与去重
summary: 补全索引的有序条目与反查表共享同一 Rc<str> 分配；intern 去重并以 inactive 状态追加新名字，惰性快照独立维护可见候选。
sources:
  - atlas-core-completions.md
kind: concept
tags:
  - 补全
  - Rust
  - 内存共享
aliases:
  - 补全索引的-rc-字符串共享与去重
provenanceState: extracted
---

# 补全索引的 Rc 字符串共享与去重

补全索引通过共享 `Rc<str>`，让有序条目与名字反查表引用同一份字符串分配，并避免重复登记名字。实现位于 `crates/atlas-core/src/frames/completions.rs`，是 `frames` 的私有子模块，由会话持有，不捕获全局缓存或运行时表。^[atlas-core-completions.md:9-20]

## 存储与去重

`CompletionCandidates` 使用 `names: Vec<(Rc<str>, bool)>` 按 intern 顺序保存名字及其 active 位，使用 `indices: BTreeMap<Rc<str>, usize>` 提供名字到下标的去重反查。两处共享同一个 `Rc<str>` 分配；惰性快照则单独存储为 `snapshot: OnceCell<Vec<String>>`。^[atlas-core-completions.md:17-24]

`intern` 遇到已有名字时直接返回原下标；遇到新名字时，将其追加为 inactive 条目。登记名字与成为可见补全候选是分开的，且 `intern` 永不使既有快照失效。^[atlas-core-completions.md:25-26]

## 顺序、可见性与快照

补全候选保持首次词法使用时的 intern 顺序，而非首次成功定义的顺序。命令发表（publication）改变可见性；名字失效后重新激活，仍回到原来的 intern 位置。这一约定对应 [[会话补全顺序与名字可见性分离]]。^[atlas-core-completions.md:17-20, atlas-core-completions.md:29-36]

借用切片 API 按需提供快照：`snapshot` 通过 `get_or_init` 过滤 active 名字，并保留 intern 顺序。只有 `set_active` 真正改变 active 位时，才调用 `snapshot.take()` 使快照失效；重复设置相同状态不会失效。^[atlas-core-completions.md:17-19, atlas-core-completions.md:24-31]

## 遗留替换接口的例外

遗留公开 API `replace` 整体重置后按调用方顺序重建，并通过 `OnceCell::from` 绕过索引，直接保存给定快照。因此，快照既保留调用方顺序，也可保留重复名字，例如 `["b","a","b"]` 会原样返回。索引的去重性质不能推广为所有快照均无重复项。^[atlas-core-completions.md:32-34]

## 测试与证据边界

三个单元测试分别提供机制锚点：`no_snapshot_until_queried` 覆盖惰性生成、指针稳定性及仅在真实状态变化时失效；`visibility_and_revival_order` 覆盖顺序语义，并检查 `Rc` 共享计数为 2；`legacy_snapshot_replacement` 覆盖替换接口的重置与保序。^[atlas-core-completions.md:38-42]

来源属于当前工作区字节的结构性阅读，不声称语言验收。惰性快照的行为验收另归于 HPC completion AFTER 门及后续 command AFTER R2，本包不重述或扩展这些证据。关于原版 `buffer.w` 的行为描述转述自模块注释与历史笔记，本次未独立重读上游。^[atlas-core-completions.md:9-13, atlas-core-completions.md:43-50]

## Sources

- [atlas-core-completions.md](../../sources/atlas-core-completions.md) — 补全候选的会话级顺序索引（frames/completions.rs）。
