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
