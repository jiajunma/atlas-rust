---
title: Weyl 语义回归的递进验证门禁
summary: AFTER-v5 与生产提交 690c2b92 仅确立限定 A1 语义验收，G2 编号、B2/C2、操作数顺序、隐式对偶及独占元素生命周期仍需后续 gate。
sources:
  - weyl-context-identity-and-sharing.md
kind: concept
createdAt: "2026-10-09T15:17:15.306Z"
updatedAt: "2026-10-09T22:53:38.078Z"
tags:
  - 回归测试
  - 证据范围
  - HPC
aliases:
  - weyl-语义回归的递进验证门禁
confidence: 1
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Weyl 语义回归的递进验证门禁
summary: 通过原版捕获与 BEFORE/fix/AFTER 回归验证 Weyl 身份和构造历史语义；A1 修复已限定验收落地，后续语义、性能与内存门禁仍须独立完成。
sources:
  - weyl-context-identity-and-sharing.md
kind: concept
tags:
  - 回归测试
  - 证据范围
  - HPC验证
aliases:
  - weyl-语义回归的递进验证门禁
---

# Weyl 语义回归的递进验证门禁

Weyl 语义回归采用“原版行为捕获 → 建立回归 → BEFORE 证明修复前失败 → 修复 → AFTER 证明通过”的递进流程。其首要目标是保持对象身份、dual 构造历史与错误行为；缓存、构建次数、速度和内存收益需要后续独立验证。截至来源的 2026-10-09 更新，A1 限定语义修复已验收并落地，不能据此宣称更高 rank 或性能验证完成。^[weyl-context-identity-and-sharing.md:138-157, weyl-context-identity-and-sharing.md:63-74, weyl-context-identity-and-sharing.md:320-330]

## 历史敏感的兼容性

original Atlas 的 `dual()` 仅在 canonical target 尚未建立 Weyl group 时共享 source 的 group；目标已经预热时不覆盖已有 group。Weyl 元素的 `=`、`!=`、`*` 在产生结果前检查 WeylGroup 地址，不同则抛出 `Weyl group mismatch`，且检查先于 `no_value` gate。因此，兼容性包含 owner 的存活期与[[dual 预热历史与 Weyl 群兼容性|dual 预热历史]]，不能仅由根数据结构或根排列判断。^[weyl-context-identity-and-sharing.md:174-187]

HPC capture job `3884807` 确认了修复前 Rust 的两类相反错误：cold canonical-dual 情形中，原版允许关系比较和乘法，Rust 却给出相反的关系结果并拒绝乘法；独立预热的不兼容 owner 情形中，原版拒绝关系与乘法，Rust 的关系运算却返回布尔值。独立检查绑定了完整原始输出、冻结原版 binary、源码和输入。^[weyl-context-identity-and-sharing.md:140-146]

## tests-first 与证据纪律

回归固定四份原版 stdout/stderr golden，在 `session.rs` 中使用两个普通 `#[test]`，完整比较输出及有序 `(ErrorKind, message)`；不使用 `should_panic`，也不把 Rust 的错误输出作为预期结果。original 与 Rust 各自在 fresh process 中保存完整输出、顺序、退出状态、wall time 和 RSS。^[weyl-context-identity-and-sharing.md:148-151, weyl-context-identity-and-sharing.md:309-311]

BEFORE 必须实际执行目标回归并证明预期失败。首个 BEFORE job `3884862` 在 checker/driver 自检阶段失败，尚未执行 Cargo、Atlas 或数学回归，因此不能作为 Rust 语义失败证据，也不能解锁生产修复。原始 stage 应保留，修正后的后继须重新完成独立 HPC BEFORE；这一要求属于[[HPC 验收证据链]]。^[weyl-context-identity-and-sharing.md:153-157]

源码预测与捕获结果必须分开保存。旧 discovery catalog 的 `source_predicted_not_captured` 是历史输入，不能修改预测数组后将其称为 golden；实际回归依据独立审查的 capture 和完整原版输出建立。^[weyl-context-identity-and-sharing.md:13-18, weyl-context-identity-and-sharing.md:208-212]

## A1 首层门禁及其上限

两个 core-only A1 fixture 覆盖 same owner、alias、fresh-equal datum、两个 warm-source/cold-target dual 方向，以及重新绑定后的旧值寿命；观察 `word`、`length`、`root_permutation`、`root_datum`、`=`、`!=`、`*`。错误路径包括 preference-distinct owner、cold source/prewarmed canonical target、owner/dual mismatch、越界与负 generator，每次错误后均设置 recovery marker。^[weyl-context-identity-and-sharing.md:301-307]

