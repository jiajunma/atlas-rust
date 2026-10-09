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
history-dependent `dual()` identity，以及 Rust 侧的对应落地实现（内容+偏好
弱驻留、cold-share、左侧坐标重放）。对应
[源码推断快照](snapshots/2026-10-01-weyl-context-source-prediction.json)
绑定修复前 Rust 字节、两个 core-only A1 fixture、已接受的 rank-one profile 和冻结
original 源码。旧预测快照保持原样；后继
[原版回归快照](snapshots/2026-10-02-weyl-core-regressions.json)记录 v8 已证实的
两处 A1 差异和新原版 goldens；[AFTER-v1 gate 冻结快照](snapshots/2026-10-03-weyl-core-after-gate-freeze.json)
绑定已提交的 after 三件套、修复补丁与离线核验的 repaired manifest；
[落地快照](snapshots/2026-10-09-weyl-owner-dual-landed.json)绑定 AFTER-v5
验收（job3900050，A1 限定）与生产提交 `690c2b92` 的字节，以及 pin `7e1b958c`
的上游行级确认（转置 canonical dual 不可由 SC/adjoint 直接构造、指针相等、
guard 先于 no-value gate、左 owner 乘积、W_elt 强持有、inner-class eager dual）。
语义修复已在 A1 限定范围验收；cache A/B 未验收，G2 capture 备好待隧道恢复。
生成页仍待 compiler 授权及 hold-all 审查，不能把来源包更新视作已批准的生成内容。

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
"nice and real"/"endgame" 情形）；重读补充四条复核备注（coefficient 过期
文档、Default 空池隐患、quotient 恒 Ok、不检查算术）与 4 个测试锚点。
对应阅读快照 [2026-10-03](snapshots/2026-10-03-kl-polynomial-table.json)（初读）
与 [2026-10-06](snapshots/2026-10-06-lattice-kl-polynomial.json)（重读，字节未变，
仅 `kl_polynomial.rs`）；两份草案均由同一 Kimi probe 路由起草、维护者对照
源码逐条核对合并改写。该包是结构性阅读，不声称 KLV 计算的数学验收。

[部分公共块：Bruhat 区间上的块构造](partial-common-block.md)记录
`partial_block.rs` 的五个构件：`StandardReprMod`、`IntegralSubsystem`、
`CommonContext` 的五个 srm 层面操作、`bruhat_below`、`PartialBlock` 的构造与
访问器语义，以及 `dual()` 的纯数据变换与部分块限制。对应[阅读快照](snapshots/2026-10-03-partial-common-block.json)；
草案由同一 Kimi probe 路由起草（exit 0，232.8s），维护者对照源码逐条核对
改写。该包是结构性阅读，不声称 partial block 的数学验收。

[完整块图：实形式与对偶实形式的纤维积](block-graph.md)记录 `block.rs`：
两个 KGB 图经 `dual_involution` 配对的纤维积构造、`BlockDescent` 八值序与
dual 映射、平铺布局、访问器的 `UndefBlock`/`None` 语义、`dual()` 纯数据变换
与 Bruhat Hasse 图；重读补充 i1/i2 的 Cayley 槽共享与 fall-through、20 个
`BlockInvariantViolation` 字面量与 7 个秩1测试锚点。对应阅读快照
[2026-10-03](snapshots/2026-10-03-block-graph.json)（初读）与
[2026-10-06](snapshots/2026-10-06-block.json)（重读，字节未变）；
两份草案均由同一 Kimi probe 路由起草、维护者对照源码逐条核对合并改写。
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
`ParabolicPieces` 的 piece 索引；重读补充等值语义张力、compose/apply 检查
层级、enumerate 的 CompactWeyl+rayon 管线、死代码观察与 7 个测试锚点。
对应阅读快照 [2026-10-03](snapshots/2026-10-03-weyl-layer.json)（初读）与
[2026-10-06](snapshots/2026-10-06-weyl-root-involution.json)（重读，字节未变，
仅 `weyl.rs`）；两份草案均由同一 Kimi probe 路由起草、维护者对照源码逐条
核对合并改写。该包是结构性阅读，不声称 Weyl 层的数学验收。

[Compact Weyl 群的 transducer 表示](weyl-transducer.md)记录
`weyl_transducer.rs`：parabolic-subquotient 表示（`[u8; WEYL_MAX_RANK]`）、
`coxeter_entry` 查表、`Transducer` 构造、`CompactWeyl::new` 三步流程与
`canonical_word` 的 piece 拼接。对应[阅读快照](snapshots/2026-10-03-weyl-transducer.json)；
草案由同一 Kimi probe 路由起草（exit 0，78.1s，420 秒期限），维护者对照
源码逐条核对改写。该包是结构性阅读，不声称 transducer 的数学验收。

