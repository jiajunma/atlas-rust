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

## 2026年10月6日 新增拓扑/命名来源包（Kimi probe 协助）

新增 `sources/topology-form-name.md` 与快照
`2026-10-06-topology-form-name.json`：dual_pi0、CorootRestriction、
对合转运管线（trivial/rank 复制的漂移风险已记录）；命名规则表与
form_type_name。同一 Kimi probe 路由起草（exit 0，264.2s），维护者对照
源码逐条核对改写。索引与 sources/index.md 已同步；未运行编译、测试或
compiler 生成。

## 2026年10月6日 新增 StructureError/全局 Tits 来源包（Kimi probe 协助）

新增 `sources/error-global-tits.md` 与快照
`2026-10-06-error-global-tits.json`：StructureError 的 53 变体字段形态
家族（14 invariant + 7 resource-limit 及 u64 例外）、GlobalTitsElement
构造门槛、crossed_generator 的 RootKind 三分支与虚根整性门槛、
crossed_word 前向顺序、10 个测试锚点与未覆盖分支清单。同一 Kimi probe
路由起草（exit 0，398.9s），维护者对照源码逐条核对改写；草案的 53 变体
普查与全部 Display 文案精确，未覆盖分支与直接下标 panic 面观察均正确并
保留。索引与 sources/index.md 已同步；未运行编译、测试或 compiler 生成。

## 2026年10月6日 新增全局 KGB 来源包（Kimi probe 协助）

新增 `sources/global-kgb.md` 与快照 `2026-10-06-global-kgb.json`：
GlobalTorusElement 的约化纪律与算术历史、x_pack 指纹、基本纤维/平方类
播种、六阶段 build 与 14 个 KgbInvariantViolation 字面量、print_X 版式
与 4 个测试锚点（含 rank-0 未测的 NOTE）。同一 Kimi probe 路由起草
（800s 期限，exit 0，520.9s；13s/KB 超时规则成立），维护者对照源码逐条
核对改写；草案的错误字面量普查、status/cross 参数序不对称、
torus_label 吞错与 print_layout 传播的不一致、panic 面清单均精确并保留。
索引与 sources/index.md 已同步；未运行编译、测试或 compiler 生成。

## 2026年10月6日 新增块图源码包（Kimi probe 协助）

新增 `sources/block.md` 与快照 `2026-10-06-block.json`：BlockDescent 表、
dual_involution、纤维积 build、i1/i2 Cayley 槽共享与 fall-through、
访问器弱下降强制、dual() 变换、7 个秩1测试锚点与未覆盖面。同一
Kimi probe 路由起草（620s 期限，exit 0，242.7s），维护者对照源码逐条
核对改写；20 个 BlockInvariantViolation 字面量普查精确，dual_position
静默覆盖等阅读观察保留。索引与 sources/index.md 已同步；未运行编译、
测试或 compiler 生成。

## 2026年10月6日 新增扭对合/分类/反射字来源包（Kimi probe 协助）

新增 `sources/twisted-involution-trio.md` 与快照
`2026-10-06-twisted-involution-trio.json`：TwistedInvolution 构造门槛、
compose_matrices 怪癖、compact/complex/split 秩核、fiber_rank、
reflection_word 贪心扫描与无上限循环。同一 Kimi probe 路由起草
（300s，exit 0，274.4s），维护者对照源码逐条核对改写；草案标记的四处
防御策略不一致均属实并保留为复核备注。索引与 sources/index.md 已同步；
未运行编译、测试或 compiler 生成。

## 2026年10月6日 新增格值/KLV 多项式来源包（Kimi probe 协助，含一次超时）

新增 `sources/lattice-kl-polynomial.md` 与快照
`2026-10-06-lattice-kl-polynomial.json`：Weight/Coweight 区分、
RationalWeight 归一纪律、KlPol trim 不变量与递归操作集、KlHashTable
种子池与 Default 隐患。Kimi probe r1 在 360s 期限超时（25.5KB 提示实测
需要 326s），保留失败证据后以 600s 期限重试成功（exit 0，326.1s）；
**超时规则上修：>20KB 的提示用 ~24s/KB（或双倍估计）**。维护者对照源码
逐条核对改写，并补充一处跨文件观察（两处 gcd_u64 副本的 (0,0) 语义已
漂移）。索引与 sources/index.md 已同步；未运行编译、测试或 compiler
生成。

## 2026年10月6日 新增 Weyl 作用/根对合来源包（Kimi probe 协助）

新增 `sources/weyl-root-involution.md` 与快照
`2026-10-06-weyl-root-involution.json`：WeylAction 溯源与双矩阵、
compose_fast 热路径、CompactWeyl+rayon 枚举管线、RootInvolutionData
校验链与子系单根选举。同一 Kimi probe 路由起草（600s，exit 0，327.7s），
维护者对照源码逐条核对改写；草案的复核清单（datum 进 Eq 的张力、
insert_action 死代码、as i32 截断、枚举序无锚定）均属实并保留。
索引与 sources/index.md 已同步；未运行编译、测试或 compiler 生成。

## 2026年10月6日 新增分级/模二来源包（Kimi probe 协助）

新增 `sources/grading-mod-two.md` 与快照
`2026-10-06-grading-mod-two.json`：Grading 语义、CartanGradingData 构造与
增广消元求逆、ModTwoSubspace RREF/低主元、CanonicalModTwoSection 64 列
掩码与 2^12 穷举 oracle、ModTwoSubquotient 与诱导映射校验。同一 Kimi
probe 路由起草（1300s 期限，exit 0，458.1s；~24s/KB 超时规则成立），
维护者对照源码逐条核对改写。索引与 sources/index.md 已同步；未运行编译、
测试或 compiler 生成。

## 2026年10月6日 新增实投影/矩阵约化来源包（Kimi probe 协助）

新增 `sources/real-projection-matreduc.md` 与快照
`2026-10-06-real-projection-matreduc.json`：(1−θ) 像基对、带符号
gcd_sweep、幺模整数逆、diagonalise 符号簿记逐行追踪、
has_solution/find_solution/in_*_image、inverse_upper_triangular、exp_i。
同一 Kimi probe 路由起草（1200s，exit 0，455.9s），维护者对照源码逐条
核对改写；草案的 row_minus 簿记追踪与非受检算术不对称观察均属实并保留。
索引与 sources/index.md 已同步；未运行编译、测试或 compiler 生成。

## 2026年10月6日 新增弱实形/对合表来源包（Kimi probe 协助）

新增 `sources/weak-real-form-involution-table.md` 与快照
`2026-10-06-weak-real-form-involution-table.json`：掩码轨道游走与编号
规则、九道闸门、像基对播种+搬运（B2 x=4 锚点）、add_cartan BFS 与查询面。
同一 Kimi probe 路由起草（1600s 期限，exit 0，658.1s；本次以 shell &
分离启动——自我管理期限安全但失去任务跟踪，已记录，优先用后台任务路由），
维护者对照源码逐条核对改写。索引与 sources/index.md 已同步；未运行编译、
测试或 compiler 生成。

## 2026年10月6日 新增 K 型来源包（Kimi probe 协助）

新增 `sources/ktype.md` 与快照 `2026-10-06-ktype.json`：KType 当选代表
不变量、六谓词链、equivalent、四个变形循环与终止预算、finals_for 分支
结构、kgp_set 位图 BFS；并记录 height 来源不对称与两个无断言观察型测试。
同一 Kimi probe 路由起草（1100s，exit 0，349.5s），维护者对照源码逐条
核对改写。索引与 sources/index.md 已同步；未运行编译、测试或 compiler
生成。


## 2026年10月6日 ktype 包重复事件与修复

发现 `ktype.rs` 已有 2026-10-03 的初读包（提交 187a6dac，本次会话的
gap 清单漏列了它）；新包在未变字节上写成后覆盖了旧包。内容无损失（新包
是严格超集），旧快照 2026-10-03-ktype.json 保留未动。修复：删除
sources/index.md 中的旧重复条目，在新条目与包正文中注明 supersession。
**教训：启动一个包之前先在 `kb/sources/` 与 `sources/index.md` 里 grep
模块名，确认没有既有包；间隙清单必须以 index.md 为准而不是凭会话记忆。**


## 2026年10月6日 block 重复包并入 block-graph（重读合并）

`block.rs` 的 2026-10-06 重读包并入 `block-graph.md`（字节未变，两次阅读
SHA-256 相同）：补充 i1/i2 Cayley 槽共享与 fall-through、descents() 判定
逻辑、20 个 BlockInvariantViolation 字面量普查、7 个秩1测试锚点、
dual_position 静默覆盖与 cross 预填 0 两处阅读观察。重复文件 block.md
删除，索引改为单条目并链接 2026-10-03 与 2026-10-06 两份快照。


## 2026年10月6日 KB 重读包合并完成（6 处重复）+ strong-real 草稿不收录

本次会话在 2026-10-03 既有包之外又起草了 7 个包，经逐字节核对确认全部为
**未变字节**上的重读。已按「保留旧名、合并新细节、双快照链接」原则合并：
block → block-graph；grading-mod-two → grading + mod-two；
lattice-kl-polynomial → lattice-types + kl-polynomial-table；
real-projection-matreduc → real-projection + matreduc；
weak-real-form-involution-table → weak-real-form + involution-table；
weyl-root-involution → weyl-layer（involution-types 仅加一行交叉引用，
其 root_involution.rs 内容已完整）。ktype 此前已单独修复（a1837fa7）。
重读新贡献均已并入对应包：错误分支普查、测试锚点、阅读观察
（静默覆盖、死代码、填报怪癖、策略不对称、过期文档等）。

strong_real.rs 的 probe 草稿（exit 0，200.9s，session 见证据目录
docs/evidence/kimi-runtime-20261006/strong-real-probe/）**不收录**：
`strong-real.md`（2026-10-03）已存在且字节未变，重读未产生新内容；证据
目录仅作调用记录保留。


## 2026年10月6日 wiki 编译首次运行受阻：codex-agent 未登录

首次 `./kb/llmwiki compile --review --instructions AGENTS.md --verbose`
在 47 个来源包完成抓取后失败：`Codex CLI authentication failed: it is
not authenticated or its login was rejected`（本机 codex-cli 0.154.0 的
登录已过期）。编译器不做任何页面变更（review.hold 下无候选写入）。
**恢复条件：用户交互式 `codex login` 后重跑同一命令**；备选提供方
（anthropic/openai/ollama）本机均未配置。源码包维护（Kimi probe 路由）
不受此影响，照常进行。


## 2026年10月6日 新增伴随纤维来源包（Kimi probe 协助）

新增 `sources/adjoint-fiber.md` 与快照 `2026-10-06-adjoint-fiber.json`：
Arc 出处绑定模型、九步构建链、两种投影形态、三条预算线公式、
「每次调用独立计费」观察与 9 个测试锚点。同一 Kimi probe 路由起草
（800s 期限，exit 0，312.4s），维护者对照源码逐条核对改写。索引与
sources/index.md 已同步；未运行编译、测试或 compiler 生成。
atlas-real-group 剩余未覆盖：`lib.rs`（crate 根）与 `dual.rs`
（root-datum-dual 已含 dual 内类构造的主要面，待核）；atlas-core
语言层仍是最大缺口。


## 2026年10月6日 新增 crate 根来源包（Kimi probe 协助）—— atlas-real-group 全覆盖

新增 `sources/lib-root.md` 与快照 `2026-10-06-lib-root.json`：60 模块
组织与 52 条再导出（topology 怪点、integer_lattice 两条、deform 双重
暴露、5 个零导出模块）、错误汇聚点、A1 原型层校验顺序与双上限。同一
Kimi probe 路由起草（600s 期限，exit 0，590.5s），维护者对照源码逐条
核对改写；草案的 60/52/5/10 四处计数全部精确。索引与 sources/index.md
已同步；未运行编译、测试或 compiler 生成。**至此 `atlas-real-group`
全部 60 个模块均有来源包或明确交叉引用；剩余最大缺口是 `atlas-core`
语言层（session/typed/domain_builtins/value/types/lex/syntax 等）。**

新增 `sources/atlas-core-root.md` 与快照 `2026-10-09-atlas-core-root.json`：
atlas-core 语言层第一包——lib.rs 的 15 个 `pub mod` + `pub(crate)`
`matreduc` + cfg(test) `session_fixture_tests` + `COMPATIBILITY_VERSION`
"atlas-language-v0"，逐模块行数与文件头自述角色（实现方陈述）。维护者
直接撰写（模块地图小，无 Kimi 调用）；git base `964f0033`，25 个文件
逐字节哈希。索引与 sources/index.md 已同步；未运行编译、测试或
compiler 生成。语言层大文件（typed/domain_builtins/session/syntax）的
内部实现仍待分包。

新增 `sources/atlas-core-session.md` 与快照 `2026-10-09-atlas-core-session.json`：
session.rs 的 SessionEvent 六变体（字节保留面）、逐命令外层循环、消费时刻
补全记录、execute_tokens 前缀保留与 SetType span 重建、drain-before-diagnose
顺序，以及 201 测试回归库的家族地图。维护者直接撰写（无 Kimi）。教训：快照
字节数第一次凭记忆写错（129981→实测 191393），已用 stat 修正——与"sha 尾巴
不凭记忆"同一纪律，字节数也一律实测。索引与 log 已同步；未运行编译/测试/
compiler。

新增 `sources/atlas-core-session-frame.md` 与快照
`2026-10-09-atlas-core-session-frame.json`：FileProvider/FileSink 边界
（sink 在解析后求值前打开）、include-once/强制/循环/64 层语义、clean 纪律
（Io 不弄脏）、Value:/void 打印、深度缩进、重定向体先按表达式解析、
abandon 级联（最内层先、line_map 物理行）、preprocess 续行。18 个测试
锚点。维护者直接撰写（无 Kimi）；索引与 log 已同步；未运行编译/测试/
compiler。

新增 `sources/atlas-core-lex.md` 与快照 `2026-10-09-atlas-core-lex.json`：
TokenKind 12 变体（OperatorBecomes 融合、命令首指令）、35 关键字 + 20 个
按位保留的原始类型、换行抑制状态机（嵌套栈 + prevent/previous 终止符）、
指令/注释/字符串边界（嵌套注释、双写引号、未闭合串 warning 且恢复 token
保留）、TokenCursor 缓存错误 peek、tokenize_with_diagnostics。维护者直接
撰写（无 Kimi）；索引与 log 已同步；未运行编译/测试/compiler。

