---
title: ambient Weyl 见证词与作用方向
summary: Weyl_orbit_ws 按权从右到左、余权从左到右的作用约定拼接反射事件，在 ambient Weyl 上下文中逐次右乘重建元素并冻结 canonical word。
sources:
  - atlas-core-weyl-subgroup.md
kind: concept
createdAt: "2026-10-09T14:39:36.694Z"
updatedAt: "2026-10-09T20:47:10.400Z"
tags:
  - Weyl群
  - 见证词
  - 作用约定
aliases:
  - ambient-weyl-见证词与作用方向
  - AW见
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: ambient Weyl 见证词与作用方向
summary: Weyl_orbit_ws 根据权与余权的作用方向排列反射事件，经 ambient Weyl 上下文重建元素并冻结规范词；优势化与见证构造均受给定子群生成元约束。
sources:
  - atlas-core-weyl-subgroup.md
kind: concept
tags:
  - Weyl群
  - 见证词
  - 对偶作用
aliases:
  - ambient-weyl-见证词与作用方向
---

# ambient Weyl 见证词与作用方向

`Weyl_orbit_ws` 为反射子群轨道返回 ambient Weyl 群中的见证词；对应的 `Weyl_orbit` 将轨道向量作为矩阵列返回。见证词按权与余权不同的作用方向组织，并经过 Weyl 元素重建与规范化。^[atlas-core-weyl-subgroup.md:44-51]

## 反射事件与作用方向

优势化过程 `dominant` 每次选择首个 level 为负的给定生成元，执行反射，并按实际应用顺序记录事件。权与余权都采用这一记录规则，且初始优势化确实使用指定子群的生成元，参见 [[反射子群轨道与生成元约束下的优势化]]。^[atlas-core-weyl-subgroup.md:18-23, atlas-core-weyl-subgroup.md:34-37]

生成见证词时，**权从右到左作用，余权从左到右作用**：dual 情形按事件正序拼接，非 dual 情形按事件逆序拼接。因此，事件的记录顺序与最终词的排列顺序需要区分。dual 形式的参数顺序为 `vec, rd, gens`，实现通过首个参数不是 RootDatum 来识别它。^[atlas-core-weyl-subgroup.md:27-28, atlas-core-weyl-subgroup.md:46-48]

## 从陪集树到规范词

见证构造沿陪集树逐个非稳定器生成元扩展。生成元可以是任意有符号根号，只要配对合法，并不限于单根。用户给定的生成元顺序控制优势化与扩展，内部稳定器 BFS 则使用 RootNbr 顺序；这些排序职责见 [[反射子群轨道的稳定排序规则]]。^[atlas-core-weyl-subgroup.md:47-48, atlas-core-weyl-subgroup.md:68-69]

拼接后的词通过 `build_weyl_context` 建立上下文，再逐次调用 `right_multiply_simple` 重建 Weyl 元素，最后由 `weyl_elt_value` 冻结为 canonical word。返回值因此经过元素重建与规范化，相关表示见 [[Weyl 元素的规范词]]。^[atlas-core-weyl-subgroup.md:48-51]

## 空子群反例与一致性

原版 `7e1b958c` 的 `rootdata.h` 在转发 `make_(co)dominant` 时忽略生成元参数 `g`，见证函数却正确传递了它。这导致 R3 反例：A2 中，空生成元集合与输入 `[-1,-2]` 得到的原版矩阵轨道为 `[2,1]`，见证却是恒等元素。Rust 实现的初始优势化使用给定生成元，保留了平凡子群应保持输入不动这一不变量。^[atlas-core-weyl-subgroup.md:18-23, atlas-core-weyl-subgroup.md:58-59]

文件内的空子群测试检查负权保持不动并返回 ambient 恒等元素，覆盖九个类型、两种编号、两种 isogeny 和两种作用。另有测试将陪集轨道与独立穷举闭包比较，通过 `word`／`root_datum` 往返及逐见证 `*` 运算重建轨道列；全单子群测试则逐项比较完整全群的轨道与见证输出。参见 [[反射子群轨道与见证的独立验证]]。^[atlas-core-weyl-subgroup.md:56-62]

## 校验与证据范围

`validate` 采用 [[BuildAndDrop 领域内建验证策略]]，仅执行 `Subgroup::new`，不计算轨道。向量大小不匹配时返回 `"Wrong vector size for Weyl subgroup orbit"`；实现不模拟上游短向量越界读取行为。^[atlas-core-weyl-subgroup.md:52-54]

本页来源仅覆盖 `domain_builtins/weyl_subgroup.rs`，记录的是结构性阅读与文件内测试锚点，不构成数学验收声明。相应数学验收仍需查阅 `docs/slices` 中的 `weyl_subgroup_orbits` 切片及其下游证据。^[atlas-core-weyl-subgroup.md:9-14, atlas-core-weyl-subgroup.md:56-64, atlas-core-weyl-subgroup.md:70-72]

## Sources

- [atlas-core-weyl-subgroup.md](../../sources/atlas-core-weyl-subgroup.md) — 反射子群轨道与 ambient Weyl 见证（domain_builtins/weyl_subgroup.rs）。
