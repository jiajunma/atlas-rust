---
title: 内类字母解析与逐字母对合查表（primitive_involution.rs）
source: atlas-rust/primitive-involution
ingestedAt: 2026-10-05T19:10:00Z
---

# 内类字母解析与逐字母对合查表（primitive_involution.rs）

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `crates/atlas-real-group/src/primitive_involution.rs`（529 行）。
它是结构性阅读，不声称该层的数学验收；上游引用（atlas-types.w:742-820、
lietype.cpp:507-605、matrix.cpp:289-295 等）仅转录自代码文档注释，本包未
核对上游字节。

## 概览

本文件是 primitive `involution(LieType,[int],string)` 与
`involution(LieType,mat,string)` 包装器背后的 `lietype::involution` 查表
实现，对应三段上游来源：`checked_inner_class_type` 及其字母坍缩规则
（atlas-types.w:742-820）、`lietype::involution(const Layout&)` 的逐字母
对合表（单连通群基本权基上，structure/lietype.cpp:507-605）、精确除法换基
`PID_Matrix::on_basis`（utilities/matrix.cpp:289-295）。模块文档明确：这些
是**纯表**，不涉及 root datum、Weyl group 或 inner-class 管线；`perm`
参数是用户提供的、对扁平化单因子的 Bourbaki 重编号，由包装器侧
`checked_permutation`（atlas-types.w:829-846）验证。

裸签名清单（完整；本文件无 `pub(crate)` 项、无 `pub struct`、无
`pub trait`）：

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InnerClassLetterError {
    TooManySymbols, TooFewSymbols, UnknownSymbol(char), ComplexPair,
    MeaninglessUnequalRank { letter: char, rank: usize },
}
impl fmt::Display for InnerClassLetterError { .. }   // 上游 runtime_error 文案逐字节
impl std::error::Error for InnerClassLetterError {}  // 无注释空实现

pub fn checked_inner_class_letters(symbols: impl AsRef<[u8]>,
    factors: &[(char, usize)]) -> Result<Vec<char>, InnerClassLetterError>
pub fn layout_involution(factors: &[(char, usize)], letters: &[char],
    perm: &[usize]) -> Vec<Vec<i32>>
pub fn on_basis(matrix: &[Vec<i32>], basis: &[Vec<i32]>) -> Option<Vec<Vec<i32>>>
```

局部项：`layout_involution` 内闭包 `compact`/`flip_last_two`（对应
lietype.cpp:622-658 的块助手）；`on_basis` 内闭包 `rational`/`product`；
六个测试。

## `InnerClassLetterError`：失败集合与文案

上游 `checked_inner_class_type` 的失败集合，`Display` 逐字节复刻上游
`runtime_error` 文案：

| 变体 | 上游出处 | `Display` 输出（精确） |
| --- | --- | --- |
| `TooManySymbols` | 符号多于单因子（:764） | `Too many inner class symbols` |
| `TooFewSymbols` | 符号少于单因子（:752） | `Too few inner class symbols` |
| `UnknownSymbol(char)` | 不在 `"Ccesu"`（:762） | ``Unknown inner class symbol `{symbol}'``（反引号开、单引号合） |
| `ComplexPair` | `'C'` 缺少两个相同连续因子（:770） | `Complex inner class needs two identical consecutive types` |
| `MeaninglessUnequalRank{letter,rank}` | `'u'` 在无 unequal-rank 类处（:805） | `Unequal rank class is meaningless for type {letter}{rank}`（如 `A1`） |

## `checked_inner_class_letters`：字母解析与坍缩规则

按单因子逐个解析内类字符串，每个因子一个字母，`'C'` 消耗两个相同连续
因子。符号按**字节**处理（`char::from(u8)`）：解释器字符串与上游
istringstream 含字节，未知符号保留为字节值字符进入 `UnknownSymbol` 诊断，
绝不做 UTF-8 解码或替换。取符号前跳过 ASCII 标点与空白
（`skip_punctuation`）。

判定顺序固定：先查 `"Ccesu"` 成员（否则 `UnknownSymbol`），再查因子存在性
（否则 `TooManySymbols`）——测试注释钉住「未知符号先于因子计数报告」。

- `'C'`：要求 `factors.get(index + 1) == Some(&(letter, rank))`，否则
  `ComplexPair`（含已到末尾的情形）；成功则 `index += 2`。
- `'c' | 'e'`：`push('c')`（`'e'` 是 `'c'` 的同义词，不出现在输出中）。
- `'s'`：存活条件 `(A && rank >= 2) || (D && rank 奇) || (E && rank == 6)
  || T`；存活则 `'s'`，否则坍缩为 `'c'`。文档注释：`'s'` 恰在 `-1` 属于
  Weyl 群处坍缩——A1、B、C、偶秩 D、E7、E8、F、G（atlas-types.w:782-790）。