[Twisted involution 表（KGB stage b）](involution-table.md)记录
`involution_table.rs`：记录格式（含 image-basis 对的播种/传送）、编号纪律、
`new`/`add_cartan` 构建与 `lookup`/`cross`/`cayley`/`simple_root_kind`
访问器；重读补充外序 BFS 细节（stepped_length、普通生成元传送）、
push_record 的奇偶不变量与边数学对账、种子插入静默覆盖观察与 7 个测试
锚点。对应阅读快照 [2026-10-03](snapshots/2026-10-03-involution-table.json)（初读）
与 [2026-10-06](snapshots/2026-10-06-weak-real-form-involution-table.json)（重读，
字节未变）；两份草案均由同一 Kimi probe 路由起草、维护者对照源码逐条核对
合并改写。该包是结构性阅读，不声称 involution 表的数学验收。

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

[弱实形式划分：adjoint fiber 的 W_im 轨道](weak-real-form.md)记录
`weak_real_form.rs`：`WeakRealFormId` 的编号约定（与上游 RealFormNbr 一致）、
`WeakRealFormPartition` 的构建与查询、`weak_real_form_at_representative` 的
代表元级归因与 provenance 门控；重读补充 walk_mask_orbits 的转移规则与
不饱和比较、seeded_class 哨兵守卫、九道闸门顺序（整性门先于虚 grading）
与 11 个测试锚点。对应阅读快照
[2026-10-03](snapshots/2026-10-03-weak-real-form.json)（初读）与
[2026-10-06](snapshots/2026-10-06-weak-real-form-involution-table.json)（重读，
字节未变）；两份草案均由同一 Kimi probe 路由起草、维护者对照源码逐条核对
合并改写。该包是结构性阅读，不声称弱实形式层的数学验收。

[精确整数格线性代数：预算、饱和核与可观测基](integer-lattice.md)记录
`integer_lattice.rs`：`IntegerLatticeBudget` 的预算分层、`IntegerMatrix`、
`saturated_kernel`、`reduce_basis_mod_two`、`negative_coweight_eigenspace`、
关系格封装与 `adapted_basis` 的可观测选举。对应[阅读快照](snapshots/2026-10-03-integer-lattice.json)；
草案由同一 Kimi probe 路由起草（exit 0，161.7s，420 秒期限），维护者对照
源码逐条核对改写。该包是结构性阅读，不声称格层的数学验收。

[紧致 grading：simple-imaginary 根的紧致性位向量](grading.md)记录
`grading.rs`：`Grading` 的位向量纪律（与 ambient coweight 坐标的类型区分）、
`CartanGradingData` 的两道门控与全一 base、`grading`/`element_from_grading`
的增广消元互转；重读补充逐虚根收集流程、faithful 门（断言改拒绝）与 9 个
测试锚点。对应阅读快照 [2026-10-03](snapshots/2026-10-03-grading.json)（初读）
与 [2026-10-06](snapshots/2026-10-06-grading-mod-two.json)（重读，字节未变）；
两份草案均由同一 Kimi probe 路由起草、维护者对照源码逐条核对合并改写。
该包是结构性阅读，不声称 grading 层的数学验收。

[权格类型层：Weight、Coweight 与有理权](lattice-types.md)记录 `lattice.rs`：
`Weight`/`Coweight` 的 newtype 纪律（同表示不可互换）、`pair` 配对、
`RationalWeight` 的公共分母与 gcd 归一化、`RationalCoweight` 的逐坐标表示；
重读补充 apply_matrix/halve/integral_coordinates/scale/dot_coroot 细节、
预算纪律与四条复核备注（dot_coroot 字段反序、actual 填报怪癖、不可达
防御分支、gcd_u64 跨文件漂移）。对应阅读快照
[2026-10-03](snapshots/2026-10-03-lattice-types.json)（初读）与
[2026-10-06](snapshots/2026-10-06-lattice-kl-polynomial.json)（重读，字节未变）；
两份草案均由同一 Kimi probe 路由起草、维护者对照源码逐条核对合并改写。
该包是结构性阅读，不声称格类型层的数学验收。

[per-involution (1-θ)X* 图像基对](real-projection.md)记录
`real_projection.rs`：`lift_mat`/`M_real` 基对、播种/传送纪律、
`transported` 的两个矩阵方向与分解不变式、`coordinates`/`lift` 接口；重读
补充 gcd_sweep 的符号纪律（E6 involution-187 注释）、幺模整数逆与 4 个
测试锚点、transported 的形状检查缺口等阅读观察。对应阅读快照
[2026-10-03](snapshots/2026-10-03-real-projection.json)（初读）与
[2026-10-06](snapshots/2026-10-06-real-projection-matreduc.json)（重读，字节未变）；
两份草案均由同一 Kimi probe 路由起草、维护者对照源码逐条核对合并改写。
该包是结构性阅读，不声称投影层的数学验收。