新增 `sources/atlas-core-syntax.md` 与快照 `2026-10-09-atlas-core-syntax.json`：
Expr/Command/Pattern/TypeExpr 的 AST 面、四条解析入口（fragment 的
Ok(None) 多行续行纪律）、TokenStream 适配与 Bison 措辞诊断、type_scope。
51 个测试。维护者直接撰写（无 Kimi）；字节数一律实测（快照脚本现场测量，
覆盖初稿占位值）。索引与 log 已同步；未运行编译/测试/compiler。

新增 `sources/atlas-core-types.md` 与快照 `2026-10-09-atlas-core-types.json`：
Type/Prim/TypeBinding/TypeTable（revision Arc 身份、matching_bindings 查全部
保留定义、先校验再等值、specialise 为唯一变异）+ polymorphic 二阶机器
（TypeScheme/TypeAssignment/InferredType，含上游 append 语义）+ recursive
图级安装 + revision_tests。59 个测试。维护者直接撰写（无 Kimi）；索引与
log 已同步；未运行编译/测试/compiler。

新增 `sources/atlas-core-value-layer.md` 与快照
`2026-10-09-atlas-core-value-layer.json`：Value 面（Union injector 打印、
Closure 载荷、不透明 BuiltinFunction、字节保留 AtlasString、Rational 符号
单走且分母恒打印）+ vec/mat/ratvec 载荷与逐字节上游打印格式（列主序、
构造时规范化）+ 算符优先级栈的奇偶结合律。11 个测试。维护者直接撰写
（无 Kimi）；索引与 log 已同步；未运行编译/测试/compiler。

新增 `sources/atlas-core-support-layer.md` 与快照
`2026-10-09-atlas-core-support-layer.json`：诊断（SourceId/Span、ErrorKind
分类、raw_message 字节权威、warning 不弄脏、back_trace 最外层在前且拷入
系统变量）+ SourceText（预存行首、Unicode 标量列）+ 强转表（29 条上游
顺序、首中即返、row_coercion、is_close 三比特、broader_eq 平衡序）。
维护者直接撰写（无 Kimi）；索引与 log 已同步；未运行编译/测试/compiler。

新增 `sources/atlas-core-typed-core.md` 与快照
`2026-10-09-atlas-core-typed-core.json`：typed.rs 上部（59–2757）——
控制/级别枚举、TypedExpr 可执行树（多项式订阅先接收方后键、饥饿内建）、
Analysis（身份守护的未移位签名缓存、活 return_type、loop_depth、
type_floor）、IdTable/TypeCell 定义处下限纪律、OverloadState 的修订守护
合并视图与 add_user 上游重放、TypedCommandEvent、TypedContext 字段面与
启动播种。convert_expr/内建注册表/TypedExpr impl/测试**不在本包**。
维护者直接撰写（无 Kimi）；索引与 log 已同步；未运行编译/测试/compiler。

新增 `sources/atlas-core-domain-values.md` 与快照
`2026-10-09-atlas-core-domain-values.json`：domain_builtins.rs 上部——Weyl
身份机器（成功才落定的 WeylIdentityCell、无环 DatumWeylIdentity、
share_group_into_if_cold、按内容弱 interning）、RootDatumHandle 结构相等
忽略身份缓存、Weyl 兼容 = 抽象群 Arc 身份 + 左系统重放、SplitValue 回绕
对偶算术、DomainValue 结构等值、多项式系数契约、派发入口与饥饿乘积。
本区域是 Weyl 修复落点；语义由 after-v5 限定验收，结构性阅读不授予验收。
维护者直接撰写（无 Kimi）；字节数实测；索引与 log 已同步。

新增 `sources/atlas-core-domain-dispatch.md` 与快照
`2026-10-09-atlas-core-domain-dispatch.json`：call 路径（三个饥饿乘积 /
打印侧通道只有 partial_extended_KL_block 用）、166 臂 match 的组织与
逐臂校验顺序契约、coerce 的逐标签转换（KpolK 的 finals_for、PolP 的
expand_final，canonical 项序）、build_real_form 的规范弱缓存。维护者
直接撰写（无 Kimi）；索引与 log 已同步；未运行编译/测试/compiler。

新增 `sources/atlas-core-weyl-subgroup.md` 与快照
`2026-10-09-atlas-core-weyl-subgroup.json`：weyl_subgroup.rs 全读——头部纪律
（dominance 真用给定生成元，R3 反例修复）、构造校验（收窄再查根号、i128
配对、原版逐字 Cartan 错误）、精确配对坐标的陪集树（正缩放不变性）、
见证作用序（权右到左、余权左到右）、BuildAndDrop、4 个测试锚点。
after-v3 教训涉事文件。维护者直接撰写（无 Kimi）；索引与 log 已同步。

新增 `sources/atlas-core-convert-expr.md` 与快照
`2026-10-09-atlas-core-convert-expr.json`：typed.rs 中部（2757–5973）——
convert_expr 的 in/out 类型模式、type_floor 调整、12 族 #[inline(never)]
划分（3839541 栈陷阱教训）、while 转换细节（循环层包住整棵 do 树、bool
措辞、WhileMode、row_coercion 回退）、赋值助手群（共享简单赋值路径、
分量赋值下标门控、保留定义做投影解析、永不丢参数优化）。维护者直接撰写
（无 Kimi）；索引与 log 已同步；未运行编译/测试/compiler。

新增 `sources/atlas-core-builtin-registry.md` 与快照
`2026-10-09-atlas-core-builtin-registry.json`：typed.rs 5973–11565——Builtin
面（hunger/overload_visible/implementation）、BuiltinImpl 变体（含
DomainNoValue 的 Skip/Validate/BuildAndDrop 无值门策略：补全名清单≠无值
策略清单）、求值辅助的上游契约（收窄逐字诊断、向量地板除、nth_set_bit
补码走、flex 修剪、convolve、ratvec LCD）、321 条目/170 名字的启动清单
（相对上游不全，缺口由 REMAINING_BUILTINS 跟踪）。维护者直接撰写
（无 Kimi）；索引与 log 已同步。

新增 `sources/atlas-core-typed-eval.md` 与快照
`2026-10-09-atlas-core-typed-eval.json`：typed.rs 11565–13741——evaluate 的
六族划分（与 convert_expr 同形）、迭代借用纪律（矩阵不再造列矩阵、多项式
保 canonical 项序与属主）、调用机器（变参元组解开、单值参数分发、空层
规则、递归 0 号槽自绑、新帧不入捕获链保无环、return 解到调用边界、带名
槽的错误附帧转储迹行）、回溯渲染（parsetree.w 的 at NAME:LINE:COL 形式）。
维护者直接撰写（无 Kimi）；索引与 log 已同步。

新增 `sources/atlas-core-domain-construction.md` 与快照
`2026-10-09-atlas-core-domain-construction.json`：构造管线（build_datum 的
两路基、商/显式 datum、build_inner_class_context 的固定装配顺序含对偶侧
只建一次、build_inner_class 的转置+from_root_involution、
build_dual_inner_class 的余根翻转+逐字母对偶、build_real_form 规范弱缓存、
build_custom_real_form 的新表+基本 Cartan+自定义种子）。维护者直接撰写
（无 Kimi）；索引与 log 已同步。

新增 `sources/atlas-core-deformation-cache.md` 与快照
`2026-10-09-atlas-core-deformation-cache.json`：形变机器（canonical 键、
锁不跨递归、只缓存完整排序结果、active 集检环、协作截止）、普通全形变
递推 F(z)=L(z)+Σc_t(1-s)F(t)（=(1+s) 展开等价原版整数递推）与 scale-zero
全保留基底、逐子项 scale/readjust/lookup/common-terms 递归、扭曲版的
后 setup 计时与翻转系数。维护者直接撰写（无 Kimi）；索引与 log 已同步。

新增 `sources/atlas-core-domain-validate-print.md` 与快照
`2026-10-09-atlas-core-domain-validate-print.json`：46 臂 validate 的逐臂
无值门顺序契约、块打印机（common_block_rows 新建打印一致 + (x,g-lambda)
init 匹配、located_ 的共享查找、partial 的调用方 gamma survives）、
print_text 与各打印机（print_KGB 同形式选择、print_gradings 的
sigma.pull_back 位约定、print_real_Weyl 的臂内先查）。维护者直接撰写
（无 Kimi）；索引与 log 已同步。

新增 `sources/atlas-core-root-numbering-alcove.md` 与快照
`2026-10-09-atlas-core-root-numbering-alcove.json`：RootNumbering 的
RootNbr 序（正根按 (level, 末坐标向前) 、负根 rootMinus npos-1-p、
signed=nbr-npos）与 alcove 机器（wall_set 的 min_coroots_for 成员、
root_components 按**最大** RootNbr 的分量序——FPP 可观察、
labels_for_component 的唯一原始正关系、sorted_by_label 降序、
from_fundamental_alcove 的留一单位墙+to_positive_system、秩+分量墙数、
精确 Cartan 逆）。维护者直接撰写（无 Kimi）；索引与 log 已同步。

新增 `sources/atlas-core-center-classifier.md` 与 `sources/atlas-core-domain-scc-root-table.md`
及对应快照：CenterClassifier 的中心陪集 tabulation（adjugate/det 与
C_denom 一致）+ adjoint 轨道 BFS（尾部递减、完成层反转）+ 词转换与
反射词约定（末字母先作用）；块图迭代 SCC + ByLastCoordinate 逆坐标序 +
宽累积矩阵积 + RootTable::build。维护者直接撰写（无 Kimi）；索引与 log
已同步。至此 domain_builtins.rs 全区域均有来源包（逐臂数学内容除外）。

新增 `sources/atlas-cli-main.md` 与快照 `2026-10-09-atlas-cli-main.json`：
CLI 前端全读（FsProvider 有损 UTF-8、字节保留 print_events、rayon 2MiB
工作栈、--path= 解析、文件按普通命令流喂、clean 退出状态——缺包含不弄脏）。
维护者直接撰写（无 Kimi）。至此 crates/ 全部三个 crate 的每个模块均有
来源包或明确交叉引用（两测试模块的组织按引用覆盖）。

新增 `sources/atlas-core-regression-library.md` 与快照
`2026-10-09-atlas-core-regression-library.json`：四个测试模块的组织地图
（session 201 / typed 133 / domain 92 / fixture 17）、原版背书范式
（include_str! + oracle 金标）与证据链位置。不含逐测试内容；不声称通过。
维护者直接撰写（无 Kimi）。**至此 atlas-core + atlas-cli 全模块来源包
齐备**（crate 根、语言层各模块、typed.rs 四区、domain_builtins.rs 六区 +
weyl_subgroup、CLI、测试库地图）。

KB 自洽审计（只读）：发现 `atlas-core-center-classifier.md` 缺索引条目（包与
快照已在，索引漏记），已补上；另有一个 2026-10-01 的候选快照无索引链接——
那是被 -v2 取代的**保留历史**快照（按纪律保留，不需索引链接）。审计结果：
73 包全部链接、85 快照全部存在、frontmatter 全部合规。

更新 `sources/weyl-context-identity-and-sharing.md` 至修复落地状态：AFTER-v4
过度断言失败与 AFTER-v5 job3900050 验收（A1 限定、flags 全 FALSE）、生产提交
`690c2b92` 的整树落地规则、上游 pin `7e1b958c` 行级确认（dualise 不重新编号、
转置 canonical dual 对 G2 不可直接构造、RootDatum `=` 为 interned 指针相等、
Weyl guard 先于 no-value gate、乘积保留左 owner、W_elt 强持有 datum、
inner_class_value::build eager dual）、G2 预期与预热空转分析、arc 现状。
新快照 `2026-10-09-weyl-owner-dual-landed.json` 绑定落地字节（与 after-gate
冻结哈希前缀一致）；历史小节以追记/标记保留。索引条目同步；维护者直接撰写。

## 2026年10月9日 wiki 编译首跑成功：真实阻碍是 strict schema，升级为 1.4.2

10月6日记录的"codex 未登录"阻碍其实从未是认证问题：当日 `codex login status`
即报已登录。今日用诊断 shim 捕获被 provider 扣留的子进程 stderr，真实错误是
`invalid_json_schema`（strict 模式要求每个对象 `additionalProperties: false`），
provider 的 stderr 正则误分类为认证失败。上游 1.4.2（2026-10-02）修复的正是
此问题（#266），核心包含 `toStrictSchema`。按既定维护纪律升级
llm-wiki-compiler 1.4.0-rc.2 → 1.4.2（读完 release notes；lockfile diff 仅
三个 atomicstrata 包；安装用 `--ignore-scripts`；安装后确认修复在位）。
`./kb/llmwiki compile --review --instructions AGENTS.md --verbose` 首跑成功：
73 个来源包全部编译，0 跳过 0 删除，533 个候选页全部进入 hold-all 审查队列
（63.7 分钟）。审查遵守既定纪律：逐候选对照来源字节核验后方可批准，不得
批量盲批。诊断 shim 与检查用 /tmp 副本均已移除。

## 2026年10月9日 533 候选的机械完整性筛查：0 异常

对每个候选 JSON 做机械核验（方法可复现：解析 `sourceStates` 钉住的来源
sha256 与当前字节比对；解析正文中 `^[来源.md:起-止, 同文件续段]` 引用——
逗号后裸行段继承前一文件名——检查文件存在、行号在界内）。结果：533 候选、
6210 条引用、0 异常；全部来源字节与生成时钉住的哈希一致（无生成后改动）。
这不替代逐候选内容审查：批准前仍须按 hold-all 纪律对照来源逐条核验内容
忠实性；首个深查样本（adjoint-fiber 的"余权坐标的投影出处绑定"）逐条通过。

## 2026年10月9日 首批候选审查批准：adjoint-fiber.md 的 7 页

逐候选对照 85 行来源包深读核验（首页"余权坐标的投影出处绑定"逐行核对引用
区间；其余 6 页逐页比对构建链、预算数字、测试锚点与未覆盖清单），7 页全部
批准后入 `wiki/concepts/`，索引与 MOC 已更新，候选 JSON 随批准清除。观察：
候选 `## Sources` 尾注链接普遍不带 `../../sources/` 前缀（434/532），批准的
页面亦如此——这是编译器的统一约定（实质引用由 `^[来源:行区间]` 锚点承担），
记录备查而非缺陷。指向尚未批准姊妹页的 [[...]] 链接暂为红链，随各包批次
审查逐步消解。批准前均核对了来源哈希与生成时钉住值一致。

## 2026年10月9日 第二批候选审查：weyl-context-identity-and-sharing.md 的 8 页

