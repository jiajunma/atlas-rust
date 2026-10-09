---
title: AtlasString 字节保留串与原始字节打印
summary: AtlasString 保存原始字节，Display 仅供 Unicode 预览；atlas_text 与 append_atlas_text 在不进行 Unicode 转换的边界递归打印值。
sources:
  - atlas-core-value-layer.md
kind: concept
createdAt: "2026-10-09T14:38:38.408Z"
updatedAt: "2026-10-09T20:46:21.390Z"
tags:
  - 字节字符串
  - 输出保真
aliases:
  - atlasstring-字节保留串与原始字节打印
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: AtlasString 字节保留串与原始字节打印
summary: AtlasString 使用 Vec<u8> 保留原始字节，Display 仅提供 Unicode 预览；Value 的 atlas_text 与 append_atlas_text 提供无 Unicode 转换的递归打印路径。
sources:
  - atlas-core-value-layer.md
kind: concept
tags:
  - 字符串
  - 字节保留
  - 值打印
aliases:
  - atlasstring-字节保留串与原始字节打印
---

# AtlasString 字节保留串与原始字节打印

`AtlasString` 是 Atlas 值层的字节保留串，以 `Vec<u8>` 保存内容，并作为 [[求值器的 Value 值模型|Value]] 中 `String(AtlasString)` 变体的载荷。它与 `str`、`&str`、`String` 支持双向 `PartialEq` 比较。^[atlas-core-value-layer.md:15-24]

## 字节表示与 Unicode 预览

`AtlasString` 的 `Display` 仅用于 Unicode 编辑或调试预览，不能作为原始字节的权威表示。保留原始字节的打印接口是 `Value::atlas_text` 与 `append_atlas_text`，它们的输出路径不经过 Unicode 转换。^[atlas-core-value-layer.md:20-24]

## 复合值的原始字节打印

上述打印接口将字符串的原始字节置于引号内，对 `Tuple` 和 `List` 递归打印，并将 `Union` 打印为 `value.injectorname`。因此，字节保留打印也涵盖嵌入这些复合值中的字符串。^[atlas-core-value-layer.md:22-24]

这些接口属于值层的输出约定；其他值的格式可参见 [[上游兼容的值打印约定]]。例如，`Value` 的 `Display` 对有理数单独处理符号，即使分母为 1 也打印分母；内建函数打印为 `{print_name}`。^[atlas-core-value-layer.md:25-29]

## 证据边界

来源包完成了 `value.rs`、`linear_values.rs` 与 `formula.rs` 的结构性阅读，并未声称完成语言或数学验收。包中记录了 11 个测试，其中值层 4 个、线性代数值层 3 个、算符优先级栈 4 个；这些计数没有说明 `AtlasString` 的具体测试覆盖范围。^[atlas-core-value-layer.md:9-11, atlas-core-value-layer.md:69-74]

来源中的上游行号引用属于实现方的移植陈述，值打印兼容性仍以 HPC 语料门为准。快照字节数与哈希仅标识对应字节，不能据此认定打印兼容性已经通过验收。^[atlas-core-value-layer.md:73-74]

## Sources

- [atlas-core-value-layer.md](../../sources/atlas-core-value-layer.md) — 值层（value.rs + linear_values.rs + formula.rs）。
