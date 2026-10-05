---
title: primitive_involution：内类字母解析、逐字母对合查表与精确除法换基
source: atlas-rust/primitive-involution
ingestedAt: 2026-10-05T18:00:00Z
---

# 草案说明

本包仅依据 `crates/atlas-real-group/src/primitive_involution.rs` 的完整字节起草。文中标注【已实现行为】者可直接在源码中指认；标注【阅读推断】者为阅读代码所得推论，需维护者核对；不作任何数学验收、性能或正确性声明。

模块文档自述：本文件是 primitive `involution(LieType,[int],string)` 与 `involution(LieType,mat,string)` 包装器背后的 `lietype::involution` 查表实现，对应三段上游来源：

- `checked_inner_class_type` 及其字母坍缩规则（interpreter/atlas-types.w:742-820）；
- `lietype::involution(const Layout&)` 的逐字母对合表，在单连通群基本权基上（structure/lietype.cpp:507-605）；
- 精确除法换基 `PID_Matrix::on_basis`（utilities/matrix.cpp:289-295）。

模块文档明确：这些是纯表，不涉及 root datum、Weyl group 或 inner-class 管线；`perm` 参数是用户提供的、对扁平化单因子的 Bourbaki 重编号，由包装器侧 `checked_permutation`（atlas-types.w:829-846）验证。

依赖（`use` 项，已实现行为）：`std::fmt`；`malachite::base::num::arithmetic::traits::Floor`；`malachite::base::num::basic::traits::Zero`；`malachite::Rational`；`crate::real_form_seed::invert_rational`。

---

## 一、裸签名清单

以下为本文件全部 `pub` 项与可见 impl 块（含无注释项）。本文件无 `pub(crate)` 项、无 `pub struct`、无 `pub trait`。

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InnerClassLetterError {
    TooManySymbols,
    TooFewSymbols,
    UnknownSymbol(char),
    ComplexPair,
    MeaninglessUnequalRank { letter: char, rank: usize },
}

impl fmt::Display for InnerClassLetterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result;
}

// 无文档注释的空实现
impl std::error::Error for InnerClassLetterError {}

pub fn checked_inner_class_letters(
    symbols: impl AsRef<[u8]>,
    factors: &[(char, usize)],
) -> Result<Vec<char>, InnerClassLetterError>;

pub fn layout_involution(
    factors: &[(char, usize)],
    letters: &[char],
    perm: &[usize],
) -> Vec<Vec<i32>>;

pub fn on_basis(matrix: &[Vec<i32>], basis: &[Vec<i32>]) -> Option<Vec<Vec<i32>>>;
```

非 `pub` 项（仅登记存在，不属对外签名）：

- `layout_involution` 内局部闭包 `compact`、`flip_last_two`（注释标注对应 lietype.cpp:622-658 的块助手，矩阵下标形式）；局部变量 `r`、`pos`。
- `on_basis` 内局部闭包 `rational`、`product`；局部变量 `inverse_columns`、`inverse`、`transported`。
- `#[cfg(test)] mod tests`，含私有助手 `fn letters(symbols: &str, factors: &[(char, usize)]) -> Vec<char>`（内部调用 `checked_inner_class_letters(...).expect("valid inner class string")`）及六个测试函数（见第五节）。

---

## 二、类型与字段

### 2.1 `InnerClassLetterError`【已实现行为】

上游 `checked_inner_class_type` 的失败集合；文档注明 `Display` 是上游 `runtime_error` 文案“逐字节”复刻。派生 `Clone, Debug, Eq, PartialEq`，并（无注释地）实现 `std::error::Error`。