[精确整数矩阵约化：matreduc 的逐操作移植](matreduc.md)记录 `matreduc.rs`：
逐操作保真的动机（被选解下游可观测）、`IntMatrix`、`diagonalise` 的符号簿记、
`has_solution`/`find_solution`、`in_left/right_image`、
`inverse_upper_triangular` 与 `exp_i`；重读补充 divide/gcd 细节、row_minus
覆盖赋值怪癖的逐行追踪、panic 面与 oracle_reference_cases 锚点。对应阅读
快照 [2026-10-03](snapshots/2026-10-03-matreduc.json)（初读）与
[2026-10-06](snapshots/2026-10-06-real-projection-matreduc.json)（重读，字节未变）；
两份草案均由同一 Kimi probe 路由起草、维护者对照源码逐条核对合并改写。
该包是结构性阅读，不声称 matreduc 移植的数学验收。

[mod-2 线性代数：位打包向量与子空间](mod-two.md)记录 `mod_two.rs`：
`ModTwoVector` 位打包（from_ones 重复下标抵消）、`ModTwoSubspace` 的低主元
RREF 与 right_kernel/pivot_rows、`CanonicalModTwoSection` 的 64 列掩码与依赖
列丢弃（2¹² 穷举 oracle）、crate 私有 `ModTwoSubquotient` 与诱导映射双侧校验。
对应阅读快照 [2026-10-03](snapshots/2026-10-03-mod-two.json)（初读）与
[2026-10-06](snapshots/2026-10-06-grading-mod-two.json)（重读，字节未变）；
初读由维护者直接读源补齐，重读草案由同一 Kimi probe 路由起草、维护者对照
源码逐条核对合并改写。该包是结构性阅读，不声称 mod-2 层的数学验收。

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

[伴随 Cartan 纤维](adjoint-fiber.md)记录 `adjoint_fiber.rs`：Arc 出处绑定
模型（ptr_eq + 坐标相等，跨投影互拒）、九步构建链（root_basis_action +
转置余权作用）、map_coweight/apply_mod_two 两种投影（可含中心核）、三条
预算线的精确公式与「每次调用独立计费」观察、9 个测试锚点（含逐坐标基
交织关系）。对应[阅读快照](snapshots/2026-10-06-adjoint-fiber.json)；草案由
同一 Kimi probe 路由起草（800s 期限，exit 0，312.4s），维护者对照源码
逐条核对改写。该包是结构性阅读，不声称伴随纤维层的数学验收。

[crate 根与 A1 原型层](lib-root.md)记录 `lib.rs`：60 模块组织与 52 条
再导出（含 topology 位置怪点、integer_lattice 两条、deform 双重暴露、
matreduc/real_projection/root_reflection/global_tits/weyl_size 零导出）、
错误类型汇聚点，以及 pub(crate) 原型层（RootDatum 的校验顺序、4096 根
闭包上限、65536 Weyl 阶上限、CartanInvolution 的 Grading 取代声明）。
对应[阅读快照](snapshots/2026-10-06-lib-root.json)；草案由同一 Kimi probe
路由起草（600s 期限，exit 0，590.5s），维护者对照源码逐条核对改写。该包
是结构性阅读，不声称 crate 门面的数学验收。至此 `atlas-real-group` 全部
60 个模块均有来源包或明确交叉引用。

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

[扭对合/对合分类/反射字三件套](twisted-involution-trio.md)记录
`twisted_involution.rs`、`involution_classification.rs` 与
`root_reflection.rs`：TwistedInvolution 的五重构造门槛与 compose_matrices
的 actual 填报怪癖、compact/complex/split 秩核与 fiber_rank 的
saturating/checked 策略并存、reflection_word 的贪心扫描与反转约定（无
迭代上限）。对应
[阅读快照](snapshots/2026-10-06-twisted-involution-trio.json)；草案由同一
Kimi probe 路由起草（300s 期限，exit 0，274.4s），维护者对照源码逐条
核对改写。该包是结构性阅读，不声称这三层的数学验收。

[K 型值与谓词/变形链](ktype.md)记录 `ktype.rs`：KType 的当选代表不变量与
sr_k 规范化、六个谓词（含 is_nonzero/is_normal 的不检查前提）、
equivalent、四个变形循环的终止预算、finals_for 的分支结构（type-2
Cayley 移位项、parity-real 投影分裂）与 kgp_set 的位图限界 BFS。对应
[阅读快照](snapshots/2026-10-06-ktype.json)；草案由同一 Kimi probe 路由
起草（1100s 期限，exit 0，349.5s），维护者对照源码逐条核对改写。本包在
**未变的字节**上取代 2026-10-03 的初读（旧快照
[2026-10-03-ktype.json](snapshots/2026-10-03-ktype.json) 保留）：初读覆盖
不变量/谓词/规范化链，本次补充 finals_for 分支结构、kgp_set、测试锚点与
错误普查。该包是结构性阅读，不声称 K 型层的数学验收。

