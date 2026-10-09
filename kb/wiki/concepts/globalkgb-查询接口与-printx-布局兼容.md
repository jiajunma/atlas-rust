---
title: GlobalKgb 查询接口与 print_X 布局兼容
summary: 查询层使用扁平表和返回 None 的边界访问，status 与 cross 的参数顺序及环面标签错误处理存在差异；打印层复现 print_X 的字符格式、字段宽度和缺失标记。
sources:
  - global-kgb.md
kind: concept
createdAt: "2026-10-09T14:49:39.618Z"
updatedAt: "2026-10-09T14:49:39.618Z"
tags:
  - 查询接口
  - 打印布局
  - 兼容性
aliases:
  - globalkgb-查询接口与-printx-布局兼容
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# GlobalKgb 查询接口与 print_X 布局兼容

`GlobalKgb` 枚举同一内类全部强实形的 KGB 元素，并按扭对合划分 tau 包。其查询与打印层移植了上游 `kgb::global_KGB` 的相关接口及 `kgb_io::print_X` 版式；从任意 `GlobalTitsElement` 播种的第二构造器和 Bruhat/Hasse 层尚未移植。^[global-kgb.md:10-15]

## 查询接口

cross、Cayley 等表使用扁平索引 `x * semisimple_rank + generator`。访问器通过 `.get` 读取，越界时返回 `None`。调用时需特别注意参数顺序：`status(element, generator)` 与 `cross(generator, element)` 相反，后者沿用上游 `KGB_base::cross(s, x)` 的约定。^[global-kgb.md:84-87]

环面标签的两条访问路径具有不同的错误语义：`torus_label()` 将 `log_2pi` 的错误转为 `None`，而 `print_layout` 会传播该错误。因此，标签查询返回 `None` 与打印布局构造失败不能视为相同的接口行为。^[global-kgb.md:87-88]

`GlobalKgb` 仅派生 `Clone, Debug`，没有实现 `Eq`；快照比较需借助 `GlobalKgbPrint`。^[global-kgb.md:89-90]

## print_X 布局约定

`render` 逐项复现上游 `setw` 填充规则：元素号宽度为 `digits(size−1)`，Cartan 类号和 length 的宽度取末行对应值的位数，标签宽度为 `3·lattice_rank+3`，缺失的 Cayley 链接显示为 `*`。其中，末行决定列宽是具体兼容约定。^[global-kgb.md:90-92]

每个 tau 包的打印字由 `canonical_involution_expr` 经 `format_involution_word` 生成：非负编码 `n` 按字符 `'1'+n` 加 `^` 打印，`!n` 编码加 `x`，末尾补 `e`；实现保留上游字符处理的特殊行为。相关背景见 [[tau packet 与 KGB 元素编号标准化]] 和 [[对合表达式的打印约定]]。^[global-kgb.md:60-62]

打印头偏移先由正根的余根坐标累加得到 `dual_two_rho`，再通过 `exp_2pi(dual_two_rho, 4).log_2pi()` 计算。^[global-kgb.md:75-76]

环面标签保留算术历史，而非始终采用规范代表元：构造入口进行约化，但 `simple_reflect` 后不再约化，因此输出可含负分子。SC B2 的元素 15 标签 `[0,-1]/2` 是这一行为的测试锚点；相关表示纪律见 [[全局环面元素的算术历史表示]]。^[global-kgb.md:19-29]

## 兼容性证据与覆盖边界

逐字节打印测试对照 `tests/reference/domain/print_x.events.json` 中的三个 `print_X` 块，覆盖 SC A1 的 5 行输出及头 `[1]/4`、adjoint A1 的 3 行输出及头 `[1]/2`，以及 SC B2 的 17 行输出及头 `[0,3]/4`。B2 用例还包含负分子标签 `[0,-1]/2`。^[global-kgb.md:96-98]

另一个 B2 结构测试检查 17 个元素、包大小 `[8,2,2,2,2,1]`、包字 `["e","1^e","2^e","1x2^e","2x1^e","1^2x1^e"]`，以及 cross 对合性与 Cayley 配对。更完整的测试背景见 [[GlobalKgb 的回归测试与证据边界]]。^[global-kgb.md:98-100]

现有测试未覆盖错误分支。半单秩为 0 的平凡群和一维环面也有意未测，因为共享内类机制会在空生成元集合上 panic。此外，维度、非零分母及直接索引等隐式前提遭到破坏时可能 panic；部分 `2 * denominator` 运算在 debug 下溢出 panic，在 release 下回绕。^[global-kgb.md:101-105]

上述材料提供源码阅读结论与现有测试锚点，不构成数学验收。源文档未核对上游源码字节，本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[global-kgb.md:14-15, global-kgb.md:109-113]

## Sources

- [global-kgb.md](../../sources/global-kgb.md) — 内类范围 KGB 图与 print_X 布局（global_kgb.rs）
