# 知识库变更日志

日志按发生顺序追加，记录知识变化和来源范围。它不替代 Git 历史或数学验收账本。

## 2026年10月1日 建立同仓库知识库

按用户要求在 `kb/` 建立可单独打开的 Obsidian vault，并将维护约定接入仓库入口。新增页面约定、主题和决策模板、来源索引，以及根坐标、ladder 成员判定、两版 ladder 比较和系统结构四篇起始页面。

来源是已有文档、历史 capture 和当前工作区源码，精确文件身份见 [初始快照](sources/snapshots/2026-10-01-initial.json)。四篇主题均保留 draft 编辑状态。本次不执行构建、测试、数学算例或新的 HPC 验证，也不改变既有验收结论。

## 2026年10月1日 明确 Rust 演进主线

用户补充：C++ 是 baseline，仅在对齐时重点比较，其他工作围绕 Rust 版本演进。相应调整入口、模板及维护规则。用户要求继续比较成熟的现成方案，现有页面保留为工具无关草稿，尚未接入或冻结具体引擎。

## 2026年10月1日 选定 llm-wiki-compiler

用户确定采用 llm-wiki-compiler。工具依赖固定为已发布的 1.4.0-rc.2，在 kb/ 本地安装并保留锁文件；增加仓库启动器、原生 hold-all 审阅策略，以及递归来源选择与索引/快照排除规则。更新根目录和本目录的 AGENTS.md：知识随 Rust 变更维护，C++ 仅在对齐时比较，生成内容须经来源与目标页审查，编译状态不能替代数学验收。

保留原有四篇笔记与历史来源快照，未进行自动迁移或模型生成。本次工具安装与文档配置不构成 Atlas 构建、测试、数学验证或编译器可靠性验收。

## 2026年10月1日 Root ladder 候选修复来源包

新增首个 compiler 原生来源包，记录 root/coroot ladder 成员查询中的固定宽度
坐标溢出、original 的 simple-coordinate/Weyl-transport 构造、Rust 候选的最小
错误边界以及 HPC BEFORE-v3 tests-first 证据。对应阅读快照绑定实际候选字节、
fixture、原版完整 stdout、BEFORE 记录和冻结 original 源文件。

该条目仍标为候选：尚未运行或接受 AFTER gate，也没有性能、内存或更高层数学
覆盖结论。即使 AFTER 通过，也还需要独立机器可读 review、注册的 claim
contract/validators 以及 checkpointed acceptance ledger 的
`accepted + math_pass` entry；之后才通过 hold-all review 更新相关主题页。

同日第一次执行
`./kb/llmwiki compile --review --instructions AGENTS.md`。compiler 识别了唯一
新来源，但 Codex provider 在生成候选前退出；随后 `review list` 确认没有 pending
candidate，也没有 state 或候选文件。沙箱外重试因可能向外部 provider 传输来源包
与 `kb/AGENTS.md`、而缺少对该精确 payload/destination 的明确授权而被拒绝。本次
没有模型产物可审阅或采用；不得更换 provider 或绕过边界。来源包仍是人工编写的
候选材料，而不是 llm-wiki-compiler 已生成页面。

## 2026年10月1日 Root ladder 页面进入待刷新状态

仓库中已存在时间上晚于当前来源包和快照的 root-ladder 记录，但本次不对其
数学、实现或性能含义作解释。为防止历史措辞被误读为当前状态，两篇 ladder
人工页面标为 `needs_refresh`，来源包、来源索引和入口索引增加同一编辑警示。

本次没有改写历史快照，没有运行 compiler、KB 校验器、Atlas 程序或 HPC 作业，
也没有新的数学验收、性能或内存结论。后续需要重新读取当前源码与证据、
追加新快照，再按 hold-all 流程审查对应的知识更新。

## 2026年10月1日 Weyl identity 与共享边界来源包

