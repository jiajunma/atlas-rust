---
title: 内建注册表（typed.rs 5973–11565）——Builtin/BuiltinImpl 面、无值门策略与启动清单
source: atlas-rust/atlas-core-builtin-registry
ingestedAt: 2026-10-09T14:30:00Z
---

# 内建注册表（typed.rs 5973–11565）

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包覆盖
`crates/atlas-core/src/typed.rs` 的 5973–11565 行：`Builtin` 面、
`BuiltinImpl`/`DomainNoValue`/`ScalarOp`、求值期辅助的上游契约、以及
`builtin_registry()` 的启动清单组织（479 个条目、240 个不同名字——
逐家族：scalar 167/59、domain 154/119、domain_validate 86/58、
domain_skip 29/19、domain_printer 21/19、domain_relation 22/2；对
`typed.rs` 的 `vec![…]` 注册体按构造器调用精确计数）。
结构性阅读，不声称语言或数学验收。

## `Builtin` 面

`{ name: &'static str, arg_type: Type, result: Type, hunger: u8,
overload_visible: bool, implementation: BuiltinImpl }`——hunger 字节对应
上游的饥饿求值位（与 `HungryBuiltinCall` 配对，见
[typed-core 包](atlas-core-typed-core.md)）；`overload_visible` 控制是否
出现在重载解析/补全中。

## `BuiltinImpl`：实现分发

- `Scalar(ScalarOp)`：标量运算（见下）。
- `Domain { name, no_value: DomainNoValue }`：领域调用（桥到
  `domain_builtins.rs`，见 [派发包](atlas-core-domain-dispatch.md)）。
- `DomainPrinter { name }`：打印包装（atlas-types.w:8944-8957,
  8850-8859）——**两个级别都写报告**、single_value 时产出空元组；其
  无值门之前无诊断。
- `DomainRelation(Relation)`：领域关系。
- `Completions`：`readline_completions`（global.w:3546-3561）——按前缀
  过滤命令层存在求值上下文里的补全候选快照。
- 四个变参数泛型：`Prints`（按当前 global.w:4480 装成普通 scheme
  T->void）、`Print`（`print@@T`：逐字打印——字符串带引号——并在要值
  级别原样返回）、`ToString`（`to_string@@T`：剥离后的拼接串值）、
  `Error`（`error@@T`：把剥离拼接作为运行时错误抛出）。

## `DomainNoValue`：包装器的无值门策略

`Skip` / `Validate` / `BuildAndDrop`——决定领域调用在 NoValue 级别做
多少：跳过、校验、或完整构造后丢弃。**补全名清单不是无值策略清单**
（AGENTS.md 的 R2 教训：`orientation_nr` 注册为 BuildAndDrop 而非
Skip）。

## 求值期辅助的上游契约（节选）

- `int_val`/`long_val` 收窄（bigint.cpp:142-162，**含笔误**的逐字诊断）；
  `check_size`（global.w:3874-3880）。
- 向量 `\`/`%` 逐项算术（global.w:3917-3937）：余数总取 `[0,|m|)`，
  商随之而来（oracle：`[7] % -3 == [1]`，`[7] \ -3 == [-2]`）。
- `nth_set_bit`（bigint.cpp `index_of_set_bit`）：二进制补码串的第 n 个
  置位（0 基）；非负值位数不足时得 -1；负值有无穷多置位——走其**补码**
  的有限清位。
- `flex_add`/`flex_sub`（global.w:3953-4048）：尺寸自适应多项式加减；
  两个参数都去尾零，结果只在**等修剪尺寸**情形去尾零。
- `convolve`（global.w:4056-4085）：去尾零参数的乘积；任一修剪后为空
  则为空。
- `ratvec ±`（global.w:4127-4139）：按最小公分母交叉相乘；
  `RatVec::new` 规范化结果。
- `prints`/`to_string`/`error` 的边界：`to_string_aux`（axis.w:8819-8840）
  对变参元组——字符串分量不带引号打印，其余按 `print`；`prints` 自己
  加 `std::endl`，`to_string`/`error` 不加；`prints_wrapper` 输出 = 剥离
  文本 + 一个换行。

## `builtin_registry()`：启动清单

`OnceLock<Vec<Builtin>>` 一次建成；479 个条目、240 个不同名字，按主题
分节（整数位工具、…），每节注释给出上游出处。自述："growing toward the
traced inventory"——注册表相对上游完整启动清单**不全**（缺口由
docs/REMAINING_BUILTINS.md 跟踪；补全名纪律见
[typed-core 包](atlas-core-typed-core.md)的 `STARTUP_COMPLETION_NAMES`
条目：309 个启动名按上游哈希序，注册缺口显式钉住）。

## 边界与限制

- 逐条目语义（479 臂）不在本包；`TypedExpr` 的求值 impl 与 133 个测试
  各待分包。上游行号引用是**实现方移植陈述**；行为兼容以 HPC 语料门
  为准。字节数/哈希只标识本快照字节（git base `964f0033`，typed.rs
  sha256 `614975c5…`）。