| 变体 | 负载 | 文档标注的上游出处 | `Display` 输出（精确字符串） |
|---|---|---|---|
| `TooManySymbols` | 无 | 符号多于单因子（atlas-types.w:764） | `Too many inner class symbols` |
| `TooFewSymbols` | 无 | 符号少于单因子（atlas-types.w:752） | `Too few inner class symbols` |
| `UnknownSymbol(char)` |  offending 字符 | 符号不在 `"Ccesu"` 内（atlas-types.w:762） | ``Unknown inner class symbol `{symbol}'``（反引号开、单引号合） |
| `ComplexPair` | 无 | `'C'` 缺少两个相同连续因子（atlas-types.w:770） | `Complex inner class needs two identical consecutive types` |
| `MeaninglessUnequalRank { letter: char, rank: usize }` | 命名字段 | `'u'` 在无 unequal-rank 类处（atlas-types.w:805） | `Unequal rank class is meaningless for type {letter}{rank}`（letter 与 rank 直接拼接，如 `A1`） |

### 2.2 因子表示与字母表【已实现行为】

- 单因子以 `(char, usize)` 二元组表示（字母 + 秩），切片 `&[(char, usize)]` 贯穿三个函数。
- 合法输入符号集合为字符串 `"Ccesu"`；解析输出字母集合为 `c / s / C / u`（`'e'` 在解析期即坍缩为 `'c'`，不出现在输出中）。
- 总秩 `rank` 在 `layout_involution` 与 `on_basis` 中分别定义为“各因子秩之和”与 `basis.len()`，注意二者定义不同【已实现行为；阅读推断：两处 `rank` 语义各自局部，调用方需保证一致】。

---

## 三、构造 / 判定流程

### 3.1 `checked_inner_class_letters`【已实现行为】

对应 `checked_inner_class_type`（atlas-types.w:742-820）：按单因子逐个解析内类字符串，每个因子一个字母，`'C'` 消耗两个相同连续因子。

逐条行为：

1. **字节迭代**：`symbols.as_ref().iter().copied().map(char::from)`。注释明确：解释器字符串与上游 istringstream 含字节；未知符号保留为无符号字节值字符以供调用方做精确字节诊断；绝不做 UTF-8 解码或替换。
2. **跳过分隔**：每次取符号前，先循环跳过 `is_ascii_punctuation() || is_ascii_whitespace()` 的字节（对应 `skip_punctuation`，atlas-types.w:224-227）。
3. **判定顺序**：先检查符号是否属于 `"Ccesu"`（否则 `Err(UnknownSymbol(symbol))`），再检查 `factors.get(index)` 是否存在（否则 `Err(TooManySymbols)`）。测试注释钉住：“Unknown symbols report before the factor count”。
4. **分字母规则**（精确条件）：
   - `'C'`：要求 `factors.get(index + 1) == Some(&(letter, rank))`，否则 `Err(ComplexPair)`（含“已到末尾无下一因子”的情形，因 `get` 返回 `None`）。成功则 `push('C')`，`index += 2`。
   - `'c' | 'e'`：`push('c')`，`index += 1`（`'e'` 是 `'c'` 的同义词）。
   - `'s'`：存活条件为 `(letter == 'A' && rank >= 2) || (letter == 'D' && rank % 2 != 0) || (letter == 'E' && rank == 6) || letter == 'T'`；存活则 `push('s')`，否则 `push('c')`。文档注释：`'s'` 恰在 `-1` 属于 Weyl 群处坍缩为 `'c'`——A1、B、C、偶秩 D、E7、E8、F、G（atlas-types.w:782-790）。
   - `'u'`：若 `letter == 'D'`，秩偶则 `push('u')`、秩奇则 `push('s')`；否则若 `(A && rank >= 2) || (E && rank == 6) || T` 则 `push('s')`；其余 `Err(MeaninglessUnequalRank { letter, rank })`。文档注释：`'u'` 仅对偶秩 D 存活，对 A(n>=2)、奇秩 D、E6、T 坍缩为 `'s'`，余者无意义（atlas-types.w:793-820）。
   - 其余符号：`unreachable!("membership in \"Ccesu\" was checked above")`（panic 分支，理论上不可达）。
5. **末尾检查**：循环结束后 `index < factors.len()` 则 `Err(TooFewSymbols)`；否则 `Ok(result)`，`result` 顺序与消耗符号的顺序一致。

### 3.2 `layout_involution`【已实现行为】

对应 `lietype::involution(const Layout&)`（lietype.cpp:507-605）：每个内类条目一个字母，输出单连通群基本权基上的行主序矩阵。`perm[k]` 是第 k 个扁平化单根的矩阵下标（上游 `Layout::d_perm`）。

- **文档载明的前置条件**（上游的、包装器侧检查）：`letters` 来自 `checked_inner_class_letters`，`perm` 是 `0..rank` 的一个排列。函数体内仅有 `debug_assert_eq!(perm.len(), rank)` 与结尾 `debug_assert_eq!(r, rank)` 两道调试断言。
- **初始化**：`rank = 各因子秩之和`；`result` 为 `rank × rank` 全零 `i32` 矩阵。
- **局部块助手**：
  - `compact(result, r, rs)`：对 `i in 0..rs` 置 `result[perm[r+i]][perm[r+i]] = 1`（块恒等）。
  - `flip_last_two(result, r, rs)`：前 `rs - 2` 个置恒等，再交换末两个下标（`perm[r+rs-2] ↔ perm[r+rs-1]`）。
- **主循环**：双指针 `r`（扁平图位置，索引 `perm`）与 `pos`（`factors` 位置）。按 `letters` 顺序分派：
  - `'c'` → `compact`。
  - `'s'` → 按因子字母分派：
    - `'A'`：反对角矩阵，`result[perm[r+i]][perm[r+rs-1-i]] = 1`。
    - `'D'`：奇秩 `flip_last_two`；偶秩 `compact`。
    - `'E'`：`rs == 6` 时固定下标 1、3，交换 `0↔5`、`2↔4`（六条显式赋值）；否则 `compact`。
    - `'T'`：对角置 `-1`（负恒等）。
    - 其余（注释：B、C、E7、E8、F、G 的恒等对合）→ `compact`。
  - `'C'`：将 `rs` 个顶点与紧随的 `rs` 个顶点平行互换（成对置 1），随后额外 `pos += 1; r += rs`（即消耗两个因子；注释：checked letters 已保证两因子相同）。
  - `'u'` → `flip_last_two`。
  - 其余字母：`unreachable!("checked letters are one of c/s/C/u")`。
  - 每轮末尾统一 `pos += 1; r += rs`。
- **无显式错误返回**：返回类型不是 `Result`；违反前置条件时的行为见第六节。

### 3.3 `on_basis`【已实现行为】

对应 `PID_Matrix::on_basis`（utilities/matrix.cpp:289-295）：计算换基 `basis^-1 * matrix * basis`。文档注明：上游的“伴随矩阵乘精确除法”计算在任何非整数条目上抛 `"Inexact integer division"`；本实现对方阵外输入、奇异基、非整数结果一律返回 `None`——包装器把这些情形统一重标为不兼容格。

逐步流程：

1. `rank = basis.len()`；若 `matrix.len() != rank`，或 `basis`/`matrix` 中任一行长度不为 `rank`，返回 `None`（非方阵早退）。
2. `rational` 闭包把 `Vec<Vec<i32>>` 逐元素转为 `Rational`。
3. `invert_rational(&rational(basis)).ok()?` 得 `inverse_columns`（奇异基 → `None` 早退）；再转置为 `inverse[row][column] = inverse_columns[column][row]`。
4. `product` 闭包为标准三重循环有理数矩阵乘（累加器 `Rational::ZERO`）；`transported = product(product(inverse, rational(matrix)), rational(basis))`，即 `basis^-1 * matrix * basis`。
5. 逐项取整检查：`floored = value.clone().floor()`；若 `*value != floored` 返回 `None`（精确除法检查）；再 `i32::try_from(&floored).ok()`（转换失败 → `None`）。全部通过后收集为 `Some(Vec<Vec<i32>>)`。

---

## 四、与 Cartan / grading 的关系

- 【已实现行为】模块文档明确声明：本文件是纯查表，**不涉及** root datum、Weyl group 或 inner-class 管线；唯一的 crate 内耦合是对 `crate::real_form_seed::invert_rational` 的调用（用于 `on_basis` 的有理数求逆）。
- 【已实现行为】`perm`（Bourbaki 重编号）的合法性不在本文件校验，文档指明由包装器侧 `checked_permutation`（atlas-types.w:829-846）负责。
- 【阅读推断】本文件不生产也不消费任何 Cartan 或 grading 数据结构；所产出的对合矩阵与换基结果如何被下游用于 Cartan/grading 构造，超出本文件字节范围，本包不作断言。

---

## 五、测试锚点

`#[cfg(test)] mod tests` 含六个测试；以下均为测试断言钉住的精确值【已实现行为】。

