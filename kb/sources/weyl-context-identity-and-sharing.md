---
title: Weyl 对象身份、dual 历史与安全共享边界
source: atlas-rust/weyl-context-identity-and-sharing
ingestedAt: 2026-10-03T10:32:42Z
---

# Weyl 对象身份、dual 历史与安全共享边界

编辑状态：**BEFORE-v4 已保留 tests-first 证据；最小语义修复已作为未验证候选实现；
AFTER-v1 gate 已冻结并提交（342a0511），HPC 提交因 SecureLink 隧道中断而暂缓**。
本来源包
解释性能线索为何同时触及可观察语义；它不是缓存实现、数学验收或加速结论。
旧 discovery catalog 的 `source_predicted_not_captured` 是不可改写的历史输入；
当前事实来自独立审查的 v8 capture、BEFORE-v4 job3886748 与新建的 regression
catalog，不把预测数组改称 golden。编译器生成页仍未刷新，外部 provider 重试的
既有授权阻碍未解除。

## 2026-10-03 AFTER-v1 gate 冻结（未提交、未验证）

changed-input 后继 `weyl-context-core-after-v1` 的 stager/driver/checker 已冻结并
随 `342a0511` 提交推送；生产修复仍以补丁数据形式随 gate 输入传输，修复后的
`domain_builtins.rs`/`typed.rs` 字节保持未提交。该 gate 计划在已修补源码上重跑
两个已知回归，要求它们通过且修补后 Rust 与四个冻结原版 golden 完整一致（冷、热
两个 case 的 stdout/stderr 与退出状态），同时保留 ladder 对照、632 项 inventory
和全部 ledger/完整性检查。它不授予数学、缓存、性能、内存或 rank 验收。

冻结身份：after stager SHA-256
`8cb4b86d9986e66c1722c87d5704ab92ee387ce426ff0f3c194b60c5320904d6`，
after driver `3d7d98903a838b3a3343482d165af4b7e9b3989cc1f30a5c9ccc49950700913b`；
checker 表 32+17+18+21+28+7=123（regression-contract 因新增四个
`classify_after` 契约测试从 17 增至 21）。离线核验：冻结 before-v4 报告的
1,567 文件源码 manifest 的 canonical SHA-256 为
`55f807cadb712377cbf6c0c79250b1d5e739e9757290a24209a086f89632850f`，应用
`REPAIRED_SOURCE_HASHES`（domain_builtins `e6987e7c…`、typed `614975c5…`）后恰为
`AFTER_SOURCE_MANIFEST_SHA256`
`3f8cf4753f29d33df8273086254a4f09170ada46f05e9c13acfdac2f5ab85c33`。

提交时 SecureLink 隧道（`tun0`）已断开，无法创建远端 stage/intent/job；精确启动
步骤见 `docs/HANDOFF.md` 2026-10-03 节。冻结前的本地静态复核（`umask 022` 加
0444 镜像目录重放十二个证据 validator）修掉了四处过渡缺陷：`progressive_submit`
缺少 before-v3 scoped campaign 文件、stage-creation 合成 fixture 缺少 before-v4
campaign 文件且 ledger 长度断言未随链增长、`validate_before_v3_failure` 仍引用已
推进到 before-v4 的裸 `PREDECESSOR`（现以移植的 `BEFORE_V3_PREDECESSOR*` 常量
本地重绑定，函数体与 HPC 验证原版逐字节一致）、以及 before-v1/v2/v3 历史证据中
冻结的 `expected_test_counts_after` 预测改用历史字面量而非当前可变常量校验。历史
字节永不改名以迎合后继。详见
[`../snapshots/2026-10-03-weyl-core-after-gate-freeze.json`](snapshots/2026-10-03-weyl-core-after-gate-freeze.json)
与 `docs/slices/weyl_context_core_after_v1_preparation_2026_10_03.md`。

## 2026-10-02 原版捕获与 tests-first 状态

HPC job `3884807` 已 FINAL `COMPLETED 0:0`。独立检查绑定完整 raw streams、
冻结原版 binary、source 和输入，确认：

- 两种 cold canonical-dual 构造历史中，原版 `=` 为 true、`!=` 为 false、乘法
  成功；Rust 返回 false/true，且两个乘法均错误拒绝。
- 对独立预热的不兼容 owner，原版先拒绝 `=`、`!=` 和乘法；Rust 的关系运算
  错误地产生布尔值。原版八条有序诊断中，Rust 只保留四条。

