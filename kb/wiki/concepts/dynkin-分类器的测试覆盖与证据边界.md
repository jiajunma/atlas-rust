---
title: Dynkin 分类器的测试覆盖与证据边界
summary: 来源记录六个成功路径测试锚点，错误路径、E7/E8 及高秩 D 缺少覆盖；结构性阅读未执行测试或核对上游字节，不构成数学验收或兼容性证明。
sources:
  - dynkin.md
kind: concept
createdAt: "2026-10-09T14:45:51.077Z"
updatedAt: "2026-10-10T00:31:10.054Z"
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
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Dynkin 分类器的测试覆盖与证据边界
summary: 六个测试均覆盖成功路径；错误路径、E7/E8 与秩大于 4 的 D 型缺少测试锚点。来源仅为结构性阅读，未执行测试或核对上游字节，不构成数学或兼容性验收。
sources:
  - dynkin.md
kind: concept
tags:
  - 测试覆盖
  - 证据边界
aliases:
  - dynkin-分类器的测试覆盖与证据边界
provenanceState: extracted
---

# Dynkin 分类器的测试覆盖与证据边界

Dynkin 分类器的源材料覆盖 `crates/atlas-real-group/src/dynkin.rs` 的结构性阅读，记录六个测试，全部针对成功路径。测试锚点涉及类型识别、Bourbaki 顶点顺序、重标号、分量排序与空输入；这些记录不构成数学正确性验收。^[dynkin.md:9-12, dynkin.md:87-103]

## 成功路径覆盖

低秩案例包括 A1/A2 判为 A，以及 B2、C2、G2 的方向约定。B2 在 `cartan[0][1] = -2` 时判为 B 且不换序；C2 在该条目为 `-1` 时判为 C；G2 的两种指向分别触发换序或保序，使短根在前。秩二 B/C 的标签由给定顺序下的 Cartan 条目决定，参见 [[基于图结构的 Dynkin 单分量分类]]。^[dynkin.md:51-56, dynkin.md:89-90]

标准形案例还包括 D4 判为 D、位置向量为 `[0,1,2,3]`，E6 判为 E、位置向量为 `[0..5]`，以及 F4 判为 F、位置向量为 `[0,1,2,3]`。E6 采用 Bourbaki 编号形状：链 1–3–4–5–6，顶点 2 连接顶点 4。^[dynkin.md:90-92]

重标号案例中，B3 经 `[2,0,1]` 重标号后仍判为 B，且 Bourbaki 置换能将重标号矩阵重建为标准形；A3 经 `[1,0,2]` 重标号后返回 `[1,0,2]`；D4 经 `[0,2,1,3]` 将分叉顶点从 1 移到 2 后返回 `[0,2,1,3]`，其中 triality 使端点重标号不可见。^[dynkin.md:92-95]

解释这些输出时须保留置换方向：`result[i]` 是占据 Bourbaki 位置 `i` 的 datum 顶点，完整结果由各分量的 `position` 顺序拼接而成，参见 [[Bourbaki 顶点排序与置换语义]]。^[dynkin.md:74-76]

分量与边界案例包括：块对角 A1+A2 返回类型串 `"AA"`，拼接后的位置顺序为 `[0,1,2]`，体现 first-vertex 顺序；标准 B2/C2 的 Bourbaki 置换为平凡置换；空矩阵返回空置换。规模为 0 的输入合法，其分类结果为空分量列表。^[dynkin.md:93-96, dynkin.md:103-103]

## 未覆盖行为与调用方责任

错误路径没有测试覆盖，E7/E8 与秩大于 4 的 D 型也没有测试锚点。来源所列的六个测试因此不能作为这些分支已经过测试验证的依据。^[dynkin.md:87-102]

分类器要求输入满足 based-datum Cartan 不变量，检查方形、对角元为 2、非对角元属于 `-3..=0` 以及零模式对称，但不校验数值配对。其前提是调用方从已检查的数据构造输入，参见 [[Cartan 类型识别的输入校验边界]]。^[dynkin.md:36-41]

`folded_cartan` 对轨道下标越界报告 `IndexOutOfRange`，但不校验轨道的完整性、不交性或覆盖性，也不校验输出是否为合法 Cartan 矩阵。实现只区分 `ExtGenKind::One`、`Three` 与其他情形，输出合法性依赖调用方，参见 [[基于轨道的折叠 Cartan 矩阵]]。^[dynkin.md:78-85, dynkin.md:106-107]

来源将全部 `expect` 涉及的 panic 条件归于 `first`、`offset` 的非空性与各型内部不变量，并说明它们按构造不可达。这是结构性阅读中的不变量判断；来源同时明确错误路径没有测试覆盖。^[dynkin.md:101-105]

## 证据来源与结论限制

“精确复现 upstream”属于代码注释的意图声明。材料中的 `structure/dynkin.cpp` 等上游引用仅转录自代码文档注释，未核对上游文件字节，不能据此认定 Rust 与上游实现已通过独立的兼容性验收。^[dynkin.md:10-12, dynkin.md:100-100]

精确读取身份记录于 `snapshots/2026-10-06-dynkin.json`，绑定 Git base、文件字节 SHA-256 与 Kimi 调用记录。草案依据完整文件字节生成，随后由维护者对照源码逐条核对改写；该次知识维护未执行 Atlas、Cargo、测试或 benchmark。因此，“成功路径”描述的是测试覆盖类别，不表示该次已运行并确认测试通过，也不提供性能测量证据。^[dynkin.md:87-96, dynkin.md:111-115]

## Sources

- [dynkin.md](../../sources/dynkin.md) — Dynkin 分类器：连通分量、Bourbaki 置换与折叠 Cartan（dynkin.rs）。
