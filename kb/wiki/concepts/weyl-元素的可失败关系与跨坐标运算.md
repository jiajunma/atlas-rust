---
title: Weyl 元素的可失败关系与跨坐标运算
summary: 等于、不等于和乘法在 no_value 门前检查抽象群身份，兼容但坐标不同的值通过在左侧重放右侧外部生成元词运算，乘积保留左属主。
sources:
  - weyl-context-identity-and-sharing.md
kind: concept
createdAt: "2026-10-09T15:16:29.646Z"
updatedAt: "2026-10-10T00:55:41.174Z"
tags:
  - Weyl
  - 运算语义
  - 坐标转换
aliases:
  - weyl-元素的可失败关系与跨坐标运算
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Weyl 元素的可失败关系与跨坐标运算
summary: Weyl 元素的关系与乘法先检查抽象群身份，包括 no_value 路径；兼容但坐标不同的值通过在左侧重放右侧外部生成元词运算，乘积保留左属主。
sources:
  - weyl-context-identity-and-sharing.md
kind: concept
tags:
  - Weyl群
  - 运算语义
  - 错误处理
aliases:
  - weyl-元素的可失败关系与跨坐标运算
---

# Weyl 元素的可失败关系与跨坐标运算

Weyl 元素的 `=`、`!=` 与 `*` 都必须先检查抽象 Weyl 群身份。因此，相等与不等关系可能报错，并非总能返回布尔值。已落地的 Rust 修复以 `AbstractWeylGroup` 的 `Arc` 身份判断兼容性，再在左侧坐标系中重放右侧的外部生成元词；关系检查在 `no_value` 级别同样执行。^[weyl-context-identity-and-sharing.md:249-261]

## 兼容性取决于身份与构造历史

original Atlas 的每个活 `root_datum_value` 持有一个惰性初始化的 `WeylGroup`。调用 `dual()` 时，若 canonical dual 的群尚未初始化，就安装 source 的同一个群对象；若目标已经预热，则保留其原有对象。兼容性因此包含 canonical owner 的存活期与 [[dual 预热历史与 Weyl 群兼容性|dual 预热历史]]，不能仅由 Cartan 矩阵、根数据结构或根排列决定。^[weyl-context-identity-and-sharing.md:174-187]

`W_elt_value` 强持有其 root datum，并引用该 datum 的 Weyl 群。二元 `=`、`!=`、`*` 在产生结果之前比较 WeylGroup 地址；地址不同便抛出 `Weyl group mismatch`。这一检查先于 `no_value` gate，即使调用方不需要表达式的值，也必须保留错误行为。^[weyl-context-identity-and-sharing.md:183-187]

## 可失败关系的 Rust 实现

修复前，Rust 的二元关系通过通用 `DomainValue::PartialEq` 比较结构 handle 与内部 `WeylElement`，乘法也用结构 handle 相等判断兼容性。这无法表达 original 的 WeylGroup 指针身份，也不能在 `no_value` 关系求值中报告 mismatch。来源已明确将这些行为标为修复前的历史对照。^[weyl-context-identity-and-sharing.md:189-206]

A1 的原版捕获确认了两类相反差异：cold canonical-dual 历史中，original 的 `=` 为 true、`!=` 为 false，乘法成功，旧 Rust 却返回相反布尔值并拒绝乘法；对于独立预热的不兼容 owner，original 拒绝关系与乘法，旧 Rust 的关系运算却返回布尔值。^[weyl-context-identity-and-sharing.md:140-146]

已实现方案使 `RootDatumHandle` 携带 `Arc<DatumWeylIdentity>`，通过按完整 datum 内容及 preference 建立的弱驻留表取得身份。二元操作先检查 abstract-group `Arc` 身份，关系检查在 `no_value` 路径也执行；RootDatum 自身的结构性 `Eq`/`Debug` 保持不变。因此，Weyl 关系的可失败兼容性检查与 RootDatum 的结构比较具有不同职责。^[weyl-context-identity-and-sharing.md:249-261]

## 跨坐标重放与乘积归属

共享抽象群身份不意味着两个值使用相同的根坐标。对于兼容但坐标 kernel 不同的值，不能直接比较或复合内部 root permutation；需要把右值的 external generator word 在左值的 `RootSystem` 中重放，再进行比较或组合，乘积归左侧 owner。来源的共享设计说明给出了这一规则，已落地修复明确实现了左侧坐标重放，详见 [[Weyl 元素兼容性与跨坐标词重放]]。^[weyl-context-identity-and-sharing.md:249-261, weyl-context-identity-and-sharing.md:279-284]

乘积保留左操作数 owner 也是 original 的明确行为。上游逐行核对还确认，新建 canonical dual RootDatum 从转置 Cartan 矩阵重新编号，而 DualTag 元数据 dual 保持原序。因此，跨 dual 运算不能仅凭共享群身份假定根编号一致，相关区别见 [[Canonical dual 与 DualTag 的根编号差异]]。^[weyl-context-identity-and-sharing.md:75-85]

## 验证状态与覆盖边界

来源记录 AFTER-v5 job `3900050` 以 `COMPLETED 0:0` 完成，修复以生产提交 `690c2b92` 落地。cold_dual 完全字节相等；prewarmed_dual 在 stdout、退出码与有序 error summary 层面一致。验收记录为 `math_weyl_context_core_after_v5_acceptance_2026_10_06.json`，接受范围仍限于 A1 语义，不授予缓存、性能、内存、高 rank 或更广数学 release。^[weyl-context-identity-and-sharing.md:63-74]

A1 的跨 dual 乘法只有 \(s_0s_0=1\)，无法发现生成元重编号错误或直接复合 foreign root permutation 的问题。既有 rebind fixture 仍由 `wc_alias` 保持旧 RootDatum 存活，也未证明仅靠保存的 WeylElt 就能维持 datum 生命周期。^[weyl-context-identity-and-sharing.md:313-318]

后续仍需覆盖非对称 G2、B2/C2、两个乘法操作数顺序、inner-class dual construction、`no_value` relations 和独立生命周期见证。来源截至 2026-10-09 的状态是：G2 gate 已冻结并彩排，等待隧道恢复后提交；后续见证仍为 provisional fixture。这些属于 [[Weyl 语义回归的递进验证门禁]]，不能由 A1 的通过结果替代。^[weyl-context-identity-and-sharing.md:320-330]

## Sources

- [weyl-context-identity-and-sharing.md](../../sources/weyl-context-identity-and-sharing.md) — Weyl 对象身份、dual 历史与安全共享边界。
