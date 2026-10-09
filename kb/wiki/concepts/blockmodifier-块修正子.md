---
title: BlockModifier 块修正子
summary: BlockModifier 组合定位器与有理权平移，支持恒等构造、无校验包装及保留整数据标识和整单根信息的重置；本来源快照中尚未接入消费方。
sources:
  - block-access-modifier.md
kind: concept
createdAt: "2026-10-09T14:40:33.332Z"
updatedAt: "2026-10-09T20:47:54.944Z"
tags:
  - 表示参数
  - 块修正子
  - Rust设计
aliases:
  - blockmodifier-块修正子
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: BlockModifier 块修正子
summary: BlockModifier 组合块定位器与有理权平移，通过 Weyl 姿态变换、整正交差和逆向恢复处理表示参数；来源记录中该切片尚未接入现存消费方。
sources:
  - block-access-modifier.md
kind: concept
tags:
  - 表示论
  - Rust设计
  - 块修正子
aliases:
  - blockmodifier-块修正子
---

# BlockModifier 块修正子

`BlockModifier` 将块定位器与有理权平移量组合起来，支持非恒等生成元姿态下的参数变换、相对化和标准参数恢复。它位于 `block_modifier.rs`，属于“nonidentity generator attitude”实现切片的第二步，第一步是定位器。在来源记录的状态下，两部分均尚未接线，块修正子没有现存消费方。^[block-access-modifier.md:44-50, block-access-modifier.md:76-81, block-access-modifier.md:99-99]

## 数据结构与初始化

`BlockModifier { locator: BlockLocator, shift: RationalWeight }` 包含一个[[Weyl 姿态定位器]]和一个有理权平移量。`trivial(system, simp_int)` 将 Weyl 元素 `w` 与简单生成元置换 `simple_pi` 设为恒等，将 `shift` 设为零，并以 `u32::MAX` 作为 `int_sys` 的局部哨兵。该哨兵对应上游的 `-1`，永不读取。^[block-access-modifier.md:52-56]

`from_locator` 直接包装定位器，不执行校验。`clear` 保留 `int_sys` 与 `simp_int`，将 `w`、`simple_pi` 和 `shift` 分别重置为恒等、恒等和零。^[block-access-modifier.md:52-56]

## Weyl 姿态变换

`RepContext::transform_srm<LEFT_TO_RIGHT>(w, srm)` 沿 Weyl 词逐字母变换参数，`LEFT_TO_RIGHT` 决定施加方向。每一步依据 `kgb_status(x, s)` 分派：Complex 分支对 `x` 执行 cross，并对有理权分子作简单反射；Real 分支保持 `x` 不变，对分子作以 \(-\rho_R\) 为中心的仿射反射；Imaginary 分支返回 `RepInvariantViolation`。变换结束后，在最终 `x` 处通过 `real_unique` 归一化。^[block-access-modifier.md:66-70]

私有函数 `simple_reflect_numerator` 执行分子更新 \(v \mathrel{-}= \alpha_s(\langle v,\mathrm{coroot}_s\rangle+\mathrm{offset})\)，保持分母不变，算术全程采用 checked 运算。Complex 分支的 offset 为零，Real 分支的 offset 为分母，参见[[有理权分子的 checked 仿射反射]]。^[block-access-modifier.md:66-69, block-access-modifier.md:83-85]

Rust 实现采用 `WeylElement::reduced_word` 给出的典范最左下降约化词，上游则使用 transducer 随元素存储的词。来源将其记录为有意偏差，并转述实现文档关于目标域内无语义差异的声明。两个方向使用同一典范词，使 `transform<false>` 成为 `transform<true>` 的逐字母逆；相对化与标准参数恢复依赖这一性质。相关词构造见[[基于左下降剥离的规范约化词]]。^[block-access-modifier.md:58-62]

## 平移、相对化与参数恢复

`shift_srm` 将 `amount` 加到 `gamma_lambda`，随后在不变的 involution 处归一化。`make_diff_integral_orthogonal` 从两个代表的差中减去其在 \((1-\theta)X^*\) 中的固定原像，经由 `IntegralSubsystem::integral`、`RepTable::integral_codec` 和 `theta_1_preimage`，使结果与 `srm.gamma_lambda()` 的整根系正交。差为零时短路，debug 构建另有正交性断言，参见[[表示参数差的整根系正交化]]。^[block-access-modifier.md:71-75]

`make_relative_to(loc, srm0, bm, srm1)` 先对定位器部分进行逆合成，再使用**更新后的** `bm.w` 执行 `transform<true>`，将 `srm1` 移回基姿态，最后把 `bm.shift` 设为两个 `gamma_lambda` 的整正交差。这一顺序连接了[[定位器的相对姿态变换]]与权参数差的处理。^[block-access-modifier.md:76-78]

`sr_with_modifier(srm, bm, gamma)` 按固定顺序读取修正子：先加 `bm.shift`，再执行 `transform<false>(bm.w)`，最后调用 `to_standard` 恢复标准参数。该流程与相对化共同构成[[块修正子的相对化与标准参数恢复]]。^[block-access-modifier.md:79-81]

## 测试与证据边界

来源记录了两个集成测试。恒等修正子测试使用 A2 的 SL(3,R) fixture，KGB 尺寸为 4，检查 `sr_with_modifier` 与无修正子的 `sr` 一致。相对化往返测试中，锚点对的定位器在同一典范数据上碰撞，`bm.w` 的词为 `[1]`，平移量为 `[0,1]/4`；shift 与 transform 的往返精确恢复查询参数，完整的 `sr_with_modifier` 也恢复查询参数本身，而非仅恢复到相差根平移的参数。^[block-access-modifier.md:89-95]

`transform_srm` 的分支仅经集成测试间接覆盖；`shift_srm` 与 `make_diff_integral_orthogonal` 没有独立单元测试。整个切片尚未接线，这些测试锚点不构成数学或正确性验收。^[block-access-modifier.md:99-105]

来源属于结构性源码阅读记录，上游行号仅转录自代码注释，未核对上游字节。其精确读取身份由快照绑定 Git base 与文件 SHA-256；本次知识维护没有执行 Atlas、Cargo、测试或 benchmark。^[block-access-modifier.md:9-13, block-access-modifier.md:109-115]

## Sources

- [block-access-modifier.md](../../sources/block-access-modifier.md) — 只读块拓扑与块修正子（block_access.rs / block_modifier.rs）
