---
title: 典范整数据驻留与 Weyl 姿态定位器（locator.rs）
source: atlas-rust/locator
ingestedAt: 2026-10-05T23:20:00Z
---

# 典范整数据驻留与 Weyl 姿态定位器（locator.rs）

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `crates/atlas-real-group/src/locator.rs`（1061 行）。它是结构性阅读，
不声称 locator 层的数学验收；上游引用（innerclass.cpp:1116-1182、
subsystem.h/cpp、repr.cpp 等行号）仅转录自代码注释，本包未核对上游字节。

## 概览与层定位

本文件是「nonidentity generator attitude」切片的第 1 步：上游
`InnerClass::int_item` 与 `subsystem::integral_datum_entry`/
`integral_datum_item` 对的**纯、未接线**移植——`RepTable::lookup` 尚不调用
本文件任何内容。三个类型的分工：`IntegralDatumTable` 是典范整数据的驻留器
（追加式序号 id，对应上游 `int_sys_nr`）；`IntegralDatumItem` 是一条已驻留
的典范数据（正根表即驻留键 + 子系统单根 + 其余根坐标缓存）；
`BlockLocator` 是查询的姿态定位器（对应 `repr::locator`）：`int_sys` 为典范
数据 id，`w` 把典范基本 alcove 整子系统映到查询实际姿态并保持整正性，
`simp_int` 为整单根之像按 upstream 正根序排列，`simple_pi` 把典范单生成元
下标映到其在 `simp_int` 中的位置。

一处刻意的存储偏差（语义无偏差，作者声明）：upstream 正根 `RootNbr` 序为
（高度、简单坐标反字典序），而 crate 的 `RootId` 序为环境字典序；所有面向
locator 的列表都按 upstream 序排序，使 `simple_pi` 可与 oracle 直接比较。

## 裸签名清单（完整）

```rust
pub struct BlockLocator { int_sys: u32, w: WeylElement, simp_int: Vec<RootId>,
    simple_pi: Vec<usize> }       // 字段全私有；4 个只读访问器
    // pub(crate) from_parts / make_relative_to
pub struct IntegralDatumItem { posroots, simple_roots, simple_coroots }
    // 私有 new；pub positive_roots/simple_roots/simple_coroots/
    // image_simples/coroots_matrix
pub struct IntegralDatumTable { items, ids }   // 仅 derive Clone/Debug/Default
    // new/len/is_empty/item/int_item 为 pub；intern 私有
```

私有辅助：`inverse_permutation`、`fundamental_alcove_walls`、
`factor_dominant`、`reflect_numerator`、`additive_closure`、`pos_simples`、
`negate_root`、`reflect_root`。

## int_item 主流程（代码注释标为 (a)–(f)，对应 innerclass.cpp:1116-1182）

- **(a)** `root_vertex_of_alcove(system, gamma)` 取 gamma 所在 alcove 的根格
  顶点，从分子中逐坐标减去 `denominator * vertex`（全程 checked）。
- **(b)** `factor_dominant`：贪心地把当前在某单余根上取负值的最低下标生成器
  反射进 word，直至 dominant（word 为施加顺序；无迭代上限，终止性由根系
  理论保证而非代码防护）。
- **(c)** 对 `fundamental_alcove_walls` 的每面墙求值：正墙（单根）与 0 比较，
  负墙（逐分量的最高余根之负）与 `-denominator` 比较，命中者入
  `on_wall`。
- **(d)** 按 `word.iter().rev()` 遍历：当前求值非整（`rem_euclid(denominator)
  != 0`）的字母左乘进 `w` 并把该反射从分子中消去；最终 `w(dominant gamma)`
  为所求姿态。
- **(e)** 典范键 = on-wall 根的加法闭包（**基于余根坐标**——整余根在根加法
  下不必封闭）取正部，按 upstream 正根序排序后驻留（命中即复用，幂等）。
- **(f)** 对典范单根逐个求 `w.image(alpha)`（缺失 → `"provenance"` 违规；
  非正像 → `"integral image positivity"` 违规——上游 `assert(is_posroot)`
  的受检形式）；`simp_int` 为像的排序副本，`simple_pi` 为各像在
  `simp_int` 中的位置。

`fundamental_alcove_walls`：所有单根的 `min_coroots_for` 之交（梯子底表）
中的负根，并上全部单根——即逐 Dynkin 分量的最高余根之负加全部单根。
`additive_closure`：以余根坐标为键建映射，生成元连同其负根并入闭包，反复
两两求余根坐标和直至不动点。`pos_simples`：要求输入已按 upstream 序排序；
对每根 α 扫描其后 β：`bracket(β, α) > 0` 时反射，像正则 β 非单根、像负则
α 非单根（`continue 'outer`）——配对符号判据与 `simpleBasis` 同一论证。

`make_relative_to`（repr.cpp:343-345 的 locator 部分）：两 locator 必须同
`int_sys`（否则不变量错误）；`w` 右乘基姿态之逆；`simple_pi` 与基的简单
置换之**逆**右复合（`simple_pi[j] = old[inv[j]]`，逆置换以 `usize::MAX`
哨兵检测越界/重复像）。

## 测试锚点（10 个，含手算推导注释）

- B2 余根和闭包回归：根加法只会错误地产出 4 个长根，余根加法给出全部 8 个
  （`LOCATOR_COROOT_REGRESSION` 标记）。
- F4 半积分定位器（case107、HPC3832609）：`w.image` 作用于 item 全部正根
  等于独立过滤的期望集。
- A2 两个切片 gamma 驻留**不同**的 A1 item（注释明确记录：典范数据依赖
  gamma 所在 alcove 而非仅其整根系——与设计简报草图的出入是已记录行为）；
  Weyl 共轭 gamma 共享同一 item（驻留幂等，`table.len() == 1`）；gamma=0
  驻留全系统（identity 姿态）。
- B2 长/短 A1 驻留不同 item，重复查询复用。
- `root_vertex_of_alcove` 跨模块锚点（[2,2]、[1,1]、[0,0]）。
- `make_relative_to` 的置换合成手算（`[2,1,0] ∘ [1,2,0]^{-1} = [0,2,1]`）
  与 `w` 右乘逆（`s0·s1^{-1}` 的简约词 `[0,1]`）；异 `int_sys` 拒绝。
- 错秩 gamma 的 `RankMismatch`。

## 限制与未覆盖面

- 不做数学/正确性验收；整个模块未接线（`RepTable::lookup` 尚不调用）。
- `IntegralDatumTable` 不持有 `RootSystem`，每次调用重传；跨调用换用不同
  `RootSystem` 不会产生错误信号但结果无定义。
- `factor_dominant` 与 `additive_closure` 无迭代/规模预算。
- 多处 `zip` 截断隐含长度一致假设；`denominator` 非零/非 `i64::MIN` 依赖
  `RationalWeight` 构造处的不变量。
- `debug_assert_eq!` 的像互异检查仅 debug 构建生效。
- 测试仅覆盖 A2/B2/F4；可约根系、rank-0 边角、多条错误路径未测。

## 来源与限制

精确读取身份见
[`2026-10-06-locator.json`](snapshots/2026-10-06-locator.json)：绑定 Git
base、文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe（无工具档案）
以完整文件字节起草，维护者对照源码逐条核对改写。本次知识维护未执行
Atlas、Cargo、测试或 benchmark。
