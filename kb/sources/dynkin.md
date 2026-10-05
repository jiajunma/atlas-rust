---
title: Dynkin 分类器：连通分量、Bourbaki 置换与折叠 Cartan（dynkin.rs）
source: atlas-rust/dynkin
ingestedAt: 2026-10-05T22:20:00Z
---

# Dynkin 分类器：连通分量、Bourbaki 置换与折叠 Cartan（dynkin.rs）

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `crates/atlas-real-group/src/dynkin.rs`（525 行）。它是结构性阅读，
不声称分类器的数学验收；上游引用（structure/dynkin.cpp:28-220 等）仅转录
自代码文档注释，本包未核对上游字节。

## 概览

移植 upstream `DynkinDiagram`（structure/dynkin.cpp:28-220）：求有限 Cartan
矩阵的连通分量，各按 Bourbaki 顶点序分类。分量顺序与每分量 `position` 向量
精确复现 upstream `type()`/`perm()`，包括其平局选择：秩二 B/C 由给定顺序
决定、A 型与 D4 从任一端开始、E 型的长臂交换。可见性：crate 外唯一入口是
`bourbaki_permutation`（语言层按 Bourbaki 编号打印 grading 需要它）；
`classify`/`DynkinComponent`/`folded_cartan` 为 `pub(crate)`。

## 裸签名清单（完整）

```rust
pub(crate) struct DynkinComponent { letter: char, support: BTreeSet<usize>,
    position: Vec<usize> }                // 字段 pub(crate)；offset()
pub(crate) fn classify(cartan: &[Vec<i32>])
    -> Result<Vec<DynkinComponent>, StructureError>
pub fn bourbaki_permutation(cartan: &[Vec<i32>]) -> Result<Vec<usize>, _>
pub(crate) fn folded_cartan(cartan: &[Vec<i32>], orbits: &[ext_block::ExtGen])
    -> Result<Vec<Vec<i32>>, _>
// 私有：first、classify_component
```

## classify：输入契约与分量划分

输入须满足 based-datum Cartan 不变量；违规以 layout invariant 错误报告
（「调用方从已检查的数据构造」）。按行主序扫描时检查：方形
（`NonSquareCartan`）；对角元 2；非对角元 ∈ `-3..=0`；零模式对称
（`cartan[i][j] != 0` 蕴含 `cartan[j][i] != 0`——不校验数值配对）。

邻接结构：`star[v]` = v 的全部邻居；`down_edges` 收集 `entry < -1` 的有序对
`(i, j, -entry)`（标号 2 或 3）。分量按 upstream 的 first-fresh-vertex 顺序
加 first-match 合并：顶点升序处理，邻居全大于自身的顶点新开分量（孤立顶点
成单点分量），否则并入第一个与其邻居集相交的分量，并把后续相交分量移除
合并进来。分量顺序等于各分量最小顶点的升序。

## classify_component：单分量判定

**秩 ≤ 2 特判**：秩 1 → A。秩 2 按 `support` 升序取 `(i, j)`，依
`cartan[i][j] * cartan[j][i]` 分派：1 → A；2 → `cartan[i][j] == -1` 判 C、
否则判 B，**顺序不变**（「Exceptionally the given order decides the type」，
dynkin.cpp:113——这是历史 B2/C2 编号教训的落点：同构的秩二系统标签由
有序 Cartan 条目决定）；3 → G 且在 `cartan[i][j] != -1` 时交换位置
（短根在前）。

**秩 > 2**：按度分析（端点 = 度 ≤ 1，fork = 度 3，度 ≥ 4 报错；端点 < 2
报「环」）；多重边在秩 > 2 分量中：label 3 → 「oversized type G」，第二条
→ 「multiple labelled edges」。字母判定：有多重边时与 fork 并存报错；
`lower ∈ extremities` → B，`upper ∈ extremities` → C，否则 F；无多重边时
无 fork → A，有 fork 则 `|star[fork] ∩ extremities| == 1` → E、否则 D。

**起点选择**（各型特判）：A 取最小编号端点；B/C 分别从端点集移除
lower/upper 后取最小；D4 任取端点，更高秩 D 移除 fork 邻居后取唯一剩余
端点（长臂末端）；E：短臂 = 端点 ∩ fork 邻居（必须恰一个），取正交长臂
端点（首次选中的若离 fork 超过两步则视为最长臂并**交换**为另一条长臂），
`position` 先压入「正交长臂端点、短臂端点」，再从公共邻居继续；F 移除
lower 的邻居后取最小端点。随后遍历：每步取当前顶点的最小编号未访问邻居；
D 型遍历中断时只允许剩余 fork 短臂（升序追加）。

## bourbaki_permutation 与 folded_cartan

`bourbaki_permutation` = 各分量 `position` 的顺序拼接（upstream
`DynkinDiagram::perm()`，dynkin.cpp:289-295）：`result[i]` 是占据 Bourbaki
位置 `i` 的 datum 顶点。

`folded_cartan`（`pub(crate)`）：对应 upstream `DynkinDiagram::folded`
（dynkin.cpp:222-261），此处经 rootdata.cpp:1578-1604 的 `cofold` Cartan
公式计算（注释声称边重数相同，未验证）。折叠单根是轨道成员之和；折叠单余根：
可交换成员（长度 2 轨道）取第一个成员的余根，不可交换（长度 3）取两者之和。
`C(i,j)` 对「轨道 j 的折叠根 a × 轨道 i 的折叠余根 b」求和 `cartan[a][b]`
（crate 约定 `cartan[i][j] = <α_i, α_j∨>`）。轨道下标越界报
`IndexOutOfRange { index: a.max(b) }`；不校验轨道的完整性/不交性/覆盖性，
也不校验输出是合法 Cartan。

## 测试锚点（6 个，全部成功路径）

A1/A2 → A；B2（`cartan[0][1] = -2`）→ B 不换序；C2（`cartan[0][1] = -1`）
→ C；G2 两指向分别换序/不换序（短根在前）。D4 标准形 → D [0,1,2,3]；E6
（1-3-4-5-6 链加 2 挂 4 的 Bourbaki 序，0 基边）→ E [0..5]；F4 → F
[0,1,2,3]。B3 经置换 [2,0,1] 重标号后分类为 B，且 Bourbaki 置换把重标号
矩阵重建为标准形。块对角 A1+A2 → "AA" 且 positions 为 [0,1,2]（first-
vertex 顺序）。A3 经 [1,0,2] 重标号 → 置换 [1,0,2]；D4 经 [0,2,1,3]（fork
从顶点 1 移到 2，triality 使端点重标号不可见）→ [0,2,1,3]。标准 B2/C2 的
Bourbaki 置换平凡；空矩阵 → 空置换。

## 限制与未覆盖面

- 不做数学/正确性验收；「精确复现 upstream」是注释意图声明。
- 错误路径无测试覆盖（6 个测试全是成功路径）；E7/E8 与秩 > 4 的 D 无测试
  锚点。
- 规模为 0 的输入合法（空矩阵 → 空分量列表）。
- panic 面：全部 `expect`（`first`/`offset` 的非空性与各型内部不变量），
  按构造不可达。
- `folded_cartan` 只区分 `ExtGenKind::One`/`Three`/其他（`Two` 字样未在本
  文件出现），其输出合法性依赖调用方。

## 来源与限制

精确读取身份见
[`2026-10-06-dynkin.json`](snapshots/2026-10-06-dynkin.json)：绑定 Git base、
文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe（无工具档案）以完整
文件字节起草，维护者对照源码逐条核对改写。本次知识维护未执行 Atlas、
Cargo、测试或 benchmark。
