---
title: 原型 Weyl 群的作用像枚举
summary: PrototypeWeylGroup 以元素对全部单根的作用像为键进行 BFS 枚举，成功阶数至多 65_536，超限返回 WeylGroupTooLarge；act_on_root 按词的逆序施加反射。
sources:
  - lib-root.md
kind: concept
createdAt: "2026-10-09T14:58:50.807Z"
updatedAt: "2026-10-09T14:58:50.807Z"
tags:
  - Weyl群
  - 广度优先搜索
  - 群作用
aliases:
  - 原型-weyl-群的作用像枚举
  - 原W群
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 原型 Weyl 群的作用像枚举

`PrototypeWeylGroup` 是 `atlas-real-group` crate 私有 A1 迁移原型层中的 Weyl 群实现。它通过元素对全部单根的作用像组织广度优先搜索（BFS），并对枚举规模设置上限。该原型层全部为 `pub(crate)`，实现注释标明待替换。^[lib-root.md:30-44]

## 作用像作为枚举键

枚举以一个元素对全部单根的作用像为键，采用 BFS 构建群元素集合。因此，枚举时用于识别元素的是这些作用像，而不是表示该元素的词本身。这里的作用像键与原型 `RootDatum::roots()` 中用于枚举根的向量集合属于不同层次：前者枚举群元素，后者计算根的反射闭包。^[lib-root.md:34-44]

相关根数据约定见 [[原型 RootDatum 的构造校验与配对约定]]；根闭包的算术与规模约束见 [[反射闭包的防御性不变量]]。原型 `RootDatum` 与 `root_datum` 模块中的 `BasedRootDatum` 是不同类型，不能因名称相近而混同。^[lib-root.md:34-41, lib-root.md:55-58]

## 容量边界

发现新元素时，若 `elements.len() >= 65_536`，实现返回 `WeylGroupTooLarge`；成功枚举的群阶至多为 `65_536`。该边界针对 Weyl 群元素，与原型根闭包的 `LIMIT = 4096` 及其 `RootSystemTooLarge` 错误相互独立。^[lib-root.md:39-44]

## 词的作用顺序

`act_on_root` 按词的**逆序**依次施加反射。理解或复现作用像键时，必须保留这一顺序约定；相关主题可参见 [[Weyl 词对根与权的作用顺序]]。^[lib-root.md:42-44]

## 测试与证据范围

来源列出三个文件级测试锚点：A1 反射取负、固定根按标志判为非紧虚根，以及 `i32::MAX` 坐标配对准确返回 `ArithmeticOverflow`。这份测试清单没有列出专门验证 Weyl 群 BFS 枚举、容量边界或逆序作用的测试，因此不能据此声称这些行为已得到全面测试覆盖。^[lib-root.md:51-58]

本页依据的是 crate 门面与原型层的结构性阅读，不代表数学验收；该次知识维护未执行 Atlas、Cargo、测试或 benchmark。原型层的整体定位见 [[A1 迁移原型层与对偶格类型设计]]。^[lib-root.md:9-14, lib-root.md:30-33, lib-root.md:62-66]

## Sources

- [lib-root.md](lib-root.md)：crate 根的模块组织、再导出面与 A1 原型层，包含 `PrototypeWeylGroup` 的枚举键、容量限制及作用顺序。
