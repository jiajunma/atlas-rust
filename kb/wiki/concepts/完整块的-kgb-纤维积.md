---
title: 完整块的 KGB 纤维积
summary: 完整块将实形式与对偶实形式的完整 KGB 集按对偶 twisted involution 配对，并保留两侧原始 KGB 坐标编号；缺失的对偶包不贡献元素。
sources:
  - block-graph.md
kind: concept
createdAt: "2026-10-09T14:41:08.924Z"
updatedAt: "2026-10-09T14:41:08.924Z"
tags:
  - 表示论
  - KGB图
  - 纤维积
aliases:
  - 完整块的-kgb-纤维积
  - 完K纤
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 完整块的 KGB 纤维积

完整块是一个实形式的 [[KGB 图与弱实形式|KGB 图]]与其对偶实形式的 KGB 图，沿 twisted involution 及其对偶配对形成的纤维积。块元素由坐标对 \((x,y)\) 表示，其中两侧 KGB 元素的对合必须满足对偶配对关系。^[block-graph.md:17-24]

## 对合的对偶配对

配对使用 twisted Weyl 群的 `dual_involution` 映射。在对合矩阵上，该映射是负转置；在 Weyl 元素上，它由 \(f(e)=w_0\) 与 \(f(s.w)=f(w)\,d(s)\) 刻画。实现从对偶最长元出发，按原元素约化字自右向左，以对偶扭曲字母右乘；约化字使用两侧共享的外部生成元编号。相关概念见 [[Twisted Weyl 群的对合对偶映射]]。^[block-graph.md:20-24]

## 完整 KGB 编号与公共 Cartan 限制

解释器使用两个实形式各自的**完整 KGB 集**构建块，以保留原有 KGB 坐标编号。显式采用 `common_Cartans` 受限重载会改变编号，因此公共 Cartan 的限制通过配对隐式实现：某个原侧对合的对偶若不在对偶包索引中，该原侧包就不贡献任何坐标对。^[block-graph.md:26-33]

对偶包使用 `HashMap<WeylElement, usize>` 索引。源码阅读发现，重复键会静默覆盖，当前没有相应防护；因此，这一索引机制本身不提供重复键检查。^[block-graph.md:30-33]

## 构造与元素编号

`BlockGraph::build` 接收原侧与对偶侧的图和对合表，以及 `dual_inner_class` 和 `weyl_budget`。对偶内类提供 distinguished twist 与最长元所需的对偶 twisted Weyl 群；预算限制定位最长元时的枚举规模。两图半单秩不一致，或对偶表与对偶内类的根系统不同，均返回 `DatumMismatch`。^[block-graph.md:66-70]

元素按原 KGB 的对合包顺序编号，每个配对包内部按 **\(x\) 外层、\(y\) 内层**枚举笛卡尔积。`xs` 和 `ys` 保存各元素在两侧完整 KGB 集中的编号，长度取原侧的 `kgb.length(x)`，供 KL 表排序使用；构造时若 `xs.len() != size`，则报告 `"block size"` 错误。^[block-graph.md:56-61, block-graph.md:70-71]

## 坐标定位

`xs` 弱增是块坐标定位的重要不变量。`first_z_of_x[x]` 记录首个满足 \(x(z)\geq x\) 的块元素，表长为 `xrange + 1`，并以块大小作为哨兵。`element(x,y)` 先确定对应的 \(x\) 区间，再按连续的 \(y\) 偏移定位，并校验所得坐标；越界或纤维不符时返回 `StructureError`。具体布局见 [[BlockGraph 存储布局与坐标定位不变量]]。^[block-graph.md:62-64, block-graph.md:88-91]

## 对偶块

`BlockGraph::dual` 通过纯数据变换交换两侧实形式的角色：反转元素序 \(z'=\mathrm{size}-1-z\)，交换 \(x/y\) 坐标，并相应变换长度、下降状态及 cross/Cayley 链接，重新计算坐标定位表。详见 [[完整块图的对偶数据变换]]。^[block-graph.md:95-104]

## 示例与证据边界

秩一测试中的 \(\operatorname{block}(\mathrm{SL}(2,\mathbb R),\mathrm{PGL}(2,\mathbb R))\) 有三个元素，坐标依次为 \((0,1),(1,1),(2,0)\)，与冻结 fixture capture 3501519 对齐。测试还检查了对偶块经 `element` 换编号后的一致性，以及覆盖全 KGB 范围时 `dual().dual()` 恢复原块及其定位表。^[block-graph.md:117-124]

源包记录的七个测试全部限于 A1，未覆盖多生成元、空对偶包或空块、`element` 失败分支等情形。该源包属于结构性源码阅读，没有执行构建、测试或原版运行；块枚举的数学正确性依赖独立 HPC 证据链，不能由本页的结构说明推出。相关覆盖见 [[完整块图的测试覆盖与证据边界]]。^[block-graph.md:9-13, block-graph.md:117-126, block-graph.md:139-139]

## Sources

- [block-graph.md](block-graph.md)：完整块图：实形式与对偶实形式的纤维积。