[atlas-core crate 根与语言模块地图](atlas-core-root.md)记录 `lib.rs`
（26 行）与全部顶层/子目录模块角色：15 个 `pub mod` + `pub(crate)`
`matreduc` + `cfg(test)` 的 `session_fixture_tests` + 唯一常量
`COMPATIBILITY_VERSION="atlas-language-v0"`；逐模块的行数与角色取自本快照
字节与文件头自述（实现方陈述，非已核验行为）。对应
[阅读快照](snapshots/2026-10-09-atlas-core-root.json)（git base
`964f0033`，工作区干净，25 个文件逐字节哈希）。本包由维护者直接撰写
（无 Kimi 调用——模块地图小，不值得 probe 往返）。这是 `atlas-core`
语言层的第一包：`typed.rs`（18519 行）、`domain_builtins.rs`（22771
行）、`session.rs`（3889 行）、`syntax.rs`（4659 行）等大文件的内部
实现仍待各自分包。结构性阅读，不声称语言或数学验收。

[会话外层循环与 SessionEvent 面](atlas-core-session.md)记录
`session.rs`（3889 行，生产面 1–178 行 + 201 个测试的回归库）：
`SessionEvent` 六变体（含 `OutputBytes`/`ReportBytes` 字节保留面与
`output()` 的 UTF-8 分流）、逐命令外层循环（Newline 为命令边界、
Directive 由会话层拒绝而归属 `session_frame`）、`next_session_token` 的
**消费时刻**补全记录、`execute_tokens` 的前缀保留与 `SetType` span 用
真实词法终止符重建、以及求值出错先 `drain_failed_printed` 再诊断的顺序
（ext_kl.cpp:947）。对应
[阅读快照](snapshots/2026-10-09-atlas-core-session.json)；维护者直接
撰写（生产面小，无 Kimi 调用）。回归库以 `include_str!` fixture +
`.oracle.*` 逐字节比对为范式（Weyl A1 goldens 为例）。结构性阅读，
不声称语言或数学验收。

[会话帧：文件包含、输出重定向与顶层输出面](atlas-core-session-frame.md)
记录 `session_frame.rs`（993 行）：FileProvider/FileSink 边界（sink 在
解析成功后、求值前打开；语法错误不留文件，求值失败留部分输出）、
include-once/`<<` 强制/循环静默跳过/64 层上限、`clean` 只在语法类型求值
错误时置位（Io 诊断与 abandon 不弄脏）、`Value:` 打印与 void 抑制、按
包含深度缩进的报告、重定向体先按表达式解析（parser.y:180-181）、
abandon 级联最内层先且经 `line_map` 报物理行、`preprocess` 先剥尾空白
再续行。18 个测试锚点含字节串三件套（冻结原版字节比对）。对应
[阅读快照](snapshots/2026-10-09-atlas-core-session-frame.json)；维护者
直接撰写（无 Kimi 调用）。结构性阅读，不声称语言或数学验收。

[有状态词法器](atlas-core-lex.md)记录 `lex.rs`（992 行）：`TokenKind`
12 变体（`OperatorBecomes` 跨空白/注释融合 `+:=`、命令首 token 才识别
`<`/`<<`/`>`/`>>` 指令）、35 关键字与 20 个按 `Prim::ALL` 位置保留的
原始类型、换行抑制状态机（Group/Let/Block 嵌套栈 + prevent/previous
终止符：`.` 后的算符不再吞换行）、指令名前跳过空白**和注释**且裸名
不含斜杠、嵌套 `{}` 注释、双写引号转义、未闭合串报 warning 且恢复出的
token 进入 pending（REPL 行为）、`TokenCursor` 缓存错误的前瞻、
`tokenize_with_diagnostics` 的恢复式扫描。23 个测试。对应
[阅读快照](snapshots/2026-10-09-atlas-core-lex.json)；维护者直接撰写
（无 Kimi 调用）。结构性阅读，不声称语言或数学验收。

[语法前端](atlas-core-syntax.md)记录 `syntax.rs`（4659 行）与
`grammar.lalrpop`（1096 行）：`Expr`/`Command`/`Pattern`/`LambdaParam`/
`TypeExpr` 的 AST 面（含 `BarList` 不经用户重载的独立脱糖、
`OperatorCast` 的自由变量、`TypeAbstraction` 离开体即消失、`IntCase` 的
负值/越界/取模三态、重复 `break` 按 levels+1 解层）、四条解析入口
（`parse_command_fragment_in` 在解析器持有的类型规格未闭合时返回
`Ok(None)`：物理换行不终止它，完整前不求值）、`TokenStream` 适配与
Bison 措辞诊断（"syntax error, unexpected …"）、`type_scope.rs` 的
解析器持有词法类型作用域。51 个测试。对应
[阅读快照](snapshots/2026-10-09-atlas-core-syntax.json)；维护者直接撰写
（无 Kimi 调用）。结构性阅读，不声称语言或数学验收。

