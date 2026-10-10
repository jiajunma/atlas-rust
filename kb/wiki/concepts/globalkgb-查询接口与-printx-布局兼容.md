---
title: GlobalKgb 查询接口与 print_X 布局兼容
summary: 查询使用扁平表与越界返回 None 的访问器，打印保留字段宽度及缺失标记；status/cross 参数顺序相反，环面标签查询与打印的错误传播不同。
sources:
  - global-kgb.md
kind: concept
createdAt: "2026-10-09T14:49:39.618Z"
updatedAt: "2026-10-10T00:33:57.434Z"
tags:
  - kgb
  - 接口设计
  - 打印兼容
aliases:
  - globalkgb-查询接口与-printx-布局兼容
  - G查P布
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: GlobalKgb 查询接口与 print_X 布局兼容
summary: GlobalKgb 使用扁平查询表，访问越界返回 None；print_X 保留字段宽度、打印字与环面标签的算术历史，标签查询与打印布局的错误传播不同。
sources:
  - global-kgb.md
kind: concept
tags:
  - KGB
  - 接口契约
  - 打印兼容
aliases:
  - globalkgb-查询接口与-printx-布局兼容
  - G查P布
---

# GlobalKgb 查询接口与 print_X 布局兼容

`GlobalKgb` 枚举同一内类全部强实形的 KGB 元素，并按扭对合划分 tau 包。实现移植了上游 `kgb::global_KGB` 的相关功能与 `kgb_io::print_X` 版式；从任意 `GlobalTitsElement` 播种的第二构造器及 Bruhat/Hasse 层尚未移植。^[global-kgb.md:10-15]

## 查询接口与错误语义

cross、Cayley 等表采用扁平索引 `x * semisimple_rank + generator`，访问器通过 `.get` 读取，越界返回 `None`。调用时须注意参数顺序：`status(element, generator)` 与 `cross(generator, element)` 相反，后者沿用上游 `KGB_base::cross(s, x)` 的约定。^[global-kgb.md:84-87]

环面标签查询与打印布局构造具有不同的错误语义：`torus_label()` 将 `log_2pi` 的错误转为 `None`，`print_layout` 则传播同一错误。`GlobalKgb` 仅派生 `Clone, Debug`，没有实现 `Eq`；快照比较需借助 `GlobalKgbPrint`。^[global-kgb.md:87-90]

## print_X 布局约定

`render` 逐项复现上游 `setw` 填充：元素号宽度为 `digits(size−1)`，Cartan 类号与 length 的宽度取**末行**对应值的位数，标签宽度为 `3·lattice_rank+3`，缺失的 Cayley 链接显示为 `*`。^[global-kgb.md:90-92]

每个 tau 包的打印字由 `canonical_involution_expr` 经 `format_involution_word` 生成：`n≥0` 时打印字符 `'1'+n` 并加 `^`，`!n` 编码加 `x`，末尾补 `e`。实现保留上游字符处理的特殊行为；这些打印字在 [[GlobalKgb 的分阶段广度优先构造]] 中作为每包派生数据生成。^[global-kgb.md:60-62]

打印头偏移先累加正根的余根坐标，得到 `dual_two_rho`，再通过 `exp_2pi(dual_two_rho, 4).log_2pi()` 计算。^[global-kgb.md:75-76]

环面标签保留[[全局环面元素的算术历史表示]]：构造入口进行约化，但 `simple_reflect` 后不再约化，因此输出可以包含负分子。B2 元素 15 的 `[0,-1]/2` 是这一行为的测试锚点；`log_2pi` 返回分子与两倍分母之比，仅进行 gcd 归一化。^[global-kgb.md:19-29]

## 测试锚点与覆盖边界

逐字节打印测试对照 `tests/reference/domain/print_x.events.json` 的三个 `print_X` 块，覆盖 SC A1 的 5 行输出及头 `[1]/4`、adjoint A1 的 3 行输出及头 `[1]/2`，以及 SC B2 的 17 行输出及头 `[0,3]/4`。B2 用例还包含负分子标签 `[0,-1]/2`。^[global-kgb.md:96-98]

另一个 B2 结构测试检查 17 个元素、包大小 `[8,2,2,2,2,1]`、包字 `["e","1^e","2^e","1x2^e","2x1^e","1^2x1^e"]`，以及 cross 对合性与 Cayley 配对。相关说明见 [[GlobalKgb 的回归测试与证据边界]]。^[global-kgb.md:98-100]

现有测试没有覆盖错误分支。半单秩为 0 的平凡群和一维环面也有意未测，因为共享内类机制会在空生成元集合上 panic，修复超出本模块范围。维度匹配、非零分母及直接下标索引等隐式前提遭到破坏时会 panic；`reduce_raw`、`evaluate_at` 等处的 `2 * denominator` 使用普通乘法，溢出时在 debug 下 panic、在 release 下回绕。^[global-kgb.md:101-105]

本页依据结构性源码阅读与源文档记录的测试锚点，不构成数学验收。源文档中的上游行号转录自代码注释，未核对上游文件字节；该次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[global-kgb.md:9-15, global-kgb.md:109-113]

## Sources

- [global-kgb.md](../../sources/global-kgb.md) — 内类范围 KGB 图与 print_X 布局（global_kgb.rs）。
