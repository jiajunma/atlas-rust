---
title: 根数据的 radical 与 coradical 饱和核基
summary: 实现分别对简单根和简单余根坐标矩阵求饱和整数核，并在无根时保留 ambient 格维度；radical_basis 文档首行与实现的措辞差异尚待核对。
sources:
  - root-datum-dual.md
kind: concept
createdAt: "2026-10-09T15:09:51.174Z"
updatedAt: "2026-10-09T15:09:51.174Z"
tags:
  - 整数格
  - 饱和核
  - 环面
aliases:
  - 根数据的-radical-与-coradical-饱和核基
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 根数据的 radical 与 coradical 饱和核基

`BasedRootDatum` 通过整数矩阵的饱和核构造两类正交格基：`coradical_basis()` 返回与所有简单余根正交的权基，`radical_basis()` 返回与所有简单根正交的余权基。两者分别使用简单余根和简单根的坐标作为矩阵的行。^[root-datum-dual.md:67-79]

## 格与矩阵约定

[[BasedRootDatum：带基根数据与构造不变量|BasedRootDatum]] 区分整个约化环面的秩 `lattice_rank` 与简单根个数 `semisimple_rank`。构造时，每个简单根和简单余根的坐标维数必须等于 `lattice_rank`；因此，求核矩阵的列数由整个格的秩决定，不能仅从简单根个数推断。合法输入包括带中心环面的根数据，以及空 Cartan 矩阵配合正 `lattice_rank` 的纯环面。^[root-datum-dual.md:18-21, root-datum-dual.md:39-51]

以 \(n=\texttt{lattice_rank}\)、\(r=\texttt{semisimple_rank}\) 记维数，令 \(A_{\mathrm{coroot}}\) 的第 \(i\) 行为简单余根 \(\alpha_i^\vee\) 的坐标，\(A_{\mathrm{root}}\) 的第 \(i\) 行为简单根 \(\alpha_i\) 的坐标。两个方法所求的整数核分别为
\[
\ker_{\mathbb Z} A_{\mathrm{coroot}}
=\{x\in\mathbb Z^n:\langle x,\alpha_i^\vee\rangle=0\text{，对所有 }i\},
\]
\[
\ker_{\mathbb Z} A_{\mathrm{root}}
=\{y\in\mathbb Z^n:\langle\alpha_i,y\rangle=0\text{，对所有 }i\}.
\]
前者的基包装为 `Weight`，后者的基包装为 `Coweight`。^[root-datum-dual.md:69-74]

## 饱和核构造与错误边界

`coradical_basis()` 调用 `integer_lattice::saturated_kernel`，使用固定预算 `IntegerLatticeBudget::new(64, 100_000, 100_000, 128)`。返回的核基按列读取，各坐标经 `i32::try_from` 转换后构造 `Weight`；转换失败返回 `ArithmeticOverflow`。`radical_basis()` 采用对称流程，将简单根矩阵的核基包装为 `Coweight`。^[root-datum-dual.md:69-74]

`BasedRootDatum` 类型本身故意不设全局秩上限，但上述求核操作使用固定的整数格预算。来源包没有解释该预算四个参数的语义，也未展开 `saturated_kernel` 的内部算法，因此不能据此进一步断言其消元方式或具体资源阈值。^[root-datum-dual.md:18-21, root-datum-dual.md:69-74, root-datum-dual.md:209-215]

## 无根情形与维数保留

当简单根或简单余根列表为空时，`annihilator_matrix` 显式构造 `IntegerMatrix::zero(0, lattice_rank, budget)`。这是一个保留 ambient 格维数的零行矩阵：空方程组的核应为整个格，若仅从空行列表推断矩阵列数，就会丢失这一维数。^[root-datum-dual.md:75-79]

来源记录的两个无根回归测试覆盖秩 \(0,1,2,4\)，固定 `coradical_basis()` 与 `radical_basis()` 都返回 ambient 格的单位坐标基。这些测试是纯环面情形下维数保留修复的回归守卫，来源链指向 `docs/slices/torus_radical_2026-09-29.md`。^[root-datum-dual.md:75-79]

## 文档差异与证据范围

`radical_basis()` 的文档首行写作 “`lattice::perp` of the coroots”，但紧随其后的定义与实现均使用简单根矩阵的核，即与所有简单根正交的余权。来源将该措辞列为疑似笔误，尚未核对上游 `rootdata.cpp:859` 的字节；解释该方法时应保留这一差异。^[root-datum-dual.md:193-197]

本页依据的是 `root_datum.rs` 与 `dual.rs` 的结构性阅读记录，不构成根数据层的数学验收。来源未定义底层整数格求核实现，也未在此次知识维护中执行 Atlas、Cargo、测试或 benchmark；上述测试覆盖是来源对已有测试的记录。^[root-datum-dual.md:9-14, root-datum-dual.md:209-216]

## Sources

- [root-datum-dual.md](root-datum-dual.md) — BasedRootDatum 与对偶内类构造（root_datum.rs / dual.rs）。
