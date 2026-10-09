---
title: Dynkin 分类器的测试覆盖与证据边界
summary: 来源列出六个成功路径测试锚点，错误路径、E7/E8 及高秩 D 缺少覆盖；本次未运行测试或核对上游字节，不构成数学验收或精确兼容性证明。
sources:
  - dynkin.md
kind: concept
createdAt: "2026-10-09T14:45:51.077Z"
updatedAt: "2026-10-09T20:51:05.833Z"
tags:
  - 测试覆盖
  - 证据边界
  - Dynkin分类
aliases:
  - dynkin-分类器的测试覆盖与证据边界
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Dynkin 分类器的测试覆盖与证据边界
summary: 六个测试全部覆盖成功路径；错误路径、E7/E8 和高秩 D 缺少测试锚点。材料仅记录结构性阅读，未执行测试或核对上游字节，不构成数学正确性或完整兼容性验收。
sources:
  - dynkin.md
kind: concept
tags:
  - 测试覆盖
  - 证据边界
  - 兼容性
---

# Dynkin 分类器的测试覆盖与证据边界

Dynkin 分类器的源材料覆盖 `crates/atlas-real-group/src/dynkin.rs` 的结构性阅读，列出 6 个测试，全部针对成功路径。测试锚点涉及类型识别、Bourbaki 顶点顺序、重标号、分量排序及空输入；这些记录不构成分类器的数学正确性验收。^[dynkin.md:9-12, dynkin.md:87-103]

## 成功路径覆盖

低秩案例包括 A1/A2 判为 A，以及 B2、C2、G2 的方向约定。B2 在 `cartan[0][1] = -2` 时判为 B 且不换序；C2 在该条目为 `-1` 时判为 C。G2 的两种指向分别触发换序或保序，使短根在前。B2/C2 的标签由给定顶点顺序下的 Cartan 条目决定，相关背景见 [[秩二 Dynkin 分类的顺序约定]]。^[dynkin.md:51-56, dynkin.md:89-90]

较高秩的标准形案例包括 D4 判为 D、位置向量为 `[0,1,2,3]`，E6 判为 E、位置向量为 `[0..5]`，以及 F4 判为 F、位置向量为 `[0,1,2,3]`。这些案例为 [[基于图结构的 Dynkin 单分量分类]] 中的分叉与多重边处理提供测试锚点。^[dynkin.md:58-62, dynkin.md:90-92]

重标号案例检查顶点排列对分类及输出顺序的影响：B3 经 `[2,0,1]` 重标号后仍判为 B，且 Bourbaki 置换能将重标号矩阵重建为标准形；A3 经 `[1,0,2]` 重标号后返回置换 `[1,0,2]`；D4 经 `[0,2,1,3]` 将分叉顶点从 1 移到 2 后返回 `[0,2,1,3]`，其中 triality 使端点重标号不可见。置换方向见 [[Bourbaki 顶点排序与置换语义]]。^[dynkin.md:92-95]

分量与边界案例包括：块对角 A1+A2 返回类型串 `"AA"`，位置顺序为 `[0,1,2]`，体现 first-vertex 顺序；标准 B2/C2 的 Bourbaki 置换为平凡置换；空矩阵返回空置换。规模为 0 的输入合法，分类结果为空分量列表。^[dynkin.md:93-96, dynkin.md:103-103]

## 未覆盖行为与调用方责任

错误路径没有测试覆盖，E7/E8 与秩大于 4 的 D 型也没有测试锚点。现有记录因此不能提供这些错误处理行为或更高秩案例的直接测试证据。^[dynkin.md:100-102]

输入契约要求满足 based-datum Cartan 不变量。分类器检查矩阵方形、对角元为 2、非对角元属于 `-3..=0` 及零模式对称，但不校验数值配对；不能将这些局部检查视为完整的输入合法性验证。相关契约见 [[Cartan 类型识别的输入校验边界]]。^[dynkin.md:36-41]

`folded_cartan` 会报告轨道下标越界，但不校验轨道的完整性、不交性、覆盖性或输出 Cartan 矩阵的合法性。实现只区分 `ExtGenKind::One`、`Three` 与其他情形，输出合法性依赖调用方，详见 [[基于轨道的折叠 Cartan 矩阵]]。^[dynkin.md:78-85, dynkin.md:106-107]

源材料将全部 `expect` 涉及的 panic 条件归于 `first`、`offset` 的非空性以及各型内部不变量，并说明它们按构造不可达。这是结构性阅读中的不变量判断，材料没有提供相应错误路径测试。^[dynkin.md:101-105]

## 证据来源与结论限制

“精确复现 upstream”是代码注释的意图声明，不是本次知识维护完成的兼容性验收。材料中的 `structure/dynkin.cpp` 等上游引用仅转录自代码文档注释，未核对上游文件字节，因而不能据此认定 Rust 与上游实现已获独立的一致性验证。^[dynkin.md:10-12, dynkin.md:100-100]

精确读取身份记录于 `snapshots/2026-10-06-dynkin.json`，绑定 Git base、文件字节 SHA-256 与 Kimi 调用记录。草案依据完整文件字节生成，随后由维护者对照源码逐条核对改写；本次知识维护未执行 Atlas、Cargo、测试或 benchmark。因此，本页所述“成功路径”指测试案例覆盖的行为类别，不表示本次已运行这些测试并确认通过，也不提供性能证据。^[dynkin.md:87-96, dynkin.md:111-115]

## Sources

- [dynkin.md](../../sources/dynkin.md) — Dynkin 分类器：连通分量、Bourbaki 置换与折叠 Cartan（dynkin.rs）
