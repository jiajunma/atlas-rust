---
title: 根数据的 radical 与 coradical 饱和核基
summary: 分别由简单根与简单余根坐标矩阵求饱和整数核，无根时保留环境格维数；radical_basis 文档首行与实现的措辞差异仍待核对。
sources:
  - root-datum-dual.md
kind: concept
createdAt: "2026-10-09T15:09:51.174Z"
updatedAt: "2026-10-10T00:50:11.608Z"
tags:
  - 根数据
  - 整数格
  - 饱和核
aliases:
  - 根数据的-radical-与-coradical-饱和核基
  - 根R与C饱
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 根数据的 radical 与 coradical 饱和核基
summary: BasedRootDatum 以简单根与简单余根坐标矩阵求饱和整数核，分别返回余权基与权基；无根时显式保留环境格维数。
sources:
  - root-datum-dual.md
kind: concept
tags:
  - 整数格
  - 饱和核
  - 根数据
aliases:
  - 根数据的-radical-与-coradical-饱和核基
---

# 根数据的 radical 与 coradical 饱和核基

`BasedRootDatum` 通过 `integer_lattice::saturated_kernel` 构造两类正交格基：`coradical_basis()` 返回与所有简单余根正交的权基，`radical_basis()` 返回与所有简单根正交的余权基。两者分别以简单余根和简单根的坐标作为矩阵行。^[root-datum-dual.md:67-79]

## 格与矩阵约定

[[BasedRootDatum：带基根数据与构造不变量|BasedRootDatum]] 区分环境格秩 `lattice_rank` 与简单根个数 `semisimple_rank`。构造时要求每个简单根、简单余根的坐标维数等于 `lattice_rank`；合法输入包括带中心环面的根数据，以及空 Cartan 矩阵配合正 `lattice_rank` 的纯环面。^[root-datum-dual.md:18-21, root-datum-dual.md:39-51]

令 $n=\texttt{lattice_rank}$，简单余根坐标组成行矩阵 $A_{\mathrm{coroot}}$，简单根坐标组成行矩阵 $A_{\mathrm{root}}$。两个接口求取的整数核分别满足以下正交条件。^[root-datum-dual.md:69-74]

$$
\ker_{\mathbb Z} A_{\mathrm{coroot}}
=\{x\in\mathbb Z^n:\langle x,\alpha_i^\vee\rangle=0\ \text{对所有 }i\},
$$

$$
\ker_{\mathbb Z} A_{\mathrm{root}}
=\{y\in\mathbb Z^n:\langle\alpha_i,y\rangle=0\ \text{对所有 }i\}.
$$

前者的核基包装为 `Weight`，后者包装为 `Coweight`，保留权与余权的类型区别。^[root-datum-dual.md:69-74]

## 构造流程与错误边界

`coradical_basis()` 使用固定预算 `IntegerLatticeBudget::new(64, 100_000, 100_000, 128)` 求饱和核，逐列读取核基，将坐标经 `i32::try_from` 转换后构造 `Weight`；转换失败返回 `ArithmeticOverflow`。`radical_basis()` 采用对称流程，以简单根坐标为行求核，并构造 `Coweight`。^[root-datum-dual.md:69-74]

根数据类型本身故意不设全局秩上限，但求核操作使用上述固定预算。来源未说明四个预算参数的语义，也未展开 `saturated_kernel` 的内部算法，因此不能据此确定具体消元方式或各参数对应的资源限制。^[root-datum-dual.md:18-21, root-datum-dual.md:69-74, root-datum-dual.md:209-215]

## 无根情形的维数保留

当矩阵行列表为空时，辅助函数 `annihilator_matrix` 显式构造 `IntegerMatrix::zero(0, lattice_rank, budget)`。空方程组的核是整个环境格；若仅从空行推断列数，就会丢失环境格的秩，因此零行矩阵仍须保留 `lattice_rank` 列。^[root-datum-dual.md:75-79]

来源记录的两个无根回归测试覆盖秩 $0,1,2,4$，固定 `coradical_basis()` 与 `radical_basis()` 均返回环境格的单位坐标基。这些测试是纯环面修复的回归守卫，来源链指向 `docs/slices/torus_radical_2026-09-29.md`。^[root-datum-dual.md:75-79]

## 文档差异与证据范围

`radical_basis()` 的文档首行写作 “`lattice::perp` of the coroots”，但随后的定义与实现均取简单根矩阵的核，即与所有简单根正交的余权。来源将此列为疑似措辞笔误，尚未核对上游 `rootdata.cpp:859` 的字节；这一差异仍属待核对事项。^[root-datum-dual.md:193-197]

在本包覆盖的 `root_datum.rs` 与 `dual.rs` 两个文件中，`coradical_basis()` 和 `radical_basis()` 均无调用点；这一观察不涉及其他文件的调用情况。^[root-datum-dual.md:184-191]

本页依据结构性阅读记录，不构成根数据层的数学验收。来源中的上游引用仅转录自代码文档注释，未核对上游字节；该次知识维护未执行 Atlas、Cargo、测试或 benchmark，因此上述测试覆盖描述是对已有测试的记录，而非本次运行结果。^[root-datum-dual.md:9-14, root-datum-dual.md:204-216]

## Sources

- [root-datum-dual.md](../../sources/root-datum-dual.md) — BasedRootDatum 与对偶内类构造（root_datum.rs / dual.rs）。
