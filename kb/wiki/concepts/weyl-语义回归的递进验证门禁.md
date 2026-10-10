---
title: Weyl 语义回归的递进验证门禁
summary: AFTER-v5 与生产提交 690c2b92 仅确立 A1 限定语义验收，尚不能证明非对称生成元编号、独占元素生命周期或缓存与性能收益。
sources:
  - weyl-context-identity-and-sharing.md
kind: concept
createdAt: "2026-10-09T15:17:15.306Z"
updatedAt: "2026-10-10T00:56:09.979Z"
tags:
  - Weyl
  - hpc
  - 证据边界
aliases:
  - weyl-语义回归的递进验证门禁
confidence: 1
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Weyl 语义回归的递进验证门禁
summary: 通过原版捕获与 BEFORE/fix/AFTER 验证 Weyl 身份、对偶历史及错误行为；A1 修复已限定验收落地，其他语义、性能与内存门禁仍需独立完成。
sources:
  - weyl-context-identity-and-sharing.md
kind: concept
tags:
  - 回归测试
  - 证据范围
  - HPC
aliases:
  - weyl-语义回归的递进验证门禁
---

# Weyl 语义回归的递进验证门禁

Weyl 语义回归采用“原版行为捕获 → 建立回归 → BEFORE 证明修复前失败 → 修复 → AFTER 验证”的递进流程，首先验证对象身份、对偶构造历史和错误行为，再独立验证构建次数、性能与内存。截至来源的 2026-10-09 更新，AFTER-v5 已完成 A1 限定语义验收，修复以提交 `690c2b92` 落地；这不代表更高 rank、缓存或性能验证完成。^[weyl-context-identity-and-sharing.md:138-157, weyl-context-identity-and-sharing.md:63-74, weyl-context-identity-and-sharing.md:320-330]

## 为什么需要历史敏感的回归

original Atlas 的 `dual()` 仅在 canonical target 的 Weyl group 尚未建立时共享 source 的 group；目标已经预热时绝不覆盖。Weyl 元素的 `=`、`!=`、`*` 在产生结果前比较 WeylGroup 地址，不同则抛出 `Weyl group mismatch`，且检查先于 `no_value` gate。因此兼容性包含 owner 的存活期与[[dual 预热历史与 Weyl 群兼容性|对偶预热历史]]，不能仅由根数据结构或根排列决定。^[weyl-context-identity-and-sharing.md:174-187]

HPC capture job `3884807` 确认了修复前 Rust 的两类错误：cold canonical-dual 情形下，原版关系结果为相等且乘法成功，Rust 却判为不等并拒绝乘法；独立预热的不兼容 owner 情形下，原版拒绝关系与乘法，Rust 的关系运算却返回布尔值。独立检查绑定完整原始输出、冻结原版 binary、源码与输入。^[weyl-context-identity-and-sharing.md:140-146]

## tests-first 与证据纪律

回归固定四份原版 stdout/stderr golden，并在 `session.rs` 中使用两个普通 `#[test]`，完整比较输出及有序 `(ErrorKind, message)`。不使用 `should_panic`，也不把 Rust 的错误输出作为预期结果。original 与 Rust 须分别在 fresh process 中保存完整输出、顺序、退出状态、wall time 和 RSS。^[weyl-context-identity-and-sharing.md:148-151, weyl-context-identity-and-sharing.md:309-311]

BEFORE 必须实际执行目标回归。首个 BEFORE job `3884862` 在 checker/driver 自检阶段失败，尚未执行 Cargo、Atlas 或数学回归，因此不是预期的 Rust 失败证据，不能解锁生产修复。原始 stage 保持不变，修正后的后继仍须完成独立 HPC BEFORE；相关原则见[[HPC 验收证据链]]。^[weyl-context-identity-and-sharing.md:153-157]

源码预测不能替代捕获结果。旧 discovery catalog 中的 `source_predicted_not_captured` 是不可改写的历史输入，不能修改预测数组后称其为 golden；回归应依据独立审查的 capture 与完整原版输出。^[weyl-context-identity-and-sharing.md:13-18, weyl-context-identity-and-sharing.md:208-212]

## A1 首层门禁与覆盖上限

两个 core-only A1 fixture 覆盖 same owner、alias、fresh-equal datum、两个 warm-source/cold-target dual 方向，以及重新绑定后的旧值寿命；观察 `word`、`length`、`root_permutation`、`root_datum`、`=`、`!=`、`*`。错误路径涵盖 preference-distinct owner、cold source/prewarmed canonical target、owner/dual mismatch、越界与负 generator，并在每次错误后设置 recovery marker。^[weyl-context-identity-and-sharing.md:301-307]

