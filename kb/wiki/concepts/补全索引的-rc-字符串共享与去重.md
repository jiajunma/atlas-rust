---
title: 补全索引的 Rc 字符串共享与去重
summary: 有序名字列表与 BTreeMap 反查索引共享同一 Rc<str> 分配，intern 对已有名字返回原下标，新名字以 inactive 状态追加。
sources:
  - atlas-core-completions.md
kind: concept
createdAt: "2026-10-09T20:30:42.604Z"
updatedAt: "2026-10-10T00:15:33.902Z"
tags:
  - 补全
  - rust
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
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
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

补全索引通过共享 `Rc<str>`，让有序名字条目与反查表引用同一份字符串分配，并避免重复登记名字。实现位于 `crates/atlas-core/src/frames/completions.rs`，属于 `frames` 的私有子模块，由会话持有，不捕获全局缓存或运行时表。^[atlas-core-completions.md:9-20]

## 存储与去重

`CompletionCandidates` 使用 `names: Vec<(Rc<str>, bool)>` 按 intern 顺序保存名字及其 active 位，使用 `indices: BTreeMap<Rc<str>, usize>` 提供名字到下标的去重反查。两者共享同一个 `Rc<str>` 分配；面向查询的惰性快照则存储为 `snapshot: OnceCell<Vec<String>>`。^[atlas-core-completions.md:17-24]

`intern` 遇到已有名字时直接返回原下标；遇到新名字时，将其追加为 inactive 条目。登记名字与成为可见候选相互分离：非 active 名字或局部名字不会改变既有快照，`intern` 永不使快照失效。^[atlas-core-completions.md:25-26]

## 顺序、可见性与快照

候选顺序采用首次词法使用时的 intern 顺序，而非首次成功定义顺序。只有命令发表（publication）改变可见性；名字失效后重新激活，仍回到原来的 intern 位置。相关语义见 [[会话补全顺序与名字可见性分离]]。^[atlas-core-completions.md:17-20, atlas-core-completions.md:29-36]

借用切片 API 按需提供快照：`snapshot` 通过 `get_or_init` 过滤 active 名字，并保持 intern 顺序。`set_active` 只有在 active 位真正变化时才调用 `snapshot.take()`；重复设置同一状态不会使快照失效。^[atlas-core-completions.md:17-19, atlas-core-completions.md:24-31]

## 遗留替换接口的例外

遗留公开 API `replace` 整体重置后按调用方顺序重建，并通过 `OnceCell::from` 绕过索引，直接保存所给快照。因此，该快照保留调用方顺序，也保留重复名字，例如 `["b","a","b"]` 会原样返回。索引去重与遗留快照保留重复项是不同的接口行为。^[atlas-core-completions.md:32-34]

## 测试与证据边界

三个单元测试提供机制锚点：`no_snapshot_until_queried` 覆盖惰性生成、指针稳定性以及仅在真实状态变化时失效；`visibility_and_revival_order` 覆盖可见性与重新激活的顺序，并检查 `Rc` 共享计数为 2；`legacy_snapshot_replacement` 覆盖 `replace` 的重置与保序。^[atlas-core-completions.md:38-42]

来源属于当前工作区字节的结构性阅读，不声称语言验收。惰性快照机制的行为验收另归于 HPC completion AFTER 门（job `3868661`，限定范围已接受）及后续 command AFTER R2（job `3868782`，过载视图修订）；本包不重述或扩展这些证据。^[atlas-core-completions.md:9-13, atlas-core-completions.md:43-49]

来源对原版 `buffer.w` 查询时遍历 intern 表的描述，转述自模块注释与历史笔记，本次未独立重读上游。^[atlas-core-completions.md:17-20, atlas-core-completions.md:49-50]

## Sources

- [atlas-core-completions.md](../../sources/atlas-core-completions.md) — 补全候选的会话级顺序索引（frames/completions.rs）。
