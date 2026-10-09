---
title: Weyl 元素的可失败关系与跨坐标运算
summary: Weyl 元素的等于、不等于和乘法须先检查群身份，包括 no_value 路径；兼容但坐标不同的值通过在左侧重放右侧外部生成元词运算，乘积保留左 owner。
sources:
  - weyl-context-identity-and-sharing.md
kind: concept
createdAt: "2026-10-09T15:16:29.646Z"
updatedAt: "2026-10-09T15:16:29.646Z"
tags:
  - Weyl群
  - 错误语义
  - 坐标转换
aliases:
  - weyl-元素的可失败关系与跨坐标运算
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Weyl 元素的可失败关系与跨坐标运算

Weyl 元素的 `=`、`!=` 与 `*` 都需要先检查抽象 Weyl 群身份是否兼容。关系运算因此可能失败，而不一定返回布尔值；兼容元素若使用不同的根坐标系，还必须先转换作用表示，再进行比较或乘法。已落地的 Rust 修复以 `AbstractWeylGroup` 的 `Arc` 身份作兼容性判断，并在左侧坐标系中重放右侧的 external generator word。^[weyl-context-identity-and-sharing.md:249-261]

## 兼容性取决于对象身份与构造历史

original Atlas 的每个活 `root_datum_value` 都持有一个惰性构造的 `WeylGroup`。调用 `dual()` 时，若 canonical dual 尚未初始化 Weyl 群，就共享 source 的群对象；若 target 已经预热，则保留其已有对象。因此，兼容性不仅取决于 Cartan 矩阵、根数据结构或根排列，还取决于 canonical owner 的存活期与 [[dual 预热历史与 Weyl 群兼容性|dual 预热历史]]。^[weyl-context-identity-and-sharing.md:174-187]

original 的 `W_elt_value` 强持有其 root datum，并引用该 datum 的 Weyl 群。二元 `=`、`!=`、`*` 在产生结果之前检查两个 WeylGroup 的地址；地址不同就抛出 `Weyl group mismatch`。该检查先于 `no_value` gate，即使不需要表达式的值，也必须执行兼容性检查。^[weyl-context-identity-and-sharing.md:183-187]

## 为什么关系运算必须可失败

修复前，Rust 的二元关系通过通用 `DomainValue::PartialEq` 比较结构 handle 与内部 `WeylElement`，乘法也以结构 handle 相等作为兼容性条件。这种方式无法表达 original 的 WeylGroup 指针身份，也不能在 `no_value` 关系求值中报告 mismatch；该描述现仅保留为历史对照。^[weyl-context-identity-and-sharing.md:189-206]

A1 的 original capture 确认了两类相反错误：cold canonical-dual 历史中，original 的相等关系成立且乘法成功，旧 Rust 却判为不等并拒绝乘法；独立预热的不兼容 owner 中，original 拒绝关系与乘法，旧 Rust 的关系运算却返回布尔值。修复因此必须让 `=`、`!=` 脱离不可失败的通用比较路径，成为带兼容性检查的 domain relation。^[weyl-context-identity-and-sharing.md:140-146, weyl-context-identity-and-sharing.md:283-284]

已实现方案让 `RootDatumHandle` 携带驻留的 `DatumWeylIdentity`，并在 `=`、`!=`、`*` 中首先比较 abstract-group `Arc` 身份；关系检查在 `no_value` 级别同样执行。该修复保留结构性 RootDatum `Eq`/`Debug`，没有把 Weyl 元素的兼容性直接等同于 RootDatum 的结构比较。^[weyl-context-identity-and-sharing.md:249-261]

## 跨坐标运算与结果归属

抽象群身份兼容不意味着内部 root permutation 可以直接比较或复合。两个值可能共享抽象 Weyl 群，却拥有不同的 coordinate kernel。此时应把右值的 external generator word 放到左值的 `RootSystem` 中重放，再进行运算；乘积归左侧 owner。相关机制见 [[Weyl 元素兼容性与跨坐标词重放]] 与 [[Rust Weyl 内核与抽象群的无环所有权模型]]。^[weyl-context-identity-and-sharing.md:279-282]

左侧 owner 的保留也是 original 的明确行为。上游逐行确认还指出，新建 dual RootDatum 从转置 Cartan 矩阵重新编号，因此跨 dual 操作不能仅凭共享群身份就假定坐标编号一致。^[weyl-context-identity-and-sharing.md:75-85]

## 验证状态与覆盖边界

AFTER-v5 job `3900050` 完成，修复以生产提交 `690c2b92` 落地。cold_dual 的输出完全字节相等；prewarmed_dual 在 stdout、退出码和有序 error summary 层面一致。接受范围仅限 A1 语义，不授予缓存、性能、内存、高 rank 或更广数学 release。^[weyl-context-identity-and-sharing.md:63-74]

A1 的跨 dual 乘法只有 \(s_0s_0=1\)，无法发现错误的生成元重编号或直接复合 foreign root permutation；既有 fixture 也未证明仅由 WeylElt 维持 datum 生命周期。因此仍需非对称 G2、B2/C2、两个乘法操作数顺序、inner-class dual construction、`no_value` relations 与独立生命周期见证。来源最新状态中，G2 gate 已冻结并彩排，等待提交，其余后续 fixture 为 provisional；这些限制属于 [[Weyl 语义回归的递进验证门禁]]。^[weyl-context-identity-and-sharing.md:313-330]

## Sources

- [weyl-context-identity-and-sharing.md](../../sources/weyl-context-identity-and-sharing.md) — Weyl 对象身份、dual 历史与安全共享边界。
