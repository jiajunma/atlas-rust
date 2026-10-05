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
[Cartan 分类：编号、预算与实形式归属](cartan-classification.md)记录
`cartan_classification.rs` 与 `cartan_class.rs`：`CartanId` 的 Atlas 顺序、
六字段预算分层、聚合的严格 Cayley 偏序、`real_form_of` 的 complex-only
canonicalize 行走与 EVEN-integer grading 规则、`TwistedConjugacyClass` 与
`CartanClass` 的分层。对应[阅读快照](snapshots/2026-10-03-cartan-classification.json)；
草案由同一 Kimi probe 路由起草（exit 0，210.1s），维护者对照源码逐条核对
改写。该包是结构性阅读，不声称 Cartan 分类的数学验收。

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

[Twisted involution 表（KGB stage b）](involution-table.md)记录
`involution_table.rs`：记录格式（含 image-basis 对的播种/传送）、编号纪律、
`new`/`add_cartan` 构建与 `lookup`/`cross`/`cayley`/`simple_root_kind`
访问器。对应[阅读快照](snapshots/2026-10-03-involution-table.json)；草案由
同一 Kimi probe 路由起草（exit 0，115.5s，420 秒期限），维护者对照源码逐条
核对改写。该包是结构性阅读，不声称 involution 表的数学验收。

[Tits 元素：torus 部分与 Tits 群操作（KGB stage c）](tits-element.md)记录
`tits_element.rs`：`TitsElement` 二元组形状、`TitsCoset` 的 full-inner-class
门控、cross/Cayley/inverse-Cayley 语义（含 inverse-Cayley 的 mod-space
grading 修复）与 `reduce` 正规形。对应[阅读快照](snapshots/2026-10-03-tits-element.json)；
草案由同一 Kimi probe 路由起草（exit 0，116.8s，420 秒期限），维护者对照
源码逐条核对改写。该包是结构性阅读，不声称 Tits 层的数学验收。

[KGB 种子 x0：stable_log、基本余权与 RealFormSeed](real-form-seed.md)记录
`real_form_seed.rs`：`stable_log` 的选举与前置条件、`fundamental_coweights`
的实际余根展开（精确有理求逆）、`RealFormSeed::build` 的门控链与 `custom`
分支。对应[阅读快照](snapshots/2026-10-03-real-form-seed.json)；草案由同一
Kimi probe 路由起草（exit 0，158.0s，420 秒期限），维护者对照源码逐条核对
改写。该包是结构性阅读，不声称种子层的数学验收。

[强实形式分类：平方类编号与 fiber 轨道](strong-real.md)记录 `strong_real.rs`：
`SquareClassId` 的编号约定（与上游 low-pivot RREF 共享）、
`StrongRealFormRep`/`StrongRealData`、`StrongRealClassification::build` 的
平方商构造与 `fiber_size` 的 `Some(0)` 语义、`StrongRealClassPrint` 打印视图。
对应[阅读快照](snapshots/2026-10-03-strong-real.json)；草案由同一 Kimi probe
路由起草（exit 0，263.0s，420 秒期限），维护者对照源码逐条核对改写。该包是
结构性阅读，不声称强实层的数学验收。

[ext_param/star 层：扩展块的参数层](ext-param.md)记录 `ext_param.rs`：
`ExtRepContext`、`ExtParam` 值类型、比较/对齐辅助、`star` 计算与三个
finalisation 驱动、两个 `StarOracle` 实现。对应[阅读快照](snapshots/2026-10-03-ext-param.json)；
草案由同一 Kimi probe 路由起草（exit 0，188.4s，420 秒期限），维护者对照
源码逐条核对改写。该包是结构性阅读，不声称 ext_param/star 层的数学验收。

[KType 层：标准表示的 K-限制](ktype.md)记录 `ktype.rs`：KType 的表示不变量、
`sr_k` 归一化、is_standard/.../is_final 判定族、`equivalent` 与
made_dominant/made_theta_stable/to_canonical_fiber/normalised 链、
`finals_for`/`kgp_set` 展开。对应[阅读快照](snapshots/2026-10-03-ktype.json)；
草案由同一 Kimi probe 路由起草（exit 0，122.4s，420 秒期限），维护者对照
源码逐条核对改写。该包是结构性阅读，不声称 KType 层的数学验收。