新回归固定四个原版 stdout/stderr golden，并在 `session.rs` 添加两个普通
`#[test]`，完整比较输出与有序 `(ErrorKind, message)`。没有 `should_panic`
或以 Rust 错误输出充当 expected 的做法。tests-only source 是 1,567 个文件，
manifest `55f807cadb712377cbf6c0c79250b1d5e739e9757290a24209a086f89632850f`。

第一项 BEFORE job `3884862` 在检查器自检中失败：84 个前置 checker 测试
通过，driver suite 的 28 项中 26 通过、1 failure、1 error。问题是旧预测分类
断言和依赖源码换行的字符串查找；Cargo、Atlas 和两个数学回归尚未执行。
这不是预期的 Rust 数学失败证据，不能解锁生产修复。原始 stage 保持不变，
修正后的后继仍须独立 HPC BEFORE，然后才是最小语义修复和同一回归 AFTER。

v8 的四次数学调用仅 0.00–0.01 秒；345 秒总作业时间主要是 281.53 秒构建。
这些数不支持 Rust/original 加速比。新证据不改变后文的历史性能数字。

## original Atlas 的两层生命周期

冻结的 original revision 是
`7e1b958c7aa9456769cc9cf09ac1542814b4800a`。它首先按完整
`PreRootDatum` 内容（包括 simple roots、simple coroots 和
`prefer_coroots`）查找 `root_datum_value`。身份 `store` 只保存 `weak_ptr`；
若相同对象仍存活就复用其 `shared_ptr`，若已释放则在原槽位重建。因此它提供
canonical live identity，却不会让不可达的重型 datum 永久存活。静态
`pool/hash` 仍保留轻量 `PreRootDatum` key，不能说整个驻留索引会自动清空。
Atlas 语言的 RootDatum `=`/`!=` 比较的也是这个 canonical `shared_ptr` identity；
活着的相等构造之所以相等，是因为先经过 weak interning，而不是另做结构比较。

每个活的 `root_datum_value` 又有一个初始为空的强
`shared_ptr<WeylGroup> W_ptr`。`W()` 在第一次需要时构造一次。`dual()` 先通过
同一个弱驻留表取得 canonical dual：

- 若目标 dual 的 `W_ptr` 为空，先确保 source 的 `W_ptr` 已建立，再把同一个
  `shared_ptr` 安装到目标；
- 若目标已经预热，则绝不覆盖它，source 与 target 后续可能拥有不同的
  `WeylGroup` identity。

每个 `W_elt_value` 强持有其 root datum，同时保存对该 datum Weyl group 的引用。
二元 `=`、`!=`、`*` 在产生结果之前先比较 WeylGroup 地址；地址不同就抛出
`Weyl group mismatch`。这个检查也先于 `no_value` gate。因此这里的兼容性
不是只由 Cartan matrix、结构相等的 root datum 或相同的 root permutation
决定；它包含 canonical owner 的存活期和 `dual()` 的预热历史。

## 当前 Rust 的差异

当前 `RootDatumHandle` 只有 `Arc<BasedRootDatum>` 加构造 provenance，并以
结构方式实现 `Eq`/`PartialEq`。每次 `build_weyl_context` 都重新枚举
`RootSystem`、重新构造 `WeylInterface`，随后把完整 handle、system 和 interface
一起放进新的 `Arc<WeylEltContext>`。`WeylEltValue` 再持有这个 context。

这带来两类独立问题：

1. 同一个长期存活 datum 上的短命 `W_elt` 会反复重建根系统和 interface；
2. 二元关系走通用 `DomainValue::PartialEq`，只比较结构 handle 和内部
   `WeylElement`；乘法也以结构 handle 相等作为兼容性判断。这个模型无法表达
   original 的 WeylGroup pointer identity，更不能在 `no_value` 二元关系中抛出
   mismatch。

因此不能先做“按 Cartan matrix 缓存”再假设行为不变。源码预期至少存在两种
相反风险：cold-target dual 在 original 中可以共享 group，但 Rust 因 handle
不同而拒绝；prewarmed-target dual 在 original 中必须保持两个 group，而 Rust
的结构关系返回布尔值而不是拒绝。上述两点现已在 A1 的限定输入范围内由 HPC
fresh-process capture 观察到；golden 取自完整原版流，而不是源码预测。

## 为什么它是当前性能目标

已接受的 rank-one loading-inclusive profile（job `3868803`，report SHA-256
`1d2baaddbfcb2bf09801a995ed9a6702646dd3cf9a8f396b1ddfd6d22c693915`）中，
`build_weyl_context` 相关栈约占 unitarity 样本 14.52–14.58 个百分点、AV-ann
样本 11.76–12.83 个百分点。这是采样归因，不是单独 kernel wall-time，也不能
与嵌套的 root-ladder 百分比相加。

