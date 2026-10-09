---
title: AtlasString 字节保留串与原始字节打印
summary: AtlasString 保存原始字节，Display 仅供 Unicode 预览；atlas_text 与 append_atlas_text 递归打印并保留字符串原始字节。
sources:
  - atlas-core-value-layer.md
kind: concept
createdAt: "2026-10-09T14:38:38.408Z"
updatedAt: "2026-10-09T22:22:52.045Z"
tags:
  - 字符串
  - 字节保真
  - 打印
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
  - 字节字符串
  - 输出保真
  - 值打印
aliases:
  - atlasstring-字节保留串与原始字节打印
---

# AtlasString 字节保留串与原始字节打印

`AtlasString` 是 Atlas 值层的字节保留串，以 `Vec<u8>` 保存内容，作为 `Value::String(AtlasString)` 的载荷。它与 `str`、`&str`、`String` 支持双向 `PartialEq` 比较。^[atlas-core-value-layer.md:15-24]

## 字节表示与 Unicode 预览

`AtlasString` 的 `Display` 仅用于 Unicode 编辑或调试预览，不能作为原始字节的权威表示。保留字节的打印接口是 `Value::atlas_text` 与 `append_atlas_text`，其打印边界不进行 Unicode 转换。^[atlas-core-value-layer.md:20-24]

## 复合值的递归打印

`atlas_text` 与 `append_atlas_text` 将字符串原始字节置于引号内，对 `Tuple` 和 `List` 递归打印，并将 `Union` 打印为 `value.injectorname`。这些规则共同规定了字符串及相关复合值的字节输出形式。^[atlas-core-value-layer.md:22-24]

其他值的输出格式可参见 [[上游兼容的值打印约定]]。例如，`Value` 的 `Display` 对有理数单独处理符号，负值打印为 `-num/den`，即使分母为 1 也保留分母；内建函数则打印为 `{print_name}`。^[atlas-core-value-layer.md:25-29]

## 证据边界

来源包覆盖 `value.rs`、`linear_values.rs` 与 `formula.rs`，属于结构性阅读，不声称语言或数学验收。包中记录的测试总数为 11 个，其中 `value` 4 个、`linear_values` 3 个、`formula` 4 个；这一计数未列出 `AtlasString` 的具体测试覆盖内容。^[atlas-core-value-layer.md:9-11, atlas-core-value-layer.md:69-74]

来源中的上游行号引用属于实现方的移植陈述，值打印兼容性仍以 HPC 语料门为准。字节数与哈希仅标识 Git base `964f0033` 对应快照的字节，不能替代兼容性验收证据。^[atlas-core-value-layer.md:71-74]

## Sources

- [atlas-core-value-layer.md](../../sources/atlas-core-value-layer.md) — 值层（value.rs + linear_values.rs + formula.rs）。