1. **`split_collapses_to_compact_exactly_where_minus_one_is_weyl`**
   - `'s'` 坍缩为 `'c'`：`A1, B2, C3, D4, D6, E7, E8, F4, G2`。
   - `'s'` 存活：`A2, A5, D5, E6, T1`。
2. **`unequal_rank_survives_only_for_even_rank_d`**
   - `'u'` → `['u']`：`D4, D6`；`'u'` → `['s']`：`A2, D5, E6, T1`。
   - `'u'` → `Err(MeaninglessUnequalRank { letter, rank })`：`A1, B2, C2, E7, F4, G2`（letter/rank 逐一对应）。
   - 文案断言：`MeaninglessUnequalRank { letter: 'A', rank: 1 }.to_string() == "Unequal rank class is meaningless for type A1"`。
3. **`letter_string_diagnostics_have_the_upstream_wording_and_order`**
   - `"e"` on `A2` → `['c']`；`". c "` on `A1` → `['c']`（标点与空白跳过）。
   - `"x"` on `A1` → `Err(UnknownSymbol('x'))`；文案 `"Unknown inner class symbol `x'"`。
   - `"ec"` on 单因子 `A2` → `Err(TooManySymbols)`；`"c"` on `[A1, A1]` → `Err(TooFewSymbols)`。
   - `"C"` on `[A1, A1]` → `['C']`；`"Cs"` on `[A1, A2]` → `Err(ComplexPair)`；`"C"` on `[A1]` → `Err(ComplexPair)`。
   - 文案断言：`ComplexPair.to_string() == "Complex inner class needs two identical consecutive types"`。
