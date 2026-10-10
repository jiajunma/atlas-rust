---
title: Canonical dual 的转置根数据与 G2 预热见证
summary: 来源预测普通 adjoint(G2,false) 无法预热 canonical dual 的转置内容，真正的预热拒绝见证需显式构造转置根数据；该预测尚待原版捕获验证。
sources:
  - weyl-context-identity-and-sharing.md
kind: concept
createdAt: "2026-10-09T21:13:30.838Z"
updatedAt: "2026-10-09T22:53:25.224Z"
tags:
  - G2
  - 根数据
  - 差分验证
aliases:
  - canonical-dual-的转置根数据与-g2-预热见证
  - CD的G预
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Canonical dual 的转置根数据与 G2 预热见证
summary: Canonical dual 从转置 Cartan 重新编号；普通 adjoint(G2,false) 无法预热其规范目标，真正的 G2 预热拒绝见证须显式构造转置内容，相关行为仍待捕获验证。
sources:
  - weyl-context-identity-and-sharing.md
kind: concept
tags:
  - weyl
  - root-data
  - regression-testing
aliases:
  - canonical-dual-的转置根数据与-g2-预热见证
provenanceState: extracted
---

# Canonical dual 的转置根数据与 G2 预热见证

Canonical dual 的构造会交换根与余根、翻转 preference，并从转置 Cartan 矩阵重新编号。对 G2，`adjoint(G2,false)` 无法构造出所需的 canonical dual 内容，因此不能用它检验 canonical target 已预热时的跨 owner 拒绝行为。来源提出以显式转置根数据构造有效见证；这一 G2 行为仍属于捕获前的预登记预测。^[weyl-context-identity-and-sharing.md:75-100]

## 转置根数据与规范身份

在冻结的 original Atlas revision `7e1b958c` 中，`PreRootDatum::dualise` 只交换 roots/coroots 并翻转 preference；随后新建的 `RootDatum` 从转置 Cartan 重新编号。这与 `DualTag` 元数据 dual 的保序行为不同，后者的上游注释注明仅供 Fokko 使用。^[weyl-context-identity-and-sharing.md:75-80]

Original Atlas 按完整 `PreRootDatum` 内容进行弱驻留，键包括 simple roots、simple coroots 和 `prefer_coroots`。相同对象仍存活时复用其共享指针，释放后则可在原槽位重建；语言层 RootDatum 的 `=`／`!=` 比较的是驻留后的指针身份。预热构造能否命中 canonical dual，因而取决于完整内容及 preference，而不能仅凭群类型名称判断，参见 [[RootDatum 弱驻留与规范活对象身份]]。^[weyl-context-identity-and-sharing.md:164-172]

## 预热如何影响 Weyl 兼容性

每个活的 root datum 都持有初始为空的 Weyl group 槽位，首次调用 `W()` 时才构造。`dual()` 先取得 canonical target：目标槽位为空时，确保 source 的 Weyl group 已建立，再将同一共享指针安装到目标；目标已预热时，绝不覆盖。因此，即使是规范对偶关系，两侧也可能因预热历史而持有不同的 WeylGroup identity，详见 [[dual 预热历史与 Weyl 群兼容性]]。^[weyl-context-identity-and-sharing.md:174-181]

Weyl 元素的 `=`、`!=`、`*` 在产生结果前先检查 WeylGroup 地址，不一致便抛出 `Weyl group mismatch`，且检查先于 `no_value` gate。元素强持有所属 root datum，乘积保留左操作数的 owner；结构相等或根置换相同都不足以代替这一身份检查。^[weyl-context-identity-and-sharing.md:81-85, weyl-context-identity-and-sharing.md:183-187]

## G2 见证与预测修正

来源的上游逐行阅读指出，G2 的 canonical dual 带有转置 coroot 矩阵，任何 `adjoint(G2,·)` 都无法构造出该内容。由此登记的预测是：`WG_DUAL_OWNER` 与 `WG_REVERSE_OWNER` 在两个引擎中均应打印 `false`。冻结 contract 原先的 `true` 预测应作为预测失准记录，而非视为引擎分歧；来源尚未将这一预测确认为捕获结果。^[weyl-context-identity-and-sharing.md:86-89]

原预热 fixture 使用 `adjoint(G2,false)`，无法占用 canonical dual 的 cold-share 槽位，因此其 dual 侧三元组并未触发所需的预热条件，预期两个引擎都不抛错。真正的 G2 预热拒绝见证需要显式调用 `root_datum`，以根矩阵 $I_2$、余根矩阵 $\begin{pmatrix}2&-3\\-1&2\end{pmatrix}$ 和 preference 参数 `false` 构造并预热准确的转置内容。^[weyl-context-identity-and-sharing.md:89-93]

B2/C2 提供了更直接的对照：C2 的固定 Cartan 矩阵正是 B2 的转置，所以 `dual(SC(B2,true))` 与 `adjoint(C2,false)` 内容一致。G2 普通 adjoint 构造无法预热 canonical dual 的问题不影响这一对照。^[weyl-context-identity-and-sharing.md:93-94]

## Rust 实现与覆盖要求

已落地的 Rust 修复使 `RootDatumHandle` 携带 `Arc<DatumWeylIdentity>`，并按完整 datum 内容加 preference 弱驻留 identity。`dual(RootDatum)` 仅在 canonical target 仍 cold 时共享 source 的 abstract group，绝不覆盖预热目标。二元运算先比较 abstract-group `Arc` 身份，再在左侧坐标系重放右侧 external word；关系运算在 `no_value` 级别同样执行身份检查，参见 [[Weyl 元素的可失败关系与跨坐标运算]]。^[weyl-context-identity-and-sharing.md:249-261]

G2 的非对称 interface-order 见证用于补足 A1 的覆盖缺口：A1 跨 dual 乘法只有 $s_0s_0=1$，无法发现生成元重编号错误或直接复合 foreign root permutation 的错误。后续还需分别覆盖 B2/C2、两个乘法操作数顺序、inner-class dual construction、`no_value` relations，以及仅由 WeylElt 维持 datum 生命周期的情形。这些属于 [[Weyl 语义回归的递进验证门禁]]。^[weyl-context-identity-and-sharing.md:313-330]

## 验证状态与证据边界

来源的 2026-10-09 更新记录，AFTER-v5 job `3900050` 已以 `COMPLETED 0:0` 完成，修复随后以生产提交 `690c2b92` 落地。cold_dual 完整字节相等，prewarmed_dual 在 stdout、退出码和有序 error summary 下保持一致；接受范围仍仅限 A1 语义，不授予缓存、性能、内存、更高 rank 或更广数学范围的验收。来源列出的验收记录为 `math_weyl_context_core_after_v5_acceptance_2026_10_06.json`，report SHA 摘要为 `3288480d…`。^[weyl-context-identity-and-sharing.md:59-74]

同一更新中，`g2-v1` 的 65 文件 payload 已冻结并彩排通过，因隧道中断暂缓提交。包括显式转置内容 G2 transposed-prewarm 在内的七个后续 fixture 仍为 provisional、未接线。因此，本页的 G2 结论限于源码预期与见证设计，不能视为已经通过的 G2 兼容性验证。^[weyl-context-identity-and-sharing.md:95-100]

## Sources

- [weyl-context-identity-and-sharing.md](../../sources/weyl-context-identity-and-sharing.md) — Weyl 对象身份、dual 历史与安全共享边界。
