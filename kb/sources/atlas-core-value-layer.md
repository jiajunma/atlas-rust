---
title: 值层（value.rs + linear_values.rs + formula.rs）——Value 面、字节保留串、上游打印格式与算符优先级栈
source: atlas-rust/atlas-core-value-layer
ingestedAt: 2026-10-09T11:20:00Z
---

# 值层（value.rs + linear_values.rs + formula.rs）

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包覆盖
求值器产出值的三个支撑文件：`value.rs`（359 行）、`linear_values.rs`
（403 行）、`formula.rs`（243 行）。结构性阅读，不声称语言或数学验收。

## `value.rs`：求值器的值面

- `Value` 变体：`Integer(BigInt)`、`Rational(BigRational)`、
  `Boolean`、`String(AtlasString)`、`Tuple`、`List`、`Vector(Vec32)`、
  `Matrix`、`RatVector(RatVec)`、`Union { tag: u16, injector_name, value }`
  （打印为 `value.injectorname`）、`Domain(DomainValue)`、
  `Closure(Rc<Closure>)`、`BuiltinFunction(Rc<BuiltinFunction>)`。
- `AtlasString(Vec<u8)`：**字节保留串**——与 `str`/`&str`/`String` 双向
  PartialEq；`Display` 只是 Unicode 编辑/调试预览，**永远不是**字节权威
  （字节串纪律，见根 AGENTS.md 的 string_bytes 条目）。`Value::atlas_text`/
  `append_atlas_text` 是无 Unicode 转换边界的打印机：String 原始字节夹在
  引号内、Tuple/List 递归、Union 打印 `value.injectorname`。
- `Display for Value`：`Rational` 符号单走（Malachite 把符号与非负分子
  分开存——负值打 `-num/den`；**分母为 1 也打印分母**）；
  `Closure` 只打 `Function defined` 头（Display 拿不到源名表；完整多行
  形式由 `typed.rs` 的 `closure_trace_string` 在回溯帧转储渲染，上游
  axis.w:3254-3271）；`BuiltinFunction` 打 `{print_name}`。
- `Closure`：`parameters`（0 = 无参，调用不再额外压帧）、`shapes`
  （`Rc<[SlotShape]>`，上游 `bind_pattern`：Leaf 一槽、Discard 零槽、
  Tuple 按元素解构且 `whole` 把未解构值绑在**元素槽之前**）、
  `recursive`（调用帧 0 号槽绑闭包自身，上游 `maybe_push`）、
  `body: Rc<TypedExpr>`（同一 lambda 字面量的所有闭包共享体）、
  `frame: Option<Rc<Frame>>`（定义域帧链在弹出后存活）、`span`
  （回溯调用行的 `defined at …`）、`param_names`（绑定序；无参或内建
  支持的成员闭包为空）。
- `BuiltinFunction` 对外不透明：只有类型化内建注册表能构造（注册表身份
  + 打印名 + 参数策略；不是用户闭包，没有词法捕获帧）。

## `linear_values.rs`：vec/mat/ratvec 载荷与上游打印格式

载荷逐字节对应上游（axis-types.w:2028-2094）：

- `Vec32(Vec<i32>)`：机器 32 位项。
- `Matrix`：**列主序** int 项；打印按列定宽、`"|"` 框、空阵打印
  `The {}x{} matrix`（`0` 行或列时）。
- `RatVec`：i64 分子 / u64 分母，**构造时即规范化**（对全部分子与分母
  取 gcd；分母为 0 → `None`）。
- 打印逐字节按 global.w:2107-2158：vec 与 ratvec 分子共享
  `write_bracketed`——右对齐于（最大项宽+1）、逗号分隔、末项后
  `" ]"`、空时 `"[ ]"`；ratvec 追加 `/denominator`。
- 自述：这些将在 phase-B stage B2 嵌入 `Value`；在此之前模块独立。

## `formula.rs`：算符优先级归约栈

结构解析器把式子看作算符与运算元的交错序列；本模块复刻上游的
formula-stack 算法（名字查找与重载解析留给后续编译阶段）：

- `FormulaOperator { symbol, priority: i32, span }`；
  `FormulaTree<T>`（Operand/Unary/Binary）；`FormulaStack` 持有
  pending（左操作数 Option + 算符）。
- 归约规则（`should_reduce`）：pending 优先级 **>** 当前，或相等**且**
  偶数——即**奇偶性结合作约定**：偶优先级左结合、奇优先级右结合
  （4 个测试钉死：`-`@4 左、`^`@7 右、混合紧绑、首元算符等后续紧算符）。
- 首元一元算符参与后续算符的优先级比较；二元算符后的一元算符属于该
  二元算符的运算元，在到达本栈前已由结构解析器归约。

## 测试与边界

11 个测试（value 4 + linear_values 3 + formula 4）。求值语义在
`typed.rs`；`Domain`/`DomainValue` 的领域内容在 `domain_builtins.rs`
（待分包）。上游行号引用是**实现方移植陈述**；值打印兼容以 HPC 语料门
为准。字节数/哈希只标识本快照字节（git base `964f0033`）。