A1 跨 dual 的乘法只有 \(s_0s_0=1\)，无法发现错误的生成元重编号或直接复合另一坐标系根排列的问题。寿命用例仍由 `wc_alias` 保持旧 RootDatum 存活，未证明仅靠保存的 WeylElt 就能维持 datum 生命周期；Atlas 输出也不能证明 fresh-equal owner 使用独立 coordinate cell。后两项分别需要额外 lifetime fixture 和 HPC-only `Weak`/work-count 单元守卫。^[weyl-context-identity-and-sharing.md:313-318]

## AFTER 验收与生产落地

AFTER 失败必须区分验证设施错误与目标语义错误。AFTER-v1 错将最终修复 manifest 摘要与 tests-only 常量比较，未执行 checker/Atlas 命令；AFTER-v2 因 sbatch 标签未随 stage 迁移，在任何 gate 前失败。后继修复保留历史证据，将 sbatch 标签纳入检查，并对没有 report 的前驱省略相应字段，而非虚构哈希。^[weyl-context-identity-and-sharing.md:22-43]

AFTER-v4 要求预热情形的 stderr 逐字节相等，但两侧错误外层格式不同，因此被归为 harness 过度断言。AFTER-v5 将该项契约改为有序 error summary 比较，其余输入逐字节不变。^[weyl-context-identity-and-sharing.md:59-62]

AFTER-v5 job `3900050` 最终为 `COMPLETED 0:0`，13 条命令全部 exit0，涵盖 127 项 checker、release build、632 项 inventory、两个回归和 ladder 对照。cold_dual 完全字节相等；prewarmed_dual 的 stdout、退出码和有序 error summary 一致。来源列出的验收记录为 `math_weyl_context_core_after_v5_acceptance_2026_10_06.json`，report SHA 前缀为 `3288480d…`。接受范围仅为 A1 限定语义，不授予 cache、performance、memory、rank 或更广数学 release，全部 release flag 保持 `FALSE`。^[weyl-context-identity-and-sharing.md:9-12, weyl-context-identity-and-sharing.md:63-69]

生产提交 `690c2b92` 落地前，整树与 v5 源码 manifest 逐字节核对，确认零缺失、零不符、零多余。因此全部 `crates/**` 变更作为一个构建一致单元提交，而非只挑出修复涉及的五个文件；后者不是 gate 单独验证过的子集。^[weyl-context-identity-and-sharing.md:70-74]

## 后续语义门禁与预测修正

A1 之后仍须逐步覆盖 G2 非对称 interface-order 见证、B2/C2、两个乘法操作数顺序、inner-class dual construction、`no_value` relations 和 sole-WeylElt lifetime。截至来源记录，`weyl-context-g2-v1` 已冻结并彩排，因隧道中断暂缓提交；七个后续见证 fixture 已起草为 provisional、尚未接线，不能视为已通过。^[weyl-context-identity-and-sharing.md:95-100, weyl-context-identity-and-sharing.md:320-330]

G2 说明 fixture 本身也需要源码核对。canonical dual 使用转置 coroot 矩阵，任何 `adjoint(G2,·)` 都不能构造该内容；capture 前登记因此预期 `WG_DUAL_OWNER` 与 `WG_REVERSE_OWNER` 在两个引擎中均打印 `false`。冻结 contract 的 `true` 是预测失准，不能预先归类为引擎分歧；编号背景见[[Canonical dual 与 DualTag 的根编号差异]]。^[weyl-context-identity-and-sharing.md:75-89]

用 `adjoint(G2,false)` 预热不能占用 canonical dual 的 cold-share 槽位，登记预期是两个引擎均不抛错。真正的 G2 预热拒绝见证需要显式构造转置内容。B2/C2 则更直接：C2 的固定 Cartan 正是 B2 的转置，`dual(SC(B2,true))` 与 `adjoint(C2,false)` 内容一致。这些均为捕获前的源码预期，而非已完成的验证结果。^[weyl-context-identity-and-sharing.md:86-94]

## 构建次数、性能与内存门禁

只有后续语义 gate 全部通过，才能加入 one-build work-count 测试，再进行同节点、交替顺序、fresh-process 的 time/CPU/RSS A/B。`59 → 至多 5` 只是调用方层面的构建次数假设，不是实测加速。v8 的四次数学调用仅耗时 0.00–0.01 秒，而总作业时间主要用于构建，也不能据此计算 Rust/original 加速比。^[weyl-context-identity-and-sharing.md:159-160, weyl-context-identity-and-sharing.md:320-324]

内存收益不能预报为单调减少。多个 WeylElt 同时存活时，共享 RootSystem 应减少重复对象；但短命元素场景中，owner 强持有 kernel 可能令 peak RSS 不变甚至略升。后续报告必须同时给出 live owner/kernel/build 数、分配与 peak RSS，不能仅凭 `Arc`/`Weak` 设计宣称节省内存。^[weyl-context-identity-and-sharing.md:332-335]

## Sources

- [weyl-context-identity-and-sharing.md](../../sources/weyl-context-identity-and-sharing.md) — Weyl 对象身份、dual 历史与安全共享边界。
