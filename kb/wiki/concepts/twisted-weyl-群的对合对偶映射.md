---
title: Twisted Weyl 群的对合对偶映射
summary: dual_involution 在对合矩阵上对应负转置，实现从对偶最长元出发，按原约化字自右向左右乘对偶扭曲生成元，并要求共享外部生成元编号。
sources:
  - block-graph.md
kind: concept
createdAt: "2026-10-09T14:41:05.161Z"
updatedAt: "2026-10-09T14:41:05.161Z"
tags:
  - Weyl群
  - 对偶性
  - 算法
aliases:
  - twisted-weyl-群的对合对偶映射
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Twisted Weyl 群的对合对偶映射

Twisted Weyl 群的对合对偶映射 `dual_involution` 将原侧的 twisted involution $w$ 配到对偶侧的 $\mathrm{dual}_w$。[[完整块的 KGB 纤维积]]使用这一映射，将一个实形式的 KGB 图与其对偶实形式的 KGB 图配对，形成完整块。^[block-graph.md:15-24]

## 数学描述与计算顺序

该映射在对合矩阵层面表现为负转置；在 Weyl 元素层面，映射 $f$ 由以下关系刻画，其中实现从对偶侧最长元出发。^[block-graph.md:20-24]

$$
f(e)=w_0,\qquad f(s.w)=f(w)\,d(s).
$$

实现接口为 `dual_involution(word, dual_system, dual_twist, dual_longest)`。算法从 `dual_longest` 开始，按 $w$ 的约化字**自右向左**遍历，以对偶扭曲字母逐次右乘。参数 `word` 携带两侧共享的外部生成元编号，因此字母编号与遍历、乘法方向都是该实现约定的一部分；相关作用顺序可参见[[Weyl 词对根与权的作用顺序]]。^[block-graph.md:20-24]

## 在完整块构造中的作用

完整块从两个实形式的完整 KGB 集构建，以保留各自的 KGB 坐标编号。构造时，对偶侧的包通过 `HashMap<WeylElement, usize>` 索引；若原侧某个对合的对偶像不在该索引中，该包就贡献零个配对。因此，对共同 Cartan 类的限制通过配对过程隐式实现。源码阅读还指出，索引中的重复键会静默覆盖，当前没有防护。^[block-graph.md:26-33]

`BlockGraph::build` 通过 `dual_inner_class` 获取对偶 twisted Weyl 群的 distinguished twist 与最长元，并用 `weyl_budget` 限制定位最长元时的枚举规模。两侧 KGB 图的半单秩不一致，或对偶表与对偶内类的根系统不同，都会返回 `DatumMismatch`。^[block-graph.md:66-71]

## 与块图对偶的关系

`dual_involution` 用于确定两侧对合的配对关系；[[完整块图的对偶数据变换]]则由 `BlockGraph::dual` 对已经构建的块执行数据变换：反转元素序、交换 $x/y$ 坐标、反射长度，并映射下降状态及 cross/Cayley 链。二者处在完整块处理的不同层次。^[block-graph.md:17-24, block-graph.md:95-104]

## 验证范围与证据边界

源包记录的 `dual_involution` 测试验证了 A1 情形下恒等元与反射互换。相关块图测试还检查了 `dual()` 与原生对偶块经坐标重新编号后的一致性，但所列七个测试全部为秩 1，未覆盖多生成元情形，不能据此扩展为任意秩的验证结论。^[block-graph.md:115-126]

本页依据结构性源码阅读材料。该源包没有执行构建、测试或原版运行；块枚举正确性另属其自身的 [[HPC 验收证据链]]，此处不扩展数学验收、性能或并行结论。^[block-graph.md:9-13, block-graph.md:139-143]

## Sources

- [block-graph.md](block-graph.md)：完整块图：实形式与对偶实形式的纤维积。
