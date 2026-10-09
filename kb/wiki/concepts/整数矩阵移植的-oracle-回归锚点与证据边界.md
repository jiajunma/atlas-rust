---
title: 整数矩阵移植的 oracle 回归锚点与证据边界
summary: C++ oracle 字面量锚定精确变换矩阵与秩亏系统被选解，但溢出、空形状等路径缺少测试，本文未重跑测试或完成数学验收。
sources:
  - matreduc.md
kind: concept
createdAt: "2026-10-09T15:01:01.099Z"
updatedAt: "2026-10-09T19:33:29.915Z"
tags:
  - 回归测试
  - 证据边界
aliases:
  - 整数矩阵移植的-oracle-回归锚点与证据边界
  - 整O回
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 整数矩阵移植的 oracle 回归锚点与证据边界

`matreduc.rs` 的 oracle 回归锚点用于约束整数矩阵移植的具体输出，包括变换矩阵、对角符号和整数系统的被选解。其目标是复现上游确切的幺模操作序列：欠定系统 \(Ax=b\) 的被选解在下游可观测，\(\tau\)/\(t\) 坐标的奇偶性会进入 `ext_block::same_sign`。模块采用 wrapping `i32`，并将相关溢出行为纳入逐操作一致性的承诺，参见 [[整数矩阵算法的逐操作保真移植]]。^[matreduc.md:20-26, matreduc.md:69-74]

## 结构性检查与 oracle 锚点

`diagonalise(m)` 返回 `(row, col, diagonal)`，其中 `row`、`col` 为幺模矩阵，使 `row * m * col` 对角化，且除首项外的对角元素均为正。对角化的 11 个测试用例检查重构关系、\(|\det(row)|=1\)、\(\det(col)=1\)、逐项对角值和首项之外的正性。^[matreduc.md:30-33, matreduc.md:69-70]

`oracle_reference_cases` 则直接固定来自 C++ oracle 的输出：`[[0,5],[0,0]]` 对应 `diagonal = [−5]` 及精确的 `row`、`col`；`[[−4]]` 对应 `[−4]`；另一个 6×6 秩亏、主元列未左对齐的用例固定完整的 `row`、`col` 字面量，以及解 `[6,14,5,−37,−421,345]`。这些逐字节锚点覆盖了对角符号、变换矩阵和被选解。^[matreduc.md:71-74]

来源共记录六组测试锚点。除对角化重构和 oracle 参考用例外，`find_solution` 覆盖满秩、秩亏和矩形三种形态；像判定包含一维例；[[单位上三角整数矩阵求逆]] 包含两个正常用例和两个拒绝用例；[[偶数指数的虚数单位幂计算]] 包含五个测试点。^[matreduc.md:69-74]

## 符号约定与失败行为

[[对角化的行列式符号簿记]] 存在注释口径差异：一处注释称应确保 `det(row)=1`，测试注释则指出上游只强制 `det(col)==1`，`det(row)` 可以为 −1。来源明确以测试为准，因此不能将 `row` 的行列式恒为正视为已确立的不变量。^[matreduc.md:42-49]

[[整数线性系统求解与像判定]] 还需要区分无整数解与输入形状错误。`find_solution` 无整数解时返回 `None`，上游此时抛异常，调用方预期已先运行 `has_solution`。长度及部分形状约束由断言处理，因此 `has_solution` 收到长度不匹配的 `b` 时会 panic，而不是返回 `Err`。^[matreduc.md:50-59]

`exp_i` 的前置条件是输入为偶数。上游使用 `assert`，Rust 移植使用 `debug_assert!`；release 模式下，奇数输入会落入 `−1` 分支而无防护。这是解释其测试覆盖时必须保留的接口边界。^[matreduc.md:65-67]

## 未覆盖范围

来源明确列出的测试缺口包括 wrapping 溢出域、`in_left_image`／`in_right_image` 的矩形和秩亏情形、空形状 `diagonalise`，以及 `from_entries`／`apply_to` 的 panic 路径。溢出行为虽被文档声明为被选解中的可观测因素，仍属于未测范围。^[matreduc.md:75-76]

## 证据来源与限制

来源包对 755 行 `matreduc.rs` 进行了两次结构性阅读，两次读取的字节及 SHA-256 相同。模块文档记载已与编译版 oracle 逐位验证，但这一结果属于模块自身的 [[HPC 验收证据链]]；来源包没有重跑该验证。源码中保留的测试锚点与文档转述的验证结果，应分别理解。^[matreduc.md:9-16]

上游文件位置及行号均转述自源码注释，未独立重读上游，行号可能随版本演进漂移。来源包未执行构建、测试或原版运行，也不提供数学验收、性能或并行结论。^[matreduc.md:80-89]

## Sources

- [matreduc.md](matreduc.md) —《精确整数矩阵约化：matreduc 的逐操作移植》
