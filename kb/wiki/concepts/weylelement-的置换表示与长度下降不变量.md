---
title: WeylElement 的置换表示与长度下降不变量
summary: 以正向置换、逆向量和缓存长度支持常数时间长度及左右 descent 查询，乘法重算长度，而单一 ambient RootSystem 的一致性由调用方保证。
sources:
  - weyl-layer.md
kind: concept
createdAt: "2026-10-09T15:18:03.389Z"
updatedAt: "2026-10-09T15:18:03.389Z"
tags:
  - Weyl群
  - 置换表示
  - 算法不变量
aliases:
  - weylelement-的置换表示与长度下降不变量
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# WeylElement 的置换表示与长度下降不变量

`WeylElement` 是 Weyl 群的词级组合表示，以 ambient `RootSystem` 的枚举根上的置换表示元素，提供 O(1) 的长度与下降查询，以及乘法、逆、扭曲共轭和按需约化词。它与矩阵级 `WeylAction` 构成[[Weyl 群的矩阵作用与词级元素双层结构]]，通过 `RootSystem::action_permutation` 与 `WeylElement::from_action` 连接。^[weyl-layer.md:19-26]

## 置换表示与来源约束

构造入口包括 `identity`、`simple_reflection` 和 `from_action`；查询接口包括 `length`、`is_identity`、`image` 和 `image_permutation`。置换的反对称性依赖“构造器是唯一入口”这一纪律保证。^[weyl-layer.md:66-71]

每次操作能够进行的来源（provenance）检查仅是根数匹配。元素必须属于同一个 ambient root system 的要求由调用方遵守，并由 KGB stages 负责；根数匹配本身并不替代这一上下文契约。^[weyl-layer.md:66-68]

## 长度与左右下降

设 \(s\) 是简单生成元，\(\alpha_s\) 是对应单根。`has_left_descent` 使用逆向量判断左下降，其条件为：^[weyl-layer.md:71-74]

\[
\ell(sw)<\ell(w)
\quad\Longleftrightarrow\quad
w^{-1}(\alpha_s)<0.
\]

`has_right_descent` 使用正向置换判断右下降，其条件为：^[weyl-layer.md:73-74]

\[
\ell(ws)<\ell(w)
\quad\Longleftrightarrow\quad
w(\alpha_s)<0.
\]

因此，左右下降查询读取的方向不同：左下降读取逆作用，右下降读取正向作用。两者与缓存的长度一起，支持 O(1) 查询。^[weyl-layer.md:22-24, weyl-layer.md:71-74]

## 运算中的不变量维护

`multiply` 的复合约定与 `WeylAction::compose` 一致，即 `self` 在右操作数之后作用。乘法在同一趟计算中依据 \((uv)^{-1}=v^{-1}u^{-1}\) 维护逆置换，并从 positivity slice 重新计算结果长度，而不是将操作数长度相加。`inverse` 则直接返回逆。^[weyl-layer.md:37-38, weyl-layer.md:75-79]

`left_multiply_simple` 和 `right_multiply_simple` 报告长度变化：\(-1\) 对应 `sigma_mult` 分支，\(+1\) 对应 `sigma_inv_mult` 分支。简单生成元乘法因而提供了明确的升降信息。^[weyl-layer.md:77-79]

[[Weyl 元素的扭曲共轭]]由 `twisted_conjugate` 实现，计算 \(s_{\mathrm{gen}}\,w\,s_{\mathrm{twist}(\mathrm{gen})}\)。`twist` 必须是生成元上的对合置换，其与 distinguished involution 的单根作用一致是调用方契约。stage (b) 所需的长度变化 \(d\in\{0,\pm2\}\) 由调用点通过缓存长度相减得到。^[weyl-layer.md:80-84]

## 下降剥离与约化词

`reduced_word` 按最小左下降逐步剥离，生成按左到右复合的约化词。`canonical_word` 则在 `WeylInterface` 的内部生成元顺序下选择字典序最小的约化词，对应上游称为 “minimal for ShortLex” 的规范选择。^[weyl-layer.md:88-94]

这一选择依据是：能够成为约化表达式首字母的生成元恰好是左下降。因此，每一步选取最小内部左下降，就能逐位确定最小字母。实现同时检查每次剥离后的长度必须恰好减少一，否则返回 `WeylElementInvariantViolation`。参见[[基于左下降剥离的规范约化词]]。^[weyl-layer.md:89-94]

内部生成元顺序包含重编号：Dynkin 分量按分类顺序排列，分量内的 Bourbaki `position` 对 A/E/F/G 型直接使用，对 B/C/D 型反转；`outward()` 将内部编号映射回 datum 生成元编号。这一顺序影响 `canonical_word` 的选词，也影响 [[ParabolicPieces 的抛物分解与排序键]]。^[weyl-layer.md:95-105]

## 证据边界

本页依据来源包对实现的结构性阅读。该来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；其中上游行号转述自源码注释，未独立重读上游，可能随版本变化。^[weyl-layer.md:9-15, weyl-layer.md:115-121]

## Sources

- [weyl-layer.md](weyl-layer.md)
