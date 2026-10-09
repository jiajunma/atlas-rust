---
title: 基于 alcove 的整数据定位流程
summary: int_item 依次执行根格顶点平移、dominant 化、基本 alcove 墙检测、逆序处理非整反射、典范闭包驻留及单根像置换构造。
sources:
  - locator.md
kind: concept
createdAt: "2026-10-09T14:59:38.938Z"
updatedAt: "2026-10-09T21:01:35.931Z"
tags:
  - Alcove
  - 整根系
  - 定位算法
aliases:
  - 基于-alcove-的整数据定位流程
  - 基A的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 基于 alcove 的整数据定位流程
summary: int_item 通过根格顶点平移、优势化、基本 alcove 墙检测、逆序处理非整反射、余根闭包驻留及单根像排序，构造典范整数据与查询姿态定位器。
sources:
  - locator.md
kind: concept
tags:
  - alcove
  - 根系算法
  - Weyl群
---

# 基于 alcove 的整数据定位流程

基于 alcove 的整数据定位由 `locator.rs` 中的 `IntegralDatumTable::int_item` 实现。给定根系与有理权 `gamma`，它确定并驻留典范整子系统，再构造从典范基本 alcove 整子系统到查询实际姿态的 Weyl 定位器。来源所述实现属于纯、未接线的移植切片，`RepTable::lookup` 尚未调用该模块。^[locator.md:16-25, locator.md:48-66]

## 数据与排序约定

`IntegralDatumTable` 负责[[典范整数据驻留]]，使用追加式序号标识条目；`IntegralDatumItem` 保存作为驻留键的正根表、子系统单根及其余根坐标缓存。`BlockLocator` 保存典范数据编号 `int_sys`、保持整正性的 Weyl 元素 `w`、排序后的整单根像 `simp_int`，以及将典范单生成元下标映到 `simp_int` 位置的置换 `simple_pi`。^[locator.md:19-25]

所有面向 locator 的根列表均按上游正根顺序排列，即先按高度、再按简单坐标反字典序排序。这与 crate 内 `RootId` 的环境字典序不同，使 `simple_pi` 能直接与 oracle 比较。^[locator.md:27-29]

## 定位步骤

### 1. 根格顶点平移

调用 `root_vertex_of_alcove(system, gamma)`，取得 `gamma` 所在 alcove 的根格顶点。随后从有理权分子逐坐标减去 `denominator * vertex`，全程使用受检算术。^[locator.md:50-51]

### 2. 优势化并记录反射词

`factor_dominant` 贪心选择当前在相应单余根上取负值的最低下标生成元，施加反射并记入 `word`，直到权成为 dominant。反射词按实际施加顺序记录；此过程没有迭代上限，终止性依赖根系理论，而非代码防护。^[locator.md:52-54]

### 3. 检测基本 alcove 的墙

`fundamental_alcove_walls` 给出全部单根，以及各 Dynkin 分量最高余根所对应的负墙。实现取所有单根的 `min_coroots_for` 梯子底表之交中的负根，再并入全部单根。检测时，正墙上的分子求值与 `0` 比较，负墙上的求值与 `-denominator` 比较，命中的墙加入 `on_wall`。^[locator.md:55-57, locator.md:68-69]

### 4. 逆序恢复查询姿态

按 `word.iter().rev()` 逆序遍历反射词。当当前求值满足 `rem_euclid(denominator) != 0`，即对应配对非整时，将该字母左乘进 `w`，同时从分子中消去该反射。最终 `w(dominant gamma)` 给出所需姿态。^[locator.md:58-60]

### 5. 构造闭包并驻留

对 `on_wall` 的闭包计算必须使用**余根坐标**：先纳入生成元及其负根，再反复求两两余根坐标和，直至不动点。取闭包正部并按上游正根序排序，得到典范驻留键；命中已有键时复用原条目，因而驻留具有幂等性。相关细节见[[整子系统的余根加法闭包]]。^[locator.md:61-62, locator.md:70-71]

余根坐标的选择不可替换为根坐标加法。来源指出，整余根在根加法下不必封闭；B2 回归锚点中，根加法错误地只产生 4 个长根，而余根加法得到全部 8 个根。^[locator.md:61-62, locator.md:82-83]

`pos_simples` 从已按上游顺序排列的正根中提取子系统单根。对每个根 α 扫描其后的 β，当 `bracket(β, α) > 0` 时考察反射像：像为正则 β 非单根，像为负则 α 非单根，并通过 `continue 'outer` 转入下一轮外层扫描。^[locator.md:71-73]

### 6. 构造单根像与生成元置换

对每个典范单根 α 计算 `w.image(alpha)`。缺失的像触发 `"provenance"` 不变量错误，非正像触发 `"integral image positivity"` 不变量错误，后者是上游 `assert(is_posroot)` 的受检形式。将像排序得到 `simp_int`，再记录各像在排序结果中的位置，形成 `simple_pi`，完成[[Weyl 姿态定位器]]的构造。^[locator.md:63-66]

## 典范性的范围

典范数据依赖 `gamma` 所在的 alcove，不能仅由其整根系确定。A2 测试记录了两个切片的 `gamma` 驻留不同 A1 条目的行为，这也是来源明确保留的、与设计简报草图不同的实现行为。另有测试检查 Weyl 共轭查询共享同一条目，以及 `gamma = 0` 驻留全系统并得到恒等姿态；B2 测试区分长根与短根 A1 条目，并检查重复查询复用。^[locator.md:86-90]

定位器还支持[[定位器的相对姿态变换]]：`make_relative_to` 要求两者具有相同 `int_sys`，然后将 `w` 右乘基姿态之逆，将 `simple_pi` 与基置换的逆右复合，即 `simple_pi[j] = old[inv[j]]`。不同 `int_sys` 会触发不变量错误。^[locator.md:75-78]

## 验证与限制

来源列出的测试覆盖 A2、B2、F4。其中 F4 半积分案例检查 `w.image` 作用于条目全部正根后，所得集合是否等于独立过滤的期望集；此外还列有错秩 `gamma` 返回 `RankMismatch` 的测试。可约根系、rank-0 边角及多条错误路径尚未覆盖。^[locator.md:80-94, locator.md:105-105]

`IntegralDatumTable` 不持有 `RootSystem`，每次调用都须重传；跨调用更换根系不会产生错误信号，但结果无定义。`factor_dominant` 与 `additive_closure` 没有迭代或规模预算，多处 `zip` 隐含长度一致假设；分母非零且不为 `i64::MIN` 依赖 `RationalWeight` 的构造不变量，像互异检查仅在 debug 构建中生效。^[locator.md:99-104]

本页依据结构性源码阅读，不构成 locator 层的数学验收。来源中的上游行号仅转录自代码注释，未核对上游字节；该次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[locator.md:9-12, locator.md:109-113]

## Sources

- [locator.md](../../sources/locator.md) — 典范整数据驻留与 Weyl 姿态定位器（locator.rs）