新增 `sources/weyl-context-identity-and-sharing.md` 及只追加阅读快照，直接绑定
冻结 original 的 weak root-datum interning、datum-local lazy WeylGroup、
history-dependent `dual()` identity，当前 Rust 的 per-call context rebuild 与
不可失败结构关系，以及两个尚未捕获的 core-only A1 fixture。来源包把
`Arc`/`Weak`/`OnceLock` 两层候选设计、强引用环风险、cross-coordinate replay、
`no_value` mismatch 和递进语义 gate 分开说明；所有内容均标为源码推断，不是
缓存实现、数学验收、速度或内存结论。

执行 `./kb/llmwiki compile --review --instructions AGENTS.md --concurrency 1
--verbose` 时，compiler 识别两个 new source，但配置的 Codex provider 在第一个
来源生成前退出。`review list` 为零候选，`status` 仍是 missing state、两个
pending source；`.llmwiki` 除原有 config 外没有文件。沙箱外重试因可能把精确
来源包和 `kb/AGENTS.md` 发送到外部 provider、缺少对该 payload/destination 的
单独授权而被拒绝；未绕过或更换 provider。故本次实际进展是来源包、阅读快照
和索引，不是 compiler 生成页。

## 2026年10月2日 原版确认 Weyl owner 差异并冻结回归

根据 v8 job3884807 的独立完整流审查，来源包从“源码预测”推进到“限定 A1
输入上的原版观察”：cold canonical dual 被 Rust 错误拒绝，prewarmed owner
关系被 Rust 错误求成布尔值。新增只追加阅读快照，绑定四个原版 goldens、
regression catalog、tests-only patch 和当前 Rust 字节。保留旧预测快照。

明确记录 BEFORE-v1 job3884862 是检查器自检失败，不是所需数学失败；没有
生产修复、缓存、性能、内存或秩释放。索引同步这一限制。没有重试此前被拒绝
的 provider 请求，没有生成或批准 compiler 页面，生成内容仍待刷新。

## 2026年10月3日 Weyl AFTER-v1 gate 冻结（未提交 HPC）

来源包推进到 AFTER-v1 gate 冻结状态：after stager/driver/checker 已随
`342a0511` 提交，修复仍以补丁数据形式传输，生产字节未提交。新增只追加阅读
快照 `2026-10-03-weyl-core-after-gate-freeze.json`，绑定三个提交、after 三件
套哈希、before-v4 证据、修复补丁与离线核验过的 repaired manifest。索引同步该
限制；生成页仍为 `needs_refresh`。未重试此前被拒绝的 provider 请求，没有生成
或批准 compiler 页面。SecureLink 隧道中断，HPC 提交暂缓；无任何数学、缓存、
性能、内存或 rank 验收。

## 2026年10月3日 Root-ladder 来源包与页面按 AFTER-v3 限定接受刷新

`root-ladder-overflow-repair.md` 的证据窗口推进到 AFTER-v3 限定接受：job
3875239 独立接受、acceptance index entry `0003-a1-torus-root-coroot-ladder-boundary`
（`accepted + math_pass`）已登记，原先的“尚未注册 claim contract”段落改写为
该 entry 的精确引用与限制。新增只追加阅读快照
`2026-10-03-root-ladder-after-v3.json`。两个 curated 页面
（ladder-bottom-membership、root-ladder-cpp-rust）重读后标记 `reviewed`；
`sources/index.md` 与 vault 索引同步。历史 candidate 快照全部保留。未运行任何
编译、测试或 compiler 生成。

## 2026年10月3日 两个 draft 页面重读后标记 reviewed

`root-coordinates` 与 `atlas-implementation-map` 按当前源码逐条核对。前者补入
`RootSystem` 新增的 `positive`/`simple_ids`/`min_roots`/`min_coroots` 字段角色，
`root_datum.rs` 字节未变、`root_system.rs` 用 AFTER-v3 接受版本；后者逐条核对
workspace 成员、`SessionEvent` 六变体、`Frame` 的 `Rc`/`RefCell`、`TypedContext`
字段，全部成立，并注明 `session.rs`/`typed.rs` 是 dirty 工作区字节。上游
ls-remote 复核：HEAD/master 仍为 `7e1b958c`，oracle pin 未漂移，收据存
`tests/reference/hpc/upstream_head_2026_10_03.json`。未运行编译、测试或
compiler 生成。

