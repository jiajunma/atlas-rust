---
title: WeylElement 的置换表示与长度下降不变量
summary: 元素维护正向置换、逆向量及缓存长度，左右下降分别读取逆向与正向根像，乘积重算长度且环境根系一致性由调用方保证。
sources:
  - weyl-layer.md
kind: concept
createdAt: "2026-10-09T15:18:03.389Z"
updatedAt: "2026-10-10T00:56:22.879Z"
tags:
  - Weyl群
  - 算法不变量
aliases:
  - weylelement-的置换表示与长度下降不变量
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: WeylElement 的置换表示与长度下降不变量
summary: WeylElement 以环境根系的根置换表示元素，维护逆置换与缓存长度；左右下降分别读取逆向和正向根像，同一环境根系由调用方保证。
sources:
  - weyl-layer.md
kind: concept
tags:
  - Weyl群
  - 不变量
aliases:
  - weylelement-的置换表示与长度下降不变量
provenanceState: extracted
---

# WeylElement 的置换表示与长度下降不变量

`WeylElement` 是 Weyl 群的词级组合表示，以环境（ambient）`RootSystem` 的枚举根上的置换表示元素，提供 O(1) 的长度与下降查询，以及乘法、求逆、扭曲共轭和按需约化词。它与矩阵级 `WeylAction` 构成[[Weyl 群的矩阵作用与词级元素双层结构]]，通过 `RootSystem::action_permutation` 与 `WeylElement::from_action` 衔接。^[weyl-layer.md:19-26]

## 表示与环境根系契约

构造入口包括 `identity`、`simple_reflection` 和 `from_action`；查询接口包括 `length`、`is_identity`、`image` 和 `image_permutation`。置换的反对称性依赖“构造器是唯一入口”的纪律。^[weyl-layer.md:66-71]

每次操作能够执行的来源一致性检查仅是根数匹配。所有元素属于同一个环境根系的要求是调用方契约，由 KGB stages 负责维持；根数匹配本身不能替代这一要求。^[weyl-layer.md:66-68]

## 长度与左右下降

设 $s$ 为简单生成元，$\alpha_s$ 为对应单根。`has_left_descent` 读取**逆向量**，以 $w^{-1}(\alpha_s)<0$ 判定 $\ell(sw)<\ell(w)$；`has_right_descent` 读取**正向置换**，以 $w(\alpha_s)<0$ 判定 $\ell(ws)<\ell(w)$。长度与这两种下降查询均为 O(1)。^[weyl-layer.md:22-24, weyl-layer.md:71-74]

`left_multiply_simple` 和 `right_multiply_simple` 报告长度变化：$-1$ 对应 `sigma_mult` 分支，$+1$ 对应 `sigma_inv_mult` 分支。^[weyl-layer.md:77-79]

## 运算中的不变量维护

`multiply` 的复合约定与 `WeylAction::compose` 一致，即 `self` 在右操作数之后作用。乘法在同一趟计算中利用 $(uv)^{-1}=v^{-1}u^{-1}$ 维护逆置换，并从 positivity slice 重新计算结果长度，而不是将操作数长度相加。`inverse` 直接返回逆。^[weyl-layer.md:37-38, weyl-layer.md:75-79]

`twisted_conjugate` 计算 $s_{\mathrm{gen}}\,w\,s_{\mathrm{twist}(\mathrm{gen})}$，是 Tits 扭曲共轭在 Weyl 层的对应操作。`twist` 必须是生成元上的对合置换；它与 distinguished involution 的单根作用一致，属于调用方契约。stage (b) 所需的长度变化 $d\in\{0,\pm2\}$ 由调用点通过缓存长度相减获得，相关概念见[[Weyl 元素的扭曲共轭]]。^[weyl-layer.md:80-84]

## 下降剥离与规范约化词

`reduced_word` 按最小左下降逐步剥离，生成按左到右复合的约化词。`canonical_word` 则使用 `WeylInterface` 携带的内部生成元顺序，选取该顺序下字典序最小的约化词，对应上游所称的 “minimal for ShortLex”。^[weyl-layer.md:88-94]

这一选择的依据是：能成为约化表达式首字母的生成元恰好是左下降，因此逐次剥离最小内部左下降能够逐位选出最小字母。实现检查每次剥离后长度必须恰好减少一，否则返回 `WeylElementInvariantViolation`；详见[[基于左下降剥离的规范约化词]]。^[weyl-layer.md:89-94]

内部生成元顺序由重编号确定：Dynkin 分量按分类顺序排列，分量内的 Bourbaki `position` 对 A/E/F/G 型直接使用，对 B/C/D 型反转；`outward()` 将内部编号映射到 datum 生成元编号。这一顺序影响规范词的选择，也影响 [[ParabolicPieces 的抛物分解与排序键]]中的 piece 索引。^[weyl-layer.md:95-105]

## 证据边界

来源材料是对 `weyl.rs` 与 `weyl_element.rs` 的结构性阅读；2026-10-06 的重读仅覆盖 `weyl.rs`。本来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；Weyl 层的正确性属于其独立的 HPC 证据链。^[weyl-layer.md:9-15, weyl-layer.md:109-121]

来源中的上游行号转述自源码注释，未独立重读上游文件，可能随版本演进而变化。因此，上游算法对应关系应保留这一证据限制。^[weyl-layer.md:115-116]

## Sources

- [weyl-layer.md](../../sources/weyl-layer.md) — Weyl 群层：矩阵作用与词级元素的双层结构。