[类型模型](atlas-core-types.md)记录 `types.rs` + `types/`（961+1106+262+78
行）：`Type` 面（`Undetermined` 只被 `specialise` 收窄、`Variable` 的刚性
由外围 scheme 的 fixed 阈值决定、长度 1 元组/联合由构造器折叠、
`Tabled`/`Applied` 的名义递归比较）、`TypeTable`（bindings+active+
constructors+`revision: Arc<()>` 非语义快照身份；`matching_bindings` 查
**全部保留定义**；`validate_applications` 不展开地校验）、
`polymorphic.rs` 二阶机器（`TypeScheme` 首次出现序打包、`TypeAssignment`
的 `append` 连待决替换一起导入、`InferredType` 体+赋值对）、
`recursive.rs` 图级递归 typedef 安装。59 个测试。对应
[阅读快照](snapshots/2026-10-09-atlas-core-types.json)；维护者直接撰写
（无 Kimi 调用）。结构性阅读，不声称语言或数学验收。

[值层](atlas-core-value-layer.md)记录 `value.rs`+`linear_values.rs`+
`formula.rs`（359+403+243 行）：`Value` 面（`Union` 的 injector 打印、
`Closure` 的 SlotShape/recursive/frame/span/param_names 载荷、不透明的
`BuiltinFunction`、字节保留 `AtlasString`——Display 永不是字节权威）、
`Rational` 符号与分子分离且分母恒打印、vec/mat/ratvec 载荷逐字节对应上游
（列主序矩阵、`[ ]`/`" ]"` 尾、ratvec 构造时 gcd 规范化）、算符优先级栈
的**奇偶结合律**（偶左奇右）。11 个测试。对应
[阅读快照](snapshots/2026-10-09-atlas-core-value-layer.json)；维护者直接
撰写（无 Kimi 调用）。结构性阅读，不声称语言或数学验收。

[支撑层](atlas-core-support-layer.md)记录 `diagnostic.rs`+`source.rs`+
`coercions.rs`（179+115+426 行）：`ErrorKind` 分类与措辞分离、
`Diagnostic` 的 `raw_message` 字节权威（非 UTF-8 时）、`warning` 报告
不弄脏会话、`back_trace` 最外层在前且拷入系统变量；`SourceText` 预存
行首、Unicode 标量列；29 条上游顺序强转表（首中即返，`mat` 上下文先遇
`[vec]->mat`）、`row_coercion` 分量类型、`is_close` 三比特（等值先于边界
检查）、`broader_eq` 平衡序（函数要求参数相等）。对应
[阅读快照](snapshots/2026-10-09-atlas-core-support-layer.json)；维护者
直接撰写（无 Kimi 调用）。结构性阅读，不声称语言或数学验收。

[类型化管线核心数据结构](atlas-core-typed-core.md)记录 `typed.rs` 上部
（59–2757 行）：控制/级别枚举、`TypedExpr` 可执行树（多项式订阅**先接收
方后键**、`HungryBuiltinCall` 的顶层饥饿操作数即目的地时立即移出旧值、
`ComponentTransform` 的范围检查在合成读时触发）、`Analysis`（`std::ptr::eq`
守护的未移位签名视图缓存、活 `return_type`、`loop_depth`、`type_floor`）、
`IdTable`/`TypeCell` 的定义处下限纪律、`OverloadState`（revision Arc 守护
的合并视图、事务克隆不带缓存、`add_user` 重放上游单表 add 并保留歧义错误
措辞）、`TypedCommandEvent`（Value 携带类型供会话层取 void 标志）、
`TypedContext` 字段面与启动播种。对应
[阅读快照](snapshots/2026-10-09-atlas-core-typed-core.json)；维护者直接撰写
（无 Kimi 调用）。`convert_expr`、内建注册表、`TypedExpr` impl 与测试各待
分包。结构性阅读，不声称语言或数学验收。

[领域值与 Weyl 身份](atlas-core-domain-values.md)记录 `domain_builtins.rs`
上部（**Weyl 修复的落点**）：`WeylIdentityCell` 只在成功时落定、
`DatumWeylIdentity`（惰性坐标 kernel + 抽象群身份，无所有权环）、
`share_group_into_if_cold`（冷 canonical dual 才共享，预热目标永不覆盖）、
`DATUM_WEYL_IDENTITIES` 按完整内容弱 interning；`RootDatumHandle` 结构相等
刻意忽略身份缓存；`WeylEltContext` 的**抽象群 Arc 身份即兼容**规则与
跨坐标重放比较；`SplitValue` 的回绕对偶算术与 `(e±|f|s)` 打印；
`DomainValue` 结构等值（`same_real_form` vs `same_real_form_owner` 指针）；
多项式系数契约（只替换 final 键、丢弃也校验、先拒外来属主）；派发入口与
三个饥饿乘积。对应
[阅读快照](snapshots/2026-10-09-atlas-core-domain-values.json)；维护者直接
撰写（无 Kimi 调用）。本区域语义由 A1 限定 HPC 门验收（after-v5）；
结构性阅读本身不授予验收。