## 2026年10月3日 新增 KGB 图来源包（Kimi probe 协助）

新增 `sources/kgb-graph-structure.md` 与快照 `2026-10-03-kgb-graph.json`：
每个弱实形式一张 KGB 图的数据布局、build 门控、分窗两相 BFS、上游一致的
排序键与计数排序标准化、链接语义与 hybrid self-contained 存储。草案经本地
Kimi probe（无工具 profile，`kimi-code/k3-256k`）起草；进程产出完整草案后未在
180 秒内退出，被 runner SIGTERM 清理、无残留进程组成员；维护者对照
`kgb_graph.rs` 逐条核对并改写，未采用的内容已剔除。经验已记入根 AGENTS.md
（超时的 probe 仍可能已产出完整回答，先解析 stdout.jsonl）。索引与
sources/index.md 已同步；未运行编译、测试或 compiler 生成。

## 2026年10月3日 新增 KLV 多项式来源包（Kimi probe 协助）

新增 `sources/kl-polynomial-table.md` 与快照 `2026-10-03-kl-polynomial-table.json`：
`KlPol` 布局与最小运算集、去重池、按列存储与两条递归填充路径。同一 Kimi
probe 路由起草；300 秒期限内正常完成（exit 0，90.7s），证实第一次 180 秒
超时只是期限过短。维护者对照源码逐条核对改写。索引与 sources/index.md 已
同步；未运行编译、测试或 compiler 生成。

## 2026年10月3日 新增部分公共块来源包（Kimi probe 协助）

新增 `sources/partial-common-block.md` 与快照 `2026-10-03-partial-common-block.json`：
`StandardReprMod`、`IntegralSubsystem`、`CommonContext` 的 srm 层面操作、
`bruhat_below`、`PartialBlock` 构造/访问器与 `dual()` 限制。同一 Kimi probe
路由起草（exit 0，232.8s）；维护者对照源码逐条核对改写。索引与
sources/index.md 已同步；未运行编译、测试或 compiler 生成。

## 2026年10月3日 新增完整块图来源包（Kimi probe 协助）

新增 `sources/block-graph.md` 与快照 `2026-10-03-block-graph.json`：纤维积
构造、`dual_involution` 配对、`BlockDescent` 八值序、布局与访问器、`dual()`
与 Bruhat Hasse 图。同一 Kimi probe 路由起草（exit 0，83.1s）；维护者对照
源码逐条核对改写。索引与 sources/index.md 已同步；未运行编译、测试或
compiler 生成。

## 2026年10月3日 新增形变驱动来源包（Kimi probe 协助）

新增 `sources/deformation-drivers.md` 与快照 `2026-10-03-deformation-drivers.json`：
移植简化契约、`SplitInteger`、积分子系统分类、父块抽象、两个 twisted KL 和、
`block_deformation_to_height` 与递归 `twisted_deformation`。同一 Kimi probe
路由起草（exit 0，168.5s）；维护者对照源码逐条核对改写。索引与
sources/index.md 已同步；未运行编译、测试或 compiler 生成。

## 2026年10月3日 新增表示参数上下文来源包（Kimi probe 部分协助）

新增 `sources/rep-context.md` 与快照 `2026-10-03-rep-context.json`：
`StandardRepr` 四元组、`RepContext` 借用视图、构造入口、lambda 派生链、
挠部分打包/提升、奇偶/朝向/reducibility 与 finals。Kimi probe 草案在 300 秒
超时处截断（30KB 摘录超出该期限）；已采纳部分经维护者对照源码核对，其余由
维护者补齐。教训：大模块应拆分摘录或提高期限。索引与 sources/index.md 已
同步；未运行编译、测试或 compiler 生成。

## 2026年10月3日 新增 Cartan 分类来源包（Kimi probe 协助）

