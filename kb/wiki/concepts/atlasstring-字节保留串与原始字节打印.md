---
title: AtlasString 字节保留串与原始字节打印
summary: AtlasString 保存原始字节，Display 仅供 Unicode 预览；atlas_text 与 append_atlas_text 递归打印字符串、容器和联合值并保留原始字节。
sources:
  - atlas-core-value-layer.md
kind: concept
createdAt: "2026-10-09T14:38:38.408Z"
updatedAt: "2026-10-10T00:25:38.835Z"
tags:
  - 字符串
  - 字节保真
aliases:
  - atlasstring-字节保留串与原始字节打印
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: AtlasString 字节保留串与原始字节打印
summary: AtlasString 使用 Vec<u8> 保留原始字节；Display 仅供 Unicode 预览，Value 的 atlas_text 与 append_atlas_text 提供无 Unicode 转换的递归打印。
sources:
  - atlas-core-value-layer.md
kind: concept
tags:
  - 字符串
  - 字节保真
  - 打印
aliases:
  - atlasstring-字节保留串与原始字节打印
---

# AtlasString 字节保留串与原始字节打印

`AtlasString` 是 Atlas 值层的**字节保留串**，以 `Vec<u8>` 保存内容，并作为 `Value::String(AtlasString)` 的载荷。它与 `str`、`&str`、`String` 支持双向 `PartialEq` 比较。^[atlas-core-value-layer.md:15-24]

## 字节表示与 Unicode 预览

`AtlasString` 的 `Display` 仅用于 Unicode 编辑或调试预览，不能作为原始字节的权威表示。`Value::atlas_text` 与 `append_atlas_text` 则提供不经过 Unicode 转换的打印路径，保留字符串原始字节。^[atlas-core-value-layer.md:20-24]

## 复合值的递归打印

`atlas_text` 与 `append_atlas_text` 将字符串原始字节置于引号内，对 `Tuple` 和 `List` 递归打印，并将 `Union` 打印为 `value.injectorname`。因此，字节保留规则也适用于这些复合值中的字符串。^[atlas-core-value-layer.md:22-24]

其他值的输出格式可结合 [[上游兼容的值打印约定]] 阅读：`Value` 的 `Display` 对有理数单独处理符号，负值打印为 `-num/den`，即使分母为 1 也保留分母；内建函数打印为 `{print_name}`。^[atlas-core-value-layer.md:25-29]

## 测试与证据边界

来源包覆盖 `value.rs`、`linear_values.rs` 和 `formula.rs`，属于结构性阅读，不声称语言或数学验收。包中记录了 11 个测试，其中 `value` 4 个、`linear_values` 3 个、`formula` 4 个；这一汇总没有说明 `AtlasString` 的具体测试覆盖范围。^[atlas-core-value-layer.md:9-11, atlas-core-value-layer.md:69-74]

来源中的上游行号引用属于实现方的移植陈述，值打印兼容性以 HPC 语料门为准，可参见 [[HPC 验收证据链]]。字节数与哈希仅标识本快照字节，来源记录的 Git base 为 `964f0033`，不能据此视为兼容性验收通过。^[atlas-core-value-layer.md:71-74]

## Sources

- [atlas-core-value-layer.md](../../sources/atlas-core-value-layer.md) — 值层（value.rs + linear_values.rs + formula.rs）。
