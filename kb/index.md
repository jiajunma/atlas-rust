# Atlas 知识索引

沿着“数学概念 → 算法 → Rust 实现与设计 → 演进及验证证据”阅读。C++ 作为 baseline，在兼容性对齐时进入比较。这里列出的页面是有明确来源范围的解释，不是整体兼容性声明。

| 主题 | 阅读目的 |
| --- | --- |
| [根坐标与格坐标](wiki/math/root-coordinates.md) | 区分简单根坐标、环境格坐标、根与余根的编号 |
| [Ladder bottom 的成员判定](wiki/algorithms/ladder-bottom-membership.md) | 成员查询溢出语义的限定修复已被 AFTER-v3 接受（ledger entry 0003，`accepted + math_pass`），范围以 entry limitations 为准 |
| [Ladder 的 C++ 与 Rust 实现比较](wiki/comparisons/root-ladder-cpp-rust.md) | 已按 AFTER-v3 限定接受重读；历史差异已闭合，不能外推到更高 rank 或其他操作 |
| [Rust 系统结构与兼容性边界](wiki/systems/atlas-implementation-map.md) | 理解 Rust 模块职责，区分早期设计与实际代码 |
| [Weyl 对象身份、dual 历史与安全共享边界](sources/weyl-context-identity-and-sharing.md) | 原版 A1 差异与 BEFORE-v4 tests-first 证据已保留；AFTER-v1 gate 已冻结待 HPC 提交；生成页 `needs_refresh`，尚无修复或缓存验收 |
| [KGB 图的结构与构造](sources/kgb-graph-structure.md) | 每个弱实形式一张图的数据布局、门控、分窗两相 BFS 与上游一致的编号；结构性阅读，不作数学验收 |
| [KLV 多项式的存储与逐列计算](sources/kl-polynomial-table.md) | `KlPol` 布局与最小运算集、去重池、按列存储与两条递归填充路径；结构性阅读，不作数学验收 |
| [部分公共块：Bruhat 区间上的块构造](sources/partial-common-block.md) | `StandardReprMod`、`CommonContext` 的 srm 层面操作、`bruhat_below`、`PartialBlock` 构造/访问器与 `dual()` 限制；结构性阅读，不作数学验收 |
| [完整块图：实形式与对偶实形式的纤维积](sources/block-graph.md) | 纤维积构造、`BlockDescent` 八值序、布局与访问器、`dual()` 与 Bruhat Hasse 图；结构性阅读，不作数学验收 |
| [形变驱动：twisted 与 block 形变](sources/deformation-drivers.md) | 移植简化契约、`SplitInteger`、积分子系统分类、父块抽象、两个 twisted KL 和与递归形变；结构性阅读，不作数学验收 |
| [表示参数上下文：StandardRepr 与 RepContext](sources/rep-context.md) | 参数四元组、借用视图、构造入口、lambda 派生链、奇偶/朝向/reducibility 与 finals；结构性阅读，不作数学验收 |

## 写作与来源

- [使用说明](README.md)、[维护规则](AGENTS.md)、[页面约定](schema.md)
- [来源索引](sources/index.md)、[变更日志](log.md)
- [主题模板](templates/topic.md)、[设计决策模板](templates/decision.md)

后续根据实际开发补充 Weyl 对象身份与缓存、KGB、KLV、变形等主题。该列表只是知识编写方向，不代表已覆盖、已实现或获准执行新的数学 gate。