新增 `sources/cartan-classification.md` 与快照 `2026-10-03-cartan-classification.json`：
`CartanId` 编号、预算分层、严格 Cayley 偏序、`real_form_of`、
`TwistedConjugacyClass`/`CartanClass` 分层。同一 Kimi probe 路由起草
（exit 0，210.1s，420 秒期限）；维护者对照源码逐条核对改写。索引与
sources/index.md 已同步；未运行编译、测试或 compiler 生成。

## 2026年10月3日 新增 inner class 来源包（Kimi probe 协助）

新增 `sources/inner-class.md` 与快照 `2026-10-03-inner-class.json`：部分实现
边界、构造入口、验证门、三阶段 canonicalize、canonical_involution_expr 与
twisted 共轭枚举族。同一 Kimi probe 路由起草（exit 0，111.5s，420 秒期限）；
维护者对照源码逐条核对改写。索引与 sources/index.md 已同步；未运行编译、
测试或 compiler 生成。

## 2026年10月3日 新增扩展块来源包（Kimi probe 协助）

新增 `sources/extended-block.md` 与快照 `2026-10-03-extended-block.json`：
`DescValue` 32 值分类、`fold_orbits`、两种构造与 `tune_signs` 调试门。同一
Kimi probe 路由起草（exit 0，180.0s，420 秒期限）；维护者对照源码逐条核对
改写。索引与 sources/index.md 已同步；未运行编译、测试或 compiler 生成。

## 2026年10月3日 新增扩展 KLV 表来源包（Kimi probe 协助）

新增 `sources/extended-kl.md` 与快照 `2026-10-03-extended-kl.json`：池/符号
分离存储、`DescentTable`、`ExtKlTable` 访问语义、`fill_columns` 错误策略与
五条 deliberate deviations。同一 Kimi probe 路由起草（exit 0，260.5s）；
维护者对照源码逐条核对改写。索引与 sources/index.md 已同步；未运行编译、
测试或 compiler 生成。

## 2026年10月3日 新增共享块存储来源包（Kimi probe 协助）

新增 `sources/rep-table.md` 与快照 `2026-10-03-rep-table.json`：
`ReducedParamKey` 键控复用、`LocatedBlock`、`with_kl_table` 并发约定、
`RepTableOwner` 与 `k_type_formula` 备忘。同一 Kimi probe 路由起草
（exit 0，112.6s，420 秒期限）；维护者对照源码逐条核对改写。索引与
sources/index.md 已同步；未运行编译、测试或 compiler 生成。

## 2026年10月3日 新增 Weyl 群层来源包（Kimi probe 协助）

新增 `sources/weyl-layer.md` 与快照 `2026-10-03-weyl-layer.json`：
WeylAction/WeylElement 双层结构、互查桥、descent 读取方向、canonical_word、
WeylInterface 重编号与 ParabolicPieces。同一 Kimi probe 路由起草
（exit 0，151.4s，420 秒期限）；维护者对照源码逐条核对改写。索引与
sources/index.md 已同步；未运行编译、测试或 compiler 生成。

## 2026年10月3日 新增 compact Weyl transducer 来源包（Kimi probe 协助）

新增 `sources/weyl-transducer.md` 与快照 `2026-10-03-weyl-transducer.json`：
parabolic-subquotient 表示、`coxeter_entry`、Transducer 构造、`canonical_word`
与 piece 根置换。同一 Kimi probe 路由起草（exit 0，78.1s）；维护者对照源码
逐条核对改写。索引与 sources/index.md 已同步；未运行编译、测试或 compiler
生成。

## 2026年10月3日 新增 twisted involution 表来源包（Kimi probe 协助）

新增 `sources/involution-table.md` 与快照 `2026-10-03-involution-table.json`：
记录格式、image-basis 播种/传送、编号纪律与访问器。同一 Kimi probe 路由起草
（exit 0，115.5s）；维护者对照源码逐条核对改写。索引与 sources/index.md 已
同步；未运行编译、测试或 compiler 生成。

## 2026年10月3日 新增 Tits 元素来源包（Kimi probe 协助）

