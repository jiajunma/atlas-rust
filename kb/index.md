# Atlas 知识索引

沿着“数学概念 → 算法 → Rust 实现与设计 → 演进及验证证据”阅读。C++ 作为 baseline，在兼容性对齐时进入比较。这里列出的页面是有明确来源范围的解释，不是整体兼容性声明。

| 主题 | 阅读目的 |
| --- | --- |
| [根坐标与格坐标](wiki/math/root-coordinates.md) | 区分简单根坐标、环境格坐标、根与余根的编号 |
| [Ladder bottom 的成员判定](wiki/algorithms/ladder-bottom-membership.md) | `needs_refresh`；历史证据窗口下的计算目标、不变量及固定宽度减法边界 |
| [Ladder 的 C++ 与 Rust 实现比较](wiki/comparisons/root-ladder-cpp-rust.md) | `needs_refresh`；历史坐标选择、已发现问题和 original capture 的连接 |
| [Rust 系统结构与兼容性边界](wiki/systems/atlas-implementation-map.md) | 理解 Rust 模块职责，区分早期设计与实际代码 |
| [Weyl 对象身份、dual 历史与安全共享边界](sources/weyl-context-identity-and-sharing.md) | 原版 A1 差异与 BEFORE-v4 tests-first 证据已保留；AFTER-v1 gate 已冻结待 HPC 提交；生成页 `needs_refresh`，尚无修复或缓存验收 |

## 写作与来源

- [使用说明](README.md)、[维护规则](AGENTS.md)、[页面约定](schema.md)
- [来源索引](sources/index.md)、[变更日志](log.md)
- [主题模板](templates/topic.md)、[设计决策模板](templates/decision.md)

后续根据实际开发补充 Weyl 对象身份与缓存、KGB、KLV、变形等主题。该列表只是知识编写方向，不代表已覆盖、已实现或获准执行新的数学 gate。
