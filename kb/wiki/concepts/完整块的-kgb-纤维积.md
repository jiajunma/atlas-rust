---
title: 完整块的 KGB 纤维积
summary: 完整块按 twisted involution 的对偶关系配对两个完整 KGB 集，保留各自坐标编号；缺失对偶包不贡献元素，对偶包索引中的重复键会静默覆盖。
sources:
  - block-graph.md
kind: concept
createdAt: "2026-10-09T14:41:08.924Z"
updatedAt: "2026-10-09T19:26:27.462Z"
tags:
  - 块图
  - KGB
  - 对偶
aliases:
  - 完整块的-kgb-纤维积
  - 完K纤
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 完整块的 KGB 纤维积

完整块是一个实形式的 [[KGB 图与弱实形式|KGB 图]]与其对偶实形式的 KGB 图，按 twisted involution 及其对偶配对形成的纤维积。每个块元素由坐标对 \((x,y)\) 表示，两侧 KGB 元素所属的对合包须满足对偶配对关系；坐标保留各自完整 KGB 集中的编号。^[block-graph.md:17-33]

## 对合的对偶配对

配对使用 twisted Weyl 群的 `dual_involution` 映射。在对合矩阵上，该映射是负转置；在 Weyl 元素上，由 \(f(e)=w_0\) 与 \(f(s.w)=f(w)\,d(s)\) 刻画。实现从对偶最长元出发，按原元素的约化字**自右向左**，以对偶扭曲字母右乘；约化字使用两侧共享的外部生成元编号。详见 [[Twisted Weyl 群的对合对偶映射]]。^[block-graph.md:20-24]

## 完整 KGB 集与公共 Cartan 限制

解释器从两个实形式的**完整 KGB 集**构建块，以保持原始坐标编号。显式使用 `common_Cartans` 受限重载会改变编号，因此公共 Cartan 的限制通过配对隐式实现：若某个原侧对合的对偶不在对偶包索引中，该原侧包便不贡献任何坐标对，对应上游 `tauPacket` 返回空区间 `(0,0)` 的行为。^[block-graph.md:26-33]

对偶包通过 `HashMap<WeylElement, usize>` 索引。来源中的源码阅读指出，重复键会静默覆盖，当前没有重复键防护；索引机制本身不能保证检测出重复的对合包键。^[block-graph.md:30-33]

## 构造与元素编号

`BlockGraph::build` 接收原侧与对偶侧的 KGB 图、对合表，以及 `dual_inner_class` 和 `weyl_budget`。对偶内类提供对偶 twisted Weyl 群的 distinguished twist 与最长元，预算限制定位最长元时的枚举规模。若两图半单秩不同，或对偶表与对偶内类的根系统不同，构造返回 `DatumMismatch`。^[block-graph.md:66-70]

元素按原 KGB 的对合包顺序编号；每个配对包内部以 **\(x\) 为外层、\(y\) 为内层**枚举笛卡尔积。`xs`、`ys` 保存两侧完整 KGB 坐标，块元素长度取原侧的 `kgb.length(x)`，供 KL 表进行长度排序。若构造后 `xs.len() != size`，则报告 `"block size"` 错误。^[block-graph.md:56-61, block-graph.md:70-71]

## 坐标定位

`xs` 弱增是坐标定位的不变量。`first_z_of_x[x]` 记录首个满足 \(x(z)\geq x\) 的块元素，表长为 `xrange + 1`，并带有值为块大小的哨兵。`element(x,y)` 先取得对应的 \(x\) 区间，再按连续的 \(y\) 偏移定位，并校验所得坐标；越界返回 `IndexOutOfRange`，纤维不符则报告 `"element fiber"`。具体布局见 [[BlockGraph 存储布局与坐标定位不变量]]。^[block-graph.md:62-64, block-graph.md:88-91]

## 与对偶块的关系

`BlockGraph::dual` 通过纯数据变换交换两个实形式的角色：反转元素序 \(z'=\mathrm{size}-1-z\)，交换 \(x/y\) 坐标，将长度变为 \(\mathrm{max\_len}-\ell(z)\)，并变换下降状态与 cross/Cayley 链接，重新计算坐标定位表。其中 `max_len` 取末元素长度，依赖末元素长度最大的排序性质；空块取零。详见 [[完整块图的对偶数据变换]]。^[block-graph.md:95-104]

## 示例与证据边界

秩一测试中的 \(\operatorname{block}(\mathrm{SL}(2,\mathbb R),\mathrm{PGL}(2,\mathbb R))\) 有三个元素，坐标依次为 \((0,1),(1,1),(2,0)\)，与冻结 fixture capture `3501519` 对齐。测试还检查了数据变换所得对偶块与原生对偶块经 `element` 换编号后的一致性，以及覆盖全 KGB 范围时 `dual().dual()` 恢复原块及其定位表。^[block-graph.md:117-124]

来源记录的七个测试全部限于 A1，未覆盖多生成元、空对偶包或空块、`element` 失败分支等情形。来源本身属于结构性源码阅读，未执行构建、测试或原版运行；块枚举的正确性归于独立的 [[HPC 验收证据链]]。相关覆盖范围见 [[完整块图的测试覆盖与证据边界]]。^[block-graph.md:9-13, block-graph.md:117-126, block-graph.md:139-139]

## Sources

- [block-graph.md](block-graph.md)：完整块图：实形式与对偶实形式的纤维积。