新增 `sources/tits-element.md` 与快照 `2026-10-03-tits-element.json`：
`TitsElement` 形状、`TitsCoset` 门控、cross/Cayley/inverse-Cayley 与
inverse-Cayley 的 grading 修复。同一 Kimi probe 路由起草（exit 0，116.8s）；
维护者对照源码逐条核对改写。索引与 sources/index.md 已同步；未运行编译、
测试或 compiler 生成。

## 2026年10月3日 新增 KGB 种子来源包（Kimi probe 协助）

新增 `sources/real-form-seed.md` 与快照 `2026-10-03-real-form-seed.json`：
`stable_log`、`fundamental_coweights`（实际余根展开）、`RealFormSeed` 门控链。
同一 Kimi probe 路由起草（exit 0，158.0s）；维护者对照源码逐条核对改写。
索引与 sources/index.md 已同步；未运行编译、测试或 compiler 生成。

## 2026年10月3日 新增强实形式分类来源包（Kimi probe 协助）

新增 `sources/strong-real.md` 与快照 `2026-10-03-strong-real.json`：
平方类编号约定、StrongRealData、分类汇总与打印视图。同一 Kimi probe 路由起草
（exit 0，263.0s）；维护者对照源码逐条核对改写。索引与 sources/index.md 已
同步；未运行编译、测试或 compiler 生成。

## 2026年10月3日 新增 ext_param/star 来源包（Kimi probe 协助）

新增 `sources/ext-param.md` 与快照 `2026-10-03-ext-param.json`：
`ExtRepContext`、`ExtParam`、`star` 与 finalisation 驱动。同一 Kimi probe
路由起草（exit 0，188.4s）；维护者对照源码逐条核对改写。索引与
sources/index.md 已同步；未运行编译、测试或 compiler 生成。

## 2026年10月3日 新增 KType 来源包（Kimi probe 协助）

新增 `sources/ktype.md` 与快照 `2026-10-03-ktype.json`：KType 表示不变量、
判定族、规范化链与 finals/KGP 展开。同一 Kimi probe 路由起草（exit 0，
122.4s）；维护者对照源码逐条核对改写。索引与 sources/index.md 已同步；未运行
编译、测试或 compiler 生成。

## 2026年10月3日 新增弱实形式划分来源包（Kimi probe 协助）

新增 `sources/weak-real-form.md` 与快照 `2026-10-03-weak-real-form.json`：
编号约定、WeakRealFormPartition、代表元级归因内核。同一 Kimi probe 路由起草
（exit 0，164.6s）；维护者对照源码逐条核对改写。索引与 sources/index.md 已
同步；未运行编译、测试或 compiler 生成。

## 2026年10月3日 新增精确整数格来源包（Kimi probe 协助）

新增 `sources/integer-lattice.md` 与快照 `2026-10-03-integer-lattice.json`：
预算分层、饱和核、mod-2 归约、关系格封装与 adapted_basis。同一 Kimi probe
路由起草（exit 0，161.7s）；维护者对照源码逐条核对改写。索引与
sources/index.md 已同步；未运行编译、测试或 compiler 生成。

## 2026年10月3日 新增紧致 grading 来源包（Kimi probe 协助）

新增 `sources/grading.md` 与快照 `2026-10-03-grading.json`：位向量纪律、
CartanGradingData 门控、grading↔元素互转。同一 Kimi probe 路由起草
（exit 0，147.9s）；维护者对照源码逐条核对改写。索引与 sources/index.md 已
同步；未运行编译、测试或 compiler 生成。

## 2026年10月3日 新增 mod-2 线性代数来源包（Kimi probe 部分协助）

新增 `sources/mod-two.md` 与快照 `2026-10-03-mod-two.json`：`ModTwoVector`
位打包、`ModTwoSubspace` 的 pivot 索引 RREF、`ModTwoSubquotient`。Kimi probe
草案因摘录按文档注释选择而漏掉未注释方法、偏薄；由维护者直接读源补齐。教训：
摘录应包含裸签名清单。索引与 sources/index.md 已同步；未运行编译、测试或
compiler 生成。