[领域派发与强转](atlas-core-domain-dispatch.md)记录 `domain_builtins.rs`
中部：`call`/`call_owned_with_printed`/`call_with_printed` 的 call 路径
（三个同结果类型饥饿乘积走 `hungry_product_owned`；打印侧通道只有
`partial_extended_KL_block` 用）、166 臂 `match name` 的组织与逐臂
"arity→提取→调用→包装"契约及校验顺序注释（如无值门之前先跑全部构造
检查）、`coerce` 的逐标签领域转换（`KpolK` 经 `finals_for`、`PolP` 经
`expand_final`，canonical 项序）、`build_real_form` 的父级规范弱缓存。
对应[阅读快照](snapshots/2026-10-09-atlas-core-domain-dispatch.json)；
维护者直接撰写（无 Kimi 调用）。166 臂的逐臂数学不在本包。结构性阅读，
不声称语言或数学验收。

[反射子群轨道与 ambient Weyl 见证](atlas-core-weyl-subgroup.md)记录
`domain_builtins/weyl_subgroup.rs`（433 行，after-v3 教训涉事文件、R3 反例
修复落点）：头部纪律（BitMap `basic_orbit` 序而非 alcove 分层；初始
dominance **真用**给定生成元——原版 rootdata.h 转发时忽略 g）、构造校验
（收窄再查根号、i128 配对、原版逐字 Cartan 错误文本）、**精确单（余）根
配对坐标**的陪集树（正缩放使符号/相等/边不变；有限 Cartan 可逆使限制
单射）、见证序（权右到左、余权左到右）、BuildAndDrop。4 个测试锚点
（空子群恒等、独立闭包等值 + 见证重建、全群序、丢弃前原版诊断、安全
溢出）。对应[阅读快照](snapshots/2026-10-09-atlas-core-weyl-subgroup.json)；
维护者直接撰写（无 Kimi 调用）。结构性阅读，不声称数学验收。

[转换遍 convert_expr](atlas-core-convert-expr.md)记录 `typed.rs` 中部
（2757–5973 行）：in/out 类型模式（只经 `specialise` 变异）、
`convert_expr_context` 的 type_floor 调整（类型抽象内的 return 操作数按
要求的实际下限解读）、12 个 `#[inline(never)]` 表达式族的机械划分
（3839541 栈陷阱教训：分区保帧小、不改语义）、while 转换（循环层在整棵
do 树外、bool 检查用上游措辞、WhileMode 由所需上下文定、row 回退
`row_coercion`）、赋值助手群（共享简单赋值路径、分量赋值下标门控、
**保留的类型定义**做投影解析、永不应用丢参数优化）。对应
[阅读快照](snapshots/2026-10-09-atlas-core-convert-expr.json)；维护者直接
撰写（无 Kimi 调用）。内建注册表、TypedExpr impl 与 133 个测试各待分包。
结构性阅读，不声称语言或数学验收。

[内建注册表](atlas-core-builtin-registry.md)记录 `typed.rs` 5973–11565：
`Builtin` 面（hunger 字节与 `HungryBuiltinCall` 配对、overload_visible）、
`BuiltinImpl` 变体（`Domain` 的 `DomainNoValue` Skip/Validate/BuildAndDrop
无值门策略——补全名清单≠无值策略清单；`DomainPrinter` 两个级别都写
报告；`Completions` 按前缀过滤快照；四个变参数泛型 print/prints/
to_string/error 的逐字边界）、求值辅助的上游契约（收窄逐字诊断、向量
地板除 `[7]\-3==[-2]`、`nth_set_bit` 补码走、`flex_*` 修剪规则、
`convolve`、ratvec LCD）、`builtin_registry()` 的 321 条目/170 名字
（相对上游追踪清单不全，缺口由 REMAINING_BUILTINS 跟踪）。对应
[阅读快照](snapshots/2026-10-09-atlas-core-builtin-registry.json)；维护者
直接撰写（无 Kimi 调用）。逐条目语义不在本包。结构性阅读，不声称语言或
数学验收。

[TypedExpr 求值](atlas-core-typed-eval.md)记录 `typed.rs` 11565–13741：
`evaluate` 的六族划分（与 convert_expr 同形、全 `#[inline(never)]`）、
迭代借用纪律（矩阵迭代不再造列矩阵、多项式保 canonical 项序与属主）、
调用机器（变参内建的元组解开、参数按**一个值**分发——多参数拆元组、
无参/全匿名不压帧、递归闭包 0 号槽自绑且新帧**不入**捕获链保 Rc 无环、
`return` 解到调用边界、带名槽的运行时错误附帧转储迹行）、回溯渲染
（`at NAME:LINE:COL-COL`，跨行双破折号）。对应
[阅读快照](snapshots/2026-10-09-atlas-core-typed-eval.json)；维护者直接撰写
（无 Kimi 调用）。133 个测试待分包。结构性阅读，不声称语言或数学验收。

