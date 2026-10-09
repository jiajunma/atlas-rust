---
title: 根系的 BFS 反射闭包枚举
summary: RootDatum::roots 从正负单根出发以 FIFO BFS 求反射闭包，使用 i128 中间运算并检查 i32 收窄溢出，第 4097 个互异向量触发 RootSystemTooLarge，最终按坐标字典序输出。
sources:
  - lib-root.md
kind: concept
createdAt: "2026-10-09T14:58:41.540Z"
updatedAt: "2026-10-09T14:58:41.540Z"
tags:
  - 根系
  - 广度优先搜索
  - 算术安全
  - 确定性
aliases:
  - 根系的-bfs-反射闭包枚举
  - 根B反
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 根系的 BFS 反射闭包枚举

根系的 BFS 反射闭包枚举是 `atlas-real-group` crate 私有 A1 迁移原型层中 `RootDatum::roots()` 的实现方式：以正、负单根为种子，通过先进先出（FIFO）的广度优先搜索构造反射闭包，最终按坐标字典序返回根向量。该原型层全部为 `pub(crate)`，实现注释标明有待替换。^[lib-root.md:30-41]

## 输入与坐标约定

原型 `RootDatum` 使用单根基：单根为标准基向量 \(e_i\)，第 \(j\) 个单余根为 Cartan 矩阵的第 \(j\) 列。`from_basis` 会逐个检查向量秩，再按行主序核对配对关系 \(\langle \mathrm{root}_i,\mathrm{coroot}_j\rangle=\mathrm{cartan}[i][j]\)。这些约定是理解反射闭包所用根与余根坐标的基础，详见 [[原型 RootDatum 的构造校验与配对约定]]。^[lib-root.md:34-41]

## 枚举流程与确定性

`roots()` 从全部正、负单根开始，以 FIFO BFS 扩展反射闭包，并以互异向量计数。枚举完成后，结果按坐标字典序排序，因此返回顺序具有确定性；最终输出顺序不应与 BFS 的发现顺序混同。^[lib-root.md:39-41]

## 算术与容量边界

计算使用 `i128` 中间精度，再收窄到 `i32`；溢出返回 `ArithmeticOverflow`。互异根向量的数量上限为 `LIMIT = 4096`，发现第 4097 个互异向量时返回 `RootSystemTooLarge`。这些检查构成该枚举实现的算术与规模边界，相关背景见 [[反射闭包的防御性不变量]]。^[lib-root.md:39-41]

## 与 Weyl 群枚举的区别

同一原型层中的 `PrototypeWeylGroup` 也采用 BFS，但以群元素对全部单根的作用像为键，枚举对象是 Weyl 群元素。其成功阶数至多为 65,536，`act_on_root` 按词的逆序执行反射。因此，根系闭包枚举与 [[原型 Weyl 群的作用像枚举]] 具有不同的状态对象与容量限制。^[lib-root.md:39-44]

## 测试与证据范围

来源列出三个测试锚点：A1 反射取负、固定根按标志判为非紧虚根，以及 `i32::MAX` 坐标配对准确返回 `ArithmeticOverflow` 而非回绕。这些锚点不构成对全部根系闭包行为的完整验证；来源仅为结构性阅读，不声称数学验收，本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[lib-root.md:9-14, lib-root.md:51-58, lib-root.md:62-66]

这里的原型 `RootDatum` 与 `root_datum` 模块中的 `BasedRootDatum` 并非同一类型。本页的算法描述限定于 crate 根文件中的原型实现；相关迁移背景见 [[A1 迁移原型层与对偶格类型设计]]，测试范围见 [[原型层的测试锚点与证据边界]]。^[lib-root.md:30-38, lib-root.md:55-58]

## Sources

- [lib-root.md](lib-root.md)：crate 根：60 模块组织、52 条再导出与 A1 原型层（lib.rs）。
