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
| [Cartan 分类：编号、预算与实形式归属](sources/cartan-classification.md) | CartanId 的 Atlas 顺序、预算分层、严格 Cayley 偏序与 real_form_of 的 complex-only 行走；结构性阅读，不作数学验收 |
| [Inner class 层：构造、验证门与 twisted 共轭枚举](sources/inner-class.md) | 部分实现边界、验证门、三阶段 canonicalize、canonical_involution_expr 与枚举族；结构性阅读，不作数学验收 |
| [扩展块：delta-不动部分与折叠生成元](sources/extended-block.md) | `DescValue` 32 值分类、`fold_orbits`、两种构造与 `tune_signs` 调试门；结构性阅读，不作数学验收 |
| [扩展 KLV 多项式表：primitivisation 符号与逐列存储](sources/extended-kl.md) | 池/符号分离存储、DescentTable、访问语义与 fill_columns 错误策略；结构性阅读，不作数学验收 |
| [共享块存储：reduced 键控复用与 RepTableOwner](sources/rep-table.md) | reduced 键、LocatedBlock、with_kl_table 并发约定与 K 型公式备忘；结构性阅读，不作数学验收 |
| [Weyl 群层：矩阵作用与词级元素的双层结构](sources/weyl-layer.md) | WeylAction/WeylElement 双层、互查桥、descent 读取方向、canonical_word 与 ParabolicPieces；结构性阅读，不作数学验收 |
| [Compact Weyl 群的 transducer 表示](sources/weyl-transducer.md) | parabolic-subquotient 表示、Transducer 表、canonical_word 与 piece 根置换；结构性阅读，不作数学验收 |
| [Twisted involution 表（KGB stage b）](sources/involution-table.md) | 记录格式、image-basis 播种/传送、编号纪律与 cross/Cayley 访问器；结构性阅读，不作数学验收 |
| [Tits 元素：torus 部分与 Tits 群操作（KGB stage c）](sources/tits-element.md) | 元素形状、TitsCoset 门控、cross/Cayley/inverse-Cayley 与 grading 修复；结构性阅读，不作数学验收 |
| [KGB 种子 x0：stable_log、基本余权与 RealFormSeed](sources/real-form-seed.md) | stable_log 前置条件、基本余权的实际余根展开、种子门控链；结构性阅读，不作数学验收 |
| [强实形式分类：平方类编号与 fiber 轨道](sources/strong-real.md) | 平方类编号约定、StrongRealData、分类汇总与打印视图；结构性阅读，不作数学验收 |
| [ext_param/star 层：扩展块的参数层](sources/ext-param.md) | ExtRepContext、ExtParam、star 计算与 finalisation 驱动；结构性阅读，不作数学验收 |
| [KType 层：标准表示的 K-限制](sources/ktype.md) | 表示不变量、判定族、规范化链与 finals/KGP 展开；结构性阅读，不作数学验收 |
| [弱实形式划分：adjoint fiber 的 W_im 轨道](sources/weak-real-form.md) | 编号约定、WeakRealFormPartition 与代表元级归因内核；结构性阅读，不作数学验收 |
| [精确整数格线性代数：预算、饱和核与可观测基](sources/integer-lattice.md) | 计算预算、饱和核、mod-2 归约与 adapted_basis 的可观测选举；结构性阅读，不作数学验收 |
| [紧致 grading：simple-imaginary 根的紧致性位向量](sources/grading.md) | 位向量纪律、CartanGradingData 门控与 grading↔元素互转；结构性阅读，不作数学验收 |
| [mod-2 线性代数：位打包向量与子空间](sources/mod-two.md) | 位打包、pivot 索引 RREF 与 crate 私有子商；结构性阅读，不作数学验收 |
| [权格类型层：Weight、Coweight 与有理权](sources/lattice-types.md) | newtype 纪律、pair 配对、公共分母归一化；结构性阅读，不作数学验收 |
| [per-involution (1-θ)X* 图像基对](sources/real-projection.md) | 播种/传送纪律、坐标/lift 接口与分解不变式；结构性阅读，不作数学验收 |
| [精确整数矩阵约化：matreduc 的逐操作移植](sources/matreduc.md) | 逐操作保真动机、diagonalise 与求解/像判定；结构性阅读，不作数学验收 |

