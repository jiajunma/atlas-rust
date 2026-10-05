---
title: Weyl 群矩阵作用与根对合分类数据（weyl.rs / root_involution.rs）
source: atlas-rust/weyl-root-involution
ingestedAt: 2026-10-06T06:20:00Z
---

# Weyl 群矩阵作用与根对合分类数据（weyl.rs / root_involution.rs）

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `weyl.rs`（478 行）与 `root_involution.rs`（354 行）。两文件互不
import，经共享 crate 词汇协作。本包是结构性阅读，不声称数学验收；上游
引用仅转录自代码注释。

## weyl.rs：带溯源的矩阵级 Weyl 作用

`WeylAction { datum: Arc<BasedRootDatum>, weight_matrix, coweight_matrix }`：
每个作用自带 datum（provenance），同时作用于两个格。**等值 = derive 逐字段
比较（含 datum 值）**——与文档「相等即矩阵作用相等」存在张力（矩阵相同而
datum 值不同的两作用按 derive 不相等）；因 Arc 的 PartialEq 委派给内层值，
同 datum 值时两表述一致。词级组合层在 `weyl_element.rs`，两层经
`RootSystem::action_permutation` 互检。

- 构造：`identity` / `simple_reflection`（根用 `.get` 越界报
  `IndexOutOfRange{upper_bound: semisimple_rank}`，余根直接下标——长度不一
  致时会 panic 的潜在面）/ `root_reflection`（任意枚举根，用于重放
  Cayley/cross 分解的强正交根反射；根的符号无关）。`reflection_matrix`：
  `M[i][j] = δ_ij − reflected[i]·pairing[j]`，i128 checked + i32 收窄。
- 复合：`compose` 先 `DatumMismatch` 后 `RankMismatch`；`compose_matrices`
  用 i64 累加后 **`sum as i32` 无检查截断**（注释论证 Weyl 矩阵条目
  Cartan-bounded；该论证未形式化——复核备注）。`compose_fast`
  （pub(crate)，枚举热循环用）完全无检查，前置条件违约即 panic 面。
- `apply_matrix`（act/act_on_coweight）：ragged 矩阵行复用
  `InvalidRootAutomorphism` 变体（变体名与场景字面语义有差距）。
- `WeylGroup`：惰性操作集合，不枚举全群、不编码全局阶上界。
  `enumerate_actions(budget)`：走 `weyl_transducer::CompactWeyl` 紧致表示
  枚举（注释：E6 约 50ms vs 矩阵 BFS 约 1.1s——转述，非本包测量），再
  rayon `par_iter` 逐元素 `compose_fast` 物化矩阵；文档声称结果按特征格
  作用矩阵字典序（来自 CompactWeyl 输出序 + rayon 保序 collect，无测试
  断言——复核备注）。
- **死代码观察**：私有 `insert_action`（VecDeque BFS 去重助手）在文件内无
  任何调用点，疑为旧矩阵 BFS 路径遗留。

测试 7 个：全格作用（格秩 2 半单秩 1 的 A1）；A2 编织关系值相等；非对称
Cartan 下双作用保配对；空半单部分的 IndexOutOfRange；A2 预算 6 成功/5 报
`ResourceLimitExceeded`；同秩异 datum 拒；i32::MAX 根坐标报
`ArithmeticOverflow`。

## root_involution.rs：根系上对合的分类数据

`RootKind { Imaginary, Real, Complex }`；`RootInvolutionData` 存对合、
逐根像表、逐根类别表、虚/实子系单根。

`new` 的校验顺序：① `DatumMismatch`；② `RankMismatch`；③
`validate_simple_root_images`——逐单根：像不是根 →
`SimpleRootImageNotRoot{simple_root}`；输送余根 ≠ 像根余根 →
`SimpleCorootImageMismatch{simple_root, image_root}`（文档强调：仅配对保持
不足以排除「固定所有根却移动余根中心环面坐标」的作用）；④ 全根循环：像
不在根系 → `InvalidRootAutomorphism`；余根输送不等 →
`InvalidRootDatumAutomorphism`；分类先判 `像==根`（Imaginary）再判
`像==−根`（Real），其余 Complex；⑤ `subsystem_simple_roots` 两次（虚/实）：
继承正系中筛选（类别匹配 + 单根坐标全非负），候选减去集合中任一成员仍
在集合则可分解，否则为子系单根；结果顺序继承 RootSystem 枚举顺序（有测试
锚定）。

测试 7 个：A2 反对对合（Real 2/Complex 4/Imaginary 0，实子系单根 [1,1]）；
不置换根的配对保持作用被拒（SimpleRootImageNotRoot）；正/负像的余根失配
两例（精确到 image_root 值）；异 datum 拒；恒等全 Imaginary；虚子系单根
顺序锚定。未测：`InvalidRootDatumAutomorphism` 全根层级（现有失配测试在
单根阶段即返回）、访问器越界 None、非平凡对合的虚子系。

## 接口关系与限制

两文件入口守卫同形（DatumMismatch + RankMismatch + 同样的
`IndexOutOfRange{upper_bound: roots().len()}` 构造）。概念衔接：
`WeylAction::root_reflection` 的任意根反射服务 Cayley/cross 重放；
`RootInvolutionData` 被 `TwistedInvolution::new` 消费（trio 包）。
复核备注汇总：derive 相等性含 datum 字段的张力；insert_action 死代码；
`as i32` 截断的安全性论证未形式化；`actual` 恒报 weight_matrix.len()；
枚举字典序无测试锚定。

## 来源与限制

精确读取身份见
[`2026-10-06-weyl-root-involution.json`](snapshots/2026-10-06-weyl-root-involution.json)：
绑定 Git base、两文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe
（无工具档案）以两文件完整字节起草（600s 期限，exit 0，327.7s），维护者
对照源码逐条核对改写。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。