对照更新后的来源包（含 2026-10-09 落地状态节）逐页深读核验：群指针身份与
no-value 先后序、内容+偏好驻留键、cold-share/预热不覆盖、左 owner 乘积、
inner-class eager dual、AFTER-v5 验收范围与整树落地规则、性能数字的证据边界
（采样归因不可相加、59→5 仅为假设）、转置 canonical dual 的 G2 预热空转
分析——全部忠实。8 页批准；wiki/concepts/ 现有 15 页，剩余候选 518。

## 2026年10月9日 第三批候选审查：weyl-layer.md 的 8 页

对照 124 行来源包逐页核验：双层分工与互查桥、反射矩阵构造的受检算术与
无检查截断面、compose_fast 前置条件、derive 逐字段等值与文档表述的张力、
CompactWeyl+rayon 枚举与无断言排序、dead-code insert_action 观察、左右下降
的逆/正向读取、canonical_word 的内部生成子序与不变量检查、ParabolicPieces
排序键与上游行号转述限制——全部忠实，候选一致保留"未执行构建/测试/原版
运行"的边界声明。8 页批准；wiki/concepts/ 现有 23 页，剩余候选 510。

## 2026年10月9日 第四批候选审查：root-datum-dual.md 的 8 页

对照 216 行来源包逐页核验：构造门控顺序与 RootPairingMismatch 详情、
standard 的列约定、validate_cartan 拒绝序列、is_finite_type 两遍精确有理
计算（意图而非验收）、饱和核基的固定预算与空行维数保留、radical 文档疑似
笔误的悬置处理、dual_datum 复用门控、longest_action 下坡行走的预算语义与
未检查 i64 内层配对的悬置观察、dual_involution 的 M=q·W0 与 -M^t/-M 分工、
dual_cartan_correspondence 的反序配对/代表元共轭不可比/不变量即错误、
dual_twisted_representative 的原词重放、计数管线逐级预算——全部忠实，
测试锚点数字（sc/adjoint A1=2/2、紧 A2=1、扭 A2=2、紧 B2=3、对应表与
oracle 锚点 3501500）逐字一致。8 页批准；wiki/concepts/ 现有 31 页，
剩余候选 502。

## 2026年10月9日 第五批候选审查：kgb-graph-structure.md 的 7 页

对照 104 行来源包逐页核验：四值状态与两步分类、下降规则（real 恒是、
imaginary 恒否、complex 比 involution 长度）、分窗两相 BFS（64 元素窗、
Rayon 纯计算相 + 顺序 intern 相）、write-once/Cayley 长度步/kgb_size 三项
不变量、编号标准化的排序键与计数排序分工、HYBRID 自包含存储与 torus_factor
的唯一表依赖、两处 IndexOutOfRange 回退差异的悬置记录、RAYON_NUM_THREADS=1
与"并行结构存在≠已有多核加速"的边界声明——全部忠实。7 页批准；
wiki/concepts/ 现有 38 页，剩余候选 495。

## 2026年10月9日 第六批候选审查：inner-class.md 的 8 页

对照 119 行来源包逐页核验：部分实现的明确边界（无 Cartan fiber/实形式/
环面数据）、三个构造入口与 Weyl 因子取舍、based_involution_twist 的三条件
与 generator_twist 语义、twisted_from_involution 的调用方前提与
θ=w·δ 分解、三阶段 canonicalize（含 active 生成元交集细节）、
canonical_involution_expr 的 external-least 选举与 signed-entry 编码、
枚举族四分工（稳定列表/轨道/分区/生成元闭包版预算差异）——全部忠实。
8 页批准；wiki/concepts/ 现有 46 页，剩余候选 487。

## 2026年10月9日 第七批候选审查：cartan-classification.md 的 7 页

对照 110 行来源包逐页核验：四类聚合事实与 Arc 共享分区、CartanId 的 Atlas
编号顺序（height→simple 坐标逆字典序、比较存储前先 canonicalize）、分层预算
六字段与 direct-classification 切换的语义边界（计数含 identity、不动
weyl_budget）、严格 Cayley 偏序的 is_below 语义与不可反身不变量、
real_form_of 的 complex-only 行走与 grading 偶整数规则、coch 不返回的理由、
real_form_of_detailed 的扩展用途——全部忠实。7 页批准；wiki/concepts/ 现有
53 页，剩余候选 480。

## 2026年10月9日 第八批候选审查：real-projection.md 的 8 页

对照 95 行来源包逐页核验：基对定义与分解不变量、播种/传送纪律（基非唯一、
列符号/次序差异是携带而非重算的理由）、gcd_sweep 符号纪律（负主元取正 +
ops 记录 + div_euclid 防定向反转 + E6 involution-187 注释）、幺模逆的
±1 主元与逐项验证、check_against 收尾自校验、transported 的方阵-only 检查
与无测试声明、zip 静默截断与 lift panic 面、非受检消元与分配不对称的阅读
观察——全部忠实；四个测试锚点（含 original3840186 的逐字矩阵字面量）一致。
8 页批准；wiki/concepts/ 现有 61 页，剩余候选 472。

## 2026年10月9日 第九批候选审查：involution-table.md 的 8 页