4. **`involution_table_pins_the_frozen_fixture_anchors`**（冻结夹具锚点）
   - `A1` 的 `'c'` 与（坍缩后的）`'s'` 均为 `[[1]]`。
   - `A2`：`'c'` → 单位阵；`'s'` 与 `'u'` → `[[0,1],[1,0]]`；`perm = [1,0]` 时 `'s'` 仍为 `[[0,1],[1,0]]`（注释：翻转对称）。
   - `B2` 的 `'s'` → 单位阵；`[A1, A1]` 的 `'C'`（`perm [0,1]`）→ `[[0,1],[1,0]]`。
5. **`involution_table_covers_the_remaining_letters`**
   - `D4` `'u'` → 固定 0、1，交换 `2↔3`；`D4` `'s'` → 4 阶单位阵；`D5` `'s'` → 固定 0、1、2，交换 `3↔4`。
   - `E6` `'s'` → 固定 1、3，交换 `0↔5`、`2↔4`（完整 6×6 矩阵钉住）。
   - `T1` `'s'` → `[[-1]]`。
   - Bourbaki 重编号作用于表输出：`[A1, A2]`、字母 `"cs"`、`perm [2,0,1]` → `[[0,1,0],[1,0,0],[0,0,1]]`。
6. **`on_basis_transports_with_the_exact_division_check`**
   - 冻结 A2 锚点：翻转阵 `[[0,1],[1,0]]` 在 `basis = [[1,0],[1,1]]`（注释：列为 (1,1) 与 (0,1)，Atlas 字面量 `[[1,1],[0,1]]`）上 → `Some([[1,1],[0,-1]])`。
   - 同一翻转阵在 `[[1,0],[0,2]]` 上 → `None`（非整除，故不兼容）。
   - A1 单位阵在偶子格 `[2]` 上 → `Some([[1]])`。
   - 奇异基 `[[1,1],[1,1]]` → `None`。
   - 单位阵在任意可逆基上仍为单位阵：`Some([[1,0],[0,1]])`。

---

## 六、限制与未覆盖面

1. **前置条件仅靠文档与调试断言**【已实现行为】：`layout_involution` 不返回 `Result`；`perm.len() == rank` 与 `r == rank` 仅为 `debug_assert_eq!`（仅调试构建生效）；`perm` 是 `0..rank` 的排列这一点完全由包装器侧保证，本函数不校验。
2. **潜在 panic 路径**【阅读推断，待核对】：
   - 若 `letters` 与 `factors` 不一致（绕过 `checked_inner_class_letters`），`factors[pos]` 的索引将越界 panic；
   - 两个 `unreachable!` 分支依赖前置条件成立；
   - `flip_last_two` 中 `rs - 2` 与 `0..rs - 2` 假定 `rs >= 2`；经 checked letters，`'u'` 仅出自偶秩 D、`'s'`+奇秩 D 走 `flip_last_two`，正常路径不会触发下溢，但该保证不在函数体内强制执行。
3. **表内若干回退分支的可达性**【阅读推断】：`'s'` 在 B、C、E7、E8、F、G 上于解析期已坍缩为 `'c'`，故 `layout_involution` 中 `'s'` 的 `_ => compact` 默认臂与 `'E'` 非 6 秩的 `compact` 回退，在遵守前置条件的调用下不可达；它们只对绕过 checked letters 的输入起防御作用。
4. **失败模式折叠**【已实现行为】：`on_basis` 把非方阵输入、奇异基、非整数（不精确除法）结果、`i32` 转换失败四类情形全部折叠为 `None`，调用方无法区分；文档说明包装器统一重标为“不兼容格”。
5. **字节语义**【已实现行为】：符号按字节处理（`char::from(u8)`），非 ASCII 输入不会产生 UTF-8 解码错误，而是以字节值字符进入 `UnknownSymbol` 诊断。
6. **无显式预算/上限**【已实现行为】：本文件无迭代次数、尺寸或递归预算；所有循环上界由输入尺寸（符号数、因子数、秩）决定。
7. **测试未覆盖面**【已实现行为 + 阅读推断】：
   - `TooManySymbols` 与 `TooFewSymbols` 的 `Display` 文案无断言（其余三个变体的文案均已钉住）；
   - `on_basis` 的非方阵早退分支（行长度不齐）无测试；
   - `'C'` 在非恒等 `perm` 下无测试；`E6` 的 `'u'`（经解析为 `'s'`）在表测试中未单独出现；
   - 空 `factors`、空符号串等退化输入无测试。