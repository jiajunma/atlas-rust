---
title: 基于 alcove 的整数据定位流程
summary: int_item 依次进行根格顶点平移、dominant 化、基本 alcove 墙检测、逆序处理非整反射、典范闭包驻留和单根像置换构造；factor_dominant 贪心选取最低下标负配对生成元且无迭代上限。
sources:
  - locator.md
kind: concept
createdAt: "2026-10-09T14:59:38.938Z"
updatedAt: "2026-10-09T14:59:38.938Z"
tags:
  - alcove
  - 根系算法
  - Weyl群
aliases:
  - 基于-alcove-的整数据定位流程
  - 基A的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 基于 alcove 的整数据定位流程

基于 alcove 的整数据定位由 `locator.rs` 中的 `IntegralDatumTable::int_item` 实现：给定根系与有理权 `gamma`，先利用 alcove 根格顶点与优势化确定典范整子系统，再驻留其数据，并构造从典范姿态到查询姿态的 Weyl 定位器。该模块是上游相关接口的纯移植切片，尚未接入 `RepTable::lookup`。^[locator.md:16-25, locator.md:48-66]

## 数据与输出

`IntegralDatumTable` 负责[[典范整数据驻留]]，以追加式序号标识数据；`IntegralDatumItem` 保存作为驻留键的正根表、子系统单根及其余根坐标缓存。`BlockLocator` 保存典范数据编号 `int_sys`、保持整正性的 Weyl 元素 `w`、排序后的整单根像 `simp_int`，以及从典范单生成元下标到该排序位置的置换 `simple_pi`。^[locator.md:19-25]

定位器面向外部的根列表统一采用上游正根顺序，即先按高度、再按简单坐标反字典序排序；这不同于 crate 内 `RootId` 的环境字典序，使 `simple_pi` 可以直接与 oracle 比较。相关编号约定见 [[RootNumbering 根编号与 RootNbr 顺序]]。^[locator.md:27-29]

## 定位步骤

### 1. 平移到根格顶点参照系

调用 `root_vertex_of_alcove(system, gamma)`，取得 `gamma` 所在 alcove 的根格顶点。随后从有理权分子逐坐标减去 `denominator * vertex`，算术全程使用受检运算。此步骤关联[[Alcove 根格顶点与基本 Alcove 约化]]。^[locator.md:50-51]

### 2. 优势化并记录反射顺序

`factor_dominant` 每次选择当前在相应单余根上取负值的最低下标生成元，施加反射并将其记入 `word`，直到权成为 dominant。`word` 按实际施加顺序记录；算法没有迭代上限，其终止性依赖根系理论，而非代码预算。^[locator.md:52-54]

### 3. 识别命中的基本 alcove 墙

`fundamental_alcove_walls` 给出全部单根，以及每个 Dynkin 分量最高余根所对应的负墙；实现通过所有单根的 `min_coroots_for` 梯子底表之交取得这些负根。对当前分子求值时，正墙与 `0` 比较，负墙与 `-denominator` 比较，命中的墙加入 `on_wall`。^[locator.md:55-57, locator.md:68-69]

### 4. 从反射词恢复查询姿态

按 `word.iter().rev()` 逆序遍历反射词。若当前求值满足 `rem_euclid(denominator) != 0`，即求值非整，则把该字母左乘进 `w`，同时从分子中消去该反射。最终 `w(dominant gamma)` 给出所需姿态。^[locator.md:58-60]

### 5. 构造并驻留典范整数据

对 `on_wall` 求加法闭包时，必须使用**余根坐标**。实现先将生成元及其负根加入闭包，再反复计算两两余根坐标和，直到达到不动点；取闭包正部，按上游正根序排序，作为驻留键。键已存在时直接复用已有条目，因此驻留是幂等的。^[locator.md:61-62, locator.md:70-71]

余根坐标的选择是实质性要求：整余根条件对应的集合在根加法下不必封闭。B2 回归锚点表明，使用根加法会错误地只产生 4 个长根，而余根加法给出全部 8 个根。^[locator.md:61-62, locator.md:82-83]

子系统单根由 `pos_simples` 提取。输入须已按上游顺序排序；对每个根 α 扫描后续根 β，当 `bracket(β, α) > 0` 时考察反射像：像为正则排除 β 的单根资格，像为负则排除 α 并进入下一轮外层扫描。^[locator.md:71-73]

### 6. 构造单根像与生成元置换

对每个典范单根 α 计算 `w.image(alpha)`。缺失的像触发 `"provenance"` 不变量错误，非正像触发 `"integral image positivity"` 不变量错误。将这些像排序得到 `simp_int`，再记录每个原始像在排序结果中的位置，形成 `simple_pi`；这些数据共同构成[[Weyl 姿态定位器]]。^[locator.md:63-66]

## 典范性的范围

这里的典范数据依赖 `gamma` 所在的 alcove，不能仅由其整根系判定。A2 测试记录了两个切片的 `gamma` 驻留不同 A1 条目的行为；另有 Weyl 共轭查询共享同一条目的幂等性测试，以及 `gamma = 0` 驻留全系统并得到恒等姿态的测试。B2 测试还区分长根与短根的 A1 条目，并检查重复查询复用。^[locator.md:86-90]

## 验证与限制

来源列出了 A2、B2、F4 测试锚点。其中 F4 半积分案例检查 `w.image` 作用于条目全部正根后，所得集合等于独立过滤的期望集。不过，该材料只完成结构性阅读，不声称数学验收；上游文件行号来自代码注释，未核对上游字节，本次知识维护也未执行测试或 benchmark。^[locator.md:9-12, locator.md:80-90, locator.md:105-113]

`IntegralDatumTable` 不持有 `RootSystem`，调用时必须保持根系上下文一致；跨调用更换根系不会报错，但结果无定义。`factor_dominant` 与 `additive_closure` 都没有迭代或规模预算，多处 `zip` 依赖长度一致，分母合法性依赖 `RationalWeight` 的构造不变量，像互异检查仅在 debug 构建中生效。现有测试未覆盖可约根系、rank-0 边界及多条错误路径。^[locator.md:99-105]

## Sources

- [locator.md](locator.md) — 典范整数据驻留与 Weyl 姿态定位器（locator.rs）
