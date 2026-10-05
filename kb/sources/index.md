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

[Alcove 几何：alcove_center 与 root_vertex_of_alcove](alcove.md)记录
`alcove.rs`：墙方程与有理求解、-θ 不动子空间校验、分母界的 rank≥63 守卫、
RootNumbering 的正根排序/负根镜像编号、wall_set 分层、root_components 并查集、
root_vertex_simple 的转置子 Cartan 逆与 labels_1 重试。对应
[阅读快照](snapshots/2026-10-06-alcove.json)；草案由同一 Kimi probe 路由以完整
文件字节起草（exit 0，269.5s），维护者对照源码逐条核对改写。该包是结构性
阅读，不声称 alcove 计算的数学验收。

[内类字母解析与逐字母对合查表](primitive-involution.md)记录
`primitive_involution.rs`：`InnerClassLetterError` 的上游逐字节文案、
`checked_inner_class_letters` 的跳过分隔/字母坍缩规则（`'s'` 恰在 -1∈W 处
坍缩、`'u'` 仅存于偶秩 D）、`layout_involution` 的逐字母表与 Bourbaki 重编号、
`on_basis` 的精确除法换基（四类失败折叠为同一个 `None`）。对应
[阅读快照](snapshots/2026-10-06-primitive-involution.json)；草案由同一 Kimi
probe 路由起草（exit 0，253.1s），维护者对照源码逐条核对改写。该包是结构性
阅读，不声称字母表或对合构造的数学验收。

[对合类型三件套](involution-types.md)记录 `involution.rs`、
`twisted_involution.rs` 与 `root_involution.rs`：`LatticeInvolution` 的
方阵/对合/配对保持三道门控与 `anti_invariant_rank`；`RootInvolutionData` 的
根置换+余根运输验证、RootKind 分类优先级与继承正系的子系统单根；
`TwistedInvolution` 的 `w·θ` 重门控与 `compose_matrices`。对应
[阅读快照](snapshots/2026-10-06-involution-types.json)；草案由同一 Kimi probe
路由以三文件完整字节起草（exit 0，362.1s），维护者对照源码逐条核对改写。
该包是结构性阅读，不声称对合层的数学验收。

[Cartan fiber 与伴随 Cartan fiber](cartan-fibers.md)记录 `cartan_fiber.rs`
与 `adjoint_fiber.rs`：子商公式 `ker_F2(I+θ_Y) / red_2 ker_Z(I+θ_Y)` 的
先分母后分子构造、元素 provenance（`Arc::ptr_eq`）语义；伴随侧的预算分层
（`16·r²+r·n` 保留坐标、`2·n²·r` 下降操作）、`AdjointProjection` 绑定语义与
`FiberToAdjoint` 的按需三步映射。对应
[阅读快照](snapshots/2026-10-06-cartan-fibers.json)；草案由同一 Kimi probe
路由起草（exit 0，403.6s），维护者对照源码逐条核对改写。该包是结构性阅读，
不声称 fiber 层的数学验收。

[实 Weyl 群与块稳定子](real-weyl.md)记录 `real_weyl.rs`：`RealWeyl` 的
14 个根/类型列表（含 `real_r` 填对偶侧这一交叉）、`dual_side` 对偶 fiber 链
的临时重建（`tw*w0` 只是典范对偶代表元的共轭）、`fiber_side` 的紧基/正交
非紧根/R-群核、`simple_basis` 的外层终止怪癖、`simple_complex` 的对合成对
删除、打印层的冒号不一致与字节契约。对应
[阅读快照](snapshots/2026-10-06-real-weyl.json)；草案由同一 Kimi probe 路由
起草（68KB 单文件，exit 0，253.7s），维护者对照源码逐条核对改写。该包是
结构性阅读，不声称实 Weyl 层的数学验收。

[普通根系的确定性枚举](root-system.md)记录 `root_system.rs`：`RootId`/
`RootSet`/`RootSystemBudget`、BFS 闭包枚举与字典序存储、访问器语义
（`bracket` 根左余右、`id_of` 二分、预计算正负表）、梯子底表及其
「溢出即非成员」修复形态、25 个测试锚点。对应
[阅读快照](snapshots/2026-10-06-root-system.json)；草案由同一 Kimi probe 路由
起草（exit 0，342.7s），维护者对照源码逐条核对改写；草案独立发现注释
「十一边界用例」与实测八组的数量差异（已记录待核）。该包是结构性阅读，
不声称根系层的数学验收；溢出修复的演进见 root-ladder-overflow-repair 包。

[Dynkin 分类器](dynkin.md)记录 `dynkin.rs`：`classify` 的输入契约与
first-fresh-vertex 分量合并、秩二 B/C 由给定顺序决定（历史编号教训的落点）、
秩 >2 的度分析字母判定、各型起点选择（含 E 型长臂交换）、
`bourbaki_permutation` 与 `folded_cartan`（经 cofold 公式）。对应
[阅读快照](snapshots/2026-10-06-dynkin.json)；草案由同一 Kimi probe 路由起草
（exit 0，375.2s），维护者对照源码逐条核对改写。该包是结构性阅读，不声称
分类器的数学验收。

