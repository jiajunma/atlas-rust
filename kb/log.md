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