冻结的 `elliptic.at` 在库加载时，对 G2/F4/E6/E7/E8 各绑定一个 adjoint datum，
再分别为 3/9/5/12/30 个 word 调用
`W_elt(rd,w).matrix.char_poly`。总计 59 次调用只保留 characteristic polynomial
向量，临时 WeylElt 可在迭代间释放。这解释了为什么只缓存一个 `Weak` context
可能没有命中；也说明把完整 context 强放回 handle 会产生
`handle -> context -> handle` 强引用环。

历史诊断 probe 对同一五类 datum 恰记录了 3/9/5/12/30 次 `weyl.context`
构造，聚合计时约为 G2 0.10 ms、F4 3.06 ms、E6 4.24 ms、E7 33.36 ms、
E8 345.76 ms。它把重复调用和 caller 对上了，但 instrumented 聚合计时不是
候选 wall-time speedup，不能代替无探针的 A/B。

已经接受的端到端结果仍是 Rust 比 original 慢约 9.15971 倍（rank-one
unitarity）和 7.53406 倍（finite AV-ann）。这些数包含加载和解释器开销，不能
归因成 Weyl kernel 的可得加速比。

另一个更晚、必须独立测量的表示线索是 element 本身。original `WeylElt` 是
固定 `RANK_MAX` 的 `unsigned char` 数组，没有 element-local heap allocation；
当前语言层 Rust `WeylElement` 则保存 root permutation、inverse 两个 `Vec` 和
length，`WeylEltValue` 另存 canonical word `Vec`。仓库已经有内部
`CompactWeyl` 与 `[u8; 32]` transducer element，但语言 `W_elt` 尚未使用它。
把语言值迁到 compact representation 可能影响 canonical word、root action、
provenance 和错误路径，不能与本轮 owner/kernel 修复混成一个 patch，也不能从
A1 identity capture 推断速度或内存收益。

## 候选 Rust 所有权模型

**2026-10-02 更新：该模型已实现为未验证候选**。`RootDatumHandle` 现携带
`Arc<DatumWeylIdentity>`，其中含 success-only lazy `DatumWeylKernel`
（`RootSystem`）与 `AbstractWeylGroup`（`WeylInterface`）两个 cell；进程级弱
注册表按完整 datum 内容加 preference 驻留 identity。`dual(RootDatum)` 在
canonical target 仍 cold 时把 source 的 group 装入 target，prewarmed target
绝不覆盖。二元 `=`/`!=`/`*` 先比较 abstract-group `Arc` identity（关系检查在
no-value 级别同样执行），再在左侧坐标系重放右侧 external word。八个 handle
构造点全部经过私有 `interned` 构造器；结构性 RootDatum `Eq`/`Debug` 不变。
补丁 `hpc/patches/weyl_context_core_repair.patch` 仅验证过能从已接受基线
重建工作区字节，未经 HPC AFTER gate，不授予任何验收。

original capture 已证实 A1 的上述可观察差异。以下仍是后续共享设计提案，
不是本轮已实现方案；先做语义修复，再分别验证缓存与性能。设计方向是两个
无反向引用的 immutable 层：

1. 每个 Rust datum owner 持有 success-only lazy
   `Arc<DatumWeylKernel>`，其中是该 datum 环境坐标下的 `RootSystem`。handle
   clone、语言 alias 和 `root_datum(WeylElt)` round trip 共享它；真正新建的
   quotient、dual、derived、integral、folded 和 explicit datum 获得新 cell。
2. 另设按 exact pre-root identity 弱驻留的 canonical cell；其成功值是
   `Arc<AbstractWeylGroup>`，拥有 `WeylInterface` 和 observable canonical-word
   ordering。`dual()` 只在 canonical target cold 时共享该 identity；已经发布的
  两个 identity 永不合并。
3. `WeylEltContext` 变为
   `{handle, Arc<DatumWeylKernel>, Arc<AbstractWeylGroup>}`。两个 kernel 都不持有
   handle，因而没有强环；`Arc` 管理活对象，`Weak` 允许驻留槽自动失效，
   `OnceLock` 只发布完整成功结果。
4. Weyl compatibility 以 `AbstractWeylGroup` 的 `Arc` identity 为准，不以
   structural handle 或 coordinate-kernel identity 为准。两个兼容但坐标 kernel
   不同的值不能直接比较或组合内部 root permutation；需要把右值的 external
   generator word 在左值 `RootSystem` 中重放，结果归左 owner。