[Cayley/Cross 分解与整对合分类](cayley-cross.md)记录 `cayley_cross.rs` 与
`involution_classification.rs`：`CayleyCrossDecomposition::build` 的
provenance 门、peeling 循环（预算检查在 descent 发现之后、步进之前）、
逆序重放收集、长根化与重放验证；`classify_involution` 的预算先行顺序、
`classify_plus_identity` 的三个秩公式与 `fiber_rank` 的 saturating_sub。
对应[阅读快照](snapshots/2026-10-06-cayley-cross.json)；草案由同一 Kimi
probe 路由起草（exit 0，347.5s），维护者对照源码逐条核对改写。该包是
结构性阅读，不声称两条线的数学验收。

[典范整数据驻留与 Weyl 姿态定位器](locator.md)记录 `locator.rs`：
`IntegralDatumTable`/`IntegralDatumItem`/`BlockLocator` 三件套、`int_item`
的 (a)–(f) 流程（alcove 顶点、factor_dominant、墙面求值、词过滤、余根
加法闭包、驻留幂等、simple_pi 构造）、`make_relative_to` 的逆置换合成。
对应[阅读快照](snapshots/2026-10-06-locator.json)；草案由同一 Kimi probe
路由起草（exit 0，440.2s），维护者对照源码逐条核对改写。该包是结构性阅读；
整个模块尚未接线（`RepTable::lookup` 不调用它），不声称数学验收。

[弱实形式标签与外部编号](real-form-labels-order.md)记录
`real_form_labels.rs` 与 `real_form_order.rs`：`RealFormLabels::build` 的
出处闸门与 grading 关联机制（Cayley 回拉翻转、cross 运送、增广子空间求解、
quasisplit 锚点）、`base_grading_extension`；`ExternalFormOrder` 的
depth+tiebreak 排序（严格无并列断言、quasisplit 居末）、`DepthTables`、
`verified_generator_map`、`special_grading_key`。对应
[阅读快照](snapshots/2026-10-06-real-form-labels-order.json)；草案由同一
Kimi probe 路由起草（53KB 两文件，exit 0，487.8s），维护者对照源码逐条
核对改写；草案发现两处真实源码观察（DepthTables::build 的死循环、
weight_sum 的恒 Some Option），已记录为清理候选。该包是结构性阅读，不声称
标签/编号层的数学验收。

[逐块 KL 支撑数据](kl-support.md)记录 `kl_support.rs`：`RankFlags` 的
u32 位集（rank ≤ 32）、`validate_topology` 构造门控、下降/good-ascent 分类
（ImaginaryTypeII 两者皆不入）、length-stop 表、懒填充的本原索引机制及其
prepare-first 前置条件链。对应
[阅读快照](snapshots/2026-10-06-kl-support.json)；草案由同一 Kimi probe 路由
起草（exit 0，321.1s），维护者对照源码逐条核对改写。该包是结构性阅读，
不声称 KL 支撑层的数学验收。

[只读块拓扑与块修正子](block-access-modifier.md)记录 `block_access.rs` 与
`block_modifier.rs`：`BlockTopology` 的密封契约与两层 None 约定、
`bruhat_hasse`、`PartialBlock` 的参数交换与下降门控；`BlockModifier` 构造器
与 `RepContext` 扩展方法（transform_srm/shift_srm/
make_diff_integral_orthogonal/make_relative_to/sr_with_modifier）。对应
[阅读快照](snapshots/2026-10-06-block-access-modifier.json)；草案由同一 Kimi
probe 路由起草但在 480s 期限处截断（2.9 节内），已覆盖部分核对无误，尾部由
维护者按完整阅读补齐。该包是结构性阅读；两文件均未接线，不声称数学验收。

[合成实形的选定余特征与初始环面部分](minimal-torus.md)记录
`minimal_torus.rs`：`elected_square_root`（字重建往返校验、delta-then-w
运输、stable_log 调用）与 `minimal_torus_part`（入口门、初始环面部分、
TitsCoset 下降循环、基本纤维 grading 目标、轨道游走与最小选举）。对应
[阅读快照](snapshots/2026-10-06-minimal-torus.json)；草案由同一 Kimi probe
路由起草（540s 期限，exit 0，414.4s），维护者对照源码逐条核对改写。该包是
结构性阅读，不声称合成种子层的数学验收。

[内类布局与限制根系](layout-restricted-roots.md)记录 `layout.rs` 与
`restricted_roots.rs`：`InnerClassLayout::build`（twist 置换、Dynkin 分支
字母判定、Complex 对旋转与上游移位顺序的逐字复制、中心环面 Smith 商对合）、
`RestrictedWeight` 的 (1-θ) 编码与 `RestrictedRootSystem` 的纤维聚合。
对应[阅读快照](snapshots/2026-10-06-layout-restricted-roots.json)；草案由
同一 Kimi probe 路由起草（exit 0，182.9s——目前最快），维护者对照源码逐条
核对改写。该包是结构性阅读，不声称这两层的数学验收。

