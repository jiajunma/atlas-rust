---
title: 内类布局（InnerClassLayout）
summary: 汇集打印序 Lie type、内类字母与 Bourbaki 单根置换，经扭转置换、Dynkin 分类及中心环面处理构造。
sources:
  - layout-restricted-roots.md
kind: concept
createdAt: "2026-10-09T14:57:56.905Z"
updatedAt: "2026-10-10T00:41:00.540Z"
tags:
  - 内类
  - 根数据
aliases:
  - 内类布局innerclasslayout
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 内类布局（InnerClassLayout）
summary: 将 distinguished 对合转换为 Lie type、内类字母及 Bourbaki 单根置换，包含半单因子分类、Complex 配对重排与中心环面处理。
sources:
  - layout-restricted-roots.md
kind: concept
tags:
  - 内类
  - 根数据
  - Rust
aliases:
  - 内类布局innerclasslayout
---

# 内类布局（InnerClassLayout）

`InnerClassLayout` 表示一个内类的布局，汇集 Lie type、内类字母和单根的 Bourbaki 置换，实现位于 `crates/atlas-real-group/src/layout.rs`。源材料将其对应到上游 `check_involution` 的计算结果 `lietype::Layout`；这一对应来自代码注释，尚未独立核对上游字节。^[layout-restricted-roots.md:10-22]

## 数据与排列约定

Lie type 中的半单因子按打印顺序排列，Complex 配对的两个因子相邻，随后为每个中心环面维度追加一个 `T1`。每个内类条目对应一个字母，其中 `'C'` 消耗两个 type 因子。Bourbaki 置换的方向为：`perm[k]` 是规范化顺序下第 `k` 个单根在 datum 中的下标。相关约定见 [[Bourbaki 顶点排序与置换语义]]。^[layout-restricted-roots.md:17-22]

构造接收 `IntegerLatticeBudget`，但预算只约束中心环面字母计算所需的 Smith 基计算，不用于半单数据。^[layout-restricted-roots.md:21-22, layout-restricted-roots.md:70-72]

## 构造流程

`build` 依次执行 `twist_permutation`、`dynkin::classify`、`inner_class_letters`，最后处理中心环面。`twist_permutation` 根据 distinguished 对合确定每个单根的像，逐根执行 `id_of`、`image`、回查和去重；任一步失败均返回 `LayoutInvariantViolation`。源材料将这一置换对应到上游 `weyl::Twist`。^[layout-restricted-roots.md:24-27]

### 半单因子的内类字母

`inner_class_letters` 逐个 Dynkin 分支检查扭转作用：逐点固定时给出 `'c'`；分支保持不变但非逐点固定时，偶秩 D 型给出 `'u'`，其他情况给出 `'s'`；分支被映到另一分支时给出 `'C'`。详见 [[Dynkin 分支的内类字母判定与 Complex 因子重排]]。^[layout-restricted-roots.md:29-35]

对于 `'C'`，算法在后续分支中寻找包含 `twist[perm[offset]]` 的配对分支；找不到时报告 `"non-matching Complex factor"`。找到后通过旋转使两个因子相邻，并将第二个因子的 Bourbaki 槽位改写为扭转像。^[layout-restricted-roots.md:31-33]

实现保留了代码注释所述的上游移位顺序：先上移 `type` 切片，再在已经移位的切片上计算移位宽度。注释指出，这一顺序仅在 Complex 对跨越多个秩不同的中间因子时可观测；当前没有构造夹具覆盖该情形。^[layout-restricted-roots.md:33-35]

### 中心环面

`torus_ranks` 从根格的 Smith 基读取商对合，使用 `adapted_basis` 及 `inverse.block × delta × basis.block`，计算时跳过零项。随后将 `tau1 = inv + I` 交给 `classify_plus_identity`，得到 `(compact, complex, split)`，按 `'c'`、`'C'`、`'s'` 顺序追加环面字母。相关概念见 [[中心环面的商对合分类]] 与 [[承载可观测量的适配基（adapted_basis）]]。^[layout-restricted-roots.md:37-41]

## 与限制根系的接口边界

`layout.rs` 与 `restricted_roots.rs` 互不调用。布局构造使用 `InnerClass::distinguished_involution()`，并接收仅用于环面分支的预算；限制根系构造使用 `RootInvolutionData::involution()`，在 `build` 内校验 datum 与格秩一致性，不接收预算。参见 [[内类布局与限制根系的接口边界]]。^[layout-restricted-roots.md:68-72]

## 测试与证据边界

源材料列出六个布局测试锚点：紧 A1 得到 `'c'`；twisted A2 得到 `'s'`；A1.A1 交换得到 `'C'`、两个因子及 `perm = [0,1]`；D4 叉尖交换得到 `'u'`；GL(2) 样式中心环面的商上 −1 作用得到字母 `'c','s'` 与 `"A1.T1"`／`"cs"`；A1.B2.A1 的 Complex 配对旋转得到 `perm = [0,3,1,2]`。^[layout-restricted-roots.md:43-46]

错误分支没有失败路径测试。中心环面字母仅测试到 `'s'`，尚无 `'c'` 或 `'C'` 的环面测试锚点；Complex 配对跨越多个不同秩中间因子的移位情形也未获夹具覆盖。^[layout-restricted-roots.md:33-35, layout-restricted-roots.md:79-82]

本页依据结构性阅读，不构成数学或正确性验收。来源快照 `2026-10-06-layout-restricted-roots.json` 绑定 Git base、两文件字节 SHA-256 与 Kimi 调用记录；草案经维护者对照源码逐条核对改写。此次知识维护未执行 Atlas、Cargo、测试或 benchmark，因此上述测试锚点不代表本次执行结果。^[layout-restricted-roots.md:9-13, layout-restricted-roots.md:79-80, layout-restricted-roots.md:89-93]

## Sources

- [内类布局与限制根系（layout.rs / restricted_roots.rs）](../../sources/layout-restricted-roots.md)
