# 来源索引

原始来源保持在项目原位置。KB 保存解释和小型来源清单，不复制源码树、完整 HPC 输出或验收账本。

## 本次阅读

[初始来源快照](snapshots/2026-10-01-initial.json)记录四篇起始主题实际使用的本地文件及其哈希。它包含工作区版本，不能只凭 Git base 当成已提交源码，也没有对历史 HPC 原始流重新执行校验。

[Root ladder 固定宽度坐标溢出修复](root-ladder-overflow-repair.md)是首个
llm-wiki-compiler 原生来源包；其
[候选后继快照](snapshots/2026-10-01-root-ladder-repair-candidate-v2.json)绑定候选
Rust 字节、tests-first fixture、BEFORE-v3 证据和冻结 original 源码。
[AFTER-v3 接受快照](snapshots/2026-10-03-root-ladder-after-v3.json)把证据窗口推进到
限定接受：job 3875239 独立接受，acceptance index entry
`0003-a1-torus-root-coroot-ladder-boundary` 为 `accepted + math_pass`，范围以该
entry 的 limitations 为准。历史 candidate snapshots 保留不改写。

[Weyl 对象身份、dual 历史与安全共享边界](weyl-context-identity-and-sharing.md)
记录 original 的 weak root-datum interning、datum-local lazy WeylGroup、
history-dependent `dual()` identity，以及当前 Rust 每次重建 context 和结构关系
路径的差异。对应
[源码推断快照](snapshots/2026-10-01-weyl-context-source-prediction.json)
绑定当前 Rust 字节、两个 core-only A1 fixture、已接受的 rank-one profile 和冻结
original 源码。旧预测快照保持原样；后继
[原版回归快照](snapshots/2026-10-02-weyl-core-regressions.json)记录 v8 已证实的
两处 A1 差异和新原版 goldens；[AFTER-v1 gate 冻结快照](snapshots/2026-10-03-weyl-core-after-gate-freeze.json)
绑定已提交的 after 三件套、修复补丁与离线核验的 repaired manifest。AFTER-v1
尚未提交 HPC（SecureLink 隧道中断），语义修复与 cache A/B 均未验收。生成页仍待
compiler 授权及 hold-all 审查，不能把来源包更新视作已批准的生成内容。

## 权威记录的位置

| 记录 | 用途 |
| --- | --- |
| [COMPATIBILITY](../../docs/COMPATIBILITY.md) | 可观察行为的兼容边界和 oracle 版本限制 |
| [LANGUAGE](../../docs/LANGUAGE.md) | 语言表面、上游来源区域及历史证据范围 |
| [DESIGN](../../docs/DESIGN.md) | 设计背景；当前结构还需核对源码 |
| [HANDOFF](../../docs/HANDOFF.md) | 当前衔接、已发现问题和历史审查记录 |
| [REMAINING_BUILTINS](../../docs/REMAINING_BUILTINS.md) | 尚待工作、限制和后续计划 |
| [数学验收账本](../../tests/reference/hpc/math_acceptance_index_2026_10_01.json) | claim 的验收与结果分类；不能由 KB 自行改写 |

后续读论文时，在这里增加论文版本、章节和已有文献管理条目；尚未读取的论文不写成支持现有命题的依据。