## 2026年10月3日 新增权格类型来源包（Kimi probe 协助）

新增 `sources/lattice-types.md` 与快照 `2026-10-03-lattice-types.json`：
Weight/Coweight newtype 纪律、pair、RationalWeight 归一化、RationalCoweight。
同一 Kimi probe 路由起草（exit 0，215.9s）；维护者对照源码逐条核对改写。
索引与 sources/index.md 已同步；未运行编译、测试或 compiler 生成。

## 2026年10月3日 新增图像基对来源包（Kimi probe 协助）

新增 `sources/real-projection.md` 与快照 `2026-10-03-real-projection.json`：
`lift_mat`/`M_real` 基对、播种/传送纪律与坐标接口。同一 Kimi probe 路由起草
（exit 0，74.5s）；维护者对照源码逐条核对改写。索引与 sources/index.md 已
同步；未运行编译、测试或 compiler 生成。

## 2026年10月3日 新增 matreduc 来源包（Kimi probe 协助）

新增 `sources/matreduc.md` 与快照 `2026-10-03-matreduc.json`：逐操作保真动机、
diagonalise、求解/像判定、inverse_upper_triangular 与 exp_i。同一 Kimi probe
路由起草（exit 0，75.6s）；维护者对照源码逐条核对改写。索引与
sources/index.md 已同步；未运行编译、测试或 compiler 生成。

## 2026年10月6日 Weyl 身份来源包推进到 AFTER v1/v2 失败与 v3 迁移

更新 `sources/weyl-context-identity-and-sharing.md`：编辑状态与新增
2026-10-06 节记录 AFTER-v1（3890328，错误比较常量）与 AFTER-v2
（3890580，sbatch 标签未随版本迁移）两次 harness 失败、v3 迁移提交
93abd29b 的要点（AFTER_V1_PREDECESSOR 本地重绑定、sbatch 标签钉到
STAGE_NAME、predecessor-v12 不递增）以及 SecureLink 隧道再次中断导致的
提交暂缓。仅人工维护来源包；未运行编译、测试或 compiler 生成。

## 2026年10月6日 新增根数据与对偶构造来源包（Kimi probe 协助）

新增 `sources/root-datum-dual.md` 与快照 `2026-10-06-root-datum-dual.json`：
`BasedRootDatum` 构造门控、radical/coradical 饱和核、简单反射；`dual.rs`
五个公开入口与原词重放。同一 Kimi probe 路由起草（exit 0，387.3s），本次
以两个文件的完整字节（42KB 提示）代替摘录，避免了 mod-two 的遗漏模式；
维护者对照源码逐条核对改写。索引与 sources/index.md 已同步；未运行编译、
测试或 compiler 生成。

## 2026年10月6日 新增 alcove 几何来源包（Kimi probe 协助）

新增 `sources/alcove.md` 与快照 `2026-10-06-alcove.json`：alcove_center 的
墙方程/通分/-θ 校验链、denominator 守卫的 rank≥63 边界、RootNumbering、
wall_set 分层、并查集分量、root_vertex_simple 的 labels_1 重试。同一
Kimi probe 路由以完整文件字节起草（exit 0，269.5s），维护者对照源码逐条
核对改写；连续第二次完整字节输入均无需事实更正，该模式成为此路由默认。
索引与 sources/index.md 已同步；未运行编译、测试或 compiler 生成。

## 2026年10月6日 新增内类字母/对合查表来源包（Kimi probe 协助）

新增 `sources/primitive-involution.md` 与快照
`2026-10-06-primitive-involution.json`：`InnerClassLetterError` 文案、
checked_inner_class_letters 的坍缩规则、layout_involution 逐字母表、
on_basis 精确除法换基。同一 Kimi probe 路由起草（exit 0，253.1s），
维护者对照源码逐条核对改写；连续第三次完整字节输入无事实更正。
索引与 sources/index.md 已同步；未运行编译、测试或 compiler 生成。

