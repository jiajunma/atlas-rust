---
title: 根对合数据（RootInvolutionData）
summary: 在格对合上验证根置换与对应余根运输，分类虚根、实根和复根，并提取继承正系中的子系统单根；置换的对合性依赖底层代数门控。
sources:
  - involution-types.md
kind: concept
createdAt: "2026-10-09T14:53:23.183Z"
updatedAt: "2026-10-10T02:22:24.203Z"
tags:
  - 根系
  - 根数据自同构
  - 根分类
aliases:
  - 根对合数据rootinvolutiondata
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 根对合数据（RootInvolutionData）

`RootInvolutionData` 在[[格对合（LatticeInvolution）]]之上，验证作用是否置换枚举根系，并将每个存储余根运输到像根对应的余根。它保存底层格对合、各根的像与分类，以及虚根和实根子系统在继承正系中的单根。^[involution-types.md:34-47, involution-types.md:82-97]

## 数学约束与类型边界

底层 `LatticeInvolution` 分别存储权格与余权格上的作用，共同验证两者均为对合且保持配对，但不保证保持有限根系。`RootInvolutionData` 补充根置换与余根运输验证：仅保持配对仍可能允许“固定所有根，却移动余根中心环面坐标”的作用通过，因此余根运输是独立且必要的检查。^[involution-types.md:63-71, involution-types.md:82-84]

构造器不独立检查根置换的平方是否为恒等；这一性质依赖底层 `LatticeInvolution` 的代数对合验证。相关构造纪律见[[对合类型的分层构造验证与错误优先级]]。^[involution-types.md:144-145]

## 构造与错误优先级

构造入口为 `RootInvolutionData::new(&RootSystem, LatticeInvolution) -> Result<Self, _>`。字段全部私有，实例必须经过构造验证。^[involution-types.md:34-39, involution-types.md:142-143]

构造首先检查 datum 一致性，再检查秩，分别可能返回 `DatumMismatch` 与 `RankMismatch`。随后执行 `validate_simple_root_images`，最后进入逐根主循环，因此单根级错误优先于主循环中的通用错误。^[involution-types.md:86-91]

单根验证中，单根像不是根时返回 `SimpleRootImageNotRoot { simple_root }`；单余根运输不符时返回 `SimpleCorootImageMismatch { simple_root, image_root }`。逐根主循环中，对应失败分别返回 `InvalidRootAutomorphism` 与 `InvalidRootDatumAutomorphism`。^[involution-types.md:87-91]

## 根分类与子系统单根

对于根 $\alpha$，分类按固定顺序进行：若 $\theta(\alpha)=\alpha$，则为 `Imaginary`（虚根）；否则若 $\theta(\alpha)=-\alpha$，则为 `Real`（实根）；其余为 `Complex`（复根）。负根通过逐坐标 `checked_neg` 计算。^[involution-types.md:91-92]

`subsystem_simple_roots` 分别计算虚根与实根子系统的单根。它选取该类中简单坐标全非负的根，继承原根系的正系，并按 `RootId` 升序处理候选；候选若可表示为集合内另一成员与某正坐标向量的差，则跳过，否则入选。输出仍按 `RootId` 升序排列，具体顺序有测试锚定。^[involution-types.md:94-97]

## 查询接口

`involution()` 返回底层格对合，`image_permutation()` 返回根像表。`image(root)` 与 `kind(root)` 分别查询根像和分类，对越界 `RootId` 返回 `None`，不会 panic。^[involution-types.md:40-43, involution-types.md:99-100]

`roots_of_kind(kind)` 惰性地按 `RootId` 升序遍历指定类别；`imaginary_simple_roots()` 与 `real_simple_roots()` 返回对应子系统的单根切片。^[involution-types.md:44-46, involution-types.md:99-100]

## 与扭曲对合的关系

[[扭曲对合（TwistedInvolution）]]构造 $w\theta$ 时，先在两个格上以 $w$ 在左、$\theta$ 在右合成矩阵，再通过 `LatticeInvolution::new` 重新验证对合与配对保持，最后通过 `RootInvolutionData::new` 验证根置换和余根运输。^[involution-types.md:108-112]

这一构造层只建立 $(w\theta)^2=1$ 的根论条件。[[扭曲对合的 Cayley/Cross 分解|Cayley/cross 分解]]由 `CayleyCrossDecomposition` 负责，规范化由 `InnerClass::canonicalize` 负责，均不属于本层。^[involution-types.md:104-106]

## 测试锚点与证据边界

A2 上的负反对角对合得到 2 个实根、4 个复根、0 个虚根，实根子系统单根为 `[id_of([1,1])]`。恒等对合将全部根分类为虚根，虚根子系统单根按枚举顺序为 `[id_of([0,1]), id_of([1,0])]`。^[involution-types.md:123-125]

配对保持但不置换根的测试使用矩阵 $W=\begin{pmatrix}-1&0\\1&1\end{pmatrix}$、$C=\begin{pmatrix}-1&1\\0&1\end{pmatrix}$。它们能通过 `LatticeInvolution::new`，却在根对合构造中返回 `SimpleRootImageNotRoot { simple_root: 0 }`；余根运输错误另有正、负单根案例，均锚定 `SimpleCorootImageMismatch`。^[involution-types.md:126-129]

`image()`、`kind()` 的越界 `None` 行为尚无测试覆盖。私有辅助函数的潜在 panic 路径仅属阅读推断，其裸下标索引依赖调用点的前置方阵及秩检查。^[involution-types.md:138-143]

来源属于结构性源码阅读，不构成数学正确性验收。上游引用转录自代码文档注释，未核对上游字节；该次知识维护未执行 Atlas、Cargo、测试或 benchmark，因此上述测试锚点不代表本次执行结果。^[involution-types.md:9-15, involution-types.md:149-156]

## Sources

- [involution-types.md](../../sources/involution-types.md) — 对合类型三件套：LatticeInvolution / RootInvolutionData / TwistedInvolution。
