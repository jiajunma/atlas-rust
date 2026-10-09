---
title: ambient Weyl 见证词与作用方向
summary: Weyl_orbit_ws 按权的从右到左作用和余权的从左到右作用组织反射事件，经 ambient Weyl 上下文逐次右乘重建元素并冻结 canonical word。
sources:
  - atlas-core-weyl-subgroup.md
kind: concept
createdAt: "2026-10-09T14:39:36.694Z"
updatedAt: "2026-10-09T14:39:36.694Z"
tags:
  - Weyl群
  - 见证词
  - 对偶作用
aliases:
  - ambient-weyl-见证词与作用方向
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# ambient Weyl 见证词与作用方向

`Weyl_orbit_ws` 为反射子群轨道返回 ambient Weyl 群中的见证词；对应的 `Weyl_orbit` 将轨道向量作为矩阵列返回。见证词的作用方向取决于输入是权还是余权，因此构造时必须区分反射事件的应用顺序与词的排列顺序。^[atlas-core-weyl-subgroup.md:44-51]

## 反射事件与作用方向

优势化过程 `dominant` 每次选择首个 level 为负的给定生成元，执行反射，并按实际应用顺序记录事件。权与余权两种作用都采用这一事件记录规则；优势化使用的是指定子群的生成元。^[atlas-core-weyl-subgroup.md:18-23, atlas-core-weyl-subgroup.md:34-37]

生成见证词时，**权从右到左作用，余权从左到右作用**。因此，dual 情形按事件正序拼接，非 dual 情形按事件逆序拼接。`Weyl_orbit_ws` 的 dual 参数顺序为 `vec, rd, gens`，实现通过首个参数不是 RootDatum 来识别该形式。相关约定可参见 [[Weyl 词对根与权的作用顺序]]。^[atlas-core-weyl-subgroup.md:27-28, atlas-core-weyl-subgroup.md:46-48]

## 从陪集树到规范词

轨道构造沿陪集树逐个非稳定器生成元扩展。生成元可以是任意有符号根号，只要配对合法，并不限于单根。用户给定的生成元顺序控制优势化和扩展，而内部稳定器 BFS 使用 RootNbr 顺序；这两个顺序承担不同职责。^[atlas-core-weyl-subgroup.md:47-48, atlas-core-weyl-subgroup.md:68-69]

拼接后的词通过 `build_weyl_context` 建立上下文，再逐步调用 `right_multiply_simple` 重建 Weyl 元素，最后由 `weyl_elt_value` 冻结为 canonical word。因此，返回表示经过了元素重建与规范化，可与 [[Weyl 元素的规范词]] 联系阅读。^[atlas-core-weyl-subgroup.md:48-51]

## 空子群反例与一致性要求

原版 `7e1b958c` 的 `rootdata.h` 在转发 `make_(co)dominant` 时忽略生成元参数 `g`，见证函数却正确传递了它。这导致 R3 反例：在 A2 中，空生成元集合与输入 `[-1,-2]` 得到的原版矩阵轨道为 `[2,1]`，见证却是恒等元素。Rust 实现的初始优势化确实使用给定生成元，相关背景见 [[反射子群轨道与生成元约束下的优势化]]。^[atlas-core-weyl-subgroup.md:18-23]

文件内的测试检查空子群保持负权不动并返回 ambient 恒等元素，覆盖九个类型、两种编号、两种 isogeny 和两种作用。另有测试将陪集轨道与独立穷举闭包比较，并通过 `word`／`root_datum` 往返及逐见证 `*` 运算重建轨道列；全单子群测试则逐项比较完整全群的轨道与见证输出。^[atlas-core-weyl-subgroup.md:56-64]

## 证据范围

本页依据的源文档仅覆盖 `domain_builtins/weyl_subgroup.rs`，记录的是结构性阅读与文件内测试锚点，不构成数学验收声明。见证重建的验证背景可参见 [[反射子群轨道与见证的独立验证]]；数学验收需另查相应 HPC 门与证据链。^[atlas-core-weyl-subgroup.md:9-14, atlas-core-weyl-subgroup.md:56-64, atlas-core-weyl-subgroup.md:70-72]

## Sources

- [atlas-core-weyl-subgroup.md](../../sources/atlas-core-weyl-subgroup.md)
