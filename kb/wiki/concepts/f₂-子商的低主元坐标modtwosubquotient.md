---
title: F₂ 子商的低主元坐标（ModTwoSubquotient）
summary: 验证共同环境空间中的分母包含于分子，以分母缺失主元对应的补基建立典范商坐标，并将表示封装为 crate 私有实现。
sources:
  - mod-two.md
kind: concept
createdAt: "2026-10-09T15:02:43.290Z"
updatedAt: "2026-10-10T00:44:29.371Z"
tags:
  - 模二线性代数
  - 子商
  - Rust
aliases:
  - f₂-子商的低主元坐标modtwosubquotient
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: F₂ 子商的低主元坐标（ModTwoSubquotient）
summary: 在共同环境空间中校验分母包含于分子，以低主元补基建立典范商坐标，并验证环境映射能否下降到子商。
sources:
  - mod-two.md
kind: concept
tags:
  - 模二线性代数
  - 子商空间
  - Rust
aliases:
  - f₂-子商的低主元坐标modtwosubquotient
---

# F₂ 子商的低主元坐标（ModTwoSubquotient）

`ModTwoSubquotient` 表示共同环境空间中两个 $\mathbb{F}_2$ 子空间的典范商，由 `numerator`、`denominator` 和 `complement_basis` 保存分子、分母及补基。它为 Cartan 纤维结构层提供低主元打包坐标，刻意保持 crate 私有；公开数学接口是 `CartanFiber` 包装，内部坐标不是序列化格式。^[mod-two.md:62-70]

## 典范基与坐标约定

底层采用[[最低主元索引的典范 RREF 子空间]]：基向量以最低置位作为主元，插入时先约化输入，再从旧行中消去新主元，使基保持与插入顺序无关的典范行最简阶梯形（RREF）。这一不变量为子商层提供确定性坐标。^[mod-two.md:30-40]

子商的补基恰好来自分母缺少主元的位置。`to_coordinates` 与 `ambient_representative` 按补基的最低置位读写坐标位，完成内部坐标与环境空间代表元之间的转换。^[mod-two.md:64-74]

## 构造校验与代表元

构造时检查分子与分母的环境维数一致、分母的秩不超过分子的秩，以及分母的每个基向量均属于分子；补基还须满足上述主元位置约定。违反这些条件时返回 `ModTwoSubquotientInvariantViolation`。^[mod-two.md:64-70]

`canonical_representative(vector)` 首先检查向量是否属于分子；若不属于，返回 `NotInModTwoSubspace`。否则调用分母的 `quotient_representative`，得到该商类的确定性代表元。^[mod-two.md:72-73]

代表元计算依赖底层 `reduce`：按主元升序扫描，逐一清除主元系数；缺失早期主元不意味着后期主元也缺失。底层 `same_coset(left, right)` 则先检查维数，再判断两向量之差是否属于子空间。^[mod-two.md:37-44]

## 环境映射的子商下降验证

`validate_induced_map_to(target, map)` 检查环境映射能否诱导子商之间的映射：分子的每个基向量的像必须属于 `target.numerator`，分母的每个基向量的像必须属于 `target.denominator`。失败时返回 `CartanFiberMapDoesNotDescend`，通过 `relation: "numerator"` 或 `"denominator"` 标明违反的条件。^[mod-two.md:75-79]

仅检查商基代表元不足以保证映射良定义：若源分母向量在目标商中的类非零，正规形选择仍可能让映射“看似”成立。因此，下降验证必须检查分母的像。^[mod-two.md:75-79]

`ModTwoAmbientMap` trait 允许领域层直接实现环境映射，避免多余的稠密矩阵分配；`ModTwoLinearMap` 是仅在 `#[cfg(test)]` 下提供的稠密测试辅助实现。^[mod-two.md:80-81]

## 测试与证据边界

源文件中的测试锚点包含子商低主元坐标，以及相关的确定性商代表、陪集判定和插入后 RREF 保持。五处 `ModTwoSubquotientInvariantViolation`、`NotInModTwoSubspace` 和 `CartanFiberMapDoesNotDescend` 的两个 `relation` 值未被覆盖；`validate_induced_map_to` 在该文件内没有调用方。^[mod-two.md:83-93]

这些记录来自源码结构性阅读。来源包未执行构建、测试或原版运行，不含数学验收、性能或并行结论；测试锚点描述不代表本次阅读已验证测试通过。^[mod-two.md:9-13, mod-two.md:103-108]

## Sources

- [mod-two.md](../../sources/mod-two.md)：mod-2 线性代数：位打包向量与子空间。
