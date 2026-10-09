---
title: 原型 Weyl 群的作用像枚举
summary: PrototypeWeylGroup 以全部单根的作用像为键进行 BFS，成功阶数至多 65_536，act_on_root 按词的逆序执行反射。
sources:
  - lib-root.md
kind: concept
createdAt: "2026-10-09T14:58:50.807Z"
updatedAt: "2026-10-09T22:38:04.471Z"
tags:
  - Weyl群
  - 轨道枚举
aliases:
  - 原型-weyl-群的作用像枚举
  - 原W群
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 原型 Weyl 群的作用像枚举
summary: PrototypeWeylGroup 以元素对全部单根的作用像为键进行 BFS 枚举，成功阶数至多为 65_536；act_on_root 按词的逆序施加反射。
sources:
  - lib-root.md
kind: concept
tags:
  - Weyl群
  - 广度优先搜索
  - 群作用
aliases:
  - 原型-weyl-群的作用像枚举
  - 原W群
---

# 原型 Weyl 群的作用像枚举

`PrototypeWeylGroup` 是 `atlas-real-group` crate 中 [[A1 迁移原型层与对偶格类型设计|A1 迁移原型层]]的 Weyl 群实现。该层全部为 `pub(crate)`，实现注释标明待替换（pending replacement）。其枚举采用广度优先搜索（BFS），以群元素对全部单根的作用像作为键。^[lib-root.md:10-14, lib-root.md:30-44]

## 作用像与枚举对象

枚举键记录一个群元素对全部单根的作用像，用于识别群元素。原型采用单根基坐标，单根为 \(e_i\)，单余根为 Cartan 矩阵的第 \(j\) 列；构造约定见 [[原型 RootDatum 的构造校验与配对约定]]。^[lib-root.md:34-44]

群元素枚举与 `RootDatum::roots()` 的根枚举具有不同的对象：前者枚举 Weyl 群元素，后者以正负单根播种，通过 FIFO BFS 构造反射闭包，并将所得根向量按坐标字典序排序。原型 `RootDatum` 与 `root_datum` 模块中的 `BasedRootDatum` 也是不同类型。^[lib-root.md:39-44, lib-root.md:55-58]

## 容量限制与失败行为

枚举新元素时，容量检查条件为 `elements.len() >= 65_536`，触发时返回 `WeylGroupTooLarge`；成功枚举的群阶至多为 `65_536`。^[lib-root.md:42-43]

根闭包具有独立的容量限制：`LIMIT = 4096`，第 4097 个互异根向量触发 `RootSystemTooLarge`。根闭包计算还使用 `i128` 中间精度并收窄至 `i32`，溢出时报 `ArithmeticOverflow`；这些约束属于 [[反射闭包的防御性不变量]]。^[lib-root.md:39-41]

## 词的作用方向

`act_on_root` 按词的**逆序**依次施加反射，即最后一个字母先作用。这是该原型计算根作用时的顺序约定。^[lib-root.md:42-44]

## 测试与证据边界

来源列出的三个文件级测试锚点为：A1 反射取负、固定根按标志判为非紧虚根，以及 `i32::MAX` 坐标配对恰报 `ArithmeticOverflow` 而非回绕。该清单未列出专门针对 Weyl 群 BFS 枚举、容量边界或逆序作用的测试；相关范围见 [[原型层的测试锚点与证据边界]]。^[lib-root.md:51-58]

本页依据 crate 门面与原型层的结构性阅读，不构成数学验收。来源所述知识维护未执行 Atlas、Cargo、测试或 benchmark，因此上述测试锚点不代表该次维护取得了新的测试运行结果。^[lib-root.md:9-14, lib-root.md:62-66]

## Sources

- [lib-root.md](../../sources/lib-root.md)：crate 根：60 模块组织、52 条再导出与 A1 原型层（lib.rs）。
