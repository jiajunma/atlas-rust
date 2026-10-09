---
title: F₂ 商空间的确定性代表元与陪集判定
summary: 按子空间典范基约化得到确定性商代表，并通过两向量之差是否属于子空间判断同陪集。
sources:
  - mod-two.md
kind: concept
createdAt: "2026-10-09T15:02:19.204Z"
updatedAt: "2026-10-09T22:40:34.544Z"
tags:
  - 模二线性代数
  - 商空间
aliases:
  - f₂-商空间的确定性代表元与陪集判定
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: F₂ 商空间的确定性代表元与陪集判定
summary: 按最低主元典范 RREF 基约化向量，得到确定性商类代表元，并通过两向量之差是否属于子空间判断同陪集。
sources:
  - mod-two.md
kind: concept
tags:
  - 有限域
  - 商空间
  - 典范代表元
aliases:
  - f₂-商空间的确定性代表元与陪集判定
---

# F₂ 商空间的确定性代表元与陪集判定

`ModTwoSubspace` 通过典范既约行阶梯形（RREF）基，为 $\mathbb{F}_2$ 商空间提供确定性代表元。`quotient_representative` 返回向量约化后的商类代表元，`same_coset` 判断两个向量之差是否属于子空间；`quotient_dimension` 返回商空间维数 `dimension - rank`。^[mod-two.md:30-40]

## 典范基与约化顺序

子空间存储环境维数 `dimension`、主元行表 `pivots: Vec<Option<ModTwoVector>>` 和秩 `rank`。基行按**最低置位主元**索引：插入时先约化输入，取剩余向量的最低置位作为新主元，再从旧行中消去该主元。这使基始终保持典范 RREF，与插入顺序无关，为商代表元及子商坐标提供确定性基础，详见 [[最低主元索引的典范 RREF 子空间]]。^[mod-two.md:32-40]

`quotient_representative` 直接使用私有方法 `reduce`。约化按主元升序扫描；缺失某个早期主元并不意味着后续主元也缺失，因此不能据此提前停止。由于典范基在每个主元处均已既约，升序扫描恰好清除每个主元系数一次，得到确定性的行阶梯代表元。^[mod-two.md:38-44]

## 成员与陪集判定

`contains(vector)` 以约化后无剩余作为子空间成员判据。`same_coset(left, right)` 先检查维数，再判断两向量之差是否属于子空间。对于共同环境空间中的子空间 $S$，这一判据写作 $x+S=y+S \Longleftrightarrow x-y\in S$；成员判定与陪集判定均以子空间约化为基础。^[mod-two.md:32-44]

## 在子商中的使用

[[F₂ 子商的低主元坐标（ModTwoSubquotient）|ModTwoSubquotient]] 表示共同环境空间中的分子与分母子空间之商。构造时检查维数一致、分母秩不超过分子秩、分母的每个基向量属于分子，以及补基恰好来自分母缺少主元的位置；违反构造不变量时报 `ModTwoSubquotientInvariantViolation`。^[mod-two.md:62-70]

子商的 `canonical_representative(vector)` 首先检查向量是否属于分子；若不属于，返回 `NotInModTwoSubspace`，否则调用分母的 `quotient_representative`。`to_coordinates` 与 `ambient_representative` 按补基最低置位读取或写入坐标位。该类型刻意保持 crate 私有，其低主元打包坐标服务于 Cartan 纤维结构层；公开数学接口由 `CartanFiber` 包装提供，这些内部坐标不是序列化格式。^[mod-two.md:64-74]

确定性代表元本身不能替代映射下降校验。`validate_induced_map_to` 要求分子基向量的像属于目标分子，分母基向量的像属于目标分母；只检查商基代表可能掩盖源分母向量映到非零目标商类的问题。相关约束见 [[Ambient 映射的子商下降验证]]。^[mod-two.md:75-81]

## 测试与证据边界

来源列出的测试锚点包括确定性商代表元与陪集判定、插入后的 RREF 保持、跨多字依赖消元、130 维动态子空间，以及子商低主元坐标。尚未覆盖 `ModTwoSubquotientInvariantViolation` 的五处检查、`NotInModTwoSubspace` 及 `CartanFiberMapDoesNotDescend` 的两个 relation 值；`validate_induced_map_to` 在文件内无调用方。^[mod-two.md:83-93]

这些信息来自源码结构性阅读及测试内容核对。来源材料未执行构建、测试或原版运行，因此不构成数学验收、性能或并行结论。^[mod-two.md:9-13, mod-two.md:103-108]

## Sources

- [mod-two.md](../../sources/mod-two.md) — mod-2 线性代数：位打包向量与子空间。
