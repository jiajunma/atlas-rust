---
title: 根对合数据（RootInvolutionData）
summary: 在格对合之上验证根置换及逐根余根运输；后者排除仅保持配对却错误移动余根中心环面坐标的作用。
sources:
  - involution-types.md
kind: concept
createdAt: "2026-10-09T14:53:23.183Z"
updatedAt: "2026-10-09T14:53:23.183Z"
tags:
  - 根数据
  - 根系自同构
  - Rust设计
aliases:
  - 根对合数据rootinvolutiondata
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 根对合数据（RootInvolutionData）

`RootInvolutionData` 在 `LatticeInvolution` 的配对保持对合之上，验证作用是否置换枚举根系，并将每个存储余根运输到像根对应的余根。它保存根的像、虚实复分类，以及虚根和实根子系统在继承正系中的单根。^[involution-types.md:13-15, involution-types.md:34-47, involution-types.md:82-97]

## 数学约束与类型边界

`LatticeInvolution` 分别存储并共同验证 character 与 cocharacter 格上的作用，保证两者都是对合且保持配对，但不保证保持有限根系。`RootInvolutionData` 补充根置换与余根运输验证：仅有配对保持仍可能允许“固定所有根，却移动余根中心环面坐标”的作用通过。^[involution-types.md:63-71, involution-types.md:82-84]

根置换是否满足二阶关系没有独立检查；这一性质依赖底层 `LatticeInvolution` 的代数对合验证。相关构造纪律见 [[对合类型的分层构造验证与错误优先级]]。^[involution-types.md:144-145, involution-types.md:67-71]

## 构造与错误优先级

构造入口为 `RootInvolutionData::new(&RootSystem, LatticeInvolution) -> Result<Self, _>`。字段全部私有，实例必须经过构造验证。^[involution-types.md:34-39, involution-types.md:142-143]

构造先检查 datum 一致性，再检查秩，分别可能返回 `DatumMismatch` 与 `RankMismatch`。随后执行 `validate_simple_root_images`，使单根级错误优先于逐根主循环中的泛型错误；最后遍历全部根，验证根像与余根运输。^[involution-types.md:86-91]

| 验证阶段 | 失败情形 | 错误 |
| --- | --- | --- |
| 单根验证 | 单根的像不是根 | `SimpleRootImageNotRoot { simple_root }` |
| 单根验证 | 单余根的运输与像根余根不符 | `SimpleCorootImageMismatch { simple_root, image_root }` |
| 逐根主循环 | 根的像不是根 | `InvalidRootAutomorphism` |
| 逐根主循环 | 余根运输不符 | `InvalidRootDatumAutomorphism` |

这些错误及其先后顺序构成构造器的诊断约定。^[involution-types.md:86-91]

## 根分类与子系统单根

对于根 \(\alpha\)，分类按固定顺序进行：若 \(\theta(\alpha)=\alpha\)，则为 `Imaginary`（虚根）；否则若 \(\theta(\alpha)=-\alpha\)，则为 `Real`（实根）；其余为 `Complex`（复根）。负根坐标通过逐坐标 `checked_neg` 计算。参见 [[对合下的虚根、实根与复根分类]]。^[involution-types.md:91-92]

`subsystem_simple_roots` 分别为虚根与实根子系统计算单根。它选取该类中简单坐标全非负的根，继承原根系的正系，并按 `RootId` 升序处理候选；依据候选与集合内另一成员之差是否为某正坐标向量排除可分解候选，保留子系统单根。输出同样按 `RootId` 升序排列。^[involution-types.md:94-97]

## 查询接口

`involution()` 返回底层格对合，`image_permutation()` 返回根像表；`image(root)` 与 `kind(root)` 分别查询根像和分类，对越界 `RootId` 返回 `None`，不会 panic。`roots_of_kind(kind)` 惰性地按 `RootId` 升序遍历指定类别，`imaginary_simple_roots()` 与 `real_simple_roots()` 返回对应子系统单根切片。^[involution-types.md:40-46, involution-types.md:99-100]

## 与扭曲对合的关系

[[扭曲对合（TwistedInvolution）]] 构造 \(w\theta\) 时，先在两格上合成矩阵，再通过 `LatticeInvolution::new` 重新验证对合与配对保持，最后通过 `RootInvolutionData::new` 验证根置换和余根运输。因此，根对合数据也是 Weyl 平移后作用的根论验证层；Cayley/cross 分解与规范化由其他层负责。^[involution-types.md:104-112]

## 测试锚点与证据边界

A2 上的负反对角对合得到 2 个实根、4 个复根、0 个虚根，实根子系统单根为 `[id_of([1,1])]`。恒等对合将全部根分类为虚根，虚根子系统单根按枚举顺序为 `[id_of([0,1]), id_of([1,0])]`。^[involution-types.md:123-125]

测试还包含配对保持但不置换根的作用：`W=[[-1,0],[1,1]]`、`C=[[-1,1],[0,1]]` 能通过 `LatticeInvolution::new`，却在根对合构造中返回 `SimpleRootImageNotRoot { simple_root: 0 }`；余根运输错误也分别以正、负单根案例锚定 `SimpleCorootImageMismatch`。^[involution-types.md:126-129]

本页依据结构性源码阅读，不构成数学正确性验收。来源未核对上游字节，且明确指出 `image()`、`kind()` 的越界 `None` 行为尚无测试覆盖；该次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[involution-types.md:10-15, involution-types.md:138-141, involution-types.md:155-156]

## Sources

- [involution-types.md](involution-types.md) — 对合类型三件套：LatticeInvolution / RootInvolutionData / TwistedInvolution
