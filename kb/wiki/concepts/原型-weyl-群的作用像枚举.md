---
title: 原型 Weyl 群的作用像枚举
summary: PrototypeWeylGroup 以全部单根的作用像为键执行 BFS，成功阶数至多 65_536，act_on_root 按词逆序执行反射。
sources:
  - lib-root.md
kind: concept
createdAt: "2026-10-09T14:58:50.807Z"
updatedAt: "2026-10-10T00:41:54.546Z"
tags:
  - Weyl群
  - 算法
aliases:
  - 原型-weyl-群的作用像枚举
  - 原W群
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 原型 Weyl 群的作用像枚举
summary: PrototypeWeylGroup 以元素对全部单根的作用像为键进行 BFS 枚举，成功阶数至多为 65_536；act_on_root 按词的逆序施加反射。
sources:
  - lib-root.md
kind: concept
tags:
  - Weyl群
  - 轨道枚举
aliases:
  - 原型-weyl-群的作用像枚举
  - 原W群
---

# 原型 Weyl 群的作用像枚举

`PrototypeWeylGroup` 是 `atlas-real-group` 中 [[A1 迁移原型层与对偶格类型设计|A1 迁移原型层]]的 Weyl 群实现，位于 `crates/atlas-real-group/src/lib.rs`。该层全部为 `pub(crate)`，实现注释标明待替换（pending replacement）。群元素通过广度优先搜索（BFS）枚举，以其对全部单根的作用像作为键。^[lib-root.md:10-14, lib-root.md:30-44]

## 作用像与枚举对象

枚举键记录群元素对全部单根的作用像。原型 `RootDatum` 使用单根基坐标：单根为 \(e_i\)，单余根为 Cartan 矩阵的第 \(j\) 列；构造与配对约定见 [[原型 RootDatum 的构造校验与配对约定]]。^[lib-root.md:34-44]

群元素枚举与根枚举的对象不同：`PrototypeWeylGroup` 枚举 Weyl 群元素，而 `RootDatum::roots()` 以正负单根播种，通过 FIFO BFS 构造反射闭包，并将所得根向量按坐标字典序排序。来源只对根枚举明确说明了这一输出排序规则。^[lib-root.md:39-44]

## 容量限制与失败行为

处理新元素时，`PrototypeWeylGroup` 使用 `elements.len() >= 65_536` 检查容量，触发时返回 `WeylGroupTooLarge`；成功枚举的群阶至多为 `65_536`。^[lib-root.md:42-43]

根闭包另有独立限制：`LIMIT = 4096`，第 4097 个互异根向量触发 `RootSystemTooLarge`。根闭包算术使用 `i128` 中间精度并收窄至 `i32`，溢出时报 `ArithmeticOverflow`。这些属于 [[反射闭包的防御性不变量]]，应与群元素枚举的容量限制区分。^[lib-root.md:39-43]

## 词的作用方向

`act_on_root` 按词的**逆序**依次施加反射，即最后一个字母先作用。这是该原型计算根作用时的顺序约定。^[lib-root.md:42-44]

## 测试与证据边界

来源列出的三个文件级测试锚点是：A1 反射取负、固定根按标志判为非紧虚根，以及 `i32::MAX` 坐标配对恰报 `ArithmeticOverflow` 而非回绕。该清单未列出专门针对 Weyl 群 BFS 枚举、容量边界或逆序作用的测试；参见 [[原型层的测试锚点与证据边界]]。^[lib-root.md:51-58]

本页依据 crate 门面与原型层的结构性阅读，不构成数学验收。原型 `RootDatum` 与 `root_datum` 模块中的 `BasedRootDatum` 是不同类型；其他模块实现不在本来源的覆盖范围内。来源所述知识维护未执行 Atlas、Cargo、测试或 benchmark，因此测试锚点不代表该次维护取得了新的运行结果。^[lib-root.md:9-14, lib-root.md:55-58, lib-root.md:62-66]

## Sources

- [lib-root.md](../../sources/lib-root.md)：crate 根：60 模块组织、52 条再导出与 A1 原型层（lib.rs）。