[弱实形式划分：adjoint fiber 的 W_im 轨道](weak-real-form.md)记录
`weak_real_form.rs`：`WeakRealFormId` 的编号约定（与上游 RealFormNbr 一致）、
`WeakRealFormPartition` 的构建与查询、`weak_real_form_at_representative` 的
代表元级归因与 provenance 门控。对应[阅读快照](snapshots/2026-10-03-weak-real-form.json)；
草案由同一 Kimi probe 路由起草（exit 0，164.6s，420 秒期限），维护者对照
源码逐条核对改写。该包是结构性阅读，不声称弱实形式层的数学验收。

[精确整数格线性代数：预算、饱和核与可观测基](integer-lattice.md)记录
`integer_lattice.rs`：`IntegerLatticeBudget` 的预算分层、`IntegerMatrix`、
`saturated_kernel`、`reduce_basis_mod_two`、`negative_coweight_eigenspace`、
关系格封装与 `adapted_basis` 的可观测选举。对应[阅读快照](snapshots/2026-10-03-integer-lattice.json)；
草案由同一 Kimi probe 路由起草（exit 0，161.7s，420 秒期限），维护者对照
源码逐条核对改写。该包是结构性阅读，不声称格层的数学验收。

[紧致 grading：simple-imaginary 根的紧致性位向量](grading.md)记录
`grading.rs`：`Grading` 的位向量纪律（与 ambient coweight 坐标的类型区分）、
`CartanGradingData` 的两道门控与全一 base、`grading`/`element_from_grading`
的增广消元互转。对应[阅读快照](snapshots/2026-10-03-grading.json)；草案由
同一 Kimi probe 路由起草（exit 0，147.9s，420 秒期限），维护者对照源码逐条
核对改写。该包是结构性阅读，不声称 grading 层的数学验收。

[权格类型层：Weight、Coweight 与有理权](lattice-types.md)记录 `lattice.rs`：
`Weight`/`Coweight` 的 newtype 纪律（同表示不可互换）、`pair` 配对、
`RationalWeight` 的公共分母与 gcd 归一化、`RationalCoweight` 的逐坐标表示。
对应[阅读快照](snapshots/2026-10-03-lattice-types.json)；草案由同一 Kimi
probe 路由起草（exit 0，215.9s，420 秒期限），维护者对照源码逐条核对改写。
该包是结构性阅读，不声称格类型层的数学验收。

[per-involution (1-θ)X* 图像基对](real-projection.md)记录
`real_projection.rs`：`lift_mat`/`M_real` 基对、播种/传送纪律、
`transported` 的两个矩阵方向与分解不变式、`coordinates`/`lift` 接口。
对应[阅读快照](snapshots/2026-10-03-real-projection.json)；草案由同一 Kimi
probe 路由起草（exit 0，74.5s，420 秒期限），维护者对照源码逐条核对改写。
该包是结构性阅读，不声称投影层的数学验收。

[精确整数矩阵约化：matreduc 的逐操作移植](matreduc.md)记录 `matreduc.rs`：
逐操作保真的动机（被选解下游可观测）、`IntMatrix`、`diagonalise` 的符号簿记、
`has_solution`/`find_solution`、`in_left/right_image`、
`inverse_upper_triangular` 与 `exp_i`。对应[阅读快照](snapshots/2026-10-03-matreduc.json)；
草案由同一 Kimi probe 路由起草（exit 0，75.6s，420 秒期限），维护者对照源码
逐条核对改写。该包是结构性阅读，不声称 matreduc 移植的数学验收。

[mod-2 线性代数：位打包向量与子空间](mod-two.md)记录 `mod_two.rs`：
`ModTwoVector` 位打包、`ModTwoSubspace` 的 pivot 索引 RREF、crate 私有
`ModTwoSubquotient`。对应[阅读快照](snapshots/2026-10-03-mod-two.json)；
Kimi probe 草案因摘录漏掉未注释方法而偏薄，由维护者直接读源补齐。该包是
结构性阅读，不声称 mod-2 层的数学验收。

[BasedRootDatum 与对偶内类构造](root-datum-dual.md)记录 `root_datum.rs` 与
`dual.rs`：`BasedRootDatum` 的两秩区分与构造门控、radical/coradical 饱和核、
简单反射与可失败克隆；`dual_datum` 转置互换、`longest_action` 下坡行走、
`dual_inner_class` 的 `negative_transposed` 装配、`dual_cartan_correspondence`
的根像置换键（上游代表元是共轭而非矩阵相等）与 `dual_real_form_count` 管线。
对应[阅读快照](snapshots/2026-10-06-root-datum-dual.json)；草案由 Kimi probe
以两文件完整字节起草（exit 0，387.3s），维护者对照源码逐条核对改写——
完整字节输入避免了 mod-two 的摘录遗漏模式。该包是结构性阅读，不声称根数据
层或对偶构造的数学验收。

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