[Weyl 群阶识别与实形展示层](weyl-size-presentation.md)记录 `weyl_size.rs`
与 `presentation.rs`：`weyl_order_of_cartan` 的分量 BFS 与分支形状分派
（B/C 不敏感、F4 内双键识别、D/E 分支长度）；`build_presentations` 与
`RealFormPresentation` 的四个状态位。对应
[阅读快照](snapshots/2026-10-06-weyl-size-presentation.json)；草案由同一
Kimi probe 路由起草（exit 0，292.1s），维护者对照源码逐条核对改写。该包是
结构性阅读，不声称这两层的数学验收。

[对偶分量群平凡性与实形命名](topology-form-name.md)记录 `topology.rs` 与
`form_name.rs`：`dual_pi0` 子商、CorootRestriction、对合转运管线
（dual_component_group_trivial/rank 共享，仅末行不同——漂移风险已记录）；
`split`/`complex_name`/`factor_name` 规则表与 `form_type_name` 的拉回循环。
对应[阅读快照](snapshots/2026-10-06-topology-form-name.json)；草案由同一
Kimi probe 路由起草（exit 0，264.2s），维护者对照源码逐条核对改写。该包是
结构性阅读，不声称这两层的数学验收。

[StructureError 分类学与全局 Tits 传输](error-global-tits.md)记录
`error.rs` 与 `global_tits.rs`：53 个错误变体的字段形态家族（invariant /
resource-limit 两大带名家族及其例外）、`GlobalTitsElement` 的三重来源门槛、
`crossed_generator` 的 RootKind 三分支（虚根整性门槛
`InvalidStrongTorusFactor`、复根用余根方向反射、实根不动）与逐坐标
mod-2 规范化、`crossed_word` 的前向折叠顺序。对应
[阅读快照](snapshots/2026-10-06-error-global-tits.json)；草案由同一
Kimi probe 路由起草（exit 0，398.9s），维护者对照源码逐条核对改写。该包是
结构性阅读，不声称错误覆盖面或 Tits 传输的数学验收。

[内类范围 KGB 图与 print_X 布局](global-kgb.md)记录 `global_kgb.rs`：
GlobalTorusElement 的「构造约化、反射不再约化」纪律（负分子 `[0,-1]/2`
锚点）、x_pack 指纹的适应基投影、基本纤维与平方类播种、六阶段 build 与
全部 14 个 `KgbInvariantViolation` 字面量、print_X 逐字节版式。对应
[阅读快照](snapshots/2026-10-06-global-kgb.json)；草案由同一 Kimi probe
路由起草（800s 期限，exit 0，520.9s——13s/KB 规则成立），维护者对照源码
逐条核对改写。该包是结构性阅读，不声称全局 KGB 的数学验收。

[块图：两 KGB 的纤维积与对偶变换](block.md)记录 `block.rs`：BlockDescent
的 Value 序/对偶表/语言码重编号、`dual_involution`、纤维积构建（空对偶包
= 上游 `(0,0)` tauPacket 的隐式限制）、i1/i2 的 Cayley 槽共享与
fall-through、访问时的弱下降强制、`dual()` 纯数据变换（含「second 仅随
first 映射」规则）。对应[阅读快照](snapshots/2026-10-06-block.json)；草案由
同一 Kimi probe 路由起草（620s 期限，exit 0，242.7s），维护者对照源码逐条
核对改写。该包是结构性阅读，不声称块层的数学验收。

[扭对合/对合分类/反射字三件套](twisted-involution-trio.md)记录
`twisted_involution.rs`、`involution_classification.rs` 与
`root_reflection.rs`：TwistedInvolution 的五重构造门槛与 compose_matrices
的 actual 填报怪癖、compact/complex/split 秩核与 fiber_rank 的
saturating/checked 策略并存、reflection_word 的贪心扫描与反转约定（无
迭代上限）。对应
[阅读快照](snapshots/2026-10-06-twisted-involution-trio.json)；草案由同一
Kimi probe 路由起草（300s 期限，exit 0，274.4s），维护者对照源码逐条
核对改写。该包是结构性阅读，不声称这三层的数学验收。

[格值类型与 KLV 多项式引擎](lattice-kl-polynomial.md)记录 `lattice.rs` 与
`kl_polynomial.rs`：Weight/Coweight 的类型区分、RationalWeight 单公分母
纪律与 halve 不归一约定、KlPol 的 trim 不变量与递归操作集、KlHashTable
种子池（及 `Default` 空池隐患）。对应
[阅读快照](snapshots/2026-10-06-lattice-kl-polynomial.json)；草案由同一
Kimi probe 路由起草（r1 在 360s 超时留有草稿残片，r2 以 600s 期限
exit 0，326.1s；超时规则上修为 ~24s/KB），维护者对照源码逐条核对改写。
该包是结构性阅读，不声称这两层的数学验收。

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
