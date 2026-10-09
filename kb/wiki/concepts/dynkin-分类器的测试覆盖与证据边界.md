---
title: Dynkin 分类器的测试覆盖与证据边界
summary: 源包列出六个成功路径测试锚点，错误路径、E7/E8 和高秩 D 缺少覆盖；结构性阅读未执行测试，也未核对上游字节，因此 upstream 精确兼容与数学正确性仍非已验收结论。
sources:
  - dynkin.md
kind: concept
createdAt: "2026-10-09T14:45:51.077Z"
updatedAt: "2026-10-09T14:45:51.077Z"
tags:
  - 测试覆盖
  - 证据边界
  - 兼容性
aliases:
  - dynkin-分类器的测试覆盖与证据边界
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Dynkin 分类器的测试覆盖与证据边界

Dynkin 分类器的源材料覆盖 `crates/atlas-real-group/src/dynkin.rs` 的结构性阅读，并列出 6 个测试，全部针对成功路径。这些测试为类型识别、Bourbaki 顶点顺序及重标号行为提供锚点，但不构成分类器的数学正确性验收。^[dynkin.md:9-12, dynkin.md:87-102]

## 成功路径覆盖

低秩测试覆盖 A1/A2 的 A 型识别，以及秩二多重边的方向约定：B2 在 `cartan[0][1] = -2` 时判为 B 且不换序；C2 在该条目为 `-1` 时判为 C；G2 的两种指向分别触发换序或保序，使短根在前。B2/C2 的标签由给定顶点顺序下的 Cartan 条目决定，测试保留了这一历史编号约定。^[dynkin.md:51-56, dynkin.md:89-90]

较高秩的标准形锚点包括 D4 → D、位置向量 `[0,1,2,3]`，E6 → E、位置向量 `[0..5]`，以及 F4 → F、位置向量 `[0,1,2,3]`。这些案例涉及分叉与多重边分类，可结合 [[基于图结构的 Dynkin 单分量分类]] 阅读。^[dynkin.md:90-92]

重标号测试检查了顶点排列对输出的影响：B3 经 `[2,0,1]` 重标号后仍识别为 B，且 Bourbaki 置换能将矩阵重建为标准形；A3 经 `[1,0,2]` 重标号后返回同一置换；D4 经 `[0,2,1,3]` 将分叉顶点从 1 移到 2 后返回 `[0,2,1,3]`，其中 triality 使端点重标号不可见。这些案例是 [[Bourbaki 顶点排序与置换语义]] 的具体测试锚点。^[dynkin.md:92-95]

分量排序与边界输入也有成功路径覆盖：块对角 A1+A2 返回类型串 `"AA"`，位置顺序为 `[0,1,2]`，对应 first-vertex 顺序；标准 B2/C2 的 Bourbaki 置换为平凡置换；空矩阵返回空置换，且规模为 0 的输入合法、对应空分量列表。^[dynkin.md:93-96, dynkin.md:103-103]

## 未覆盖的行为与调用方责任

错误路径没有测试覆盖，E7/E8 与秩大于 4 的 D 型也没有测试锚点。因此，现有测试不能为这些错误处理行为或更高秩案例提供直接验证依据。^[dynkin.md:100-102]

分类器检查方形、对角元为 2、非对角元属于 `-3..=0` 以及零模式对称，但不校验数值配对；输入仍须满足 based-datum Cartan 不变量。这一契约应与 [[Cartan 类型识别的输入校验边界]] 一起理解。^[dynkin.md:36-41]

[[基于轨道的折叠 Cartan 矩阵]] 对应的 `folded_cartan` 会报告轨道下标越界，但不校验轨道的完整性、不交性或覆盖性，也不校验输出是否为合法 Cartan 矩阵。实现只区分 `ExtGenKind::One`、`Three` 与其他情形，输出合法性依赖调用方。^[dynkin.md:78-85, dynkin.md:106-107]

源材料还将全部 `expect` 的 panic 面归于 `first`、`offset` 的非空性及各型内部不变量，并说明它们按构造不可达；这属于结构性阅读中的不变量判断。^[dynkin.md:104-105]

## 证据来源与结论限制

“精确复现 upstream”是代码注释表达的意图，而非本次知识维护完成的兼容性验收。源材料中的 `structure/dynkin.cpp` 等上游引用仅转录自代码文档注释，未核对上游文件字节，因此不能据此声称已独立验证 Rust 与上游实现完全一致。^[dynkin.md:10-12, dynkin.md:100-100]

读取身份由 `snapshots/2026-10-06-dynkin.json` 记录，绑定 Git base、文件字节 SHA-256 与 Kimi 调用记录。草案依据完整文件字节生成，随后由维护者对照源码逐条核对改写；本次知识维护未执行 Atlas、Cargo、测试或 benchmark。这里列出的测试覆盖是源码中的测试锚点记录，不是本次运行结果或性能证据。^[dynkin.md:111-115]

## Sources

- [dynkin.md](../../sources/dynkin.md) — Dynkin 分类器：连通分量、Bourbaki 置换与折叠 Cartan（dynkin.rs）