| [伴随 Cartan 纤维：构建、投影与 mod-2 商（adjoint_fiber.rs）](sources/adjoint-fiber.md) | 伴随 Cartan 纤维：构建、投影与 mod-2 商；结构性阅读，不作数学验收 |
| [Alcove 几何：alcove_center 与 root_vertex_of_alcove](sources/alcove.md) | Alcove 几何：alcove_center 与 root_vertex_of_alcove；结构性阅读，不作数学验收 |
| [CLI 前端（atlas-cli/main.rs）——会话帧驱动、--path 解析与 clean 退出状态](sources/atlas-cli-main.md) | CLI 前端——会话帧驱动、--path 解析与 clean 退出状态；结构性阅读，不作数学验收 |
| [内建注册表（typed.rs 5973–11565）——Builtin/BuiltinImpl 面、无值门策略与启动清单](sources/atlas-core-builtin-registry.md) | 内建注册表——Builtin/BuiltinImpl 面、无值门策略与启动清单；结构性阅读，不作数学验收 |
| [中心分类器与轨道词（domain_builtins.rs 6102–7992）——CenterClassifier 的中心陪集 tabulation 与 adjoint 轨道 BFS/词转换](sources/atlas-core-center-classifier.md) | 中心分类器与轨道词——CenterClassifier 的中心陪集 tabulation 与 adjoint 轨道 BFS/词转换；结构性阅读，不作数学验收 |
| [转换遍 convert_expr（typed.rs 中部）——in/out 类型模式、族划分与赋值助手契约](sources/atlas-core-convert-expr.md) | 转换遍 convert_expr——in/out 类型模式、族划分与赋值助手契约；结构性阅读，不作数学验收 |
| [形变缓存机器（domain_builtins.rs 2657–2933）——full/twisted full deformation 的递归、缓存纪律与协作截止](sources/atlas-core-deformation-cache.md) | 形变缓存机器——full/twisted full deformation 的递归、缓存纪律与协作截止；结构性阅读，不作数学验收 |
| [领域构造管线（domain_builtins.rs 1628–2527）——datum/内类/实形的构造顺序与预算](sources/atlas-core-domain-construction.md) | 领域构造管线——datum/内类/实形的构造顺序与预算；结构性阅读，不作数学验收 |
| [领域派发与强转（domain_builtins.rs 中部）——call 路径、coerce 与 166 臂派发匹配](sources/atlas-core-domain-dispatch.md) | 领域派发与强转——call 路径、coerce 与 166 臂派发匹配；结构性阅读，不作数学验收 |
| [块图 SCC 与根表（domain_builtins.rs 3022–5005 + 7992–9606）——strong_components、ByLastCoordinate 序、对合校验与 RootTable](sources/atlas-core-domain-scc-root-table.md) | 块图 SCC 与根表——strong_components、ByLastCoordinate 序、对合校验与 RootTable；结构性阅读，不作数学验收 |
| [校验与打印（domain_builtins.rs 9730–12263）——46 臂 validate、块打印机与 print_text 面](sources/atlas-core-domain-validate-print.md) | 校验与打印——46 臂 validate、块打印机与 print_text 面；结构性阅读，不作数学验收 |
| [领域值与 Weyl 身份（domain_builtins.rs 上部）——DatumWeylIdentity、RootDatumHandle 与 DomainValue 面](sources/atlas-core-domain-values.md) | 领域值与 Weyl 身份——DatumWeylIdentity、RootDatumHandle 与 DomainValue 面；结构性阅读，不作数学验收 |
| [有状态词法器（lex.rs）——TokenKind 面、换行抑制状态机、指令与字符串/注释边界](sources/atlas-core-lex.md) | 有状态词法器——TokenKind 面、换行抑制状态机、指令与字符串/注释边界；结构性阅读，不作数学验收 |
| [回归测试库地图（session/typed/domain_builtins/session_fixture_tests 的测试模块）——家族组织与原版背书模式](sources/atlas-core-regression-library.md) | 回归测试库地图（session/typed/domain_builtins/session_fixture_tests 的测试模块）——家族组织与原版背书模式；结构性阅读，不作数学验收 |
| [领域层接缝（domain_builtins.rs 三处缝隙）——值提取器、alcove 助手与 Weyl 词/生成元校验](sources/atlas-core-domain-seams.md) | 领域层接缝——值提取器（0xN 空行表示）、两份并存 root_vertex_simple、Weyl 校验配套；结构性阅读，不作数学验收 |
| [补全候选的会话级顺序索引（frames/completions.rs）](sources/atlas-core-completions.md) | 补全候选顺序索引——intern 顺序、惰性快照与失效纪律；结构性阅读，不作数学验收 |
| [组内名解析（typed/type_groups.rs）——递归图构造前的验证与形参转发](sources/atlas-core-type-groups.md) | 组内名解析——BFS 验证顺序、局部名禁显式参数、裸自引用形参转发；结构性阅读，不作数学验收 |
| [根编号与 alcove 机器（domain_builtins.rs 5005–5924）——RootNumbering 的 RootNbr 序与 alcove 墙/标签/词](sources/atlas-core-root-numbering-alcove.md) | 根编号与 alcove 机器——RootNumbering 的 RootNbr 序与 alcove 墙/标签/词；结构性阅读，不作数学验收 |
| [crate 根：atlas-core 语言门面（lib.rs）——15 个公开模块、1 个 crate 私有矩阵约化与兼容契约版本](sources/atlas-core-root.md) | crate 根：atlas-core 语言门面——15 个公开模块、1 个 crate 私有矩阵约化与兼容契约版本；结构性阅读，不作数学验收 |
| [会话帧：文件包含、输出重定向与顶层输出面（session_frame.rs）](sources/atlas-core-session-frame.md) | 会话帧：文件包含、输出重定向与顶层输出面；结构性阅读，不作数学验收 |
| [会话外层循环与 SessionEvent 面（session.rs）——逐命令执行、字节保留输出与回归测试库](sources/atlas-core-session.md) | 会话外层循环与 SessionEvent 面——逐命令执行、字节保留输出与回归测试库；结构性阅读，不作数学验收 |
| [支撑层（diagnostic.rs + source.rs + coercions.rs）——结构化诊断、源位置与强转表/邻近谓词](sources/atlas-core-support-layer.md) | 支撑层——结构化诊断、源位置与强转表/邻近谓词；结构性阅读，不作数学验收 |
| [语法前端（syntax.rs + grammar.lalrpop）——AST 面、LALRPOP 适配与 Bison 风格诊断](sources/atlas-core-syntax.md) | 语法前端——AST 面、LALRPOP 适配与 Bison 风格诊断；结构性阅读，不作数学验收 |
| [类型化管线核心数据结构（typed.rs 上部）——TypedExpr 树、Analysis/OverloadState 与 TypedContext](sources/atlas-core-typed-core.md) | 类型化管线核心数据结构——TypedExpr 树、Analysis/OverloadState 与 TypedContext；结构性阅读，不作数学验收 |
| [TypedExpr 求值（typed.rs 11565–13741）——六族求值、调用机器与回溯渲染](sources/atlas-core-typed-eval.md) | TypedExpr 求值——六族求值、调用机器与回溯渲染；结构性阅读，不作数学验收 |
| [类型模型（types.rs + types/）——Type 面、TypeTable 与二阶机器](sources/atlas-core-types.md) | 类型模型——Type 面、TypeTable 与二阶机器；结构性阅读，不作数学验收 |
| [值层（value.rs + linear_values.rs + formula.rs）——Value 面、字节保留串、上游打印格式与算符优先级栈](sources/atlas-core-value-layer.md) | 值层——Value 面、字节保留串、上游打印格式与算符优先级栈；结构性阅读，不作数学验收 |
| [反射子群轨道与 ambient Weyl 见证（domain_builtins/weyl_subgroup.rs）](sources/atlas-core-weyl-subgroup.md) | 反射子群轨道与 ambient Weyl 见证；结构性阅读，不作数学验收 |
| [只读块拓扑与块修正子（block_access.rs / block_modifier.rs）](sources/block-access-modifier.md) | 只读块拓扑与块修正子；结构性阅读，不作数学验收 |
| [Cartan fiber 与伴随 Cartan fiber（cartan_fiber.rs / adjoint_fiber.rs）](sources/cartan-fibers.md) | Cartan fiber 与伴随 Cartan fiber；结构性阅读，不作数学验收 |
| [Cayley/Cross 分解与整对合分类（cayley_cross.rs / involution_classification.rs）](sources/cayley-cross.md) | Cayley/Cross 分解与整对合分类；结构性阅读，不作数学验收 |
| [Dynkin 分类器：连通分量、Bourbaki 置换与折叠 Cartan（dynkin.rs）](sources/dynkin.md) | Dynkin 分类器：连通分量、Bourbaki 置换与折叠 Cartan；结构性阅读，不作数学验收 |
| [StructureError 错误分类学与全局 Tits 交叉作用传输层（error.rs / global_tits.rs）](sources/error-global-tits.md) | StructureError 错误分类学与全局 Tits 交叉作用传输层；结构性阅读，不作数学验收 |
| [内类范围 KGB 图与 print_X 布局（global_kgb.rs）](sources/global-kgb.md) | 内类范围 KGB 图与 print_X 布局；结构性阅读，不作数学验收 |
| [对合类型三件套：LatticeInvolution / RootInvolutionData / TwistedInvolution](sources/involution-types.md) | 对合类型三件套：LatticeInvolution / RootInvolutionData / TwistedInvolution；结构性阅读，不作数学验收 |
| [逐块 KL 支撑数据：KlSupport 与 RankFlags（kl_support.rs）](sources/kl-support.md) | 逐块 KL 支撑数据：KlSupport 与 RankFlags；结构性阅读，不作数学验收 |
| [内类布局与限制根系（layout.rs / restricted_roots.rs）](sources/layout-restricted-roots.md) | 内类布局与限制根系；结构性阅读，不作数学验收 |
| [crate 根：60 模块组织、52 条再导出与 A1 原型层（lib.rs）](sources/lib-root.md) | crate 根：60 模块组织、52 条再导出与 A1 原型层；结构性阅读，不作数学验收 |
| [典范整数据驻留与 Weyl 姿态定位器（locator.rs）](sources/locator.md) | 典范整数据驻留与 Weyl 姿态定位器；结构性阅读，不作数学验收 |
| [合成实形的选定余特征与初始环面部分（minimal_torus.rs）](sources/minimal-torus.md) | 合成实形的选定余特征与初始环面部分；结构性阅读，不作数学验收 |
| [内类字母解析与逐字母对合查表（primitive_involution.rs）](sources/primitive-involution.md) | 内类字母解析与逐字母对合查表；结构性阅读，不作数学验收 |
| [弱实形式标签与外部编号（real_form_labels.rs / real_form_order.rs）](sources/real-form-labels-order.md) | 弱实形式标签与外部编号；结构性阅读，不作数学验收 |
| [实 Weyl 群与块稳定子：real_weyl.rs 的构造、对偶 fiber 重放与打印层](sources/real-weyl.md) | 实 Weyl 群与块稳定子：real_weyl.rs 的构造、对偶 fiber 重放与打印层；结构性阅读，不作数学验收 |
| [BasedRootDatum 与对偶内类构造（root_datum.rs / dual.rs）](sources/root-datum-dual.md) | BasedRootDatum 与对偶内类构造；结构性阅读，不作数学验收 |
| [Root ladder 固定宽度坐标溢出修复](sources/root-ladder-overflow-repair.md) | 溢出即非成员的限定修复与 tests-first 验证链（AFTER-v3 已验收，ledger entry 0003）；结构性阅读，不作数学验收 |
| [普通根系的确定性枚举：RootSystem、RootId 与梯子底表](sources/root-system.md) | 普通根系的确定性枚举：RootSystem、RootId 与梯子底表；结构性阅读，不作数学验收 |
| [对偶分量群平凡性与实形命名（topology.rs / form_name.rs）](sources/topology-form-name.md) | 对偶分量群平凡性与实形命名；结构性阅读，不作数学验收 |
| [扭对合、对合分类与环境根反射字（twisted_involution.rs / involution_classification.rs / root_reflection.rs）](sources/twisted-involution-trio.md) | 扭对合、对合分类与环境根反射字；结构性阅读，不作数学验收 |
| [Weyl 群阶识别与实形展示层（weyl_size.rs / presentation.rs）](sources/weyl-size-presentation.md) | Weyl 群阶识别与实形展示层；结构性阅读，不作数学验收 |

## 写作与来源

- [使用说明](README.md)、[维护规则](AGENTS.md)、[页面约定](schema.md)
- [来源索引](sources/index.md)、[变更日志](log.md)
- [主题模板](templates/topic.md)、[设计决策模板](templates/decision.md)

上表现已覆盖全部 76 个来源包（585 页概念 wiki 见 [MOC](wiki/MOC.md)，Fresh 无待审候选）。页面均为结构性阅读记录，不代表数学验收；验收以 HPC 门与 append-only 账本为准。