- `'u'`：D 偶秩 → `'u'`，D 奇秩 → `'s'`；A(n≥2)、E6、T → `'s'`；其余
  `MeaninglessUnequalRank { letter, rank }`（atlas-types.w:793-820）。
- 末尾：`index < factors.len()` → `TooFewSymbols`。

## `layout_involution`：逐字母对合表

输出单连通群基本权基上的行主序 `i32` 矩阵；`perm[k]` 是第 k 个扁平化单根
的矩阵下标（上游 `Layout::d_perm`）。前置条件仅靠文档与两道
`debug_assert_eq!`（`perm.len() == rank`、末尾 `r == rank`）：`letters` 须
来自 `checked_inner_class_letters`，`perm` 须为 `0..rank` 的排列；函数
**不返回 `Result`**，违反前置条件时下标越界 panic 或命中两个
`unreachable!`。

逐字母分派（双指针 `r` 扁平图位置、`pos` 因子位置）：

- `'c'` → `compact`（块恒等）；
- `'s'` → A：反对角；D：奇秩 `flip_last_two`（末两顶点互换）、偶秩
  `compact`；E6：固定下标 1、3，交换 0↔5、2↔4（六条显式赋值），其他秩
  `compact`；T：负恒等（对角置 -1）；其余（B、C、E7、E8、F、G 的恒等
  对合）`compact`；
- `'C'`：把 `rs` 个顶点与紧随的 `rs` 个顶点平行互换，并额外消耗一个因子
  （`pos += 1; r += rs`——checked letters 已保证两因子相同）；
- `'u'` → `flip_last_two`。

## `on_basis`：精确除法换基

计算 `basis^-1 * matrix * basis`（上游 `PID_Matrix::on_basis` 的
adjugate-乘-精确除法在非整条目上抛 "Inexact integer division"）。流程：
非方阵早退 `None`；`real_form_seed::invert_rational` 求基的有理逆（奇异 →
`None`），转置得逆矩阵；两次三重循环有理乘得
`basis^-1 * matrix * basis`；逐项 `floor` 检查整性（非整 → `None`）并
`i32::try_from`（失败 → `None`）。**四类失败折叠为同一个 `None`**（非方阵、
奇异基、非整结果、转换失败），调用方无法区分——文档说明包装器统一重标为
不兼容格。

## 测试锚点（断言值照录）

- `'s'` 坍缩为 `'c'`：A1、B2、C3、D4、D6、E7、E8、F4、G2；存活：A2、A5、
  D5、E6、T1。
- `'u'` 存活仅 D4/D6；坍缩为 `'s'`：A2、D5、E6、T1；
  `MeaninglessUnequalRank`：A1、B2、C2、E7、F4、G2；文案断言
  `Unequal rank class is meaningless for type A1`。
- 表锚点：A1 `'c'`/坍缩 `'s'` 均为 `[[1]]`；A2 `'s'`/`'u'` 均为
  `[[0,1],[1,0]]`（`perm [1,0]` 下不变）；B2 `'s'` 为单位阵；
  `[A1,A1]` `'C'` 互换两因子；D4 `'u'` 交换末两顶点而 D4 `'s'` 为单位阵；
  D5 `'s'` 交换末两顶点；E6 `'s'` 固定 1、3 并交换 0↔5、2↔4；T1 `'s'`
  为 `[[-1]]`；`[A1,A2]` + `"cs"` + `perm [2,0,1]` 钉住 Bourbaki 重编号
  作用于表输出。
- `on_basis`：A2 翻转阵在 `[[1,0],[1,1]]` 基上 → `Some([[1,1],[0,-1]])`；
  在 `[[1,0],[0,2]]` 上 → `None`（非整除即不兼容）；A1 单位阵在偶子格
  `[2]` 上 → `Some([[1]])`；奇异基 → `None`；单位阵在任意可逆基上不变。

## 限制与未覆盖面

- 不做数学/正确性验收；上游行号引用来自代码注释，未独立验证。
- 前置条件（letters 来自 checked 解析、perm 为排列、`rs >= 2`）只在调试
  构建中断言；绕过 `checked_inner_class_letters` 的输入可走通防御分支或
  panic。
- `'s'` 在 B/C/E7/E8/F/G 上已于解析期坍缩为 `'c'`，故表中对应默认臂在
  遵守前置条件的调用下不可达（仅对绕过解析的输入起防御作用）。
- 测试未覆盖：`TooManySymbols`/`TooFewSymbols` 的 Display 文案、`on_basis`
  的非方阵早退分支、`'C'` 在非恒等 `perm` 下的行为、退化空输入。

## 来源与限制

精确读取身份见
[`2026-10-06-primitive-involution.json`](snapshots/2026-10-06-primitive-involution.json)：
绑定 Git base、文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe（无
工具档案）以完整文件字节起草，维护者对照源码逐条核对改写。本次知识维护
未执行 Atlas、Cargo、测试或 benchmark。
