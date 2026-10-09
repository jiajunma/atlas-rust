---
title: 求值器的 Value 值模型
summary: Value 统一表示数值、字节串、容器、线性代数载荷、带标签联合、领域值及函数值；实际求值语义由 typed.rs 承担。
sources:
  - atlas-core-value-layer.md
kind: concept
createdAt: "2026-10-09T14:38:56.402Z"
updatedAt: "2026-10-09T14:38:56.402Z"
tags:
  - Rust设计
  - 值模型
  - 求值器
aliases:
  - 求值器的-value-值模型
  - 求V值
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 求值器的 Value 值模型

`Value` 是求值器的运行时值表示，定义于 `value.rs`；线性代数载荷由 `linear_values.rs` 支撑，求值语义位于 `typed.rs`。它涵盖基础数据、复合数据、领域值及可调用值。^[atlas-core-value-layer.md:13-19, atlas-core-value-layer.md:41-49, atlas-core-value-layer.md:71-73]

## 值的种类

基础与复合变体包括 `Integer(BigInt)`、`Rational(BigRational)`、`Boolean`、`String(AtlasString)`、`Tuple` 和 `List`。联合值使用 `Union { tag: u16, injector_name, value }`，领域值使用 `Domain(DomainValue)`；可调用值分为 `Closure(Rc<Closure>)` 与 `BuiltinFunction(Rc<BuiltinFunction>)`。^[atlas-core-value-layer.md:15-19]

线性代数变体包括 `Vector(Vec32)`、`Matrix` 和 `RatVector(RatVec)`。`Vec32` 保存 `i32` 项；`Matrix` 采用列主序整数存储；`RatVec` 使用 `i64` 分子和 `u64` 分母，构造时对全部分子与分母取最大公因数进行规范化，分母为零则返回 `None`。^[atlas-core-value-layer.md:15-17, atlas-core-value-layer.md:45-49]

## 字节保留与打印

`AtlasString(Vec<u8>)` 是字节保留串，支持与 `str`、`&str`、`String` 双向进行 `PartialEq` 比较。其 `Display` 仅提供 Unicode 编辑或调试预览，不能作为原始字节的权威表示。`Value::atlas_text` 与 `append_atlas_text` 提供无 Unicode 转换边界的打印：字符串原始字节放在引号内，元组和列表递归打印，联合值打印为 `value.injectorname`。参见 [[AtlasString 字节保留串与原始字节打印]]。^[atlas-core-value-layer.md:20-24]

`Display for Value` 对有理数单独处理符号，负值打印为 `-num/den`，即使分母为 1 也保留分母。闭包只打印 `Function defined` 头部；完整多行形式由 `typed.rs` 的 `closure_trace_string` 在回溯帧转储时渲染，因为 `Display` 无法取得源名表。内建函数打印为 `{print_name}`。^[atlas-core-value-layer.md:25-29]

线性代数载荷遵循各自的打印约定：矩阵按列定宽并使用 `|` 框，零行或零列时打印 `The {}x{} matrix`；向量和有理向量分子共享 `write_bracketed`，按最大项宽加一右对齐、以逗号分隔，末项后接 `" ]"`，空向量为 `"[ ]"`。有理向量再追加 `/denominator`。参见 [[上游兼容的值打印约定]]。^[atlas-core-value-layer.md:46-52]

## 闭包与调用环境

`Closure` 保存参数信息、绑定形状、递归标志、表达式体、定义环境帧链及源码跨度。`body: Rc<TypedExpr>` 使同一 lambda 字面量产生的闭包共享表达式体；`frame: Option<Rc<Frame>>` 使定义域帧链在弹出后仍然存活。`span` 用于回溯中的 `defined at …`，`param_names` 按绑定顺序保存参数名，无参闭包或内建支持的成员闭包中为空。参见 [[TypedExpr 可执行表达式树]]。^[atlas-core-value-layer.md:30-37]

参数绑定由 `shapes: Rc<[SlotShape]>` 描述：`Leaf` 占一个槽，`Discard` 不占槽，`Tuple` 按元素解构；若有 `whole` 绑定，未解构的完整值先于元素槽绑定。`parameters` 为 0 表示无参，调用时不再额外压帧；递归闭包在调用帧的 0 号槽绑定自身。^[atlas-core-value-layer.md:30-33]

`BuiltinFunction` 对外不透明，只能由类型化内建注册表构造，携带注册表身份、打印名和参数策略。它不是用户闭包，也没有词法捕获帧。参见 [[内建函数元数据与实现分发]]。^[atlas-core-value-layer.md:38-39]

## 证据与边界

源文档记录的是结构性阅读，不构成语言或数学验收。三个支撑模块合计有 11 个测试，其中 `value` 4 个、`linear_values` 3 个、`formula` 4 个。`DomainValue` 的领域内容位于 `domain_builtins.rs`，不在该包的展开范围内；上游行号引用属于实现方的移植陈述，值打印兼容性仍以 HPC 语料门为准。参见 [[HPC 验收证据链]]。^[atlas-core-value-layer.md:9-11, atlas-core-value-layer.md:69-74]

源文档同时列出了 `Value` 的线性代数变体，并保留了线性载荷模块“将在 phase-B stage B2 嵌入 `Value`，此前独立”的阶段性自述，因此其集成时间状态存在表述差异。^[atlas-core-value-layer.md:15-19, atlas-core-value-layer.md:53-53]

## Sources

- [atlas-core-value-layer.md](../../sources/atlas-core-value-layer.md)
