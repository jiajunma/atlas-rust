---
title: 原型 RootDatum 的构造校验与配对约定
summary: new 在空输入检查后先调用外部校验器，再检查形状与 Cartan 条件；单余根取 Cartan 列，from_basis 另检查维度及逐项配对。
sources:
  - lib-root.md
kind: concept
createdAt: "2026-10-09T14:58:31.132Z"
updatedAt: "2026-10-09T22:37:48.300Z"
tags:
  - 根数据
  - 构造校验
aliases:
  - 原型-rootdatum-的构造校验与配对约定
  - 原R的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 原型 RootDatum 的构造校验与配对约定
summary: 原型 RootDatum 按固定顺序校验构造输入，单根取标准基、单余根取 Cartan 矩阵列；from_basis 先检查维数，再按行主序校验配对。
sources:
  - lib-root.md
kind: concept
tags:
  - 根数据
  - 构造校验
  - Cartan矩阵
---

# 原型 RootDatum 的构造校验与配对约定

原型 `RootDatum` 位于 `crates/atlas-real-group/src/lib.rs`，属于全部以 `pub(crate)` 可见性保留、实现注释标明待替换的 [[A1 迁移原型层与对偶格类型设计|A1 迁移原型层]]。它与 `root_datum` 模块中的 [[BasedRootDatum：带基根数据与构造不变量|BasedRootDatum]] 是不同类型。^[lib-root.md:10-14, lib-root.md:30-38, lib-root.md:55-58]

## 构造校验顺序

`RootDatum::new` 的校验顺序为 `EmptyRootDatum` → `BasedRootDatum::standard(...)?` → `NonSquareCartan` → `InvalidCartanMatrix`：先检查空输入，再调用外部校验器，随后检查方阵形状，最后检查对角元是否等于 2、非对角元是否不大于 0。外部校验器的错误先于原型自身的形状检查传播，因此必须保留这一错误优先级。^[lib-root.md:34-36]

`RootDatum::from_basis` 对显式给定的根与余根逐个检查维数，不匹配时返回 `RankMismatch`；随后按行主序核对每一项根与余根的配对，不符合对应 Cartan 条目时返回 `RootPairingMismatch`。^[lib-root.md:37-38]

## 单根基与配对方向

在单根基坐标下，第 \(i\) 个单根取标准基向量 \(e_i\)，第 \(j\) 个单余根取 Cartan 矩阵的第 \(j\) **列**。`from_basis` 核对的等式是 \(\langle \mathrm{root}_i,\mathrm{coroot}_j\rangle=\mathrm{cartan}[i][j]\)，即根索引对应矩阵行，余根索引对应矩阵列。^[lib-root.md:34-38]

原型使用的 `LatticeVector(Vec<i32>)` 是不带校验的 newtype。其文档声明将由 `Weight`／`Coweight` 的编译期对偶格区分取代；这是待替换原型的迁移方向，来源未声明替换已经完成。^[lib-root.md:30-33]

## 根系展开与算术边界

`roots()` 从正负单根播种，以 FIFO BFS 构造反射闭包。计算使用 `i128` 中间精度，再收窄至 `i32`；溢出返回 `ArithmeticOverflow`。出现第 4097 个互异向量时返回 `RootSystemTooLarge`，对应容量限制 `LIMIT = 4096`；输出按坐标字典序排序，以保持确定性。^[lib-root.md:39-41]

## 测试与证据范围

源文件包含三个测试锚点：A1 反射取负、固定根按标志判为非紧虚根，以及含 `i32::MAX` 坐标的配对恰好返回 `ArithmeticOverflow` 而非回绕。最后一项还间接要求 `StructureError: PartialEq`；相关范围见 [[原型层的测试锚点与证据边界]]。^[lib-root.md:51-54]

本页依据 crate 门面与原型层的结构性阅读材料，不构成数学验收，也不覆盖其他模块的实现。来源记录了维护者对照源码逐条核对的过程，但该次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[lib-root.md:9-14, lib-root.md:55-58, lib-root.md:62-66]

## Sources

- [lib-root.md](../../sources/lib-root.md) — crate 根：60 模块组织、52 条再导出与 A1 原型层（lib.rs）。