对照 106 行来源包逐页核验：记录字段清单与图像基对的例外地位、幂等添加与
包含式容量上限、种子公式 (W+#Cayley)/2 与奇偶拒绝、stepped_length 的
±2/∓1 规则、投影传送用普通生成元矩阵的理由（δ 已并入 θ）、check_against
边对账、index_by_permutation 静默覆盖的调用纪律、lookup 键契约、Cayley
None 的 stage-e 语义、simple_root_kind 三合一——全部忠实；7 个测试锚点
（含 B2 投影传送的 assert_ne 字面量与 arm64 oracle 标注）逐字一致，未触
分支清单一致。8 页批准；wiki/concepts/ 现有 69 页，剩余候选 464。

## 2026年10月9日 第十批候选审查：rep-context.md 的 8 页

对照 103 行来源包逐页核验：四元组字段与相等性（height 不参与比较）、
undefined_print_weights 仅 UndefKGB 携带、两道 DatumMismatch 构造闸门与
from_derived 的 debug_assert 复核、sr_gamma/sr 构造链与 KType 互转、
lambda_rho 重建的减半奇偶拒绝、lambda_unique 的 div_euclid(2) 约定理由、
mod_reduce/build_srm、is_parity 与 orientation_number 的步骤、
reducibility_points 的"分子/分母对升序"措辞保留、deformation_terms 两个
边界与逆向累积顺序——全部忠实。8 页批准；wiki/concepts/ 现有 77 页，
剩余候选 456。

## 2026年10月9日 第十一批候选审查：tits-element.md 的 7 页

对照 77 行来源包逐页核验：元素形状与不携带逐元素 Weyl 数据的理由、裸构造器
不自动归约、序关系仅对 REDUCED 代表元有意义、reduce 幂等、grading offset 的
两种来源（stage d 的 square-class cocharacter 与 adjoint 约定）、FULL
inner-class 门控而非 datum-only、simple_grading 公式与 IMAGINARY 守卫、
cross 闭式映射的两步分解加 offset 修正、cayley/inverse_cayley 的目标/源
mod-space 分工与修复机制、三处上游行号转述限制——全部忠实。7 页批准；
wiki/concepts/ 现有 84 页，剩余候选 449。

## 2026年10月9日 第十二批候选审查：integer-lattice.md 的 7 页

对照 85 行来源包逐页核验：四字段预算的"计算预算而非秩限制"定位、
IntegerMatrix 构造顺序、BezoutTransform 的系数来源、关系格族的预检时序
（preflight_shape/from_i32_iter/try_collect）、saturated_kernel 的幺模右因子
与零对角元列构造及刻意不用有理行约化、reduce_basis_mod_two 只保留 Y/2Y
张成、negative_coweight_eigenspace 不再转置的理由、adapted_basis 的
observable-bearing 动机与逐字主元策略及条目总量预检——全部忠实。
7 页批准；wiki/concepts/ 现有 91 页，剩余候选 442。

## 2026年10月9日 第十三批候选审查：real-form-seed.md 的 7 页

对照 84 行来源包逐页核验：observable-bearing 选举链（stable_log 代表元 →
g_rho_check → 每个下游 torus_factor）、stable_log 四步与已检查前置条件、
基本余权公式与"按实际简单余根展开"的坐标约定、invert_rational/solve_mod_two/
fractional_part 三辅助、RealFormSeed 私有字段与 grading_offset 不变量、
build 四门控链、custom 分支的两项一致性——全部忠实。7 页批准；
wiki/concepts/ 现有 98 页，剩余候选 435。

## 2026年10月10日 第十四批候选审查：weak-real-form.md 的 7 页

对照 109 行来源包逐页核验：编号约定（canonical 坐标序最小元、class 0 为
quasisplit）、stage-(d) 排序审计与上游 RealFormNbr 的对齐及 adapter 边界、
walk_mask_orbits 的 FiberAction 转移规则与 u128 故意不饱和的规模比较、
MAX_MASK_BITS/CLASS_SENTINEL/seeded_class 三边界、代表元级归因内核的
投影-整性-grading-标签链与闸门顺序（整性门先于虚 grading 提取、覆盖每个
单根）、provenance/分解两门、Tits 搬运职责边界——全部忠实；11 个测试锚点
（含合成 A1 归因与 rank-33/rank-64 两种不同拒绝）一致。7 页批准；
wiki/concepts/ 现有 105 页，剩余候选 428。

## 2026年10月10日 第十五批候选审查：block-graph.md 的 8 页

对照 143 行来源包逐页核验：八值 BlockDescent 体系（0x4 规则、TAB 重编号、
对偶配对）、块级状态判定（complex 看 is_descent、i1/i2 看 cross 动不动、
r1/r2 看对偶侧）、两种平铺布局（descent 按 z*rank+s、cross 按 s*size+z）、
first_z_of_x 弱增不变量与 element() 校验、直接与逆 Cayley 共享槽与
weak-descent 互补语义、i1 单值/i2 双值构建与回填、dual() 的反转/互换/反射
算术与"第二像仅在有定义时映射"、无序集比较约定、Bruhat Hasse 递归与
n_bruhat_comparable 的拓扑序前提、HashMap 静默覆盖观察、A1 七测试锚点
（capture 3501519 对齐）与未覆盖清单——全部忠实。8 页批准；
wiki/concepts/ 现有 113 页，剩余候选 420。

## 2026年10月10日 第十六批候选审查：kl-polynomial-table.md 的 7 页

对照 138 行来源包逐页核验：KlPol 布局（零=空向量、trim 维持首一、degree(0)=0
需 is_zero 区分）、非负/首一属算法输出而非类型不变量、i32 无溢出通道的悬置
判断、coefficient() 越界返回 0 的过期文档以实际实现为准、quotient_by_1_plus_q
恒 Ok 的签名形态、去重池 0/1 种子与 Default 空池风险、primitive 投影与
UndefBlock 哨兵及零/恒等回退、mu 的 None 二义性、fill 的幂等与两条递归路径
的分派条件、运算集与上游逐条对应——全部忠实；4 个测试锚点与未测面清单一致。
7 页批准；wiki/concepts/ 现有 120 页，剩余候选 413。

## 2026年10月10日 第十七批候选审查：deformation-drivers.md 的 8 页

对照 151 行来源包逐页核验：SplitInteger 的 wrapping 算术与逐条上游对应、
两个 twisted KL 和的长度函数差异（扩展块自身 vs 父块）、KlSumParent/
DeformParent 的借用/拥有分工与存活纪律、lambda_rho 一次提供契约及 SL(2,R)
反例、IntegralBlockScope 三变体与 A1 ν=[1]/2 陷阱、奇异集在平凡 bm 下的
一致性条件、递归 twisted_deformation 的 flip/收缩/取消语义、
block_deformation_to_height 的逆向顺序与 consumed flags 及 plug_hole 填表
差异——全部忠实。8 页批准；wiki/concepts/ 现有 128 页，剩余候选 405。

## 2026年10月10日 第十八批候选审查：partial-common-block.md 的 7 页

对照 113 行来源包逐页核验：五构件及其上游对应、CommonContext 五个生成元
操作（status 的布尔旗标随根类型而异、cross 的 pos_to_neg 平移修正、
up_cayley 的 α_s/2 奇偶修正）、StandardReprMod 两条构造路径、bruhat_below
→ PartialBlock::build 的区间消费与 (length,x,y) 终排（与 oracle 行号一致）、
访问器的 None=UndefBlock 语义、survives 判定、dual() 的部分块限制
（外出链接保持未定义、KlTable 拒绝、BareBlock 无参数池对偶）、assert 省略
约定——全部忠实。7 页批准；wiki/concepts/ 现有 135 页，剩余候选 398。

## 2026年10月10日 第十九批候选审查：strong-real.md 的 6 页

对照 81 行来源包逐页核验：SquareClassId 的商空间坐标约定与 stage-(d) 审计的
上游对齐及换基不变量边界、StrongRealFormRep 的轨道编号依赖与大小不变性、
StrongRealData 访问器族（含 wrf_preimage_mask 方程与 square_class_representative
的两处选举差异）、fiber_size 的 Some(0) 求和语义、StrongRealClassPrint 三字段
与跨类重复编号、MAX_MASK_BITS 资源边界——全部忠实。6 页批准；
wiki/concepts/ 现有 141 页，剩余候选 392。

## 2026年10月10日 第二十批候选审查：rep-table.md 的 6 页

对照 85 行来源包逐页核验：ReducedParamKey 三字段的构成与私有化、块复用的
Weyl 姿态差记录（block_modifier/make_relative_to）、LocatedBlock 各访问器与
has_identity_generator_attitude 门控、两个 lookup 入口的语义差异、
with_kl_table 的全回调持锁与 ActiveKlCallback 重入禁令、k_type_formula 的
严格身份键/更高截断复用/锁外计算提交复核（并与 with_kl_table 的锁范围明确
区分）——全部忠实。6 页批准；wiki/concepts/ 现有 147 页，剩余候选 386。

## 2026年10月10日 第二十一批候选审查：block-access-modifier.md 的 8 页

对照 115 行来源包逐页核验：BlockTopology 的密封模式与四项结构不变量
（rank≤32、非降长度、格子存在、链接目标<size）、双层 None 语义、
PartialBlock 的参数交换与下降门控编码、bruhat_hasse 三个分支的分量取舍、
transform_srm 的逐字母分派与偏移规则（Complex offset=0、Real offset=分母、
Imaginary 报错）、reduced_word 有意偏差与逐字母互逆性、make_relative_to/
sr_with_modifier 的顺序、simple_reflect_numerator 的 checked 算术、
BlockModifier 字段语义与 u32::MAX 哨兵、两个集成测试细节——全部忠实。
8 页批准；wiki/concepts/ 现有 155 页，剩余候选 378。

## 2026年10月10日 第二十二批候选审查：matreduc.md 的 8 页

对照 93 行来源包逐页核验：逐操作复现的动机（被选解的 τ/t 奇偶性进入
ext_block::same_sign）、wrapping i32 镜像 C++ int 含溢出域、divide 的
负被除数分支避开 i32::MIN 取负、gcd 的 flip/记录矩阵符号纪律、diagonalise
簿记怪癖（覆盖赋值/^=/退出后 ^=/pull_back_columns/首项负号归一）与
行列式口径以测试为准、has_solution/find_solution 的失败差异、assert 的
panic 面、exp_i 的 debug_assert 前置、oracle_reference_cases 逐字节锚点
（含 6×6 秩亏案例的解字面量）、未测面清单——全部忠实。8 页批准；
wiki/concepts/ 现有 163 页，剩余候选 370。

## 2026年10月10日 第二十三批候选审查：weyl-transducer.md 的 7 页

对照 72 行来源包逐页核验：parabolic-subquotient 表示与固定栈数组的零堆
分配、WEYL_MAX_RANK=32 是表示上界而非枚举预算、表编码的 shift/transduction
分界、CompactWeyl::new 三步（分类/反转 BCD/逐生成元建表）、d_out 与
piece_offset 的编号分工、coxeter_entry 的分派表（含 BC(0,1)/F(1,2)=4、G=6；
D/E 分叉未展开如实声明）、canonical_word 的重建-拼接-映射回流程与输入词
无关性、piece_root_permutations 免矩阵、E6 数字为文档转述而非性能结论——
全部忠实。7 页批准；wiki/concepts/ 现有 170 页，剩余候选 363。

## 2026年10月10日 第二十四批候选审查：root-system.md 的 8 页

对照 139 行来源包逐页核验：RootId 三表索引对齐与访问器越界语义、枚举流程
（预算四步检查 → 自有快照 → 播种 → BFS → BTreeMap 键序定终序）、Closure
插入的两道防御检查与基数拒绝、预算公式的 u128 饱和算术与兼容包装器的错误
映射、梯子底表的定义/二分/坐标映射、3868832 修复的"溢出即非成员"语义及其
边界（仅限该查询路径、其它错误照传）、RootSet 的只读位图与不可达 panic 面、
测试锚点与 11-vs-8 数量差异的待核记录——全部忠实。8 页批准；
wiki/concepts/ 现有 178 页，剩余候选 355。

## 2026年10月10日 第二十五批候选审查：root-ladder-overflow-repair.md 的 6 页

对照 143 行来源包逐页核验：3868832 发现（原版接受全部 11 个 A1+环面
坐标边界案例、Rust 溢出拒绝六个）、"可表示的相反根之差不可表示则必不在
i32 根集内"的推理链、修复仅限 build_ladder_bottoms 成员查询单点
（combine_roots 与 i128 反射路径不动、禁用 wrapping/saturating）、原版
RootSystem 在压缩抽象单纯根坐标中构造梯子底（环境格尚未存在，环面坐标
不参与减法）与 Rust 逐对相减的结构性差异、tests-first 链（BEFORE-v3
job3873400 证明两域一核三处失败 → 修复 → AFTER-v3 job3875239 全绿）、
账本状态歧义（前文称接受范围不含 index 登记、后文记载 entry
0003-a1-torus-root-coroot-ladder-boundary 已 accepted/math_pass——候选
如实标注为歧义而非擅自消解）、"不授予性能/排名/更广数学验收"的限定声明——
全部忠实。6 页批准；wiki/concepts/ 现有 184 页，剩余候选 349。

## 2026年10月10日 第二十六批候选审查：cartan-fibers.md 的 8 页

对照 171 行来源包逐页核验：子商公式 ker_F2(I+θ_Y)/red_2 ker_Z(I+θ_Y)
与 low-pivot 坐标约定（同构声明如实标注为注释声明）、先分母后分子的构造
顺序与预算执行点、分子按行不转置加 from_ones XOR 实现对角 +I、元素
ptr_eq 绑定的 provenance 语义与 CartanFiberMismatch 锚点、canonical 代表
与基代表 XOR 关系、validate_induced_map 的分子/分母双层下降与按需应用、
伴随构造五门控（DatumMismatch → InvolutionMismatch → 分配前预算 16r²+rn
与 2n²r → 逐根作用矩阵按列写入 → transpose_square 的逆转置论证如实标注）
——全部忠实；A2 twisted 作用矩阵字面量与 transpose 关系核验一致，覆盖缺口
（DatumMismatch 无锚点、from_source 隐含一致）如实保留，跨页链接目标全部
可解析（两页为已上线页、六页为本批或管线内候选）。另修正 kb/AGENTS.md 的
编译器 pin 记录与 package.json 的 1.4.2 保持一致。8 页批准；
wiki/concepts/ 现有 192 页，剩余候选 341。

## 2026年10月10日 第二十七批候选审查：cayley-cross.md 的 7 页

对照 111 行来源包逐页核验：分解四分量与"cross_word 存生成器下标而非
RootId"的区分、跨 port 不唯一与 replay-invariant 比较契约、provenance 双门
（weight/coweight 两侧比较，支撑 w^{-1}=δwδ 终止论证）、lowest-external-
descent-first 剥离与预算检查时点（找到下降后、步进前，故零需求输入预算 0
可通过；Complex 一步两次反射只计一步）、逆序重放时 Cross 反射已收集 Cayley
根、长根化的 B2 对替换与终止性如实标注为源码声明、六种不变量错误无专门负
测试的覆盖缺口、整对合分类的检查顺序（形状 → 预算门 → i128 checked
is_involution → checked_add(1)）与 compact+2·complex+split=n 恒等式、
奇偶测试矩阵字面量、fiber_rank 的 saturating_sub vs checked_sub 阅读观察
及其零测试状态——全部忠实。7 页批准；wiki/concepts/ 现有 199 页，
剩余候选 334。

## 2026年10月10日 第二十八批候选审查：locator.md 的 8 页

对照 113 行来源包逐页核验：三类型分工与"纯、未接线移植"状态、int_item
六步流程（alcove 根格顶点平移 checked 算术 → factor_dominant 无迭代上限、
终止性依赖根系理论的如实标注 → 正墙对 0/负墙对 −denominator 的命中检测 →
逆序遍历反射词、rem_euclid 非整字母左乘并消去 → 余根坐标加法闭包取正部
作驻留键 → w.image 的 provenance/positivity 两道不变量）、upstream
RootNbr 序（高度+简单坐标反字典序）与 crate RootId 环境字典序的刻意存储
偏差（作者声明语义无偏差）、make_relative_to 的右乘逆与
simple_pi[j]=old[inv[j]]（手算 [2,1,0]∘[1,2,0]⁻¹=[0,2,1] 复核一致）、
usize::MAX 哨兵、B2 余根闭包回归（根加法 4 vs 余根加法 8，
LOCATOR_COROOT_REGRESSION）、A2 两切片驻留不同 item 的"典范性依赖 alcove
而非仅整根系"记录、IntegralDatumTable 不持有 RootSystem 的未定义风险与
debug_assert 仅 debug 生效等边界——全部忠实。8 页批准；
wiki/concepts/ 现有 207 页，剩余候选 326。

## 2026年10月10日 第二十九批候选审查：minimal-torus.md 的 8 页

对照 108 行来源包逐页核验：elected_square_root 的门序
（RankMismatch→DatumMismatch）、word.iter().rev() 重建与 from_action
往返钉字方向、运输顺序（先 distinguished 余权矩阵后元素余权作用）与
stable_log 唯一预算透传；minimal_torus_part 的四道入口门（含 actual 取两
长度较大者、rank>63 的 mask bits 预算门）、torus-part 整性与奇偶置位、
首个左下降生成元（Real→逆 Cayley 且 Ok(None) 亦错误，否则 cross_pregated）、
末尾幂等 coset.reduce 的决定性情形、"逐步约化不动最终类"如实标注为注释
声明、逐位置配对 vs 上游前导段约定的翻译差异及其一致性边界、轨道游走的
LIFO+BTreeSet<u64> 与 2^rank 上限、ModTwoVector 整数序选举、空候选具名
错误对应上游断言、测试覆盖边界如实记录（三正例皆 coch==factor、测试 3 与
测试 2 首组输入相同、encode 的 Result 为预留）——全部忠实。8 页批准；
wiki/concepts/ 现有 215 页，剩余候选 318。

## 2026年10月10日 第三十批候选审查：extended-block.md 的 7 页

对照 95 行来源包逐页核验：DescValue 32 值三族分类、is_descent 对应奇数
枚举值、generator_length 按族 1/2/3、零链接类型（OneRealNonparity/
OneImaginaryCompact 不记录 cross action）、has_october_surprise 的定义式
与 2016-10 注释、ExtGen 的 usize::MAX 哨兵对应上游 ~0、fold_orbits 的
cartan[i][j]=<α_i,α_j^v> 约定与轨道按 s0 递增、全块 build 在平凡 bm 下
transformed_twisted 退化为双 kgb.twisted、build_partial 的
x+gamma_lambda 不动点测试（y 是合成子系统计数而非对偶 KGB 元素）、子系统
Cartan/twist 上的 fold、cofold 在 complete_construction 之后且当前仅恒等
姿态（非恒等 simple_pi 显式失败）、element(zz) 的下界查找语义与
is_present 的成员判定分工、length(n)=parent.length(z(n))、StarOracle 注入
边界与 debug_assertions 对应 #ifndef NDEBUG、dirty 工作区快照如实记录——
全部忠实。7 页批准；wiki/concepts/ 现有 222 页，剩余候选 311。

## 2026年10月10日 第三十一批候选审查：extended-kl.md 的 7 页

对照 118 行来源包逐页核验：池/符号分离（KlHashTable 条目 i32 KlPol 对应
上游 IntPolEntry，索引不打包符号位，prim_flip 独立 bitmap，raw_ext_KL 的
inx.second ? -inx.first : inx.first 渲染层）、DescentTable 预计算
（is_descent 置 descents、否则 !has_double_image 置 good_ascents 即"至多
一个向上邻居"、rank>MAX_FOLDED_RANK 的资源拒绝）、prim_index 逐 mask 递减
构造（首个 good ascent、like-nonparity/跨 partial-block 边记 DEAD_END、沿
cross 继承按 epsilon 差置 flip、收尾反转为递增）、very_easy/easy 集合与
is_extremal（D(x)⊇D(y)）/is_primitive（G(x)∩D(y)=∅）判定、候选中
"extremal 蕴含 primitive"的推导复核成立且反向限制如实声明、qk_plus_1/
qk_minus_1/qk_minus_q 辅助多项式、mu(i,x,y) 取 q^{(ℓ(y/x)-i)/2} 系数
（i=1,2,3）、fill_columns 的 limit==0 约定与"出错清空出错列并传播"对上游
catch(...) 的有意偏离（候选如实指出偏离清单的"不改变可观察结果"概括不覆盖
失败路径行为差异）、未移植项（get_M/down-set defect/check_polys/共享池
swallow/StandardRepr 侧 ext_kl_matrix 前段）——全部忠实。7 页批准；
wiki/concepts/ 现有 229 页，剩余候选 304。

## 2026年10月10日 第三十二批候选审查：kl-support.md 的 8 页

对照 79 行来源包逐页核验：RankFlags 的 u32 私有位集与 rank≤32 硬上限、
set/is_set 无边界检查由构造门控兜底、contains 是超集判定、非 Copy；
validate_topology 集中检查（rank/长度存在且非降/逐生成元 descent·cayley·
inverse_cayley 存在/cross 与 Cayley 像在块内）；ImaginaryTypeII 既非下降
亦非 good ascent 的三分类；length_stop[l] 首个长度≥l、末尾追加 size、
max_length 显式丢弃记为清理候选；is_primitive（good(x)∩desc_y 为空）与
is_extremal（desc(x)⊇desc_y）一行组合语义；unique_ascent 按类型取 cross
像或第一个 Cayley 像；prim_back_up 先自减再判定、失败时 *x 已置 0 的原地
契约；prepare_prim_index 幂等降序扫描、DEAD_END=usize::MAX、结尾
哨兵→range、其余→range-1-slot 的反转、"上升像序号更大"如实标注为无防护
的阅读观察、四访问器未 prepare 即 panic（仅 prim_index 文档显式声明）；
测试覆盖边界如实（4 个 FakeTopology 锚点、成功路径/判定/索引机制无单元
测试、经 KL 层 HPC 门覆盖的措辞保留）——全部忠实。8 页批准；
wiki/concepts/ 现有 237 页，剩余候选 296。

## 2026年10月10日 第三十三批候选审查：ktype.md 的 7 页

对照 87 行来源包逐页核验：KType 表示（lam_rho 恒为 (1−θ_x)X* 陪集的
lambda_unique 当选代表、规范化只在 sr_k 一次、height 预计算、(1+θ)λ 公式）、
new 不校验不变量而 crate 外只能经 sr_k 的纪律划分、theta_plus_1_eval 四
项公式、六谓词前提（is_nonzero 假设 is_standard 但不检查、is_normal 因
total 而不查四联前提、is_final 的 ic/Real 奇配对/Complex 下降拒绝与 inc
放行）、三变形的终止预算（weight_defect / 图大小"慷慨"界 /
weight_defect+图大小+1）与各自错误名、finals_for 的五分支结构（含
type-2 移位项 λ_ρ+α、Real 的 shift=(eval+1)/2 投影与逆 Cayley 分裂、
None→"parity real inverse Cayley"）、height 来源不对称（todo 项重算 vs
结果项沿用）如实标注为阅读观察、simple_reflect 第三参数 im_wt=0/lr=1、
kgp_set 的 Levi 生成元静默跳过/先 second 后 first/shift=eval/2 不查奇偶、
equivalent 的同 Cartan 类→双 to_canonical_fiber→严格相等流程、测试锚点
普查（9 个，其中两个 su21 测试仅 eprintln! 无断言如实标注为观察型）与未
覆盖清单——全部忠实。7 页批准；wiki/concepts/ 现有 244 页，剩余候选 289。

## 2026年10月10日 第三十四批候选审查：real-weyl.md 的 8 页

对照 196 行来源包逐页核验：RealWeyl 根列表统一存 primal RootId、
imaginary/real 按 upstream RootNbr 键排序而 complex 保留
makeSimpleComplex 输出序、对偶侧 real_compact/real_orth 经余根向量映回、
real_type/real_compact_type 用转置子系统 Cartan（B/C 互换入口，Sp(4,R)
Cartan#3→B2 锚点 (2,3)）、real_r 填对偶侧 R-群向量而 imaginary_r 填
primal 侧的交错归属、fiber_side 的 grading 线性扩张与 parity_dot 平移、
Σbracket 必偶否则不变量错误、simple_basis 的"候选移除即终止外层扫描"
上游怪癖保留、r_vectors 按自由列升序的核生成元序（[free]+含 free 位的
主元行）、复根生成元 s_rn·s_θ(rn) 的构造顺序、dual_side 每次调用重建无
缓存的原因（−θ 只是典范对偶代表之共轭 tw·w0）与"是否有意未确认/仅性能
线索"的如实标注、七 fixture 表与 rev 4d3e9449/2026-08-11 逐字节复制的
测试注释声明、冒号不一致的字节契约、format_word 空词 e 与 1-based 逗号、
两个按构造不可达的 .expect、printDualRealWeyl 与 NDEBUG 尺寸断言未移植——
全部忠实。8 页批准；wiki/concepts/ 现有 252 页，剩余候选 281。

## 2026年10月10日 第三十五批候选审查：weyl-size-presentation.md 的 7 页

对照 84 行来源包逐页核验：weyl_order_of_cartan 的 pub(crate) 定位与阶商
公式 |W|/(|W_im|×|W_re|×|W_cx|) 用途、B/C 同阶 2ⁿn! 故不区分取向、精确
Integer 算术的理由、NonSquareCartan/环面行列贡献阶 1/checked_mul 唯一受检
算术、m=3/2/1 分派表（G2=12、F4=1152、B/C 链、A=(n+1)!、D=2ⁿn!/2、
E6=51840、E7=2903040、E8=696729600）逐数字复核、branch_lengths 无环检测
与三处校验留白（单节点任意非零对角返 2、负乘积计度数不提升重数、rank>4
双键链不查位置）如实标注为阅读观察、factorial 的 Result 为预留、
presentation 四状态位判定（ms_tau 逐元素比 ±1、quasisplit 只按 external
编号、connected 委托对偶分量群平凡性）、扫描先于秩检查且失败整体丢弃无
副作用、LayoutInvariantViolation 四 reason 与 "most split Cartan" 两处
共用、两文件互不导入仅共享 StructureError 两条再导出路径、
CartanClassification 是否调用不可见的诚实声明、测试锚点（A4=120 至空系统
=1；sc A1/adjoint A1/sc B2 三展示案例）与未覆盖清单——全部忠实。7 页批准；
wiki/concepts/ 现有 259 页，剩余候选 274。

## 2026年10月10日 第三十六批候选审查：mod-two.md 的 7 页

对照 108 行来源包逐页核验：ModTwoVector 与 Malachite 层刻意分离、填充位
清零支撑派生 Ord 的健全性（仅 map 键、非数学序）、from_ones 重复下标偶次
抵消（[0,63,64,127,128,63] 锚点）、dot 的逐字 parity 折叠与 pub(crate)
可见性、ModTwoSubspace 的最低置位 pivot（沿用 BitVector::firstBit()）、
insert 先约化再消旧行保持序无关 RREF、reduce 升序扫描对齐 normalSpanAdd
的注释、right_kernel 的 v_f=e_f+Σe_p 规则与其重新约化同 pivot_rows 直接
供 real_weyl R-group 读位的接口差异、CanonicalModTwoSection 保留首批独立
列而丢弃依赖列（固定可观测实形种子代表）、u64 掩码 64 列上限、
solve 的 then_some 语义与 2^12×8 穷举"解=数值最小源掩码"、
ModTwoSubquotient 的 crate 私有定位与四道构造校验、validate_induced_map_to
的分子/分母双查与"只查商基代表会看似成立"的注释论据、12 测试锚点与未
覆盖清单（5 处不变量违规/两个 relation/dot 无直接测试/文件内无调用方）
——全部忠实。7 页批准；wiki/concepts/ 现有 266 页，剩余候选 267。

## 2026年10月10日 第三十七批候选审查：primitive-involution.md 的 7 页

对照 149 行来源包逐页核验：纯表定位（不涉及 root datum/Weyl/inner-class
管线）、perm 方向（扁平位置 k → 矩阵下标 perm[k]，对应 Layout::d_perm）与
包装器侧 checked_permutation 的职责划分、字节级解析（char::from(u8)、
绝不 UTF-8 解码、skip_punctuation）、诊断顺序钉住"未知符号先于因子计数"、
五变体 Display 逐字节复刻上游文案（含反引号开单引号合）、's' 坍缩恰在
−1∈Weyl 群处（A1/B/C/偶 D/E7/E8/F/G）与存活条件、'u' 的三路分派、'C'
消耗两个相同连续因子、逐字母表（A 反对角/奇 D 与 u 换末两顶点/E6 固定
1,3 换 0↔5,2↔4/T 负恒等/其余坍缩臂仅防御作用）、不返回 Result 的前置
条件契约与两道 debug_assert_eq、on_basis 的有理逆+三重循环+floor 整性+
i32 收窄四类失败折叠为同一 None（包装器重标不兼容格）、B⁻¹MB 手算复核
（[[1,1],[0,-1]] 一致）、偶子格 [2] 上单位阵不变、测试缺口四项如实——
全部忠实。7 页批准；wiki/concepts/ 现有 273 页，剩余候选 260。

## 2026年10月10日 第三十八批候选审查：involution-types.md 的 7 页

对照 156 行来源包逐页核验：三层类型的职责阶梯（配对保持对合 → 根置换+
余根运输 → wθ 再为对合）、LatticeInvolution 门序（方阵 → W²=I、C²=I 短路
i128 → W^T·C=I 配对保持）、anti_invariant_rank 的 (r−tr θ)/2 精确式与负/
奇拒绝、RootInvolutionData 的 DatumMismatch→RankMismatch→单根级先于主循环
的错误优先级、余根运输排除"固定所有根却移动余根中心坐标"的设计动机测试、
分类优先级（自身→Imaginary、checked_neg 负根→Real、否则 Complex）、子系统
单根提取（继承正系、候选−成员差为正坐标向量则跳过、输出按 RootId 升序）、
TwistedInvolution 先三个 datum 一致性后秩检查、合成结果重走完整门控、
distinguished 不存储、compose_matrices 的 actual 恒为 right.len() 阅读观察、
全部测试锚点数值（pair=−34 手算复核一致、A2 负反对角 Real=2/Complex=4/
Imaginary=0、[id_of([1,1])] 等）——全部忠实。批准过程中发现两处候选把
1x1 矩阵写成裸 `[[-1]]` 字面量，与 wiki 链接语法碰撞被批准门拒绝
（broken citation targets）；已改写为 pmatrix 形式并复核后批准，教训记入
kb/AGENTS.md 第 5 条。另有四个未来批次候选存在同类模式，到批处理。7 页
批准；wiki/concepts/ 现有 280 页，剩余候选 253。

## 2026年10月10日 第三十九批候选审查：real-form-labels-order.md 的 8 页

对照 112 行来源包逐页核验：RealFormLabels 五道构造门控的顺序与错误名、
Cayley 回拉翻转位（root+alpha 为根的个数为奇则翻转）、根列表 cross 运送
须全为 distinguished 虚根、base_grading_extension 的转置 bracket 子 Cartan
精确求解与"整性不是虚根判据"的显式门控、增广 ModTwoSubspace（根位置+哨兵
位）与余量置位→ImpossibleGrading、quasisplit 首标签锚点（空分区必然触发）；
ExternalFormOrder 的 depth 升序+specialGrading 位集平局、Rust 断言严格序
而上游是不稳定 std::sort、"compact 深度 0 为 external 0"仅文档声明显式
区分、DepthTables 的 M=C^T 有理逆列和与整性闸门、贪心极大正交集的短根对
紧性翻转、候选如实标注"极大≠最大基数无证明"、depth 内无效果空转循环的
死代码观察、special_grading_key 的 >= 替换取最大 popcount 最高下标与
取补 unslice、verified_generator_map 逐位校验实际有序基（非抽象双射）、
MAX_KEY_GENERATORS=127 与 MAX_MASK_BITS 职责分离、两文件互不导入与
"局部→内部→外部编号"串联为未验证推断的诚实声明、E6 [1,3] 期望来自
置换而非 Rust 输出——全部忠实。8 页批准；wiki/concepts/ 现有 288 页，
剩余候选 245。

## 2026年10月10日 第四十批候选审查：twisted-involution-trio.md 的 6 页

对照 92 行来源包逐页核验：TwistedInvolution 的四步门槛（datum 三连查 →
三个 RankMismatch（WeylAction 用 rank() 余用 lattice_rank()）→ 双格复合
w 左 θ 右 → 重走 LatticeInvolution/RootInvolutionData 门控，InvalidInvolution
为传播而非本体构造点）、compose_matrices 的 actual 恒报 right.len() 怪癖、
分类的预算闸门先于对合检验（临时矩阵即 drop 不抬 live-entry 记账的注释
明示）、complex/compact/split 公式与三处 checked_sub、entry%2!=0 含负奇、
fiber_rank 公式 dim ker((q+I) mod 2) − dim span(plusBasis(q)) mod 2 及
其防御不一致（saturating_sub 钳零 vs checked_sub 报错、普通减法与普通
i32、debug 溢出 panic 无防护）如实标注为阅读观察、reflection_word 唯一
移植点与禁止私有拷贝、to_dominant(reflection(α,2ρ)) 原样反转、贪心首个
负配对无迭代上限、wrapping 静默回绕+debug_assert 长度+release zip 截断、
三文件算术策略分层-vs-漂移存疑备查、未测路径清单——全部忠实。6 页批准；
wiki/concepts/ 现有 294 页，剩余候选 239。

## 2026年10月10日 第四十一批候选审查：grading.md 的 6 页

对照 94 行来源包逐页核验：Grading 作为 ModTwoVector newtype 的位语义
（第 i 位=第 i 个 simple-imaginary 根、置位=NONCOMPACT、与 ambient/adjoint
坐标维数可同而必须由类型区分）、quasisplit 规范化（零元基点全一、其余为
canonical 代表的仿射线性求值即逐根 !dot）、逐虚根收集（m_alpha=余根的
ambient mod-2 像、伴随像经 Π(y)_j=⟨α_j,y⟩ 投影且配对只保留投影内单一
实现、simple_mod_two 的 %2!=0 含负奇数、B2 (2,−1)→(0,1) 锚点）、刻意不
接收 ambient fiber 参数而对着 AdjointCartanFiber::ambient_fiber 构建的
来源绑定（值相等不能表达 fiber 同一性）、ensure_faithful_shifts 的上游
断言→无条件拒绝转变与"无已知公共路径"的防御声明、element_from_grading
的增广消元（marker 位=imaginary_rank+adjoint_basis_index、右端 target XOR
base 标记 compact 位、低位余量置位→ImpossibleGrading、marker 位 xor_assign
汇总）、唯一性=faithful 不变量但不保证可实现、9 个测试锚点（含 A2 根序
index 0=α₂、A1×A1 imaginary_rank==0、33 个 A1 因子突破打包位宽）与未
覆盖清单——全部忠实。6 页批准；wiki/concepts/ 现有 300 页，剩余候选 233。

## 2026年10月10日 第四十二批候选审查：dynkin.md 的 6 页

对照 115 行来源包逐页核验：classify 输入契约（方形/对角 2/非对角
-3..=0/零模式对称但不校验数值配对）、first-fresh-vertex 顺序与
first-match 合并的分量构造（分量按最小顶点升序）、秩二特判
（乘积 2 时 cartan[i][j]==-1 判 C 否则判 B 且保序——历史 B2/C2 编号
教训落点；乘积 3 判 G 且 cartan[i][j]!=-1 时交换使短根在前）、高秩度数
分析（度≥4 报错、端点<2 报环、label 3 报 oversized G、第二多重边报错、
多重边与 fork 并存报错、lower/upper∈端点分判 B/C、否则 F、无多重边时
|star[fork]∩端点|==1 判 E 否则 D）、各型起点选择（A 最小端点、B/C 移除
lower/upper、D4 任取/高秩 D 取长臂末端、E 的短臂唯一交点与长臂超两步交换、
F 移除 lower 邻居）与逐步最小未访问邻居遍历、D 型中断只补 fork 短臂、
bourbaki_permutation 的 result[i]=占据 Bourbaki 位置 i 的原顶点方向、
folded_cartan 的 cofold 公式（注释声称边重数相同未验证如实保留）、长度 2
轨道取第一成员余根/长度 3 取两者和、C(i,j) 的下标方向（i 选余根轨道、j
选根轨道）、IndexOutOfRange{index:a.max(b)}、不校验轨道完整性/输出合法性、
6 个全成功路径测试与 E7/E8/高秩 D/错误路径无锚点——全部忠实。6 页批准；
wiki/concepts/ 现有 306 页，剩余候选 227。

## 2026年10月10日 第四十三批候选审查：lib-root.md 的 7 页

对照 66 行来源包逐页核验：crate 定位（只含数学值、作为解释器 domain
values 的适配边界）、60 mod 声明与 5 pub mod/2 个 allow(dead_code) 注释
（global_tits 的消费者在 synthetic builder、weyl_size 停放 task #9）、
52 条 pub use（topology 再出口的位置怪点、integer_lattice 两条、deform
双重暴露）、5 个零再导出模块、三个根部错误类型、pair_coordinates 内部
使用不导出、A1 原型层全 pub(crate) 且自述 pending replacement、
LatticeVector 无校验将被 Weight/Coweight 编译期区分取代、原型 RootDatum≠
BasedRootDatum 的同名界限、构造校验顺序（EmptyRootDatum→外部校验器先
传播→NonSquareCartan→InvalidCartanMatrix）、单余根=Cartan 第 j 列、
from_basis 行主序配对核对、roots() 的 FIFO BFS/i128 收窄/4096 上限/字典
序确定性、PrototypeWeylGroup 的作用像键 BFS 与 65536 上限、act_on_root
逆序、compact_imaginary 未校验标志已被 Grading 取代、simple_real_rank
刻意窄于 real rank、3 测试锚点（i32::MAX 恰报 ArithmeticOverflow 间接
要求 StructureError: PartialEq）——全部忠实。7 页批准；wiki/concepts/
现有 313 页，剩余候选 220。

## 2026年10月10日 第四十四批候选审查：lattice-types.md 的 7 页

对照 93 行来源包逐页核验：四格类型（Weight/Coweight 独立 newtype 不可
互换、RationalWeight 公共分母 Vec<i64>+i64、RationalCoweight 逐坐标
Rational）、checked 固定宽度存储+领域边界转换的设计决策、pair 先秩查再
委托 pair_coordinates（i128 累加 i32 收窄、zip 截断靠调用方）、
checked_mul(sign) 拦截 i32::MIN*-1、try_reserve_exact 预算纪律与
halve/normalized/to_rationals 的裸 clone 例外、构造即 gcd 归一与拒绝
非正分母（上游 normalize 只拒零的对照保留）、halve 刻意不归一、
apply_matrix 保持分母只作用分子、integral_coordinates 的断言可检查化、
dot_coroot 返回已约分 (i64,i64)、两处错误字段怪癖如实保留
（dot_coroot 的 expected/actual 反序、apply_matrix 的 actual=matrix.len()）、
防御性溢出口不可达的备注、gcd_u64 两处实现漂移（gcd(0,0)=1 vs 0，各自
调用点因分母恒正而安全但已漂移）、RationalCoweight 无算术无 Hash 与
malachite 不进公开 API——全部忠实。7 页批准；wiki/concepts/ 现有 320 页，
剩余候选 213。

## 2026年10月10日 第四十五批候选审查：alcove.md 的 8 页

对照 190 行来源包逐页核验：分母界守卫（rank<63 且 denominator>2^rank；
rank≥63 一律 false 的移位理由与 rank62/63/64 测试边界）、RootNumbering
（level 升序+从末坐标向前的反向字典序、正根占 [npos,total) 负根镜像占
[0,npos)、positive_index 缺键即 panic 与 id 越界 panic、unwrap_or 静默
兜底 vs ok_or? 的风格差异如实标注未确认）、wall_set（负根整值改写为
denominator 的不对称约定、integrals⊆walls、α∨−β∨ 是 coroot 则丢弃 β 的
逐层筛除与 n_min saturating_sub）、barycentre_eq（整值墙保持 (0,1)、
非整值墙 (1, n_off*label)）、labels_for_component（恰一个自由列否则
"alcove wall component must have one coroot relation"、首元素为负才取负
而首元素为 0 不翻）、alcove_center 六步（墙行系数 coroot×scale、radical
行、None→"no unique solution"、checked_lcm 通分、i64::try_from(&scaled)
对整个有理数转换而非 numerator_ref——signed-rational 教训落点、−θ 不动
子空间校验、sr_gamma 重建）、root_vertex_of_alcove 显式忽略 integrals 且
用朴素 div_euclid（注释强调非负根修正版 floor_eval）、root_vertex_simple
（丢第一面系数 1 墙、转置子 Cartan、C^{-T}·floors、labels_1 重试循环、
"outside the root lattice"）、solve_rational_system/rational_inverse 的
None/Ok(None) 语义与 d>0 未断言、潜在 panic 路径清单与测试稀薄如实——
全部忠实。注意：候选 ada058cd 把 wall_set 链向了解释器层的
[[Alcove 墙集与整值墙筛选]]（另一包页面），内容无误但链接指向邻层，
记录为观察留待该包批次复核。8 页批准；wiki/concepts/ 现有 328 页，
剩余候选 205（另 7 页属 atlas-core-root-numbering-alcove.md，随其批次审）。
（本行原误写 219，已按 candidates 目录实际计数更正。）

## 2026年10月10日 第四十六批候选审查：layout-restricted-roots.md 的 6 页

对照 93 行来源包逐页核验：InnerClassLayout 三要素（Lie type 打印序+
Complex 对相邻+每中心环面维一个 T1、每条目一字母 'C' 耗两因子、perm[k]
方向=规范化序第 k 个单根的 datum 下标）、budget 只约束环面 Smith 基、
build 流程（twist_permutation 的逐根 id_of/image/回查/去重全归
LayoutInvariantViolation）、inner_class_letters 三路判定（逐点固定→'c'、
留分支内→偶秩 D 'u' 否则 's'、映到别支→'C' 配对旋转+"non-matching
Complex factor"）、上游移位顺序怪癖逐字复制且无可观测夹具如实记录、
torus_ranks 的 adapted_basis 商对合读出与 'c'/'C'/'s' 追加顺序、
RestrictedWeight 的 (1−θ) 编码是商到像格的单射而非环境坐标（split A1
alpha 类编码为 2alpha 但不等于 2alpha 类的文档原例）、restrict/doubled
的 checked 算术、纤维聚合 BTreeMap 与 roots 键序、rank 取自
anti_invariant_rank 而非纤维计数、is_multipliable 判二倍类、三测试锚点
（含 A2 [[0,-1],[-1,0]] 下 lambda 纤维 multiplicity 2 可乘）、两文件
互不调用与并行消费者关系如实标注为推断——全部忠实。6 页批准；
wiki/concepts/ 现有 334 页，剩余候选 199。另：随本批提交第四十五批
log 行的计数更正（219→205）。

## 2026年10月10日 第四十七批候选审查：error-global-tits.md 的 7 页

对照 162 行来源包逐页核验：StructureError 53 变体（22 无字段+31 带字段）
与六大家族划分、invariant 家族 14 变体的子系统前缀清单与 Display 模板、
IntegerLatticeResourceLimit 的 limit 为 u64 全家唯一、RootPairingMismatch
唯一含 i32、SimpleCorootImageMismatch 唯一含 Weight 且 Display 用 Debug
格式、NotYetImplemented 唯一带文档注释的"大声报错而非错误近似"语义、
error.rs 无构造点/无测试的声明；GlobalTitsElement 的完整有理余特征保留
（含中心坐标）与 [0,2) 典范代表、new 的三步门序、validate_context 的
datum→w·δ 矩阵比较顺序、crossed_generator 每次重验上下文与三分支更新
（Complex 反射/Imaginary 整性门槛 InvalidStrongTorusFactor/Real 不变）、
Weyl 侧 s_i∘w∘s_{δ(i)} 重建、crossed_word 前向顺序与不预检生成元、
规范化式 x−2⌊x/2⌋ 与全部数值锚点手算复核（(−1/2,9/2)→(3/2,1/2)、
(0,7/3)→(1,1/3)、A2 (1/3,1/2)→(5/6,3/2)+s1∘s0∘s1）、10 测试与未覆盖
清单、潜在 panic 面阅读观察——全部忠实。7 页批准；wiki/concepts/ 现有
341 页，剩余候选 192。

## 2026年10月10日 第四十八批候选审查：global-kgb.md 的 8 页

对照 113 行来源包逐页核验：GlobalTorusElement 的"构造入口约化、反射不再
约化"算术历史纪律（B2 元素 15 的 [0,-1]/2 锚点）、add 的 lcm+gcd+每坐标
至多一次 [2,4) 条件减法、negative_at 整性门槛与奇为紧、fingerprint 的
θ+I 饱和像适应基投影+rem_euclid 与幺模左因子无损性论证（注释声明如实
标注）、fundamental_fiber 与 build_owned 同公式保基序、from_ones 对角
xor、square_class_generators 的非主元位置升序当选、fundamental_coweights
的 [C|I] 精确消元与 lcm 公分母、build 六阶段（A 装 Cartan→B 长度区间 BFS
与 hasTwistedCommutation 的 (change>0)==has_left_descent→C 包元数据与
format_involution_word 字符怪癖→D 播种 2^generators×2^fiber_rank 全落包 0
与 (identity_id, fingerprint) 去重→E 包区间 BFS 的 cross length parity/
虚根 imaginary_cross_act/实根 cross 像=自身/新指纹只入正开启包/Cartan 类
一致/Cayley 仅 ImaginaryNoncompact 且逆槽首写居 .0→F 打印头偏移
exp_2pi(dual_two_rho,4)）、收尾 "element status" 扫描与 cross 槽无哨兵的
推断如实标注为推断、查询层 status/cross 参数顺序相反与 torus_label vs
print_layout 错误语义不一致的字节事实、无 Eq 由 GlobalKgbPrint 承担快照
比较、render 的 setw 复现细节、4 测试（三个逐字节+B2 结构）与无错误分支
/半单秩 0 有意未测（weyl_transducer.rs:485 panic）——全部忠实。8 页批准；
wiki/concepts/ 现有 349 页，剩余候选 184。

## 2026年10月10日 第四十九批候选审查：ext-param.md 的 7 页

对照 83 行来源包逐页核验：模块范围（ExtRepContext/ExtParam/比较对齐辅助/
fixed_conjugate_simple/complex_cross/star/三 finalisation 驱动）、两条保真
约定（Weight/Coweight/int 全 wrapping i32 匹配上游 int、有理权分子保持
i64；上游 assert→debug_assert 或 debug-only validate、数据失败走
StructureError）、ExtRepContext 的 delta 根系置换+不动根集+诱导 twist 与
to_simple_shift/is_very_complex/shift_flip 的上游行号转述、ExtParam 六
字段与 x(ctx) 由 (tw, l mod 2) 重建 KGB、默认扩展一族（at/def_ext 族、
default_extend_srm 要求 gamma_lambda 已 real_unique）、star 返回
(DescValue, Vec<ExtParam>)、finalisation 队列重放与净翻转跟踪、
extended_finalise 返回 Vec 而 scaled 返回单个并缩放 ν 保持 λ、两个
StarOracle 实现的分工（ExtParamOracle 经 def_ext 重建默认扩展、
PartialBlockOracle 服务 build_partial 之后的 tune_signs）、dirty 工作区
快照与"正确性属自身 HPC 证据链"的边界声明——全部忠实。7 页批准；
wiki/concepts/ 现有 356 页，剩余候选 177。

## 2026年10月10日 第五十批候选审查：topology-form-name.md 的 7 页

对照 95 行来源包逐页核验：连通性=最分裂 Cartan 的对偶分量群平凡、
B 列序（简单余根+radical 基）与 i_sw=B^t·θ·B^{-t} 转运对应上游
theta.transposed().on_basis(basis).transposed()、dualPi0 子商定义
（ker_F2(θ+1) 模饱和 +1 特征格 mod-2 像）、B_z 清 radical 列与
CorootRestriction 逐输出列奇偶、validate_induced_map_to 下降验证、
核秩=源维数−像秩与两函数整段复制仅末行不同的同步漂移风险如实记录、
integral_entry 逐项整性、θ 的对合性由调用方保证；命名侧的 pulled[k]=
grading[perm[k]] 拉回方向、'C' 消费两因子两段切片而环面不消费位、
split 弱递减（su(1,2)→su(2,1)）、m=最低置位+1、各型分派表（含不等秩 A
奇秩+平凡双条件、C 的 m==rank 分派、D 的 so* rank%4 规则、D4 任何非零
grading→so(5,3)、E8 的 0xCC 掩码与 {2,3,6,7} 位）、perm≥128 拒绝与
u32 切片无防护、折叠 'f'/'g' 不覆盖、5+7 测试锚点与未覆盖清单——全部
忠实。批准前按已记录的教训修正三个候选中的 5 处 `[[1]]`/`[[2]]` 矩阵
字面量（pmatrix 化后复核批准）。7 页批准；wiki/concepts/ 现有 363 页，
剩余候选 170。

## 2026年10月10日 第五十一批候选审查：atlas-core-root-numbering-alcove.md 的 7 页

对照 56 行来源包（维护者直接撰写、git base 964f0033）逐页核验：
RootNumbering 的 (level, root_compare) 排序（从最后坐标向前）、
prefer_coroots 选坐标系、正根 [npos,total) 与负根镜像 npos-1-p、
signed(nbr)=nbr−npos、BTreeMap 坐标索引；wall_set 的小 dominant 位移
语义与 integrals=on_wall_coroots、min_coroots_for 的 α∨−β∨ 非余根过滤；
root_components 分量内 RootNbr 升序而分量间按最大 RootNbr（原版追加
行为所致，FPP 乘积向量可观察——与第四十五批 alcove.rs 侧"按首次出现
顺序"的记录分属两层，如实保留各自表述）；labels_for_component 唯一正
本原关系与环境余根表核计算；sorted_by_label 降序+RootNbr 并列；
from_fundamental_alcove 留单位标签墙+to_positive_system+逆序得词；
基本 alcove 墙数=秩+分量数且只有大小可观察（"Too few walls"）；精确
Cartan 逆的 (分子,分母) 分数自由消元（候选如实不展开未覆盖细节）——
全部忠实。7 页批准；wiki/concepts/ 现有 370 页，剩余候选 163。

## 2026年10月10日 第五十二批候选审查：atlas-core-weyl-subgroup.md 的 7 页

对照 74 行来源包（维护者直接撰写、git base 964f0033、涉事文件
86d52b4f…）逐页核验：排序遵循 BitMap basic_orbit/extend_orbit 而非
alcove 分层序、初始 dominance 真用给定生成元（原版 rootdata.h669/678
转发忽略 g 的 R3 反例如实保留：A2 空生成元 [-1,-2] 矩阵轨道 [2,1] 但
见证恒等）、Subgroup::new 的三参数与 dual 形式识别、i128 配对矩阵与
收窄错误文案、infer_lie_type 失败的原版逐字文本与单生成元 [[c]] 特例、
cosets 的精确配对坐标替代格核选举（正 c 取 1 的缩放论证+有限 Cartan
可逆性单射）、BFS 只对新层去重保留首插、活动生成元按根号序与用户序
分管 dominance/扩展的分离、Weyl_orbit 列矩阵 vs Weyl_orbit_ws 见证词
（权右到左/余权左到右、dual 正序否则逆序拼接、build_weyl_context+
right_multiply_simple 重建+weyl_elt_value 冻结）、BuildAndDrop 只构造
不算轨道、向量大小不匹配报安全错误而不模拟原版越界读、4 测试锚点
（9 类型×2 编号×2 isogeny×2 作用的空子群不变量、独立穷举闭包交叉核验、
全单子群逐等、丢弃前诊断匹配）——全部忠实。7 页批准；wiki/concepts/
现有 377 页，剩余候选 156。

## 2026年10月10日 第五十三批候选审查：atlas-core-center-classifier.md 的 7 页

对照 60 行来源包（维护者直接撰写、git base 964f0033）逐页核验：
CenterClassifier 的 (adjugate, det) 制表与 rem_euclid 分桶/div_euclid
入 shift_of、与上游 C_denom 一致为移植陈述如实标注、shifts 的借位规则
（fix_entry≤entry 直减，否则 rts+1 且 entry-=fix_entry-det）与
pos & subset == subset 输出条件；adjoint 轨道机器的共享 BFS 核（finish
后递减、层完成反转递增）、basic_orbit_adjoint 的前 i+1 生成元 Levi 子商、
vertex_orbit 的模变体沿 label>1 扩展、convert_to_words 的反射词左乘父段、
reflection_word 的首个下降降到单根再逆序回溯、word_act_root/
word_act_weight 均为最后一个字母先作用、与 subgroup 包见证序的约定配对
——全部忠实。7 页批准；wiki/concepts/ 现有 384 页，剩余候选 149。

## 2026年10月10日 第五十四批候选审查：atlas-core-domain-scc-root-table.md 的 6 页

对照 45 行来源包（维护者直接撰写）逐页核验：strong_components 的迭代
Tarjan 形（rank/class_of/partition/induced 与 active 四元组、nil=size/
infinity=size+1 哨兵）、ByLastCoordinate 的 iter().rev() 与上游
root_compare 一致、integer_matrix_product 的 i128 宽累积只喂扭曲兼容等值
测试的限定用途如实保留、对合构造器先检查值再适配成行（Vec<Vec<_>> 的
0xN 维数不可恢复、Atlas 以行数为期望秩）、RootTable::build 的
prefer_coroots 转置-生成-换回、components/express/length_flags 分工——
全部忠实（本包为头部精读，候选均如实声明未展开细节）。6 页批准；
wiki/concepts/ 现有 390 页，剩余候选 143。

## 2026年10月10日 第五十五批候选审查：atlas-core-root.md 的 7 页

对照 64 行来源包（维护者直接撰写、git base 964f0033）逐页核验。审查中
发现包行 17 误记"15 个 pub mod"而清单只有 14 个名字；对照 git base
964f0033 与当前工作区的 lib.rs 实际声明均为 14 个 pub mod + 1 个
pub(crate) matreduc + 1 个私有测试模块，确认为包的计数笔误。已更正包
（15→14），并将候选 atlas-core-语言门面与兼容契约 的 summary 两处与正文
一段从"存在不一致，不能确定遗漏模块"改写为更正后的陈述加更正注记（该
候选此前如实保留歧义而非擅自消解，行为正确）。其余核验：15→14 模块清单
与角色地图、COMPATIBILITY_VERSION="atlas-language-v0"、typed 整数收窄
保留上游逐字错误文本含笔误、session 逐命令执行不预切分的原因、
domain_builtins 句柄的 Arc 束+急切种子+惰性 KGB/表示属主与结构比较对应
上游 memoized 句柄可观察相等、matreduc 私有但需独立包的声明——全部忠实。
7 页批准；wiki/concepts/ 现有 397 页，剩余候选 136。

## 2026年10月10日 第五十六批候选审查：atlas-core-lex.md 的 8 页

对照 91 行来源包（维护者直接撰写）逐页核验。审查中又发现一处包计数
笔误：包行 31-32 写"20 个上游原始类型名"但清单 21 个；对照 git base
964f0033 与当前 lex.rs 的 PRIMITIVE_TYPES 均为 21 个，包已更正
（20→21），候选 atlas-词法-token-模型 的正文段落相应改写为更正后陈述
加更正注记（该候选同样先如实保留歧义）。其余核验：TokenKind 12 变体与
OperatorBecomes 融合规则（跨空白/注释、失败回退）、Directive 仅命令首
识别与四种种类、lexeme 精确源拼写含引号、35 保留字、FILE_NAME_CHARS
不含斜杠、换行抑制状态机的 nesting/prevent_termination 双空透出条件与
各关键字/标点状态转换、operator_termination 的"."例外、裸 ! 永不融合、
<=> 极大连续段为单算符、不支持字符清空状态使下一换行终止（对应 oracle
恢复行为）、未闭合注释/字符串的诊断文本与 pending 恢复 token、
recover_command 语义、TokenCursor 错误与 token 同缓存、tokenize 全有
或全无 vs tokenize_with_diagnostics 保留诊断、23 测试与职责边界——全部
忠实。8 页批准；wiki/concepts/ 现有 405 页，剩余候选 128。

## 2026年10月10日 第五十七批候选审查：atlas-core-syntax.md 的 8 页

对照 110 行来源包（维护者直接撰写）逐页核验：Expr 全变体清单（BarList
的独立节点保留 oracle 精确诊断且不受用户 ^/mat 重载影响、Subscription 的
reversed 标志、Slice 省略界解析器填零、MultiAssignment 目标是已存在变量
区别于 let、OperatorCast 的自由形式参数不是抽象、TypeAbstraction 离开体
时抽象成 scheme 后在类型分析中消失、Conditional 的 elif 解析期脱糖、
Sequence/Next 的效果与值分工、Do 的词法帧必须同时包住两表达式、Break 的
levels+1、IntCase 的 then 收负值/else 收越界/皆缺取模、Die 通过任何类型
分析而求值抛 I die）、Pattern 四变体（Discard 非元组成员、Omitted 消费
分量不约束不绑名、Name 的 0x8/0x4 独立位域、whole 必为 Name）、TypeExpr
九变体与 Named 的两种指向、Command 面（SetType 仅括号形式进 tabled 表、
Forget 永不报错、SetOption 未知选项中止不动 verbosity、PolymorphicSet
逗号兄弟各自独立）、解析入口纪律（词法器拥有命令边界、适配层绝不预切分、
TypeTable 活跃可见、has_virtual_group 的 Ok(None) 续行、重定向体打开前
解析）、TokenStream 惰性转换与 Bison 措辞渲染（措辞可精化如实声明）、
51 测试边界——全部忠实。8 页批准；wiki/concepts/ 现有 413 页，剩余
候选 120。

## 2026年10月10日 第五十八批候选审查：atlas-core-types.md 的 8 页

对照 91 行来源包（维护者直接撰写）逐页核验。前置计数核对：候选称
"Prim::ALL 20 个原始类型"——对照 types.rs:84 的 `pub const ALL:
[Prim; 20]`，真实确为 20（Void 不在其中，void 即空元组），与 lex.rs
的 21 名列表（含 "void"）是两份不同清单，本包计数正确无需更正。
核验要点：Type 九变体与 rigidity 由外围 scheme fixed 阈值决定、
specialise 唯一变异路径与失败部分特化的上游语义（回滚先
can_specialise）、expanded 的 Cow 与循环上界=绑定数、equivalent 先
validate_applications 两侧再结构递归、递归名终止边界、validate_• 
不展开定义而 expand_application 只展一层、TypeTable 四组件与 forget
只摘活名、matching_bindings 查全部保留定义（axis-types.w:1454）、
revision: Arc<()> 的非语义快照身份语义（克隆共享/突变即换/阻地址复用/
保 Send+Sync）与已验收跨命令缓存门的交叉引用、TypeScheme::wrap 的首次
出现序与重复变量共享槽位、constructor 保留声明 arity 含未用参数、
TypeAssignment 的 [fixed,fixed+degree) 无环替换与 append 连同待决替换
导入、unify 失败部分变异 vs try_unify 回滚、InferredType 接口清单、
recursive.rs 的命名 RHS 槽位优先与环上匿名后代保留身份、59 测试分布——
全部忠实。8 页批准；wiki/concepts/ 现有 421 页，剩余候选 112。

## 2026年10月10日 第五十九批候选审查：atlas-core-session.md 的 8 页

对照 80 行来源包（维护者直接撰写）逐页核验：SessionEvent 六变体与
Value 独占 void 标志、output() 的 UTF-8 分流（非法字节进 OutputBytes
不被替换字符改写）、run_source/run_source_with_context 分工、Newline/Eof/
Unsupported/Directive 的分流（Directive 在会话层只能得 Io 诊断）、
next_session_token 消费时刻记录标识符保首次使用序、execute_tokens 的
allow_more/Ok(None) 保留前缀不求值无诊断、SetType 按真实词法终止符重建
span（Newline 列+1）、drain_failed_printed 先排空已打印再发诊断
（ext_kl.cpp:947 顺序的移植陈述）、201 测试家族统计与
include_str!+oracle.stdout/stderr 逐字节范式、硬规则 7 的原版背书回归
——全部忠实。8 页批准；wiki/concepts/ 现有 429 页，剩余候选 104。

## 2026年10月10日 第六十批候选审查：atlas-core-session-frame.md 的 7 页

对照 99 行来源包（维护者直接撰写）逐页核验：FileProvider 有损 UTF-8
（游离字节不得变成打开失败）、sink 只在解析成功后求值前打开（语法错误
不留文件、求值失败留部分输出）、search_path 空前缀最后试与 .at 补名、
包含判定链（typed 名 completed 静默跳过→resolve 失败 Io+Abort→已在
active 静默跳过算成功→超深 64 Io+Abort→Starting/压栈/Finished 才记
completed 打 Completely）、clean 纪律（语法/类型/求值错误置脏、打开失败
与 abandon 刻意不弄脏、missing_file 测试断言 is_clean 仍真）、quit 在
包含内结束整个会话、Value:/void 抑制、按深度每层两空格缩进、重定向体先
按表达式解析（parser.y TOFILE expr 与 file_commands_b9 的 '=' 拒绝形状）、
打开失败只留裸 stderr 且保持 clean、abandon 的 offset()-1+line_map 与
最内层先读、preprocess 先剥尾空白再拼接 \ 续行（foo\ 仍续行）、
describe_bytes 输出形状与无 span 裸头、18 测试锚点清单与 eval/
file_commands_b9 唯一允许写 /tmp——全部忠实。7 页批准；wiki/concepts/
现有 436 页，剩余候选 97。

## 2026年10月10日 第六十一批候选审查：atlas-core-value-layer.md 的 7 页

对照 74 行来源包（维护者直接撰写）逐页核验：Value 全变体清单、
AtlasString(Vec<u8>) 字节保留与双向 PartialEq、Display 只是预览而
atlas_text/append_atlas_text 才是无 Unicode 边界的打印机、Rational 符号
单走且分母为 1 也打印、Closure 只打 Function defined 头（完整形式由
closure_trace_string 渲染）、Closure 的 shapes（whole 绑在元素槽之前）、
parameters=0 不再压帧、recursive 0 号槽绑自身、frame 弹出后存活、
BuiltinFunction 不透明只能由注册表构造、Vec32/Matrix 列主序/RatVec 构造
即规范化且分母 0 返 None、write_bracketed 的右对齐逗号分隔" ]"/"[ ]"
细节与 ratvec 追加 /denominator、formula 栈的 should_reduce 奇偶结合
约定（偶左奇右）与 4 测试钉死、首元一元算符参与比较而二元后一元属其
运算元——全部忠实；候选如实保留"Value 已列线性变体"与"phase-B B2 才
嵌入"的阶段性表述差异。7 页批准；wiki/concepts/ 现有 443 页，剩余
候选 90。

## 2026年10月10日 第六十二批候选审查：atlas-core-typed-core.md 的 8 页

对照 114 行来源包（维护者直接撰写、typed.rs sha256 614975c5… 与
after-v5 清单一致）逐页核验：Control/Level/WhileMode 三枚举、赋值目的地
族（Global 分析时捕获 cell、Local 留词法坐标、MultiAssignmentPlan 整体
最后、TransformOperation 的用户重载重组为普通调用即脱糖应用）、TypedExpr
节点面（Captured 冻结重载值带表达式拼写、BarList 直接构矩阵不受用户
重载拦截、ComponentAssignment 先值后下标、ComponentTransform 范围检查在
合成读触发、Subscription 领域系数读先接收方再求键而普通订阅先下标、
FunctionCall 参数按一个值传入多参数为元组、For 的七种聚合与下标类型随
接收方、Break 的 levels+1、Die 分析通过任何类型求值抛 I die）、Analysis
字段面（return_type 活结果要求独立当前上下文、loop_depth 分析期拒游离
break、type_floor、ConversionType 共享单元绝不持 RefCell 借用递归）、
IdTable 重定义只换名而旧代码保留捕获 cell、TypeCell 按定义处词法下限
解读且克隆共享精化单元导入不写、OverloadState 的启动表静态+forget/set
合并单列表、add_user 重放上游 add（孪生原地替换、过近邻保留完整歧义
措辞、返回前后变体数选报告措辞）、views 缓存纪律（revision Arc 守护、
只放未移位结构签名、事务克隆不带缓存、ptr::eq 检查同一性）、
TypedCommandEvent 的 Value 携带类型供 is_void、STARTUP_COMPLETION_NAMES
的 35 关键字+21 原始类型名+注册序（与第六十五批更正一致）、三系统变量
不在其中由 new() 播种并标 prelude_log 为 const、type_locations 属于当前
绑定而非复用槽——全部忠实。8 页批准；wiki/concepts/ 现有 451 页，剩余
候选 82。

## 2026年10月10日 第六十三批候选审查：atlas-core-typed-eval.md 的 7 页

对照 72 行来源包（维护者直接撰写）逐页核验：evaluate(context, level)
签名与六族分派表（与转换遍同形、#[inline(never)] 栈帧纪律）、Level
NoValue/SingleValue 一路下传、迭代借用纪律（矩阵不得再造列矩阵、多项式
保持 canonical 项序与属主形式）、apply_function 不加调用迹与被调/参数
求值留在迹外、变参数内建解元组而 bare 变量参数即使元组也按一个值消费、
function_origin 的 "built-in"/"defined <loc>"、apply_closure 的一个值
传入/元组拆分/无参不压帧/全匿名不占帧/递归 0 号槽自绑而新帧不在捕获链
保持 Rc 无环、return 在调用边界解开经 at_level 供值、运行时错误穿带名
槽调用附加局部变量迹行而无参闭包无此行、trace_location 的
at NAME:LINE:COL-COL（行 1 基列 0 基、单行结束 exclusive、跨行双破折号）
与 Rust span 1 基的差异、frame_dump 按绑定序打印槽名——全部忠实。
7 页批准；wiki/concepts/ 现有 458 页，剩余候选 75。

## 2026年10月10日 第六十四批候选审查：atlas-core-convert-expr.md 的 8 页

对照 69 行来源包（维护者直接撰写）逐页核验：convert_expr 入口把
required 装进共享 ConversionType 再写回、convert_expr_context 的
type_floor 调整（return 操作数引用外层要求的实际 fixed 下限）、12 族
分派清单与 #[inline(never)] 机械分区（original3839541 教训：GDB 命中的
是分析帧非求值器）、conform_types 的特化→强转→错误次序、非行上下文
列表显示查 row_coercion（mat: [[1,2]] 元素定型为 vec）、while 转换
（循环层装在整棵 do 树外、条件先 a-priori 转换再查 bool、上游措辞
"found … while … was needed."、WhileMode 三模式与 row 先试 [*] 特化
再回退 row_coercion）、convert_simple_assignment 两形式共享路径、
lookup_assignable 局部遮蔽全局与赋值专用诊断、component_type_for_assignment
的行/vec/mat/KTypePol/ParamPol 允许与 ratvec 上游只读、resolve_projector
用保留类型定义而非投影当前值（具名接收方不能唯一确定字段选择）、
factor_transform_call 永不应用 x+1→succ(x) 丢参数优化——全部忠实。
补充第 38 批记录的链接碰撞教训：候选 85526734 的 `[[1,2]]` 位于反引号
代码段内，批准器正常通过——碰撞校验只针对裸 `[[...]]`，代码段免疫。
8 页批准；wiki/concepts/ 现有 466 页，剩余候选 67。

## 2026年10月10日 第六十五批候选审查：atlas-core-builtin-registry.md 的 8 页

对照 80 行来源包（维护者直接撰写）逐页核验：Builtin 六字段（hunger
对应上游饥饿求值位、overload_visible 控制重载/补全可见性）、BuiltinImpl
六分支（DomainPrinter 两级都写报告且 single_value 产空元组、无值门前无
诊断；Prints/Print/ToString/Error 四变参泛型的各自契约）、DomainNoValue
三策略与"补全名清单不是无值策略清单"的 R2 教训（orientation_nr 注册为
BuildAndDrop）、求值期辅助契约（int_val/long_val 含笔误逐字诊断、向量
\ /% 的余数总取 [0,|m|) 与 oracle 例 [7]%-3=[1]/[7]\-3=[-2] 复核一致、
nth_set_bit 的 0 基与非负耗尽得 -1/负值走补码有限清位、flex_add/flex_sub
只在等修剪尺寸时去结果尾零、convolve 任一修剪后为空则为空、ratvec ± 按
最小公分母交叉相乘后 RatVec::new 规范化、to_string_aux 对变参元组的字符串
分量不带引号、prints 加换行而另两个不加）、321 条目/170 名/309 启动名
三个不同清单统计如实区分、注册表相对上游不全由 REMAINING_BUILTINS.md
跟踪——全部忠实。8 页批准；wiki/concepts/ 现有 474 页，剩余候选 59。

## 2026年10月10日 第六十六批候选审查：atlas-core-domain-dispatch.md 的 7 页

对照 72 行来源包（维护者直接撰写）逐页核验：call/call_with_printed/
call_owned_with_printed 三入口分工与 printed 侧通道只有
partial_extended_KL_block 使用（ext_kl.cpp:945-948 中途 stdout）、
hungry_product_owned 的三乘积表（先校验后消费：RANK_MAX 组合秩/权与余权
大小匹配，factors 合并/word_act_weight/simple_coreflect）、166 臂统一
形状（arity→类型提取→调用→包装→四类错误路径）、臂内校验顺序契约
（build_KGB_element_wrapper 全部构造检查在无值门之前、KL_block_wrapper
先 test_standard、classify_involution_wrapper 先方形与 M²=I）、signed32
再 unsigned32 的收窄次序、real_form 按参数个数分派、coerce 六类标签
（LT/IcRf 调派发臂、RdIc/RdRf 句柄导航不经函数调用、SpI/Sp(I,I) 先收窄、
KpolK 经 finals_for+merge_ktype_term+K_type_pol 项序、PolP 经
expand_final+SR_poly 项序）、未知 tag 的运行时错误文案、
build_real_form 的 internal(external) 翻译与 Illegal real form number、
canonical_forms 父级弱缓存与 same_real_form_owner 指针规则配对、自定义
构造即使数学相等也新建属主——全部忠实。7 页批准；wiki/concepts/ 现有
481 页，剩余候选 52。

## 2026年10月10日 第六十七批候选审查：atlas-core-domain-values.md 的 7 页

对照 126 行来源包（维护者直接撰写，Weyl owner/dual 修复落点）逐页核验：
WeylIdentityCell 只在成功时落定+失败不占单元+毒化/二次初始化报
RepInvariantViolation、DatumWeylKernel/AbstractWeylGroup/DatumWeylIdentity
三层与无所有权环、share_group_into_if_cold 只装冷目标且并发竞态只有一个
发布、DATUM_WEYL_IDENTITIES 按完整内容弱驻留与 4096 才扫死槽、
RootDatumHandle::interned 唯一入口与 PartialEq 刻意忽略身份缓存、
WeylEltContext 的 kernel+抽象群（重编号固定 canonical-word 选择）与
"兼容=Arc 身份永不是结构 handle"、WeylEltValue 构造时冻结 canonical word、
SplitValue 的机器位宽回绕与 (e±|f|s) 打印、split_keeps 零因子筛选、
DomainValue 13 变体结构等值分层（same_real_form 四要件 vs
same_real_form_owner 指针）、weyl_elements_equal 的 Arc 同一性+外生成元
词在左系统重放、require_weyl_compatible 在无值门之前、check_weyl_word 先
unsigned 再 <半单秩、多项式系数契约（只替换精确 final 键不累加、触碰前
拒绝不兼容属主的刻意偏离、dominant 化只改副本、loop_terms 借用 canonical
序保留属主）——全部忠实。7 页批准；wiki/concepts/ 现有 488 页，剩余
候选 45。

## 2026年10月10日 第六十八批候选审查：atlas-core-domain-construction.md 的 8 页

对照 75 行来源包（维护者直接撰写）逐页核验：build_datum 的两路格基
（单连通用权格基根=Cartan 行余根=基、伴随用根格基根=基余根=Cartan 列）、
T1 因子追加在半单之后且不保留交错输入也不改调用方 LieType、中央商覆盖
中间商不只端点、显式 datum 保留空维矩阵维数、build_inner_class_context
固定装配序与三道预算门（分类/FIBER/INTEGER）、对偶侧只建一次的四件
（dual_inner_class+对偶分类+dual_form_count+dual_cartan_correspondence）、
build_presentations 在 canonical_forms（每形式一个 Weak 槽的 Mutex 向量）
之前、build_inner_class 的余权部转置与上游接受任何根数据对合再左合成为
distinguished、build_dual_inner_class 的余根偏好翻转+逐字母对偶 Lie 类型
+内容弱驻留共享 Weyl 身份、build_real_form 的编号翻译与非法号错误、
build_custom_real_form 的 fresh_table→基本（第一个）Cartan→
RealFormSeed::custom 顺序与 RealFormContext 的 FallibleOnce kgb/rep+
双形变缓存——全部忠实。8 页批准；wiki/concepts/ 现有 496 页，剩余
候选 37。

## 2026年10月10日 第六十九批候选审查：atlas-core-domain-validate-print.md 的 8 页

对照 81 行来源包（维护者直接撰写）逐页核验：validate 46 臂的逐臂顺序
契约（integrality 四件先 check_integrality_dimension、W_refl 的 int_val
收窄先于索引校验且可观察、KGB 先按 kgb_size 查界、KGB_elt 全部构造检查
在无值门前、KL_block 先 test_standard、real_form 按参数个数分发）、
common_block_rows 每次调用新建的兼容性依据（Rep_table 池只是记忆化+
dominant gamma 修饰符平凡）与 init 按 (x, gamma-lambda) 匹配（单按 x 在
R 包内有歧义）、located_common_block_rows 与 KL_column/KL_block 共享
查找序列的刻意区分、partial_block_rows 的种子归约链与 survives 用调用方
gamma（即使种子先规范化）、完整块路径 mod_reduce 但不 make_dominant 无池
无修饰符、involution_expression 的 1 基数字/^交叉/x 共轭/e 收尾、
print_KGB 选择形式的同实形要求与逐字错误、print_gradings 的
gr_print[i]=gr[sigma[i]] 回拉方向（候选明确警示反向解读）、print_X 与
print_blockstabilizer 上游无检查、print_real_Weyl 检查须在臂内先跑否则
静默翻译——全部忠实。8 页批准；wiki/concepts/ 现有 504 页，剩余候选 29。

## 2026年10月10日 第七十批候选审查：atlas-core-deformation-cache.md 的 8 页

对照 65 行来源包（维护者直接撰写）逐页核验：FullDeformKey=(x,y_bits,
gamma) 是 canonical 单元而非顶层参数键、锁只在读写瞬间持有不跨递归、
只缓存完整且 canonical 排序的结果、active 集显式检环（"revisited an
active parameter"）、deadline_expired 阶段间检查且超限返 None 不缓存部分
结果、普通递推 F(z)=L(z)+Σc_t(1-s)F(t) 与原版整数递推经 (1-s)²=2(1-s)
等价、scale-zero 基底保留全部 final 项（set_LKTs）、子项链
scale→deform_readjust→rep lookup→common_deformation_terms→带 c(1-s) 递归、
"停在前一点变成单 K 型会丢 Split 因子与后代"的修复课、compute_full_deform
逐 final 组分形变按系数缩放合并排序、扭曲流程 distinguished_twist→
ExtRepContext→extended_finalise 且计时在 finalise 之后开始（setup 在截止
外）、finalise 翻转不同系数为 s 否则为 1、矩阵辅助三件（候选如实声明
cramer_solution 只记录了"分数自由变量消元"概述而不补充细节）——全部忠实。
8 页批准；wiki/concepts/ 现有 512 页，剩余候选 21。

## 2026年10月10日 第七十一批候选审查：atlas-core-support-layer.md 的 7 页

对照 74 行来源包（维护者直接撰写）逐页核验：SourceId(0)=匿名、
SourcePosition 行列均 1 基、SourceSpan 含头不含尾、ErrorKind 七类别与
措辞刻意分离（Program=表达式分析失败区别于类型合一）、raw_message 精确
字节 vs message 转义预览的 new_bytes 分流、warning 报告但不弄脏会话、
back_trace 最外层在前且 trace() 前插对应 push_front、命令层拷入
back_trace 系统变量（候选如实标注空回溯时的行为未说明）、SourceText 的
position 钳制+字符边界+partition_point 与列=Unicode 标量数+1、coercions
29 条注册保持上游顺序与首中即返线性扫描（mat 上下文必先遇 [vec]->mat）、
row_coercion 取第一个 from 为行的条目、is_close 三比特与相等先于边界
检查/void 与 * 只与自己邻近/Tabled/Applied 递归名返 0、broader_eq 平衡序
（void 最宽、* 最窄、原始吸收可转入者、函数要求参数相等）——全部忠实。
7 页批准；wiki/concepts/ 现有 519 页，剩余候选 14。

## 2026年10月10日 第七十二批候选审查：atlas-cli-main.md 的 8 页

对照 45 行来源包（维护者直接撰写、175 行全覆盖）逐页核验：FsProvider
有损 UTF-8 的动机（游离字节不得变成打开失败）、FsSink 的 OpenOptions
模式、print_events 分流（Output/ReportLine 走 print!、字节事件写原始
字节到 stdout、诊断经 describe_bytes 到 stderr、Value 分支防御性）、
rayon 2MiB 工作栈配置及其动机如实标注为源码说明而非实测、--path 可重复
与文件参数按普通命令流喂入（prelude-capture 明确 non-goal）、stdin
终端分流与横幅逐字内容、退出状态=clean 标志与缺包含文件不置脏、quit
提前结束、本文件无测试由 HPC 语料门覆盖——全部忠实。8 页批准；
wiki/concepts/ 现有 527 页，剩余候选 6。

## 2026年10月10日 第七十三批（最终批）候选审查：atlas-core-regression-library.md 的 6 页

对照 55 行来源包（维护者直接撰写）逐页核验：四模块测试计数（session
201/typed 133/domain_builtins 92/session_fixture_tests 17）与各家族前缀
统计、原版背书范式（include_str!+逐字节 oracle 比对、金标永不来自 Rust
输出、244 处 include_str!）、session_fixture_tests 走 run_source 全程且
刻意不保留已移除的动态求值器、验收证据链组成（分阶段证据 JSON+
append-only 账本+HANDOFF.md，KB 不改写账本）、"计数只标识快照字节、不
声称任何测试通过"的边界——全部忠实。枢纽页 [[HPC 验收证据链]] 上线，
消解了此前数十批候选的前向引用。6 页批准。**至此 533 个候选全部审完，
wiki/concepts/ 共 533 页上线（533 = 533 候选全部入库；早前 73 包编译
产生的全部候选已清零）**。
## [2026-10-09T18:14:39Z] compile | 2 source(s) → 0 page(s)
- Sources: atlas-core-lex.md, atlas-core-root.md


## 2026年10月10日 候选清零后的 stale 刷新与收尾

第 55/56 批的两处包计数更正（15→14、20→21）使 15 个已上线页在编译器
看来 stale（源哈希变了）。按 kb/AGENTS.md 第 6 条的 deliberate
maintenance 路径：先 `refresh --stale --dry-run` 检查计划，再正式
`refresh --stale`（80.7s，2 包重编译）：7 页内容不变直接刷新，8 个再生
候选按 hold-all 进入审查。逐一对照更正后的包与现页核验：6 个同名候选为
等价改写（门面页已正确写 14 个公开模块，更正注记随包更正而不再需要），
2 个为编译器从新包提取的新概念（递归类型组的名字解析与校验分离、会话
补全顺序与名字可见性分离，均核验忠实）。8 页全部批准后 `llmwiki status`：
535 页、Fresh、无 stale/孤儿、无待审候选。wiki 与 73 个来源包完全同步。
## [2026-10-09T18:27:55Z] compile | 1 source(s) → 0 page(s)
- Sources: atlas-core-builtin-registry.md


## 2026年10月10日 注册表计数更正（AGENTS.md 审计的延伸）

审计延伸核对 docs/REMAINING_BUILTINS.md 时发现包 atlas-core-builtin-registry.md
的"321 个条目、170 个不同名字"只数了 scalar_builtin+domain_builtin 两个
构造器家族（167+154=321/170 精确吻合）。对 typed.rs 注册体 vec![…] 按
构造器调用精确计数（git base 964f0033 与 HEAD 字节相同）：479 条目/240
不同名字（scalar 167/59、domain 154/119、domain_validate 86/58、
domain_skip 29/19、domain_printer 21/19、domain_relation 22/2）。包三处
更正后 refresh --stale：4 页原地刷新、4 个再生候选（含新提取的
readline_completions 页）重审批准；docs/REMAINING_BUILTINS.md 顶部加注
LATEST 2026-10-10 更正说明。wiki 现 536 页、Fresh。

## 2026年10月10日 kb/index.md 补齐（46 个缺失来源包）

vault 入口索引此前只链接 73 个来源包中的 28 个；按现有表格风格补齐其余
46 个（链接文字取包 frontmatter 标题，描述列保留"结构性阅读，不作数学
验收"的诚实后缀；root-ladder-overflow-repair 行保留其 AFTER-v3/ledger
0003 验收指针）。末尾"后续补充"的过期注记更新为当前状态（73 包全覆盖、
536 页、Fresh、验收权威在 HPC 门与账本）。74 个 sources/ 链接全部可解析。
