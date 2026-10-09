---
title: F₂ 子商的低主元坐标（ModTwoSubquotient）
summary: 在共同环境空间中校验分母包含于分子，以分母缺失主元对应的补基建立典范商坐标，并通过 crate 私有实现服务 CartanFiber。
sources:
  - mod-two.md
kind: concept
createdAt: "2026-10-09T15:02:43.290Z"
updatedAt: "2026-10-09T15:02:43.290Z"
tags:
  - 子商空间
  - 坐标表示
  - Rust
aliases:
  - f₂-子商的低主元坐标modtwosubquotient
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# F₂ 子商的低主元坐标（ModTwoSubquotient）

`ModTwoSubquotient` 表示共同环境空间（ambient space）中两个 $\mathbb{F}_2$ 子空间的典范商，以 `numerator`、`denominator` 和 `complement_basis` 保存分子、分母及补基。它为 Cartan 纤维结构层提供低主元打包坐标，刻意保持 crate 私有；公开的数学接口是 `CartanFiber` 包装，这套内部坐标不是序列化格式。^[mod-two.md:62-70]

## 典范基与低主元坐标

底层子空间采用[[最低主元索引的典范 RREF 子空间]]：每个基向量以最低置位作为主元，插入时先约化输入，再从旧行中消去新主元，使基始终保持与插入顺序无关的典范行最简阶梯形（RREF）。这一不变量为子商提供确定性坐标。^[mod-two.md:30-40]

子商的补基来自分母缺少主元的位置。`to_coordinates` 与 `ambient_representative` 按补基的最低置位读写坐标位，在内部打包坐标与环境空间代表元之间转换。^[mod-two.md:64-74]

## 构造不变量

构造时要求分子与分母的环境维数一致、分母的秩不超过分子的秩，并逐一检查分母的基向量属于分子；还要求补基恰好来自分母缺少主元的位置。违反这些条件会返回 `ModTwoSubquotientInvariantViolation`。^[mod-two.md:64-70]

`canonical_representative(vector)` 首先检查向量是否属于分子；若不属于，返回 `NotInModTwoSubspace`。否则调用分母的 `quotient_representative`，得到该商类的确定性代表元。^[mod-two.md:72-73]

这一代表元选择依赖分母子空间的约化过程：按主元升序扫描并消去各主元系数。底层还通过判断两向量之差是否属于子空间来判定陪集相同，参见[[F₂ 商空间的确定性代表元与陪集判定]]。^[mod-two.md:37-44]

## 环境映射下降为子商映射

`validate_induced_map_to(target, map)` 检查环境映射能否诱导子商之间的映射：分子的每个基向量必须映入 `target.numerator`，分母的每个基向量必须映入 `target.denominator`。失败时返回 `CartanFiberMapDoesNotDescend`，并以 `relation: "numerator"` 或 `"denominator"` 标明违反的条件。^[mod-two.md:75-79]

仅检查商基代表元不足以保证映射良定义：若源分母向量在目标商中具有非零类，正规形选择仍可能使映射“看似”成立。完整检查必须包含分母的像，详见[[Ambient 映射的子商下降验证]]。^[mod-two.md:75-79]

`ModTwoAmbientMap` trait 允许领域层直接实现环境映射，避免为此额外分配稠密矩阵；`ModTwoLinearMap` 则是仅在测试配置下提供的稠密辅助实现。^[mod-two.md:80-81]

## 测试与证据边界

源文件的测试锚点包含子商低主元坐标，以及相关的确定性商代表、陪集判定和插入后 RREF 保持。但五处 `ModTwoSubquotientInvariantViolation`、`NotInModTwoSubspace` 和 `CartanFiberMapDoesNotDescend` 的两个 `relation` 值未被覆盖；`validate_induced_map_to` 在该文件内没有调用方。^[mod-two.md:83-93]

这些记录来自源码结构性阅读；来源包没有执行构建、测试或原版运行，也不提供数学验收、性能或并行结论。^[mod-two.md:9-13, mod-two.md:103-108]

## Sources

- [mod-two.md](mod-two.md)：mod-2 线性代数：位打包向量与子空间。
