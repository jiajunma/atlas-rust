---
title: 根数据的 radical 与 coradical 饱和核基
summary: 分别对简单根与简单余根坐标矩阵求饱和整数核，无根时显式保留环境格维度；radical_basis 的文档首行与实现存在待核对措辞差异。
sources:
  - root-datum-dual.md
kind: concept
createdAt: "2026-10-09T15:09:51.174Z"
updatedAt: "2026-10-09T21:08:45.185Z"
tags:
  - 整数格
  - 饱和核
  - 中心环面
aliases:
  - 根数据的-radical-与-coradical-饱和核基
  - 根R与C饱
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 根数据的 radical 与 coradical 饱和核基
summary: BasedRootDatum 分别对简单根和简单余根坐标矩阵求饱和整数核，返回余权基和权基；无根时显式保留环境格维数，坐标转换受溢出检查保护。
sources:
  - root-datum-dual.md
kind: concept
tags:
  - 整数格
  - 饱和核
  - 环面
aliases:
  - 根数据的-radical-与-coradical-饱和核基
---

# 根数据的 radical 与 coradical 饱和核基

`BasedRootDatum` 通过饱和整数核构造两类正交格基：`coradical_basis()` 返回与所有简单余根正交的权基，`radical_basis()` 返回与所有简单根正交的余权基。两个方法分别以简单余根和简单根的坐标作为矩阵行，再调用 `integer_lattice::saturated_kernel`。^[root-datum-dual.md:67-79]

## 格与矩阵约定

[[BasedRootDatum：带基根数据与构造不变量|BasedRootDatum]] 区分整个约化环面的秩 `lattice_rank` 与简单根个数 `semisimple_rank`。构造门控要求每个简单根和简单余根的坐标维数等于 `lattice_rank`；合法输入包括带中心环面的根数据，以及空 Cartan 矩阵配合正 `lattice_rank` 的纯环面。^[root-datum-dual.md:18-21, root-datum-dual.md:39-51]

令 \(n=\texttt{lattice_rank}\)，以简单余根坐标为行的矩阵记为 \(A_{\mathrm{coroot}}\)，以简单根坐标为行的矩阵记为 \(A_{\mathrm{root}}\)。两类整数核为
\[
\ker_{\mathbb Z}A_{\mathrm{coroot}}
=\{x\in\mathbb Z^n:\langle x,\alpha_i^\vee\rangle=0\text{，对所有 }i\},
\]
\[
\ker_{\mathbb Z}A_{\mathrm{root}}
=\{y\in\mathbb Z^n:\langle\alpha_i,y\rangle=0\text{，对所有 }i\}.
\]
前者的核基包装为 `Weight`，后者包装为 `Coweight`，保留权与余权的类型区别。^[root-datum-dual.md:69-74]

## 构造流程与错误边界

`coradical_basis()` 使用固定预算 `IntegerLatticeBudget::new(64, 100_000, 100_000, 128)` 求饱和核，按列读取核基，各坐标经 `i32::try_from` 转换后构造 `Weight`；转换失败返回 `ArithmeticOverflow`。`radical_basis()` 采用对称流程，对简单根矩阵求核并构造 `Coweight`。^[root-datum-dual.md:69-74]

根数据类型本身故意不设全局秩上限，但求核操作仍使用上述固定的[[整数格计算预算（IntegerLatticeBudget）]]。来源未解释四个预算参数的语义，也未展开 `saturated_kernel` 的内部算法，因此不能据此断言具体消元方式或各项资源阈值。^[root-datum-dual.md:18-21, root-datum-dual.md:69-74, root-datum-dual.md:209-215]

## 无根情形的维数保留

当矩阵行列表为空时，辅助函数 `annihilator_matrix` 显式构造 `IntegerMatrix::zero(0, lattice_rank, budget)`。空方程组的核是整个环境格；若仅从空行列表推断列数，就会丢失环境格的维数，因此零行矩阵仍须保留 `lattice_rank` 列。^[root-datum-dual.md:75-79]

来源记录的两个无根回归测试覆盖秩 \(0,1,2,4\)，固定 `coradical_basis()` 与 `radical_basis()` 均返回环境格的单位坐标基。这些测试是纯环面修复的回归守卫，来源链指向 `docs/slices/torus_radical_2026-09-29.md`。^[root-datum-dual.md:75-79]

## 文档差异与证据范围

`radical_basis()` 的文档首行写作 “`lattice::perp` of the coroots”，但随后的定义与实现均取简单根矩阵的核，即与所有简单根正交的余权。来源将此列为疑似措辞笔误，尚未核对上游 `rootdata.cpp:859` 的字节；方法含义应依据所记录的定义与实现，同时保留这一待核对差异。^[root-datum-dual.md:193-197]

本页依据 `root_datum.rs` 与 `dual.rs` 的结构性阅读记录，不构成根数据层的数学验收。来源中的上游引用转录自代码文档注释，未核对上游字节；本次知识维护未执行 Atlas、Cargo、测试或 benchmark，因此上述测试内容是对已有回归测试的记录。^[root-datum-dual.md:9-14, root-datum-dual.md:204-216]

## Sources

- [root-datum-dual.md](../../sources/root-datum-dual.md) — BasedRootDatum 与对偶内类构造（root_datum.rs / dual.rs）。
