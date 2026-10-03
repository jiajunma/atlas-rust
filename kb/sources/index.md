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

[KGB 图的结构与构造](kgb-graph-structure.md)记录 `kgb_graph.rs` 的数据布局、
build 门控、分窗两相 BFS（Rayon 纯计算 + 顺序 intern）、上游一致的排序键与
计数排序标准化、cross/Cayley/inverse-Cayley 链接语义和 hybrid self-contained
存储。对应[阅读快照](snapshots/2026-10-03-kgb-graph.json)；草案由 Kimi probe
起草、维护者对照源码逐条核对改写，调用记录见快照的 `kimi_assist`。该包是结构
性阅读，不声称 KGB 枚举的数学验收。

[KLV 多项式的存储与逐列计算](kl-polynomial-table.md)记录 `kl_polynomial.rs`
的 `KlPol` 布局（低次在前、无尾零、零为空向量）、恰好够递归与 μ-修正的运算
集、`KlHashTable` 去重池（`zero`/`one` 固定索引 0/1），以及 `kl_table.rs`
按列存储（primitive-index 位置索引的池索引列、非零 μ-对列、`holes`）与
`fill` 的两条递归分派（`recursion_column` 与 `new_recursion_column` 的
"nice and real"/"endgame" 情形）。对应[阅读快照](snapshots/2026-10-03-kl-polynomial-table.json)；
草案由同一 Kimi probe 路由起草（300 秒期限，exit 0，90.7s），维护者对照源码
逐条核对改写。该包是结构性阅读，不声称 KLV 计算的数学验收。

[部分公共块：Bruhat 区间上的块构造](partial-common-block.md)记录
`partial_block.rs` 的五个构件：`StandardReprMod`、`IntegralSubsystem`、
`CommonContext` 的五个 srm 层面操作、`bruhat_below`、`PartialBlock` 的构造与
访问器语义，以及 `dual()` 的纯数据变换与部分块限制。对应[阅读快照](snapshots/2026-10-03-partial-common-block.json)；
草案由同一 Kimi probe 路由起草（exit 0，232.8s），维护者对照源码逐条核对
改写。该包是结构性阅读，不声称 partial block 的数学验收。

[完整块图：实形式与对偶实形式的纤维积](block-graph.md)记录 `block.rs`：
两个 KGB 图经 `dual_involution` 配对的纤维积构造、`BlockDescent` 八值序与
dual 映射、平铺布局、访问器的 `UndefBlock`/`None` 语义、`dual()` 纯数据变换
与 Bruhat Hasse 图。对应[阅读快照](snapshots/2026-10-03-block-graph.json)；
草案由同一 Kimi probe 路由起草（exit 0，83.1s），维护者对照源码逐条核对改写。
该包是结构性阅读，不声称块枚举的数学验收。

[形变驱动：twisted 与 block 形变](deformation-drivers.md)记录 `deform.rs`：
四个移植驱动入口、冻结的 domain/deform 简化契约、`SplitInteger` 算术、
`IntegralBlockScope` 三变体（含 A1 陷阱）、奇异集与父块抽象、两个 twisted
KL 和的长度函数差异、`block_deformation_to_height` 与递归
`twisted_deformation`（去 memoisation、alcove_center 收缩、可取消变体）。
对应[阅读快照](snapshots/2026-10-03-deformation-drivers.json)；草案由同一
Kimi probe 路由起草（exit 0，168.5s），维护者对照源码逐条核对改写。该包是
结构性阅读，不声称形变计算的数学验收。

[表示参数上下文：StandardRepr 与 RepContext](rep-context.md)记录
`rep_context.rs`：参数四元组、借用视图与派生常量、构造入口、lambda 派生链、
挠部分打包/提升、奇偶与朝向、mod_reduce/build_srm、reducibility points、
finals 与 deformation terms。对应[阅读快照](snapshots/2026-10-03-rep-context.json)；
Kimi probe 草案在超时处截断，已采纳部分经核对、其余由维护者补齐。该包是
结构性阅读，不声称参数层的数学验收。

