---
title: 反射子群轨道与 ambient Weyl 见证（domain_builtins/weyl_subgroup.rs）
source: atlas-rust/atlas-core-weyl-subgroup
ingestedAt: 2026-10-09T13:30:00Z
---

# 反射子群轨道与 ambient Weyl 见证（domain_builtins/weyl_subgroup.rs）

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包覆盖
`crates/atlas-core/src/domain_builtins/weyl_subgroup.rs`（433 行）——
`Weyl_orbit`/`Weyl_orbit_ws` 两个领域内建的实现。这是 after-v3 教训
（E0609：修复补丁漏掉本文件）的涉事文件，也是 R3 反例修复的落点。
结构性阅读，不声称数学验收（其 HPC 门见 docs/slices 的
weyl_subgroup_orbits 切片与下游目录）。

## 头部纪律（文件自述）

排序遵循 rootdata.cpp 的 BitMap `basic_orbit`/`extend_orbit`，**不是**
alcove 变体的分层排序；与 original 7e1b958c 的 rootdata.h 不同，初始
dominance 操作**真的用**给定生成元（rootdata.h669/678 在转发
make_(co)dominant 时忽略 g；见证函数才传对 g——R3 反例：A2 空生成元、
输入 [-1,-2] 时原版矩阵轨道 [2,1] 但见证是恒等）。保留的原版反例与
独立的平凡子群不变量见 `docs/slices/weyl_subgroup_orbits_2026-09-29.md`。

## 构造与校验（`Subgroup::new`）

- 参数形状：3 个；dual 形式（`Weyl_orbit_ws` 的 vec, rd, gens 序）由
  首参不是 RootDatum 识别。
- 生成元行：逐个 `internal_root_nbr`（`check_W_subsystem` 先收窄再查每个
  有符号根）；配对矩阵用 i128 中间精度，窄化失败即
  "Integer value too big for Cartan pairing"；`infer_lie_type` 失败时报
  原版逐字错误 "Matrix for root indices is not a Cartan matrix: \n  …"
  （单生成元特例打印 `[[c]]`）。
- `level`/`reflect`：配对后反射，i128→i32 校验窄化（
  "…subgroup orbit coordinate"）。
- `dominant`：首个负 level 生成元，反射，按**应用顺序**记录事件（两种
  作用都如此）。
- `cosets`：**精确单（余）根配对坐标**的陪集树——原版的核选向量在旧
  stab 上配对为零、在生成元 i 上为正 c，本实现取 1：放大子群上配对向量
  按正 c 缩放，每个符号/相等/BFS 边完全一致；有限 Cartan 可逆使限制在
  轨道差上单射；活动子群外的坐标不改变相等。由此避免任意的格核选举与
  定宽溢出。BFS 只对**新层**去重、保留首次插入；活动生成元按根号排序。

## `call`：轨道矩阵 vs 见证词

- `Weyl_orbit`：轨道列组成矩阵（列带校验过的格秩）。
- `Weyl_orbit_ws`：见证词——**权从右到左作用、余权从左到右**（dual 时
  事件正序拼接，否则逆序）；陪集树逐非 stab 生成元扩展；词经
  `build_weyl_context` + 逐 `right_multiply_simple` 重建为元素，再经
  `weyl_elt_value` 冻结 canonical word（见
  [领域值包](atlas-core-domain-values.md)）。
- `validate`：只构造（`Subgroup::new`）——BuildAndDrop 策略，不计算轨道。
- 向量大小不匹配 → "Wrong vector size for Weyl subgroup orbit"（上游会
  越界读短向量；本实现**不模拟**未定义行为）。

## 测试锚点（4 个，本文件内）

1. 空子群不动负权且返回 ambient 恒等（9 个类型 × 两编号 × 两 isogeny ×
   两作用）。
2. 陪集轨道 == 独立穷举闭包，且每个见证可重建（`word`/`root_datum` 往返
   + 逐见证 `*` 重建轨道列）。
3. 全单子群保持完整全群轨道序（与全群 `Weyl_orbit`/`_ws` 逐等）。
4. 非法生成元在**丢弃前**匹配原版诊断（非法根号、非 Cartan 矩阵文本、
   i32 收窄）；溢出与错秩是安全错误而非回绕轨道。

## 边界与限制

- 生成元是任意有符号根号（配对合法即可），不是单根；原版先校验后无值
  门。内部稳定器 BFS 用 RootNbr 序，用户序管 dominance/扩展——两者分离。
- 本包只覆盖本文件；其在派发中的两臂见
  [派发包](atlas-core-domain-dispatch.md)；Weyl 身份机器见
  [领域值包](atlas-core-domain-values.md)。
- 字节数/哈希只标识本快照字节（git base `964f0033`，本文件 sha256
  `86d52b4f…`——即已验收 after-v5 清单中的同一字节）。
