---
title: 最小环面部分的 grading 轨道搜索
summary: 算法用 LIFO 栈与 BTreeSet<u64> 遍历 grading 轨道，在置位方向施加余根奇偶平移和 grading 翻转，筛选满足目标约束的候选并按位向量整数序取最小值。
sources:
  - minimal-torus.md
kind: concept
createdAt: "2026-10-09T15:01:39.154Z"
updatedAt: "2026-10-09T15:01:39.154Z"
tags:
  - 轨道搜索
  - 环面部分
  - 算法
aliases:
  - 最小环面部分的-grading-轨道搜索
  - 最G轨
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 最小环面部分的 grading 轨道搜索

`minimal_torus_part` 将给定强代表下降到基本纤维，再搜索基本虚 grading 的轨道，选出数值最小的环面部分，使其在 datum-单根上的紧致性模式复现目标弱实形。该函数服务于自定义种子构造分支，与[[合成实形的选定余特征]]共同确定初始数据。^[minimal-torus.md:17-25]

## 搜索的起点

初始环面部分来自 `factor − coch`：每个坐标必须为整数，奇数坐标对应置位。随后通过逆 Cayley 变换或 based twisted 共轭完成[[强代表下降到基本纤维]]，并在末尾执行 `coset.reduce`。下降后的环面部分 `tp` 提升为余权
\[
\mathrm{coweight}=\mathrm{coch}+\mathrm{lift}(tp),
\]
其中 `lift(tp)` 在置位坐标上取 1。^[minimal-torus.md:50-62]

## Grading 与目标约束

搜索使用基本纤维 `CartanId(0)` 的虚单根基。`start` 由各基根与 `coweight` 的配对偶性确定，置位表示紧致；`constrained` 标记其中属于 datum-单根的位置；`target` 保存这些受约束位置上目标实形的非紧致性，通过翻转适配 crate 的 `Grading` 存储约定。理解候选判定时，需要区分这些位的含义，不能将紧致性位与非紧致性位直接混同。^[minimal-torus.md:63-67]

每个基位置还关联两个模二数据：`m_alpha[i]` 是相应基余根的奇偶向量，`grading_shift[i]` 来自基间配对的 mod 2 矩阵。前者用于平移环面部分，后者用于同步翻转 grading。^[minimal-torus.md:67-74]

目标约束采用逐位置配对。源码注释说明，上游依赖根编号将 datum-单根排在基本单虚根基的前导位置；Rust 的虚根基具有自己的确定性根序，因此显式按位置对应。注释声称，在上游前导段不变量成立时，特别是 distinguished 对合平凡时，两种方式一致；本来源未独立验证该一致性。相关位置约束见[[基本虚根 grading 的位置约束]]。^[minimal-torus.md:68-71, minimal-torus.md:95-96]

## 轨道遍历与最小值选择

轨道遍历使用 LIFO 栈和 `BTreeSet<u64>` 去重，状态数量上限为 \(2^{\mathrm{rank}}\)。对当前 grading 的每个置位位置 \(i\)，以 `m_alpha[i]` 平移环面部分，同时按 `grading_shift[i]` 翻转 grading；受约束位置满足目标条件的状态成为候选。^[minimal-torus.md:72-74]

若候选集合为空，函数返回具名错误 `"minimal torus part candidates"`，对应上游的断言情形。否则，函数选取最小候选：`ModTwoVector` 对等维位向量的排序等同于整数序，因此这里的“最小”指环面部分位向量的数值最小。^[minimal-torus.md:75-76]

## 资源限制与证据边界

搜索的 `u64` 编码由 `MAX_MASK_BITS = 63` 限制格秩；入口遇到 `rank > 63` 时返回 `SeedResourceLimit { resource: "mask bits" }`。`minimal_torus_part` 没有显式预算参数，仅依靠该位数限制与分配防护；相关门控可参见[[合成实形种子算法的门控与资源限制]]。^[minimal-torus.md:30-31, minimal-torus.md:46-48, minimal-torus.md:99-100]

来源记录了四个测试锚点，均为 rank 2、紧致内类。三个正例均满足 `coch == factor`，没有通过可区分的断言刻画非平凡运输行为；非紧致 distinguished、多数具名错误分支以及 `rank > 63` 的资源门也未覆盖。这些材料属于结构性阅读，不构成该算法的数学验收，详见[[最小环面算法的测试覆盖边界]]。^[minimal-torus.md:10-13, minimal-torus.md:78-98]

## Sources

- [minimal-torus.md](minimal-torus.md) — 合成实形的选定余特征与初始环面部分（minimal_torus.rs）
