---
title: Weyl 群的矩阵作用与词级元素双层结构
summary: WeylAction 承载全格矩阵作用与 datum，WeylElement 承载根置换，两层通过 action_permutation 与 from_action 桥接。
sources:
  - weyl-layer.md
kind: concept
createdAt: "2026-10-09T15:17:23.076Z"
updatedAt: "2026-10-09T19:38:21.156Z"
tags:
  - Weyl群
  - 架构设计
aliases:
  - weyl-群的矩阵作用与词级元素双层结构
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Weyl 群的矩阵作用与词级元素双层结构

Weyl 群层由矩阵级作用 `WeylAction` 与词级组合元素 `WeylElement` 构成。前者在特征格与余特征格两个全格上表示作用，并携带根数据来源；后者以 ambient `RootSystem` 的枚举根置换表示元素，提供长度、下降、乘法、逆、扭曲共轭及按需约化词。两层通过 `RootSystem::action_permutation` 与 `WeylElement::from_action` 桥接互查。^[weyl-layer.md:17-26]

## 矩阵级作用：WeylAction

`WeylAction` 保存 `datum: Arc<BasedRootDatum>`、`weight_matrix` 和 `coweight_matrix`。单位作用使用 `lattice_rank` 阶单位矩阵；简单反射依据根与余根的对偶配对构造，矩阵公式为 \(M_{ij}=\delta_{ij}-\mathrm{reflected}_i\mathrm{pairing}_j\)。`root_reflection` 支持任意枚举根上的反射，与根的符号无关，用于 Cayley/cross 分解的重放。参见 [[WeylAction 的对偶全格作用]]。^[weyl-layer.md:28-36]

`compose` 表示先作用 `right`、再作用 `self`；`act` 与 `act_on_coweight` 分别提供两个格上的作用。作用构造不要求枚举 Weyl 群，完整枚举则由独立的 `WeylGroup::enumerate_actions(budget)` 接口承担，并接受显式基数预算。^[weyl-layer.md:19-21, weyl-layer.md:37-41]

等值比较采用派生的逐字段比较，包含 datum 的值。`Arc` 的 `PartialEq` 委派给内层值，因此矩阵相同但 datum 值不同的两个作用仍不相等；只有在 datum 值相同时，才与概述中的“相等即矩阵作用相等”一致。参见 [[WeylAction 的等值与 datum 身份语义]]。^[weyl-layer.md:43-47]

枚举实现通过 `CompactWeyl` 的紧致表示生成元素，再以 Rayon 并行物化矩阵。来源描述结果按特征格作用矩阵的字典序排列，并将该顺序归于 `CompactWeyl` 输出与保序收集，但同时指出没有测试断言验证这一排序。参见 [[带基数预算的 Weyl 群作用枚举]]。^[weyl-layer.md:39-41, weyl-layer.md:52-55]

## 词级元素：WeylElement

`WeylElement` 属于 KGB map 的 stage (a)，提供 \(O(1)\) 的长度与下降查询。它的来源检查只能核对根数；所有操作使用同一 ambient system 的要求属于调用方契约，由 KGB stages 负责。置换的反对称性依靠构造器作为唯一入口保证。^[weyl-layer.md:22-24, weyl-layer.md:64-68]

左右下降使用不同方向的置换数据：`has_left_descent` 读取逆向量，以 \(w^{-1}(\alpha_s)<0\) 判定 \(\ell(sw)<\ell(w)\)；`has_right_descent` 读取正向置换，以 \(w(\alpha_s)<0\) 判定 \(\ell(ws)<\ell(w)\)。相关不变量见 [[WeylElement 的置换表示与长度下降不变量]]。^[weyl-layer.md:70-74]

`multiply` 的复合方向与 `WeylAction::compose` 一致，在同一趟计算中依据 \((uv)^{-1}=v^{-1}u^{-1}\) 维护逆置换。乘积长度从 positivity slice 重新计算，不能通过操作数长度相加得到。左右乘简单反射的接口报告长度变化，`inverse` 则直接返回逆。^[weyl-layer.md:75-79]

`twisted_conjugate` 计算 \(s_{\mathrm{gen}}\,w\,s_{\mathrm{twist}(\mathrm{gen})}\)，是 Tits 扭曲共轭的 Weyl 影子。`twist` 必须是生成元上的对合置换；它与 distinguished involution 的单根作用一致这一条件由调用方保证。stage (b) 使用的长度变化 \(d\in\{0,\pm2\}\) 由调用点对缓存长度作差获得。参见 [[Weyl 元素的扭曲共轭]]。^[weyl-layer.md:80-84]

## 约化词与规范排序

`reduced_word` 通过逐次剥离最小左下降生成约化词，按从左到右复合解释。`canonical_word` 则复刻上游 transducer 的规范约化词，在 `WeylInterface` 的内部生成元顺序下取字典序最小值。能够成为约化表达式首字母的生成元恰是左下降，因此逐次选择最小内部左下降即可确定规范词；每步长度必须恰减一，否则返回 `WeylElementInvariantViolation`。参见 [[Weyl 元素的规范词]]。^[weyl-layer.md:86-94]

内部生成元顺序由 `WeylInterface::new(cartan)` 保存：Dynkin 分量按分类顺序排列，各分量的 Bourbaki `position` 对 A/E/F/G 型直接使用，对 B/C/D 型反转。`outward()` 将内部编号映射为 datum 生成元编号。这套重编号的可观察效果体现在规范词选择与 `ParabolicPieces` 的内部序 piece 索引中。^[weyl-layer.md:95-99]

`ParabolicPieces::key` 返回按 internal-level 顺序排列的 piece 列表，对应唯一分解 \(w=w_1\cdots w_n\)，其中 \(w_i\) 是相应右陪集的最小代表元。列表的字典序比较复刻上游 `WeylElt::operator<`，还用于 involution 排序的平局判定，并由 KGB 重编号使用。参见 [[ParabolicPieces 的抛物分解与排序键]]。^[weyl-layer.md:100-105]

## 实现边界与证据范围

反射矩阵构造使用经过检查的 `i128` 运算并收窄至 `i32`；矩阵复合则在 `i64` 累加后以 `sum as i32` 无检查转换，源码注释关于矩阵条目受 Cartan 界约束的论证尚未形式化。枚举热循环使用的 `compose_fast` 完全无检查，违反前置条件存在 panic 风险；简单反射构造对余根的直接下标访问，也存在长度不一致时的潜在 panic 风险。^[weyl-layer.md:31-35, weyl-layer.md:48-51]

来源列出的七个测试锚点涵盖 A1 加中心环面的全格作用、A2 编织关系、非对称 Cartan 下的配对保持、空半单部分的索引越界、A2 枚举预算、同秩异 datum 拒绝，以及极大根坐标的算术溢出错误。它们是结构性阅读记录的测试锚点，并非该来源执行测试所得的验收结果。^[weyl-layer.md:59-62, weyl-layer.md:121-121]

该来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；上游行号转述自源码注释，未独立重读核实。Weyl 层的正确性属于独立的 [[HPC 验收证据链]]，包括 capacity gate 与 Weyl owner 语义线，本页不扩展其证据范围。^[weyl-layer.md:9-12, weyl-layer.md:115-121]

## Sources

- [weyl-layer.md](weyl-layer.md) — Weyl 群层：矩阵作用与词级元素的双层结构。