[Inner class 层：构造、验证门与 twisted 共轭枚举](inner-class.md)记录
`inner_class.rs`：有意为之的部分实现边界、构造入口、`based_involution_twist`
验证门、`twisted_from_involution` 成员判定、三阶段 `canonicalize`、
`canonical_involution_expr` 的 signed-entry 编码，以及 twisted 共轭枚举族
（含 generated partition 的预算差异）。对应[阅读快照](snapshots/2026-10-03-inner-class.json)；
草案由同一 Kimi probe 路由起草（exit 0，111.5s），维护者对照源码逐条核对
改写。该包是结构性阅读，不声称 inner class 的数学验收。

[扩展块：delta-不动部分与折叠生成元](extended-block.md)记录 `ext_block.rs`：
`DescValue` 32 值分类与其谓词、`ExtGen` 轨道折叠、`extended_type` 局部识别、
`ExtBlock::build` 与 `build_partial`（含 cofolded 生成元姿态）、`tune_signs`
与 debug_assertions 下的 `check_quadratic`/`check_braid`。对应[阅读快照](snapshots/2026-10-03-extended-block.json)；
草案由同一 Kimi probe 路由起草（exit 0，180.0s，420 秒期限），维护者对照
源码逐条核对改写。该包是结构性阅读，不声称扩展块的数学验收。

[扩展 KLV 多项式表：primitivisation 符号与逐列存储](extended-kl.md)记录
`ext_kl.rs`：池/符号分离存储（`prim_flip` bitmap）、`DescentTable` 预计算、
`ExtKlTable` 列式访问语义、`fill_columns` 的错误传播策略，以及模块文档载明的
五条 deliberate deviations。对应[阅读快照](snapshots/2026-10-03-extended-kl.json)；
草案由同一 Kimi probe 路由起草（exit 0，260.5s，420 秒期限），维护者对照
源码逐条核对改写。该包是结构性阅读，不声称扩展 KL 的数学验收。

[共享块存储：reduced 键控复用与 RepTableOwner](rep-table.md)记录
`rep_table.rs`：`ReducedParamKey` 的键控复用、`LocatedBlock` 访问器、
`with_kl_table` 的互斥与重入禁令、`RepTableOwner` 入口与 `k_type_formula`
的备忘语义（锁外计算、提交时再复核）。对应[阅读快照](snapshots/2026-10-03-rep-table.json)；
草案由同一 Kimi probe 路由起草（exit 0，112.6s，420 秒期限），维护者对照
源码逐条核对改写。该包是结构性阅读，不声称块存储的数学验收。

[Weyl 群层：矩阵作用与词级元素的双层结构](weyl-layer.md)记录 `weyl.rs` 与
`weyl_element.rs`：WeylAction/WeylElement 双层分工与互查桥、descent 读取
方向（左读逆向量、右读正向置换）、`canonical_word` 的不变量检查、
`WeylInterface` 的内部生成子重编号（A/E/F/G 直取、B/C/D 反转）与
`ParabolicPieces` 的 piece 索引。对应[阅读快照](snapshots/2026-10-03-weyl-layer.json)；
草案由同一 Kimi probe 路由起草（exit 0，151.4s，420 秒期限），维护者对照
源码逐条核对改写。该包是结构性阅读，不声称 Weyl 层的数学验收。

[Compact Weyl 群的 transducer 表示](weyl-transducer.md)记录
`weyl_transducer.rs`：parabolic-subquotient 表示（`[u8; WEYL_MAX_RANK]`）、
`coxeter_entry` 查表、`Transducer` 构造、`CompactWeyl::new` 三步流程与
`canonical_word` 的 piece 拼接。对应[阅读快照](snapshots/2026-10-03-weyl-transducer.json)；
草案由同一 Kimi probe 路由起草（exit 0，78.1s，420 秒期限），维护者对照
源码逐条核对改写。该包是结构性阅读，不声称 transducer 的数学验收。

[Cartan 分类：编号、预算与实形式归属](cartan-classification.md)记录
`cartan_classification.rs` 与 `cartan_class.rs`：`CartanId` 的 Atlas 顺序、
六字段预算分层、聚合的严格 Cayley 偏序、`real_form_of` 的 complex-only
canonicalize 行走与 EVEN-integer grading 规则、`TwistedConjugacyClass` 与
`CartanClass` 的分层。对应[阅读快照](snapshots/2026-10-03-cartan-classification.json)；
草案由同一 Kimi probe 路由起草（exit 0，210.1s），维护者对照源码逐条核对
改写。该包是结构性阅读，不声称 Cartan 分类的数学验收。

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
