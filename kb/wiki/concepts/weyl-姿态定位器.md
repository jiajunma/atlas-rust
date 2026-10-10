---
title: Weyl 姿态定位器
summary: BlockLocator 以 int_sys、w、simp_int 和 simple_pi 描述典范整子系统到查询姿态的映射；该来源快照中尚未接入 RepTable::lookup。
sources:
  - locator.md
kind: concept
createdAt: "2026-10-09T14:59:23.876Z"
updatedAt: "2026-10-10T00:42:15.854Z"
tags:
  - Weyl群
  - 整子系统
  - 定位器
aliases:
  - weyl-姿态定位器
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Weyl 姿态定位器
summary: BlockLocator 以典范整数据标识、Weyl 元素及整单根像的排序置换记录查询姿态；来源版本中尚未接入 RepTable::lookup。
sources:
  - locator.md
kind: concept
tags:
  - Weyl群
  - 参数定位
aliases:
  - weyl-姿态定位器
provenanceState: extracted
---

# Weyl 姿态定位器

Weyl 姿态定位器 `BlockLocator` 描述典范整子系统在一次查询中的实际姿态。其 Weyl 元素 `w` 将典范基本 alcove 整子系统映到查询姿态，并保持整正性；整单根像及其排序置换记录生成元之间的对应关系。来源版本的实现位于 `crates/atlas-real-group/src/locator.rs`，属于纯、尚未接线的移植，`RepTable::lookup` 尚不调用该模块。^[locator.md:10-12, locator.md:16-25]

## 数据结构与排序契约

定位器与[[典范整数据驻留]]分工明确：`IntegralDatumTable` 以追加式序号管理典范数据，`IntegralDatumItem` 保存作为驻留键的正根表、子系统单根及其余根坐标缓存，`BlockLocator` 保存查询姿态。^[locator.md:19-25]

`BlockLocator` 有四个私有字段，并提供四个只读访问器：`int_sys: u32` 标识典范数据；`w: WeylElement` 表示姿态映射；`simp_int: Vec<RootId>` 保存按上游正根序排列的整单根像；`simple_pi: Vec<usize>` 将典范单生成元下标映到其像在 `simp_int` 中的位置。^[locator.md:22-25, locator.md:33-36]

所有面向定位器的列表均采用上游 `RootNbr` 正根顺序，即按高度、简单坐标反字典序排列，而 crate 的 `RootId` 使用环境字典序。来源将此描述为存储偏差，并声明没有语义偏差；采用上游顺序使 `simple_pi` 可以直接与 oracle 比较。^[locator.md:27-29]

## 构造流程

`IntegralDatumTable::int_item` 首先调用 `root_vertex_of_alcove(system, gamma)`，取得参数所在 alcove 的根格顶点，再从分子中逐坐标减去 `denominator * vertex`，全程使用受检算术。随后，`factor_dominant` 反复选择当前在单余根上取负值的最低下标生成元进行反射，按施加顺序记录反射词，直至参数优势化。详见[[基于 alcove 的整数据定位流程]]。^[locator.md:48-54]

算法检测优势化参数所在的基本 alcove 墙：正墙对应单根，与零比较；负墙对应各 Dynkin 分量最高余根之负，与 `-denominator` 比较，命中者组成 `on_wall`。随后按 `word.iter().rev()` 逆序遍历反射词，将当前求值非整的字母左乘进 `w`，同时从分子中消去该反射；最终 `w(dominant gamma)` 给出所求姿态。^[locator.md:55-60, locator.md:68-69]

典范驻留键是 `on_wall` 的加法闭包正部，按上游正根序排序后驻留，命中时复用已有数据。闭包以**余根坐标**为键，将生成元及其负根纳入集合，反复求两两余根坐标和直至不动点；不能用根加法替代，因为整余根在根加法下不必封闭。详见[[整子系统的余根加法闭包]]。^[locator.md:61-62, locator.md:70-71]

最后，对每个典范单根计算 `w.image(alpha)`：像缺失时触发 `"provenance"` 不变量违规，像非正时触发 `"integral image positivity"` 违规，后者是上游 `assert(is_posroot)` 的受检形式。像的排序副本构成 `simp_int`，各像在其中的位置构成 `simple_pi`，详见[[定位器整单根像的正性与置换校验]]。^[locator.md:63-66]

## 相对姿态变换

`make_relative_to` 将定位器改写为相对于基定位器的姿态。两者必须具有相同的 `int_sys`，否则返回不变量错误。更新时，`w` 右乘基姿态之逆，`simple_pi` 与基的简单置换之逆右复合，即 `simple_pi[j] = old[inv[j]]`；逆置换构造使用 `usize::MAX` 哨兵检测越界或重复像。详见[[定位器的相对姿态变换]]。^[locator.md:75-78]

手算测试覆盖置换合成 `[2,1,0] ∘ [1,2,0]^{-1} = [0,2,1]`，以及 `w` 右乘逆的例子：`s0·s1^{-1}` 的简约词为 `[0,1]`。测试也覆盖不同 `int_sys` 的拒绝路径。^[locator.md:92-93]

## 测试锚点与行为边界

B2 余根和闭包回归记录：根加法会错误地仅产生 4 个长根，余根加法则得到全部 8 个根。F4 半积分案例（case107、HPC3832609）检查 `w.image` 作用于 item 全部正根所得的集合是否等于独立过滤的期望集合。^[locator.md:80-85]

A2 测试保留了一个关键行为：两个切片参数驻留不同的 A1 item，典范数据依赖参数所在 alcove，而非仅依赖其整根系；这与设计简报草图的差异已记录在注释中。另有测试覆盖 Weyl 共轭参数共享同一 item、零参数驻留全系统且姿态为恒等，以及 B2 长、短 A1 驻留不同 item、重复查询复用。^[locator.md:86-90]

其他测试锚点包括 `root_vertex_of_alcove` 的跨模块案例 `[2,2]`、`[1,1]`、`[0,0]`，以及错秩参数触发 `RankMismatch`。整体测试仅覆盖 A2、B2、F4，可约根系、rank-0 边角和多条错误路径尚未测试。^[locator.md:91-94, locator.md:105-105]

## 使用限制与证据范围

`IntegralDatumTable` 不持有 `RootSystem`，调用方每次调用时重新传入；跨调用换用不同根系不会产生错误信号，但结果无定义。`factor_dominant` 没有迭代上限，其终止性依赖根系理论而非代码防护；`additive_closure` 也没有迭代或规模预算。^[locator.md:52-54, locator.md:99-101]

多处 `zip` 截断隐含长度一致假设；分母非零且不为 `i64::MIN` 依赖 `RationalWeight` 的构造不变量。像互异检查使用 `debug_assert_eq!`，仅在 debug 构建中生效。^[locator.md:102-104]

来源属于结构性阅读，不构成 locator 层的数学或正确性验收。上游文件行号仅转录自代码注释，未核对上游字节；读取身份记录于 `snapshots/2026-10-06-locator.json`，绑定 Git base、文件字节 SHA-256 与 Kimi 调用记录。来源所述知识维护未执行 Atlas、Cargo、测试或 benchmark，因此上述测试锚点不代表本次执行结果。^[locator.md:9-12, locator.md:98-98, locator.md:109-113]

## Sources

- [locator.md](../../sources/locator.md) — 典范整数据驻留与 Weyl 姿态定位器（locator.rs）。
