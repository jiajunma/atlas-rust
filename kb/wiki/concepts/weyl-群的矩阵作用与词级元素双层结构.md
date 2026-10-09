---
title: Weyl 群的矩阵作用与词级元素双层结构
summary: WeylAction 表示携带根 datum 的全格矩阵作用，WeylElement 表示枚举根的置换，两层通过 action_permutation 与 from_action 桥接互查。
sources:
  - weyl-layer.md
kind: concept
createdAt: "2026-10-09T15:17:23.076Z"
updatedAt: "2026-10-09T15:17:23.076Z"
tags:
  - Weyl群
  - 系统设计
  - 表示转换
aliases:
  - weyl-群的矩阵作用与词级元素双层结构
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Weyl 群的矩阵作用与词级元素双层结构

Weyl 群层分为矩阵级作用 `WeylAction` 与词级组合元素 `WeylElement`：前者在 character/cocharacter 两个全格上表示作用，并携带根数据来源；后者用 ambient `RootSystem` 的枚举根置换表示元素，支持长度、下降、乘法和约化词查询。两层通过 `RootSystem::action_permutation` 与 `WeylElement::from_action` 衔接。^[weyl-layer.md:17-26]

## 矩阵级作用：WeylAction

`WeylAction` 保存 `datum: Arc<BasedRootDatum>`、`weight_matrix` 和 `coweight_matrix`。单位元使用 `lattice_rank` 阶单位矩阵；简单反射与任意根反射依据根、余根的对偶配对构造，反射矩阵满足 \(M_{ij}=\delta_{ij}-\mathrm{reflected}_i\mathrm{pairing}_j\)。任意根反射与根的符号无关，可用于 Cayley/cross 分解的重放。相关概念见 [[WeylAction 的对偶全格作用]]。^[weyl-layer.md:28-36]

`compose` 的方向是先作用 `right`，再作用 `self`；`act` 与 `act_on_coweight` 分别作用于两个格。完整作用枚举独立于单个作用的构造：`WeylGroup::enumerate_actions(budget)` 接受显式基数预算，通过 `CompactWeyl` 枚举后并行物化矩阵。来源描述其结果按 character-lattice 作用矩阵的字典序排列，但同时指出这一排序没有测试断言。^[weyl-layer.md:37-41, weyl-layer.md:52-55]

等值语义需要区分概述与实现。概述将等值描述为矩阵作用相等；实际派生比较逐字段进行，也包含 datum 的值。由于 `Arc` 的 `PartialEq` 比较内层值，矩阵相同但 datum 值不同的作用仍不相等。参见 [[WeylAction 的等值与 datum 身份语义]]。^[weyl-layer.md:19-21, weyl-layer.md:43-47]

## 词级元素：WeylElement

`WeylElement` 是 KGB map 的 stage (a) 所使用的组合层，提供 \(O(1)\) 的长度与下降查询。其元素以枚举根置换表示，操作中的来源检查只能核对根数；所有操作属于同一 ambient system 的纪律由调用方及 KGB stages 负责。置换的反对称性依靠构造器作为唯一入口保证。^[weyl-layer.md:22-24, weyl-layer.md:64-68]

左右下降分别使用逆向量与正向置换判定：
\[
\ell(s w)<\ell(w)\iff w^{-1}(\alpha_s)<0,
\qquad
\ell(w s)<\ell(w)\iff w(\alpha_s)<0.
\]
这些判定与缓存长度共同支撑词级操作，参见 [[WeylElement 的置换表示与长度下降不变量]]。^[weyl-layer.md:70-74]

`multiply` 的复合方向与 `WeylAction::compose` 一致，并在同一趟计算中依据 \((uv)^{-1}=v^{-1}u^{-1}\) 维护逆置换。乘积长度从 positivity slice 重新计算，不能直接相加操作数长度。左右乘简单反射的接口报告长度变化，`inverse` 则直接返回逆。^[weyl-layer.md:75-79]

`twisted_conjugate` 计算 \(s_{\mathrm{gen}}\,w\,s_{\mathrm{twist}(\mathrm{gen})}\)，是 Tits 扭曲共轭的 Weyl 影子。`twist` 必须是生成元上的对合置换；它与 distinguished involution 的单根作用一致这一条件属于调用方契约。stage (b) 所需的长度变化 \(d\in\{0,\pm2\}\) 由调用点对缓存长度作差获得。参见 [[Weyl 元素的扭曲共轭]]。^[weyl-layer.md:80-84]

## 约化词与上游排序兼容

`reduced_word` 通过逐次剥离最小左下降生成约化词，按从左到右复合解释。`canonical_word` 则复刻上游 transducer 的规范约化词，在 `WeylInterface` 的内部生成元顺序下取字典序最小值：能够作为约化表达式首字母的生成元恰是左下降，因此每步选择最小内部左下降即可逐位确定规范词。实现要求每次剥离使长度恰减一，否则返回 `WeylElementInvariantViolation`。^[weyl-layer.md:86-94]

内部顺序来自 `WeylInterface::new(cartan)` 保存的生成元重编号：Dynkin 分量按分类顺序排列，各分量的 Bourbaki `position` 对 A/E/F/G 型直接使用，对 B/C/D 型反转；`outward()` 将内部编号映射到 datum 生成元编号。这一顺序影响 [[Weyl 元素的规范词]] 和 [[ParabolicPieces 的抛物分解与排序键]]。^[weyl-layer.md:95-99]

`ParabolicPieces::key` 返回按 internal-level 顺序排列的 piece 列表，对应唯一分解 \(w=w_1\cdots w_n\)，其中 \(w_i\) 是右陪集 \(W_{i-1}.w\) 的最小代表元。列表的字典序比较复刻上游 `WeylElt::operator<`，也用于 involution 排序的平局判定，并由 KGB 重编号使用。^[weyl-layer.md:100-105]

## 实现边界与证据范围

矩阵反射构造使用经过检查的 `i128` 运算并收窄到 `i32`，但矩阵复合在 `i64` 累加后以 `sum as i32` 无检查转换；关于条目受 Cartan 界约束的注释论证尚未形式化。枚举热循环使用的 `compose_fast` 完全无检查，违反前置条件存在 panic 风险；简单反射构造中余根的直接下标访问也存在长度不一致时的潜在 panic 面。^[weyl-layer.md:31-35, weyl-layer.md:48-51]

来源列出的测试锚点涵盖全格作用、A2 编织关系、非对称 Cartan 下的配对保持、索引越界、枚举预算、datum 不匹配及算术溢出。不过，该来源仅记录结构性阅读，未执行构建、测试或原版运行，也不提供数学验收、性能或并行结论；其中上游行号转述自源码注释，未独立重读核实。Weyl 层的正确性需结合其独立的 [[HPC 验收证据链]] 判断。^[weyl-layer.md:9-12, weyl-layer.md:59-62, weyl-layer.md:115-121]

## Sources

- [weyl-layer.md](weyl-layer.md) — Weyl 群层：矩阵作用与词级元素的双层结构。