[领域构造管线](atlas-core-domain-construction.md)记录 `domain_builtins.rs`
1628–2527（Weyl 弧中心区）：`build_datum` 的两路基（单连通：根=Cartan 行、
余根=基；伴随：根=基、余根=Cartan **列**）与 T1 因子追加序、商/显式 datum、
`build_inner_class_context` 的固定装配顺序与预算门（**对偶侧只建一次**：
dual_inner_class + 对偶分类 + 对偶弱实形计数 + Cartan 对应）、
`build_inner_class`（转置 + from_root_involution，上游再左合成为
distinguished）、`build_dual_inner_class`（余根偏好翻转 + 逐字母对偶
Lie 类型——即 dual-identity 路径）、`build_real_form` 规范弱缓存、
`build_custom_real_form`（新表 + 基本 Cartan + 自定义种子 + 两个形变缓存）。
对应[阅读快照](snapshots/2026-10-09-atlas-core-domain-construction.json)；
维护者直接撰写（无 Kimi 调用）。结构性阅读，不声称数学验收。

[形变缓存机器](atlas-core-deformation-cache.md)记录 `domain_builtins.rs`
2657–3022（ordinary-full-deform 弧落点）：`FullDeformKey` 的 canonical 单元、
缓存纪律（锁不跨递归、只缓存完整 canonical 排序结果、`active` 集显式检环、
协作截止分段检查）、普通全形变递推 `F(z)=L(z)+Σc_t(1-s)F(t)`（因
`(1-s)²=2(1-s)` 等于原版整数递推）、scale-zero 基底保留全部 final 项、逐子项
scale→readjust→lookup→common-terms 的 c(1-s) 递归、扭曲版的
**后 setup 计时**与 finalise 翻转的 s/1 系数。对应
[阅读快照](snapshots/2026-10-09-atlas-core-deformation-cache.json)；维护者直接
撰写（无 Kimi 调用）。结构性阅读，不声称数学验收。

[校验与打印](atlas-core-domain-validate-print.md)记录 `domain_builtins.rs`
9730–12263：`validate` 的 46 臂逐臂无值门顺序契约（BuildAndDrop 的
Weyl_orbit、integrality 先查维数、W_refl 收窄可观察在先、KGB_elt 全管线
再丢弃、KL_block 先 test_standard）、块打印机（`common_block_rows`
**每次新建**打印一致且按 `(x, gamma-lambda)` 匹配 init——单按 x 在 R 包内
有歧义；`located_common_block_rows` 参与共享查找；`partial_block_rows` 用
**调用方** gamma 的 survives 标志）、`print_text` 与各打印机
（print_KGB 的同形式选择、print_gradings 的 `sigma.pull_back` 位约定、
print_real_Weyl 臂内先查防静默翻译、`involution_expression` 的
`^`/`x`/`e` 渲染）。对应
[阅读快照](snapshots/2026-10-09-atlas-core-domain-validate-print.json)；
维护者直接撰写（无 Kimi 调用）。结构性阅读，不声称数学验收。

[根编号与 alcove 机器](atlas-core-root-numbering-alcove.md)记录
`domain_builtins.rs` 5005–5924：`RootNumbering` 的 RootNbr 序（正根按
`(level, root_compare)`——末坐标向前比；负根 `npos-1-p`；
`signed=nbr-npos`）与 alcove 机器（`wall_set` 只留余根不能被减去的根、
`root_components` 分量按**最大** RootNbr 序——FPP 乘积向量可观察、
`labels_for_component` 的唯一原始正关系、`sorted_by_label` 降序+RootNbr
并列、`from_fundamental_alcove` 留一单位墙+`to_positive_system`、
秩+分量的墙数只以大小可观察、精确 Cartan 逆）。对应
[阅读快照](snapshots/2026-10-09-atlas-core-root-numbering-alcove.json)；
维护者直接撰写（无 Kimi 调用）。结构性阅读，不声称数学验收。

[中心分类器与轨道词](atlas-core-center-classifier.md)记录
`domain_builtins.rs` 6102–7992：`CenterClassifier` 按根格陪集 tabulate 基本权
子集和（adjugate/行列式表示，与上游 `C_denom` 一致；`shifts` 的借位算术）、
adjoint 轨道 BFS（尾部递减、完成层反转递增）、`basic_orbit_adjoint` 与
`vertex_orbit`（label>1 的模变体）、`convert_to_words` 经陪集树**左乘**展开、
反射词约定（沿首个下降降到单根再逆序回溯；词作用**末字母先**）。对应
[阅读快照](snapshots/2026-10-09-atlas-core-center-classifier.json)；维护者直接
撰写（无 Kimi 调用）。结构性阅读，不声称数学验收。

