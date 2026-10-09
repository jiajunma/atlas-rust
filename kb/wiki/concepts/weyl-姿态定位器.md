---
title: Weyl 姿态定位器
summary: BlockLocator 以 int_sys 标识典范数据，以 w 保持整正性地映到查询姿态，并通过 simp_int 与 simple_pi 记录整单根像及生成元位置；本来源中的模块尚未接入 RepTable::lookup。
sources:
  - locator.md
kind: concept
createdAt: "2026-10-09T14:59:23.876Z"
updatedAt: "2026-10-09T21:01:22.592Z"
tags:
  - Weyl群
  - 表示参数
  - Rust设计
aliases:
  - weyl-姿态定位器
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Weyl 姿态定位器
summary: BlockLocator 以 int_sys 标识典范整数据，以 Weyl 元素 w 记录查询姿态，并通过 simp_int 和 simple_pi 保存整单根像的排序与生成元对应关系；来源中的实现尚未接入 RepTable::lookup。
sources:
  - locator.md
kind: concept
tags:
  - Weyl群
  - 整根子系统
  - 姿态定位
aliases:
  - weyl-姿态定位器
---

# Weyl 姿态定位器

Weyl 姿态定位器 `BlockLocator` 描述典范整数据在一次查询中的实际姿态：它记录典范数据标识、将典范基本 alcove 整子系统映到查询姿态并保持整正性的 Weyl 元素，以及整单根像的排序和生成元编号对应关系。该实现位于 `locator.rs`，属于纯、尚未接线的移植；来源所描述的版本中，`RepTable::lookup` 尚不调用该模块。^[locator.md:16-25]

## 数据结构与排序契约

`BlockLocator` 与[[典范整数据驻留]]分工明确：`IntegralDatumTable` 以追加式序号管理典范数据；`IntegralDatumItem` 保存作为驻留键的正根表、子系统单根及其余根坐标缓存；定位器保存查询相对于典范数据的姿态。^[locator.md:19-25]

定位器包含四个私有字段，并提供四个只读访问器：`int_sys: u32` 标识典范数据；`w: WeylElement` 表示姿态；`simp_int: Vec<RootId>` 保存排序后的整单根像；`simple_pi: Vec<usize>` 将典范单生成元下标映到其像在 `simp_int` 中的位置。^[locator.md:22-25, locator.md:33-41]

所有面向定位器的列表采用上游正根 `RootNbr` 顺序，即按高度、简单坐标反字典序排列。crate 内部的 `RootId` 则采用环境字典序。来源将这种差异描述为存储偏差，并声明没有语义偏差；使用上游顺序使 `simple_pi` 可以直接与 oracle 比较。^[locator.md:27-29]

## 构造流程

`IntegralDatumTable::int_item` 首先通过 `root_vertex_of_alcove(system, gamma)` 取得参数所在 alcove 的根格顶点，再从参数分子中逐坐标减去 `denominator * vertex`，全程使用受检算术。随后，`factor_dominant` 反复选择当前在某单余根上取负值的最低下标生成元，施加反射并按施加顺序记录成词，直至参数优势化。详见[[基于 alcove 的整数据定位流程]]。^[locator.md:48-54]

算法在优势化参数上检测基本 alcove 的墙：正墙对应单根，与零比较；负墙对应各 Dynkin 分量最高余根之负，与 `-denominator` 比较。命中的墙组成 `on_wall`。随后逆序遍历优势化反射词，将当前求值非整的字母左乘进 `w`，同时从分子中消去该反射；最终 `w(dominant gamma)` 给出所求姿态。^[locator.md:55-60, locator.md:68-69]

典范驻留键取自 `on_wall` 的加法闭包正部，并按上游正根顺序排序。闭包使用**余根坐标加法**：将生成元及其负根纳入闭包，反复求两两余根坐标和，直至不动点。不能改用根加法，因为整余根在根加法下不必封闭。驻留命中时复用既有数据；相关主题见[[整子系统的余根加法闭包]]。^[locator.md:61-62, locator.md:70-71]

最后，对每个典范单根计算 `w.image(alpha)`。缺失的像触发 `"provenance"` 不变量违规，非正像触发 `"integral image positivity"` 违规，后者是上游 `assert(is_posroot)` 的受检形式。将这些像排序得到 `simp_int`，再记录各像在排序结果中的位置得到 `simple_pi`。^[locator.md:63-66]

## 相对姿态变换

`make_relative_to` 将定位器改写为相对于基定位器的姿态。两者必须具有相同的 `int_sys`，否则返回不变量错误。`w` 右乘基姿态之逆，`simple_pi` 则与基的简单置换之逆右复合；详见[[定位器的相对姿态变换]]。^[locator.md:75-78]

若旧置换为 `old`，基置换的逆为 `inv`，更新规则为 `simple_pi[j] = old[inv[j]]`。逆置换构造使用 `usize::MAX` 哨兵检测越界或重复像。来源中的手算测试覆盖 `[2,1,0] ∘ [1,2,0]^{-1} = [0,2,1]`，也覆盖 `w` 右乘逆及不同 `int_sys` 的拒绝路径。^[locator.md:75-78, locator.md:92-93]

## 测试锚点与证据边界

B2 回归用于区分根加法与余根加法：前者错误地仅产生 4 个长根，后者得到全部 8 个根。F4 半积分案例（case107、HPC3832609）检查 `w.image` 作用于 item 全部正根后，是否等于独立过滤得到的期望集合。^[locator.md:82-85]

A2 测试记录了一个重要行为：两个切片参数可驻留不同的 A1 item，说明典范数据依赖参数所在 alcove，而非仅依赖其整根系；这与设计简报草图的差异已在注释中记录。测试还覆盖 Weyl 共轭参数共享同一 item、零参数驻留全系统并得到恒等姿态，以及 B2 长、短 A1 驻留不同 item 且重复查询复用。^[locator.md:86-90]

来源属于结构性阅读，不构成 locator 层的数学或正确性验收。上游文件行号仅转录自代码注释，未核对上游字节；本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[locator.md:9-12, locator.md:98-98, locator.md:109-113]

## 使用限制

`IntegralDatumTable` 不持有 `RootSystem`，每次调用均由调用方重传。跨调用换用不同根系不会产生错误信号，但结果无定义，因此使用时必须保持根系上下文一致。^[locator.md:99-100]

`factor_dominant` 没有迭代上限，其终止性依赖根系理论；`additive_closure` 也没有迭代或规模预算。多处 `zip` 截断隐含长度一致假设，分母非零且不为 `i64::MIN` 依赖 `RationalWeight` 的构造不变量，像互异检查仅在 debug 构建中生效。测试覆盖限于 A2、B2、F4，尚未覆盖可约根系、rank-0 边角及多条错误路径。^[locator.md:52-54, locator.md:101-105]

## Sources

- [locator.md](../../sources/locator.md) — 典范整数据驻留与 Weyl 姿态定位器（locator.rs）
