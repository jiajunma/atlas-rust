---
title: Weyl 元素的扭曲共轭
summary: twisted_conjugate 计算 s_gen·w·s_twist(gen)，要求 twist 为生成子对合置换，其与 distinguished involution 的一致性由调用方保证。
sources:
  - weyl-layer.md
kind: concept
createdAt: "2026-10-09T15:17:54.222Z"
updatedAt: "2026-10-09T15:17:54.222Z"
tags:
  - Weyl群
  - 扭曲共轭
  - 调用方契约
aliases:
  - weyl-元素的扭曲共轭
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Weyl 元素的扭曲共轭

Weyl 元素的扭曲共轭由词级组合层 `WeylElement::twisted_conjugate` 实现。对生成元编号 `gen`，它计算
\[
s_{\mathrm{gen}}\cdot w\cdot s_{\mathrm{twist}(\mathrm{gen})},
\]
其中 \(w\) 是当前元素。该操作是 Tits 扭曲共轭在 Weyl 层的影子。^[weyl-layer.md:80-84]

## twist 与调用方契约

`twist` 必须是生成元集合上的对合置换；它与 distinguished involution 在单根上的作用一致，是调用方必须保证的契约。公式左侧使用生成元 `gen`，右侧使用其经 `twist` 映射后的生成元。^[weyl-layer.md:80-84]

`WeylElement` 用 ambient `RootSystem` 的枚举根置换表示元素。每次操作能够表达的来源检查仅为根数匹配，因此同一 ambient system 的使用纪律由调用方负责，具体由 KGB stages 维持；置换的反对称性则通过构造器作为唯一入口来保证。相关表示约束见 [[WeylElement 的置换表示与长度下降不变量]]。^[weyl-layer.md:64-68]

## 长度变化与下降查询

KGB stage (b) 使用扭曲共轭前后的长度变化
\[
d=\ell\!\left(s_{\mathrm{gen}}w s_{\mathrm{twist}(\mathrm{gen})}\right)-\ell(w)
\in\{0,\pm2\}.
\]
调用点通过缓存长度相减取得这个变化量。词级组合层提供 \(O(1)\) 的长度与下降查询。^[weyl-layer.md:22-24, weyl-layer.md:80-84]

左右下降的判定分别为
\[
\ell(s w)<\ell(w)\iff w^{-1}(\alpha_s)<0,
\qquad
\ell(w s)<\ell(w)\iff w(\alpha_s)<0.
\]
因此 `has_left_descent` 读取逆向量，`has_right_descent` 读取正向置换。一般乘法会从 positivity slice 重新计算长度，不能将操作数长度直接相加。^[weyl-layer.md:70-79]

## 所属层次与证据边界

扭曲共轭属于 [[Weyl 群的矩阵作用与词级元素双层结构]] 中的词级组合层。矩阵级 `WeylAction` 同时作用于 character 与 cocharacter 两个全格；两层通过 `RootSystem::action_permutation` 和 `WeylElement::from_action` 桥接。^[weyl-layer.md:17-26]

本页依据的来源是结构性源码阅读，未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。Weyl 层正确性由其自身的 [[HPC 验收证据链]] 支持，本来源不扩展该证据范围。^[weyl-layer.md:9-12, weyl-layer.md:121-124]

## Sources

- [weyl-layer.md](weyl-layer.md) — Weyl 群层：矩阵作用与词级元素的双层结构。
