---
title: 求值器的 Value 值模型
summary: Value 统一承载数值、字节串、容器、线性代数载荷、联合、领域值及函数值，求值语义由 typed.rs 实现。
sources:
  - atlas-core-value-layer.md
kind: concept
createdAt: "2026-10-09T14:38:56.402Z"
updatedAt: "2026-10-10T00:25:56.869Z"
tags:
  - 值模型
  - 解释器
aliases:
  - 求值器的-value-值模型
  - 求V值
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 求值器的 Value 值模型
summary: Value 统一承载数值、字节串、容器、线性代数载荷、联合、领域值和函数值；实际求值语义由 typed.rs 实现。
sources:
  - atlas-core-value-layer.md
kind: concept
tags:
  - Rust设计
  - 值模型
  - 求值器
aliases:
  - 求值器的-value-值模型
---

# 求值器的 Value 值模型

`Value` 是 `value.rs` 中的求值器值表示，统一承载基础数据、复合数据、线性代数载荷、领域值和函数值。`linear_values.rs` 提供线性代数载荷，实际求值语义位于 `typed.rs`。^[atlas-core-value-layer.md:13-19, atlas-core-value-layer.md:41-49, atlas-core-value-layer.md:71-73]

## 值的种类与载荷

基础与容器变体包括 `Integer(BigInt)`、`Rational(BigRational)`、`Boolean`、`String(AtlasString)`、`Tuple` 和 `List`。带标签联合表示为 `Union { tag: u16, injector_name, value }`，领域值表示为 `Domain(DomainValue)`；函数值分为 `Closure(Rc<Closure>)` 与 `BuiltinFunction(Rc<BuiltinFunction>)`。^[atlas-core-value-layer.md:15-19]

线性代数变体包括 `Vector(Vec32)`、`Matrix` 和 `RatVector(RatVec)`。`Vec32(Vec<i32>)` 保存机器 32 位整数项，`Matrix` 采用列主序整数存储。`RatVec` 使用 `i64` 分子和 `u64` 分母，构造时对全部分子与分母取最大公因数进行规范化；分母为零时返回 `None`。^[atlas-core-value-layer.md:15-17, atlas-core-value-layer.md:45-49]

## 字节保留与打印契约

`AtlasString(Vec<u8>)` 是字节保留串，支持与 `str`、`&str`、`String` 双向进行 `PartialEq` 比较。其 `Display` 仅提供 Unicode 编辑或调试预览，不能作为字节权威表示。`Value::atlas_text` 与 `append_atlas_text` 不经过 Unicode 转换：字符串原始字节置于引号内，元组和列表递归打印，联合值打印为 `value.injectorname`。参见 [[AtlasString 字节保留串与原始字节打印]]。^[atlas-core-value-layer.md:20-24]

`Display for Value` 单独处理有理数的符号，负值打印为 `-num/den`，即使分母为 1 也保留分母。闭包仅打印 `Function defined` 头部，因为 `Display` 无法取得源名表；完整多行形式由 `typed.rs` 的 `closure_trace_string` 在回溯帧转储时渲染。内建函数打印为 `{print_name}`。^[atlas-core-value-layer.md:25-29]

矩阵按列定宽并使用 `|` 框，零行或零列时打印 `The {}x{} matrix`。向量与有理向量分子共享 `write_bracketed`：按最大项宽加一右对齐，以逗号分隔，末项后接 `" ]"`，空向量打印为 `"[ ]"`；有理向量再追加 `/denominator`。相关格式见 [[上游兼容的值打印约定]]。^[atlas-core-value-layer.md:46-52]

## 闭包与调用环境

`Closure` 保存参数信息、绑定形状、递归标志、表达式体、定义环境帧链及源码跨度。`body: Rc<TypedExpr>` 使同一 lambda 字面量产生的所有闭包共享表达式体；`frame: Option<Rc<Frame>>` 使定义域帧链在弹出后仍然存活。`span` 用于回溯调用行的 `defined at …`，`param_names` 按绑定顺序保存参数名，在无参闭包或内建支持的成员闭包中为空。^[atlas-core-value-layer.md:30-37]

绑定形状由 `shapes: Rc<[SlotShape]>` 描述：`Leaf` 占一个槽，`Discard` 不占槽，`Tuple` 按元素解构，`whole` 将未解构的完整值绑定在元素槽之前。`parameters` 为 0 表示无参，调用时不再额外压帧；递归闭包在调用帧的 0 号槽绑定自身。^[atlas-core-value-layer.md:30-33]

`BuiltinFunction` 对外不透明，只能由类型化内建注册表构造，包含注册表身份、打印名和参数策略。它不是用户闭包，没有词法捕获帧，可结合 [[内建函数元数据与实现分发]] 阅读。^[atlas-core-value-layer.md:38-39]

## 实现阶段与证据边界

源文档列出了 `Value` 的线性代数变体，同时保留了线性载荷模块“将在 phase-B stage B2 嵌入 `Value`，此前模块独立”的阶段性自述。这两种表述并存于同一源包，不能据此消除其阶段差异。^[atlas-core-value-layer.md:15-19, atlas-core-value-layer.md:53-53]

本页依据结构性阅读，不构成语言或数学验收。源包记录三个支撑文件合计 11 个测试：`value` 4 个、`linear_values` 3 个、`formula` 4 个。`Domain`／`DomainValue` 的领域内容位于 `domain_builtins.rs`，在该源包中尚待分包说明；上游行号引用属于实现方的移植陈述，值打印兼容性以 HPC 语料门为准。^[atlas-core-value-layer.md:9-11, atlas-core-value-layer.md:69-74]

## Sources

- [atlas-core-value-layer.md](../../sources/atlas-core-value-layer.md) — 值层（value.rs + linear_values.rs + formula.rs）。