## 2026年10月6日 新增对合类型三件套来源包（Kimi probe 协助）

新增 `sources/involution-types.md` 与快照 `2026-10-06-involution-types.json`：
LatticeInvolution 三道门控、RootInvolutionData 的根置换+余根运输验证与
RootKind 分类、TwistedInvolution 的 w·θ 重门控。同一 Kimi probe 路由以
三文件完整字节起草（exit 0，362.1s），维护者对照源码逐条核对改写；
草案自带「高风险核对点」清单恰好覆盖最需细读处（配对条件语义、错误载荷
不对称），加速了核对。索引与 sources/index.md 已同步；未运行编译、测试或
compiler 生成。

## 2026年10月6日 新增 Cartan fiber 对来源包（Kimi probe 协助）

新增 `sources/cartan-fibers.md` 与快照 `2026-10-06-cartan-fibers.json`：
CartanFiber 的先分母后分子子商构造、元素 provenance 语义；
AdjointCartanFiber 的预算分层、绑定语义与 FiberToAdjoint 按需映射。
同一 Kimi probe 路由起草（50KB 两文件完整字节，exit 0，403.6s），
维护者对照源码逐条核对改写。索引与 sources/index.md 已同步；未运行
编译、测试或 compiler 生成。

## 2026年10月6日 新增实 Weyl 群来源包（Kimi probe 协助）

新增 `sources/real-weyl.md` 与快照 `2026-10-06-real-weyl.json`：RealWeyl
载荷、dual_side 临时重建、fiber_side 三包、simple_basis/simple_complex
怪癖、twisted_orbit_size、打印层字节契约。同一 Kimi probe 路由起草（68KB
单文件，exit 0，253.7s），维护者对照源码逐条核对改写；草案正确捕捉到
real_r/imaginary_r 交叉赋值与 simple_basis 外层终止怪癖。索引与
sources/index.md 已同步；未运行编译、测试或 compiler 生成。

## 2026年10月6日 新增普通根系来源包（Kimi probe 协助）

新增 `sources/root-system.md` 与快照 `2026-10-06-root-system.json`：
RootSystem 的三表对齐存储、BFS 闭包枚举、RootSystemBudget 语义、梯子底表
与「溢出即非成员」修复形态、25 个测试锚点。同一 Kimi probe 路由起草
（exit 0，342.7s），维护者对照源码逐条核对改写；草案独立发现注释
「十一边界用例」与实测八组的数量差异，已记录待核。索引与
sources/index.md 已同步；未运行编译、测试或 compiler 生成。

## 2026年10月6日 新增 Dynkin 分类器来源包（Kimi probe 协助）

新增 `sources/dynkin.md` 与快照 `2026-10-06-dynkin.json`：classify 输入契约、
first-fresh-vertex 分量合并、秩二 B/C 给定序规则、各型起点选择与 E 型长臂
交换、bourbaki_permutation、folded_cartan。同一 Kimi probe 路由起草（exit 0，
375.2s），维护者对照源码逐条核对改写；草案的错误位点表与可达性标注加速了
核对。索引与 sources/index.md 已同步；未运行编译、测试或 compiler 生成。

## 2026年10月6日 新增 Cayley/Cross 与整对合分类来源包（Kimi probe 协助）

新增 `sources/cayley-cross.md` 与快照 `2026-10-06-cayley-cross.json`：
CayleyCrossDecomposition 的 provenance 门、peeling 预算位置、逆序重放与
重放验证；classify_involution 的预算先行、classify_plus_identity 公式、
fiber_rank。同一 Kimi probe 路由起草（33KB 两文件，exit 0，347.5s），
维护者对照源码逐条核对改写。索引与 sources/index.md 已同步；未运行编译、
测试或 compiler 生成。

## 2026年10月6日 新增 locator 来源包（Kimi probe 协助）