[块图 SCC 与根表](atlas-core-domain-scc-root-table.md)记录
`domain_builtins.rs` 3022–5005 + 7992–9606：`strong_components` 的迭代
Tarjan 形 SCC（块图消费者）、`ByLastCoordinate` 的**坐标逆序**字典序
（同 root_compare）、矩阵辅助（`integer_matrix_product` 用 i128 宽累积、
只喂等值测试）、对合构造器的包装侧校验（`mat` 保留 0xN 维；行数为期望
秩）、`RootTable::build`（prefer_coroots 时先转置再换回、express 到环境
格基、长度旗）。对应
[阅读快照](snapshots/2026-10-09-atlas-core-domain-scc-root-table.json)；
维护者直接撰写（无 Kimi 调用）。结构性阅读，不声称数学验收。

[CLI 前端](atlas-cli-main.md)记录 `atlas-cli/src/main.rs`（175 行，全部）：
会话帧驱动的 CLI（`FsProvider` 有损 UTF-8、`FsSink`、字节保留的
`print_events`、rayon 2MiB 工作栈压 RSS、`--path=` 解析、文件参数按
**普通命令流**喂——上游文件参数的 prelude-capture 是 non-goal、交互横幅
与 `atlas> ` 循环、退出状态 = 上游 clean 标志：缺包含文件本身不弄脏）。
对应[阅读快照](snapshots/2026-10-09-atlas-cli-main.json)；维护者直接撰写
（无 Kimi 调用）。本文件无测试；行为由 HPC 语料门覆盖。结构性阅读。

[回归测试库地图](atlas-core-regression-library.md)记录四个测试模块的组织
（**不含**逐测试内容）：`session.rs` 201 个（家族地图 + `include_str!` fixture
+ `.oracle.*` 逐字节金标的原版背书范式）、`typed.rs` 133 个
（`convert*`/`overload*`/`evaluate*` 族）、`domain_builtins.rs` 92 个
（`as_*`/`build_*`/`block*`/`print*`/`weyl*`…）、`session_fixture_tests.rs`
17 个（经 `session::run_source` 的全程回归，不留第二实现）。证据链：
`tests/reference/hpc/` 分阶段证据 + append-only 验收账本 + HANDOFF。对应
[阅读快照](snapshots/2026-10-09-atlas-core-regression-library.json)；维护者
直接撰写（无 Kimi 调用）。本包不声称任何测试通过——可执行性以 HPC 门为准。
结构性阅读。

[领域层接缝](atlas-core-domain-seams.md)记录 `domain_builtins.rs` 既有各包
之间的三个区间：值提取器组（2538–2653，`as_matrix_rows` 的 0xN 空行表示与
显式 datum 臂消费链）、alcove/FPP 助手（5924–6085，Bareiss 无分数行列式、
余子式转置伴随、**两份并存**的 `root_vertex_simple` 移植及其三处差异）、
Weyl 词/生成元校验与值冻结（9606–9723，AFTER-v5 身份纪律的派发侧配套）。
对应[阅读快照](snapshots/2026-10-10-atlas-core-domain-seams.json)；维护者
直接撰写（无 Kimi 调用）。结构性阅读，不声称语言或数学验收。

[补全候选的会话级顺序索引](atlas-core-completions.md)记录
`frames/completions.rs`（112 行，全部）：intern 顺序即首次词法使用顺序、
`Rc<str>` 索引/条目共享分配、`OnceCell` 惰性快照只在 active 位真实变化时
失效、`replace` 遗留 API 保序保重复。对应
[阅读快照](snapshots/2026-10-10-atlas-core-completions.json)；维护者直接
撰写（无 Kimi 调用）。行为验收属 HPC completion AFTER 门（3868661）与
command AFTER R2（3868782），本包不重述。结构性阅读。

[组内名解析](atlas-core-type-groups.md)记录 `typed/type_groups.rs`（90 行，
全部）：递归图构造前的 BFS 验证（结果先于参数、应用先于其参数）、三类
Program 诊断、局部名永不接受显式参数、`forward_formals` 把组内裸自引用
改写为转发全部形参的 Applied。对应
[阅读快照](snapshots/2026-10-10-atlas-core-type-groups.json)；维护者直接
撰写（无 Kimi 调用）。结构性阅读，不声称语言或数学验收。

[数学测试与基准库结构](tests-math-library.md)记录 `tests/math/`、
`tests/fixtures/` 与 `tests/reference/` 的结构：顶层三 JSON 的 schema 与
群×操作清单（8 群 × 8 操作 + 拒绝模板 + 语言操作 + 13 grids）、31 个
模板、generics 探针库（273 用例）、progressive 秩 1 门、rank6 清单与
证据纪律（金标只来自原版完整流、清单≠正确率、append-only 账本种子）。
对应[阅读快照](snapshots/2026-10-10-tests-math-library.json)——取证时
三个文件带 owner 未提交修改、rank6 目录清单未跟踪，快照已如实标注；
维护者直接撰写（无 Kimi 调用）。结构性阅读，不声称数学验收。

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