A1 跨 dual 的乘法只有 \(s_0s_0=1\)，无法揭示错误的生成元重编号或直接复合外部坐标系根排列的问题。寿命用例仍由 `wc_alias` 保持旧 RootDatum 存活，未证明仅靠 WeylElt 就能维持 datum 生命周期；Atlas 输出也不能证明 fresh-equal owner 使用独立 coordinate cell。这些缺口分别需要额外 lifetime fixture 和 HPC-only `Weak`/work-count 单元守卫。^[weyl-context-identity-and-sharing.md:313-318]

## AFTER 验收与生产落地

AFTER 的失败需要区分验证设施错误与目标语义错误。AFTER-v1 因源码 manifest 摘要比较对象错误而失败；AFTER-v2 因 sbatch 标签未随 stage 迁移，在任何 gate 前失败。后继修复保留历史证据，并将 sbatch 标签纳入检查；无报告的前驱不虚构 report 哈希。^[weyl-context-identity-and-sharing.md:22-43]

AFTER-v4 要求预热情形的 stderr 逐字节相等，但两侧错误外层格式不同，因此被归为 harness 过度断言。AFTER-v5 将该项契约改为有序 error summary 比较，其余输入逐字节不变。^[weyl-context-identity-and-sharing.md:59-62]

AFTER-v5 job `3900050` 最终为 `COMPLETED 0:0`，13 条命令全部 exit0，涵盖 127 项 checker、release build、632 项 inventory、两个回归和 ladder 对照。cold_dual 完全字节相等；prewarmed_dual 的 stdout、退出码及有序 error summary 一致。来源列出的验收记录为 `math_weyl_context_core_after_v5_acceptance_2026_10_06.json`，report SHA 前缀为 `3288480d…`；接受范围仅为 A1 限定语义，不授予 cache、performance、memory、rank 或更广数学 release。^[weyl-context-identity-and-sharing.md:63-69]

修复以生产提交 `690c2b92` 落地。由于 gate 验证整树源码 manifest，生产落地前逐字节比对确认零缺失、零不符、零多余，再将全部 `crates/**` 变更作为同一构建单元提交；仅挑选修复涉及的文件不能保持已验证树的一致性。^[weyl-context-identity-and-sharing.md:70-74]

## 后续语义门禁与预测修正

A1 之后仍须逐步覆盖 G2 非对称 interface-order 见证、B2/C2、两个乘法操作数顺序、inner-class dual construction、`no_value` relations，以及 sole-WeylElt lifetime。截至来源记录的 2026-10-09 状态，`weyl-context-g2-v1` 已冻结并彩排，因隧道中断暂缓提交；后续见证仍为 provisional，不能视为已通过。^[weyl-context-identity-and-sharing.md:95-100, weyl-context-identity-and-sharing.md:320-330]

G2 还说明 fixture 本身需要源码核对：canonical dual 带转置 coroot 矩阵，不能由 `adjoint(G2,·)` 构造。因此，capture 前登记将 `WG_DUAL_OWNER` 与 `WG_REVERSE_OWNER` 的预期修正为两个引擎均输出 `false`，原 contract 的 `true` 属于预测失准，不能预先归类为引擎分歧。相关背景见[[Canonical dual 的转置根数据与 G2 预热见证]]。^[weyl-context-identity-and-sharing.md:75-89]

使用 `adjoint(G2,false)` 的预热无法占用 canonical dual 的 cold-share 槽位，预期不会产生所需的拒绝行为；真正的 G2 预热拒绝见证需要显式构造转置内容。B2/C2 则不同：C2 的固定 Cartan 正是 B2 的转置，`dual(SC(B2,true))` 与 `adjoint(C2,false)` 内容一致。这些仍是捕获前的登记预期。^[weyl-context-identity-and-sharing.md:86-94]

## 工作量、性能与内存门禁

只有后续语义 gate 全部通过，才能加入 one-build work-count 测试，再进行同节点、交替顺序、fresh-process 的 time/CPU/RSS A/B。`59 → 至多 5` 只是 caller-level 构建次数假设，并非实测加速；详见[[Weyl 上下文共享的性能与内存证据边界]]。^[weyl-context-identity-and-sharing.md:320-324]

内存收益也不能预报为单调减少。多个 WeylElt 同时存活时共享 RootSystem 应减少重复对象，但短命元素场景中，owner 强持有 kernel 可能使 peak RSS 不变甚至略升。后续报告必须同时给出 live owner/kernel/build 数、分配和 peak RSS，不能仅凭 `Arc`/`Weak` 设计宣称节省内存。^[weyl-context-identity-and-sharing.md:332-335]

## Sources

- [weyl-context-identity-and-sharing.md](../../sources/weyl-context-identity-and-sharing.md) — Weyl 对象身份、dual 历史与安全共享边界。