新增 `sources/locator.md` 与快照 `2026-10-06-locator.json`：驻留三件套、
int_item 的 (a)–(f) 流程、fundamental_alcove_walls、pos_simples、
make_relative_to。同一 Kimi probe 路由起草（46KB 单文件，exit 0，440.2s），
维护者对照源码逐条核对改写；草案的「接口使用面」表格是有用的段落模式。
索引与 sources/index.md 已同步；未运行编译、测试或 compiler 生成。

## 2026年10月6日 新增实形式标签/编号来源包（Kimi probe 协助）

新增 `sources/real-form-labels-order.md` 与快照
`2026-10-06-real-form-labels-order.json`：RealFormLabels 的出处闸门与
grading 关联机制、base_grading_extension；ExternalFormOrder 的严格
(depth, tiebreak) 排序、DepthTables、verified_generator_map、
special_grading_key。同一 Kimi probe 路由起草（53KB 两文件，exit 0，
487.8s），维护者对照源码逐条核对改写；草案发现 DepthTables::build 的死
循环与 weight_sum 恒 Some 两处真实源码观察，已记录为清理候选。索引与
sources/index.md 已同步；未运行编译、测试或 compiler 生成。

## 2026年10月6日 新增 KL 支撑来源包（Kimi probe 协助）

新增 `sources/kl-support.md` 与快照 `2026-10-06-kl-support.json`：
RankFlags 位集、validate_topology 门控、下降/good-ascent 分类、
length-stop 表、懒填充本原索引及其 prepare-first 前置条件链。同一
Kimi probe 路由起草（16KB 单文件，exit 0，321.1s），维护者对照源码
逐条核对改写。索引与 sources/index.md 已同步；未运行编译、测试或
compiler 生成。

## 2026年10月6日 新增块拓扑/修正子来源包（Kimi probe 部分协助）

新增 `sources/block-access-modifier.md` 与快照
`2026-10-06-block-access-modifier.json`：BlockTopology 密封契约、
bruhat_hasse、PartialBlock 委托细节；BlockModifier 构造器与
RepContext 扩展方法。Kimi probe 在 480s 期限截断（SIGTERM；单个 assistant
记录是完整 JSON 但内容断在句中——教训：按 ~13s/KB 估期限）；已覆盖部分
核对无误，测试/限制章节由维护者按完整阅读补齐。索引与 sources/index.md
已同步；未运行编译、测试或 compiler 生成。

## 2026年10月6日 新增合成实形种子来源包（Kimi probe 协助）

新增 `sources/minimal-torus.md` 与快照 `2026-10-06-minimal-torus.json`：
elected_square_root 与 minimal_torus_part 的完整流程。同一 Kimi probe 路由
起草（期限按新教训放宽到 540s，exit 0，414.4s），维护者对照源码逐条核对
改写；草案独立指出 encode 的恒 Ok 签名与正例测试 coch==factor 的覆盖缺口。
索引与 sources/index.md 已同步；未运行编译、测试或 compiler 生成。

## 2026年10月6日 新增内类布局/限制根来源包（Kimi probe 协助）

新增 `sources/layout-restricted-roots.md` 与快照
`2026-10-06-layout-restricted-roots.json`：InnerClassLayout 的构建与
Complex 对旋转、上游移位顺序怪癖、环面 Smith 商对合；RestrictedWeight 的
(1-θ) 编码与纤维聚合。同一 Kimi probe 路由起草（exit 0，182.9s——目前
最快），维护者对照源码逐条核对改写。索引与 sources/index.md 已同步；
未运行编译、测试或 compiler 生成。

## 2026年10月6日 新增 Weyl 阶/展示层来源包（Kimi probe 协助）

新增 `sources/weyl-size-presentation.md` 与快照
`2026-10-06-weyl-size-presentation.json`：weyl_order_of_cartan 的分量 BFS
与分支形状分派、branch_lengths；build_presentations 与状态位。同一
Kimi probe 路由起草（exit 0，292.1s），维护者对照源码逐条核对改写；
草案的两处阅读观察（秩检查落后于扫描、负边乘积计度数不提重数）均正确
并保留。索引与 sources/index.md 已同步；未运行编译、测试或 compiler 生成。