5. 二元 Weyl `=`/`!=` 必须离开不可失败的通用 `PartialEq` 路径，成为可失败的
   domain relation，并在 `no_value` 级别仍执行 identity mismatch 检查。

此外，original 的 `inner_class_value::build` 会立即调用 `srd->dual()`，并在
inner-class owner 中强持有 primal 与 dual 两个 datum。当前 Rust
`InnerClassContext` 只保存 primal handle，而 `build_dual_inner_class` 直接重新
构造 handle。只修显式 `dual(RootDatum)` 因而不完整：inner-class construction
也必须通过同一个 canonical dual/identity 路径，并保留 original 同等的 target
生命周期；是否在 context 中增存 dual handle 要由相应 original-backed history
和无环所有权测试共同决定。

Rust 的自动内存管理只解决生命周期实现，不自动证明缓存 key、共享范围或历史
语义正确。尤其不能把失败的 `Diagnostic` 或首次调用的 `SourceSpan` 缓存进
cell：构造失败后下一次必须重试，并把错误定位到当前调用。现有
`FallibleOnce` 的 poison 文本专属于 lazy real-form，不能未经修改直接复用。

## tests-first 与递进 gate

当前两个 core-only A1 fixture 分别冻结：

- same owner、alias、fresh-equal datum、两种 warm-source/cold-target dual 方向、
  rebind 后旧值寿命，以及 `word`、`length`、`root_permutation`、`root_datum`、
  `=`、`!=`、`*`；
- preference-distinct owner、cold source/prewarmed canonical target、owner/dual
  mismatch、越界和负 generator，以及每个错误后的 recovery marker。

它们必须在 original 与 Rust 各自的 fresh process 中保存完整 stdout/stderr、
顺序、退出状态、wall time 和 RSS。现在已经确认差异并保存原版回归；仍须
完成语义 BEFORE/fix/AFTER，在此之前不得写生产缓存。

A1 的证明力有明确上限：跨 dual 的乘法只是 $s_0s_0=1$，不能发现错误的
generator renumbering 或直接复合 foreign root permutation；而 rebind 后仍有
`wc_alias` 保持旧 RootDatum 存活，所以它也没有证明“仅由保存的 WeylElt 维持
datum 生命周期”。Atlas 输出只能约束可观察语义，不能证明 fresh-equal owner
使用独立 coordinate cell。后两项需要另一个 lifetime fixture 和 HPC-only
`Weak`/work-count 单元守卫。

A1 之后仍需逐步覆盖 G2（非对称 interface-order witness）、B2/C2、两个乘法
operand order、inner-class dual construction 和 `no_value` relations。只有这些
语义 gate 全部通过，才能加入 one-build work-count 测试，然后做同节点、交替
顺序、fresh-process 的 time/CPU/RSS A/B。`59 -> 至多 5` 只是 caller-level
work-count 假设，不是已测加速。

内存方向也不能预报成单调节省。多个 WeylElt 同时存活时共享 RootSystem 应减少
重复对象；`elliptic.at` 的值却是短命的，owner 强持有一个 kernel 可能令 peak
RSS 不变甚至略升。后续报告必须同时给出 live owner/kernel/build 数、分配和
peak RSS，不能只引用 `Arc`/`Weak` 设计。

## 来源与限制

精确读取身份见
历史源码推断见
[`2026-10-01-weyl-context-source-prediction.json`](snapshots/2026-10-01-weyl-context-source-prediction.json)；
增量证据见
[`2026-10-02-weyl-core-regressions.json`](snapshots/2026-10-02-weyl-core-regressions.json)
与
[`2026-10-03-weyl-core-after-gate-freeze.json`](snapshots/2026-10-03-weyl-core-after-gate-freeze.json)。
Rust 文件处于 dirty/untracked 工作区；哈希只能标识所读字节。original checkout
在读取时是 clean，并以 commit+文件哈希标识，不把 `/tmp` 绝对路径作为长期
依赖。

本次知识维护不额外执行 Atlas、Cargo、测试或 benchmark。引用的 v8 和 BEFORE
作业均运行在 HPC 计算节点；知识包不拥有数学验收状态。没有证明 cache、数学
修复、速度提升、内存节约或高 rank 覆盖。原有 compiler 请求被拒绝后未重试、
未更换 provider；这里是人工维护来源包，不是已生成并批准的 wiki 页面。
