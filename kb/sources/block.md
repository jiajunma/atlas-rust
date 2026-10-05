---
title: 块图：两实形式 KGB 的纤维积与对偶变换（block.rs）
source: atlas-rust/block
ingestedAt: 2026-10-06T04:40:00Z
---

# 块图：两实形式 KGB 的纤维积与对偶变换（block.rs）

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `crates/atlas-real-group/src/block.rs`（1095 行）：上游
`blocks::Block`（gkmod/blocks.cpp:526-606）的移植——块 = 实形式 KGB 图与
对偶实形式 KGB 图在扭对合 `w`/`dual_w` 上的纤维积。上游行号仅转录自代码
注释，本包未核对上游字节；不声称数学验收。

## BlockDescent 与语言码

8 个变体按上游 `descents::DescentStatus::Value` 顺序（descents.h:40）：
`ComplexAscent, RealNonparity, ImaginaryTypeI, ImaginaryTypeII,
ImaginaryCompact, ComplexDescent, RealTypeII, RealTypeI`——前四个是上升，
索引位 `0x4` 置位的后四个是弱下降（`is_descent`）。`dual()` 静态表：
`ComplexAscent↔ComplexDescent`、`RealNonparity↔ImaginaryCompact`、
`ImaginaryTypeI↔RealTypeII`、`ImaginaryTypeII↔RealTypeI`。`language_code()`
经 `TAB = [4,5,6,7,1,0,3,2]` 映到 Atlas 语言码（0=C-, 1=ic, 2=r1, 3=r2,
4=C+, 5=rn, 6=i1, 7=i2；atlas-types.w:4911-4913 的重编号）。

## dual_involution 与纤维积构建

`dual_involution(word, dual_system, dual_twist, dual_longest)`（上游
blocks.cpp:1701-1711）：从对偶最长元素出发，按 `w` 的约化字**自右向左**
以对偶扭曲字母右乘；`word` 携带两侧共享的外部生成元编号。对合矩阵上是
负转置；Weyl 元素上由 `f(e)=w0`、`f(s.w)=f(w).dwist(s)` 刻画。

`BlockGraph::build(graph, table, dual_graph, dual_table, dual_inner_class,
weyl_budget)`：

1. 秩一致性门槛：两半单秩不等或对偶表根系 ≠ 对偶内类根系 →
   `DatumMismatch`。`weyl_budget` 仅供 `dual::longest_action`。
2. 对偶包以 `HashMap<WeylElement, usize>` 索引（重复键会静默覆盖——
   无防护，阅读观察）。
3. 逐原侧包算 `dual_w`；对偶包缺失即上游 `tauPacket` 返回 `(0,0)`，
   贡献零对——共同 Cartan 的限制是隐式的。模块文档强调：解释器从两形式
   的**完整** KGB 建块（atlas-types.w:4753-4758），坐标保持各形式自身
   KGB 编号，`common_Cartans` 受限重载会改变编号。
4. 元素编号：包按原 KGB 对合序，包内 **x 外层、y 内层**的笛卡尔积
   （blocks.cpp:548-558）；`xs` 弱增是 `first_zs`/`element_at` 赖以定位
   的不变量。`element_at` 用 `first_z_of_x[x]` + 连续 y 偏移定位并**验证**
   坐标（承担上游 assert 的角色）；`xs.len() != size` 报 `"block size"`。
5. 每个生成元的下降状态由 `descents()` 判定：复根看 `is_descent`；虚非紧
   看 cross 是否动（动 → i1，不动 → i2）；实/虚紧看对偶侧——对偶虚非紧
   且 cross 动 → r2、不动 → r1，否则 Real → rn、ImaginaryCompact → ic。
6. cross/Cayley 表扁平布局 `generator * size + z`（descent 是
   `z * rank + generator`）。i1：单值直接 Cayley + `first_free_slot` 回填
   逆 Cayley；i2：**双值**直接 Cayley（z→z1 经 dual_second，随后落入 i1
   分支处理 dual_first；blocks.cpp:575-590 的 fall-through）。
   `cayley_first/second` 两槽同时服务直接 Cayley（上升处）与逆 Cayley
   （弱下降处）；槽满报 `"Cayley pair slots"`。

## 访问器语义

`cayley(z, g)` 在弱下降处强制 `Some((None, None))`（内层 None = 上游
`UndefBlock`），上升处返回槽原值；`inverse_cayley` 互补（非弱下降返回
`(None, None)`）。`cross` 对每个生成元有定义。访问器越界一律 `None`。

## `dual()`：纯数据变换

上游 `Bare_block::dual`（blocks.cpp:474-509）：元素序反转
（`z' = size−1−z`）、x/y 互换、长度反射 `max_len − length(z)`（`max_len`
取末元素长度，依赖排序；空块取 0）、下降状态逐点 `dual()`、链接
`c → size−1−c`；Cayley 槽**仅当 first 有定义**才映射（second 随之），
first 为 None 则两槽均 None。`first_z_of_x` 按 `max_y+1` 范围重算。上游
`orbits`/`dd` 字段（上游自注 "probably not right"）不复现。上游双值
Cayley 对在对偶中不重排（故与原生对偶块比较时按无序集比）。

`bruhat_hasse()` 委托 `crate::block_access`（算法本体在 block-access 包）；
`n_bruhat_comparable`（poset.cpp:197-229）逐行 BTreeSet 闭包累计，含自身
可比；要求 Hasse 行按拓扑序（直接索引 `closure[j]`，乱序会 panic——
阅读观察）。

## 测试锚点与限制

7 个测试，全部秩 1（A1）：`dual_involution` 交换恒等/反射；
block(SL(2,R), PGL(2,R))（大小 3，坐标 (0,1),(1,1),(2,0)，状态 i1/i1/r1，
语言码 6/2，两 i1 共享单值 r1 像，inverse 双值）对齐冻结 fixture
capture 3501519；block(PGL(2,R), SL(2,R))（i2 双值 Cayley、r2 单值逆）；
Bruhat-Hasse `[[],[],[0,1]]`；descent dual 静态表逐项 + 对合性；
`dual()` 的反转/互换/链接映射逐点核对；`dual()` 与原生对偶块经 `element`
换编号后一致（覆盖全 KGB 范围时 `dual().dual()` 为恒等）。

未覆盖：多生成元（两种扁平布局的区分）、C±/rn/ic 块级状态、全部错误路径、
空对偶包/空块、`element` 失败分支、`n_bruhat_comparable`。`cross` 表先填
0 再全量覆写（0 不可观测，但若循环边界改变会成静默默认值——阅读观察）。
`size * rank.max(1)` 等乘法未做 checked（debug 溢出 panic）。

## 来源与限制

精确读取身份见
[`2026-10-06-block.json`](snapshots/2026-10-06-block.json)：
绑定 Git base、文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe
（无工具档案）以完整字节起草（620s 期限，exit 0，242.7s），维护者对照
源码逐条核对改写。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。
