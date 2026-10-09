---
title: 最小环面部分的 grading 轨道搜索
summary: 使用 LIFO 栈和 BTreeSet<u64> 遍历轨道，在置位 grading 方向执行余根奇偶平移及 grading 翻转，从满足约束的候选中选取数值最小环面部分。
sources:
  - minimal-torus.md
kind: concept
createdAt: "2026-10-09T15:01:39.154Z"
updatedAt: "2026-10-09T22:40:06.843Z"
tags:
  - 轨道搜索
  - grading
  - 环面
aliases:
  - 最小环面部分的-grading-轨道搜索
  - 最G轨
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 最小环面部分的 grading 轨道搜索
summary: minimal_torus_part 将强代表下降到基本纤维，遍历基本虚 grading 轨道，并从满足目标约束的候选中选取数值最小的环面部分。
sources:
  - minimal-torus.md
kind: concept
tags:
  - 轨道搜索
  - 分级
  - 环面部分
---

# 最小环面部分的 grading 轨道搜索

`minimal_torus_part` 服务于自定义实形种子构造：将给定强代表下降到基本纤维，再遍历基本虚 grading 轨道，选出数值最小的环面部分，使其在 datum-单根上的紧致性模式复现目标弱实形。它与[[合成实形的选定余特征]]共同提供构造所需的初始数据。^[minimal-torus.md:17-25]

## 搜索起点

初始环面部分由 `factor − coch` 构造：逐坐标检查整性，非整数触发 `"torus-part integrality"`，整数中的奇数坐标置位。随后通过逆 Cayley 与 based twisted 共轭完成[[强代表下降到基本纤维]]，末尾执行 `coset.reduce`。下降后的环面部分 `tp` 提升为余权 \(\mathrm{coweight}=\mathrm{coch}+\mathrm{lift}(tp)\)，其中 `lift(tp)` 在置位坐标上取 1。^[minimal-torus.md:50-62]

下降在每个中间环面部分的目标对合处约化，而来源所述的上游实现仅在末尾约化。源码注释以这些操作是 mod-2 商上的类映射为依据，声明逐步约化不改变最终约化类；来源未独立验证这一声明。^[minimal-torus.md:27-29]

## Grading 与目标约束

搜索使用基本纤维 `CartanId(0)` 的虚单根基。`start` 记录各基根与 `coweight` 的偶配对，置位表示紧致；`constrained` 标记其中属于 datum-单根的位置；`target` 保存受约束位置上目标实形的非紧致性，并通过翻转适配 crate 的 `Grading` 存储约定。阅读搜索逻辑时须保留这些位含义及其转换。^[minimal-torus.md:63-67]

各基位置 \(i\) 还具有两项模二数据：`m_alpha[i]` 是对应基余根的奇偶向量，用于平移环面部分；`grading_shift[i]` 来自基间配对的 mod 2 矩阵，用于同步翻转 grading。^[minimal-torus.md:67-74]

目标模式采用逐位置配对。源码注释指出，上游把目标 grading 切片与基本单虚根基中前导的 datum-单根条目配对，依赖根编号将这些条目排在最前；Rust 虚根基有自己的确定性根序，因此改为逐位置对应。注释声明，上游前导段不变量成立时，特别是 distinguished 对合平凡时，两者一致；来源未独立验证这一一致性，参见[[基本虚根 grading 的位置约束]]。^[minimal-torus.md:68-71, minimal-torus.md:95-96]

## 轨道遍历与最小值选择

遍历使用 LIFO 栈及 `BTreeSet<u64>` 去重，状态数量上限为 \(2^{\mathrm{rank}}\)。在当前 grading 的每个**置位**位置 \(i\)，算法以 `m_alpha[i]` 平移环面部分，并按 `grading_shift[i]` 翻转 grading；受约束位置与目标一致的状态成为候选。^[minimal-torus.md:72-74]

若候选集合为空，函数返回具名错误 `"minimal torus part candidates"`，对应上游的断言情形。否则按 `ModTwoVector` 的顺序选取最小候选；等维位向量的该顺序就是整数序，因此这里的“最小”指环面部分位向量的数值最小。^[minimal-torus.md:75-76]

## 门控与资源限制

入口依次检查 table 的 inner class、`coch`／`factor` 长度、置换长度及格秩。inner class 不符或置换长度不符返回 `DatumMismatch`；长度检查失败返回 `RankMismatch`，其中 `actual` 取 `coch` 与 `factor` 长度的较大值；`rank > 63` 返回 `SeedResourceLimit { resource: "mask bits" }`。^[minimal-torus.md:46-48]

`MAX_MASK_BITS = 63` 为轨道去重所用的 `u64` 编码限定格秩。`minimal_torus_part` 没有显式预算参数，资源防护仅包含该位数限制与分配防护；相关主题见[[合成实形种子算法的门控与资源限制]]。^[minimal-torus.md:30-31, minimal-torus.md:99-100]

## 测试与证据边界

来源记录四个测试锚点，均为 rank 2、紧致内类，涉及 A1.T1 中心因子、A2 非典范种子、等价因子的种子选择及门控拒绝。A1.T1 示例得到零环面部分；A2 的两组种子分别得到坐标 0 与坐标 1 置位的环面部分。^[minimal-torus.md:78-87]

三个正例均满足 `coch == factor`，没有通过可区分的断言刻画非平凡运输行为；第三个测试与第二个测试第一组输入完全相同。测试还未覆盖非紧致 distinguished、多数具名错误分支及 `rank > 63` 的资源门，参见[[最小环面算法的测试覆盖边界]]。^[minimal-torus.md:89-98]

来源属于结构性源码阅读，不构成数学验收。上游位置仅转录自代码注释，未核对上游字节；本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[minimal-torus.md:9-13, minimal-torus.md:104-108]

## Sources

- [minimal-torus.md](../../sources/minimal-torus.md) — 合成实形的选定余特征与初始环面部分（minimal_torus.rs）。
