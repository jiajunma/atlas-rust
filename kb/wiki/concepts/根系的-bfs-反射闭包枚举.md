---
title: 根系的 BFS 反射闭包枚举
summary: 原型 roots 从正负单根执行 FIFO BFS，以 i128 中间精度和受检 i32 收窄计算反射，第 4097 个互异向量触发限制，结果按坐标字典序输出。
sources:
  - lib-root.md
kind: concept
createdAt: "2026-10-09T14:58:41.540Z"
updatedAt: "2026-10-09T22:37:51.578Z"
tags:
  - 根系
  - 广度优先搜索
  - 算术安全
aliases:
  - 根系的-bfs-反射闭包枚举
  - 根B反
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 根系的 BFS 反射闭包枚举
summary: 原型 RootDatum::roots 从正负单根出发，以 FIFO BFS 构造反射闭包；采用 i128 中间运算与受检 i32 收窄，最多容纳 4096 个互异向量，最终按坐标字典序输出。
sources:
  - lib-root.md
kind: concept
tags:
  - 根系
  - 广度优先搜索
  - 算术安全
  - 确定性
---

# 根系的 BFS 反射闭包枚举

根系的 BFS 反射闭包枚举是原型 `RootDatum::roots()` 的实现方式：从正、负单根出发，以先进先出（FIFO）的广度优先搜索构造反射闭包，最终按坐标字典序返回根向量。该实现属于 `atlas-real-group` 的 [[A1 迁移原型层与对偶格类型设计|A1 迁移原型层]]，仅在 crate 内可见，且实现注释标明待替换。^[lib-root.md:30-41]

## 输入与坐标约定

原型 `RootDatum` 使用单根基坐标：单根为标准基向量 \(e_i\)，第 \(j\) 个单余根为 Cartan 矩阵的第 \(j\) 列。`from_basis` 先逐个检查向量维度，不符时返回 `RankMismatch`，再按行主序核对配对关系 \(\langle \mathrm{root}_i,\mathrm{coroot}_j\rangle=\mathrm{cartan}[i][j]\)，不符时返回 `RootPairingMismatch`。详见 [[原型 RootDatum 的构造校验与配对约定]]。^[lib-root.md:34-38]

## 搜索与输出顺序

`roots()` 以正负单根播种，通过 FIFO BFS 扩展反射闭包，并按互异向量计数。完成枚举后，结果按坐标字典序排序，保证输出的确定性；FIFO 规定搜索顺序，坐标字典序规定最终返回顺序。^[lib-root.md:39-41]

## 算术与容量边界

闭包计算采用 `i128` 中间精度，再收窄至 `i32`；发生溢出时返回 `ArithmeticOverflow`。实现设定 `LIMIT = 4096`，发现第 4097 个互异向量时返回 `RootSystemTooLarge`。该限制约束的是互异根向量数量。^[lib-root.md:39-41]

同一原型层中的 [[原型 Weyl 群的作用像枚举|PrototypeWeylGroup]] 也使用 BFS，但以群元素对全部单根的作用像为键，成功构造的群阶至多为 65,536；其 `act_on_root` 按词的逆序施加反射。这是另一套枚举对象与容量约定。^[lib-root.md:42-44]

## 测试与证据范围

来源列出三个测试锚点：A1 反射取负、固定根按标志判为非紧虚根，以及 `i32::MAX` 坐标配对返回 `ArithmeticOverflow` 而非回绕。相关范围见 [[原型层的测试锚点与证据边界]]。^[lib-root.md:51-54]

本页限定于 crate 根文件中的原型 `RootDatum`，它与 `root_datum` 模块中的 [[BasedRootDatum：带基根数据与构造不变量|BasedRootDatum]] 并非同一类型。来源属于维护者对照源码核对的结构性阅读，不声称数学验收；该次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[lib-root.md:9-14, lib-root.md:55-58, lib-root.md:62-66]

## Sources

- [lib-root.md](../../sources/lib-root.md)：crate 根：60 模块组织、52 条再导出与 A1 原型层（lib.rs）。
