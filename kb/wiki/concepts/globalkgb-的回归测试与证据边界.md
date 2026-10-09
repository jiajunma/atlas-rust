---
title: GlobalKgb 的回归测试与证据边界
summary: 来源记录三个 A1/B2 逐字节打印测试及一个 B2 结构测试，缺少错误分支与半单秩零覆盖；本次未执行测试或核对上游字节，不构成数学验收。
sources:
  - global-kgb.md
kind: concept
createdAt: "2026-10-09T14:49:48.238Z"
updatedAt: "2026-10-09T20:53:48.389Z"
tags:
  - 回归测试
  - 证据边界
  - KGB
aliases:
  - globalkgb-的回归测试与证据边界
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: GlobalKgb 的回归测试与证据边界
summary: GlobalKgb 包含三个 A1/B2 逐字节打印回归测试与一个 B2 结构不变量测试，未覆盖错误分支和半单秩零；源包记录的是结构性阅读，不能据此宣称新的测试通过或数学验收。
sources:
  - global-kgb.md
kind: concept
tags:
  - 回归测试
  - 证据范围
  - 实现限制
aliases:
  - globalkgb-的回归测试与证据边界
---

# GlobalKgb 的回归测试与证据边界

`GlobalKgb` 枚举同一内类全部强实形的 KGB 元素，并按扭对合划分 tau 包。源码包含四个测试：三个小秩实例的逐字节打印比较，以及一个 B2 结构不变量测试。源包属于结构性阅读记录，不声称数学验收。^[global-kgb.md:9-15, global-kgb.md:94-105]

## 打印回归

三个打印测试分别对照 `tests/reference/domain/print_x.events.json` 中对应的 `print_X` 块：单连通（SC）A1 为 5 行，打印头为 `[1]/4`；伴随（adjoint）A1 为 3 行，打印头为 `[1]/2`；单连通 B2 为 17 行，打印头为 `[0,3]/4`，并包含负分子标签 `[0,-1]/2`。比较采用逐字节匹配。^[global-kgb.md:96-98]

B2 元素 15 的 `[0,-1]/2` 还锚定了[[全局环面元素的算术历史表示]]：构造入口会约化环面坐标，但 `simple_reflect` 后故意不再约化，因此输出允许保留负分子。^[global-kgb.md:19-25]

打印版式包含元素号宽度 `digits(size−1)`、按末行位数确定的 Cartan 类号与长度宽度、标签宽度 `3·lattice_rank+3`，以及用 `*` 表示缺失的 Cayley 链接。`GlobalKgb` 仅派生 `Clone, Debug`，没有 `Eq`，快照比较借助 `GlobalKgbPrint`；接口与版式细节见[[GlobalKgb 查询接口与 print_X 布局兼容]]。^[global-kgb.md:89-92]

## B2 结构不变量

第四个测试检查 B2 的 17 个元素、tau 包大小 `[8,2,2,2,2,1]`，以及依次排列的包字 `["e","1^e","2^e","1x2^e","2x1^e","1^2x1^e"]`；同时检查 cross 的对合性与 Cayley 配对。这些是[[GlobalKgb 的分阶段广度优先构造]]在 B2 实例上的结构回归锚点。^[global-kgb.md:98-100]

## 未覆盖路径与失败边界

现有测试没有覆盖错误分支。半单秩 0 的平凡群与一维环面也有意未测：共享内类机制在空生成元集合上会于 `weyl_transducer.rs:485` 触发 panic，修复超出该模块范围。^[global-kgb.md:101-103]

实现还依赖维度匹配、`denominator != 0` 和直接下标索引有效等隐式前置条件，违反时会 panic。`reduce_raw`、`evaluate_at` 等位置的 `2 * denominator` 使用普通乘法，溢出时 debug 模式 panic，release 模式回绕。^[global-kgb.md:103-105]

查询与打印的错误语义也存在差异：访问器使用 `.get`，越界返回 `None`；`torus_label()` 将 `log_2pi` 的错误转为 `None`，而 `print_layout` 传播同一错误。源包记录了这些实现行为，但没有相应错误分支测试。^[global-kgb.md:84-88, global-kgb.md:101-105]

## 证据来源与适用范围

源包覆盖 `crates/atlas-real-group/src/global_kgb.rs` 的 1370 行。上游从任意 `GlobalTitsElement` 播种的第二构造器及 Bruhat/Hasse 层尚未移植；材料中的上游行号仅转录自代码注释，未核对上游文件字节。^[global-kgb.md:10-15]

精确读取身份记录在 `snapshots/2026-10-06-global-kgb.json` 中，绑定 Git base、文件字节 SHA-256 与 Kimi 调用记录。草案基于完整字节起草，再由维护者对照源码逐条核对改写。本次知识维护没有执行 Atlas、Cargo、测试或 benchmark，因此上述内容描述的是源码中的测试覆盖与限制，不是一次新的测试运行结果。^[global-kgb.md:109-113]

## Sources

- [global-kgb.md](../../sources/global-kgb.md) — 内类范围 KGB 图与 print_X 布局（global_kgb.rs）。
