---
title: Weyl 群的矩阵作用与词级元素双层结构
summary: WeylAction 承载带 datum 的全格双矩阵作用，WeylElement 承载环境根置换，两层通过 action_permutation 与 from_action 桥接。
sources:
  - weyl-layer.md
kind: concept
createdAt: "2026-10-09T15:17:23.076Z"
updatedAt: "2026-10-10T00:56:05.777Z"
tags:
  - Weyl群
  - 架构
aliases:
  - weyl-群的矩阵作用与词级元素双层结构
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Weyl 群的矩阵作用与词级元素双层结构
summary: WeylAction 携带根数据并表示两个全格上的矩阵作用；WeylElement 以环境根系的根置换支持词级组合运算，两层通过根置换转换衔接。
sources:
  - weyl-layer.md
kind: concept
tags:
  - Weyl群
  - 系统设计
aliases:
  - weyl-群的矩阵作用与词级元素双层结构
provenanceState: extracted
---

# Weyl 群的矩阵作用与词级元素双层结构

Weyl 群层分为矩阵级作用 `WeylAction` 与词级组合元素 `WeylElement`。前者同时表示特征格与余特征格两个全格上的作用，并携带根数据 datum；后者以环境根系（ambient `RootSystem`）的枚举根置换表示元素，提供长度、下降、乘法、逆、扭曲共轭与按需约化词。两层通过 `RootSystem::action_permutation` 互查，`WeylElement::from_action` 提供从矩阵作用到词级元素的转换。^[weyl-layer.md:17-26]

## 矩阵作用层：全格作用与等值

`WeylAction` 保存 `datum: Arc<BasedRootDatum>`、`weight_matrix` 和 `coweight_matrix`。单位作用使用 `lattice_rank` 阶单位矩阵；简单反射依据根与余根的对偶配对构造，矩阵条目为 \(M_{ij}=\delta_{ij}-\mathrm{reflected}_i\mathrm{pairing}_j\)。`root_reflection` 支持任意枚举根上的反射，且与根的符号无关，可用于重放 Cayley/cross 分解。详见 [[WeylAction 的对偶全格作用]]。^[weyl-layer.md:28-36]

`compose` 的顺序是先作用 `right`、再作用 `self`；`act` 与 `act_on_coweight` 分别作用于两个格。构造矩阵作用无需枚举 Weyl 群，完整枚举由独立的 `WeylGroup::enumerate_actions(budget)` 接口提供，并要求显式基数预算。^[weyl-layer.md:19-21, weyl-layer.md:37-41]

实现中的等值判断是派生的逐字段比较，**包含 datum 的值**；`Arc` 的 `PartialEq` 委派给内层值。因此，矩阵相同但 datum 值不同的两个作用并不相等。来源概述中的“相等即矩阵作用相等”应限定在 datum 值相同的条件下理解，参见 [[WeylAction 的等值与 datum 身份语义]]。^[weyl-layer.md:43-47]

枚举使用 `CompactWeyl` 的紧凑表示生成元素，再通过 Rayon 的 `par_iter` 与 `compose_fast` 物化矩阵。来源将输出描述为按特征格作用矩阵的字典序排列，并将顺序归于 `CompactWeyl` 输出序与保序收集，但明确记录该排序没有测试断言。详见 [[带基数预算的 Weyl 群作用枚举]]。^[weyl-layer.md:39-41, weyl-layer.md:52-55]

## 词级元素层：根置换与调用方契约

`WeylElement` 属于 KGB map 的 stage (a)，提供 \(O(1)\) 的长度与下降查询。每次操作能够进行的来源一致性检查仅是根数匹配；始终使用同一环境根系属于调用方契约，由 KGB stages 负责。根置换的反对称性依靠“构造器是唯一入口”保证，构造入口包括 `identity`、`simple_reflection` 与 `from_action`。^[weyl-layer.md:22-24, weyl-layer.md:64-70]

左右下降读取不同方向的置换。`has_left_descent` 读取逆置换，以 \(w^{-1}(\alpha_s)<0\) 判定 \(\ell(sw)<\ell(w)\)；`has_right_descent` 读取正向置换，以 \(w(\alpha_s)<0\) 判定 \(\ell(ws)<\ell(w)\)。详见 [[WeylElement 的置换表示与长度下降不变量]]。^[weyl-layer.md:71-74]

`multiply` 的复合方向与 `WeylAction::compose` 一致，并在同一趟计算中按 \((uv)^{-1}=v^{-1}u^{-1}\) 维护逆置换。乘积长度从 positivity slice 重新计算，不能直接相加操作数长度。`left_multiply_simple` 与 `right_multiply_simple` 报告长度变化，其中 \(-1\) 对应 `sigma_mult` 分支，\(+1\) 对应 `sigma_inv_mult`；`inverse` 直接返回逆。^[weyl-layer.md:75-79]

`twisted_conjugate` 计算 \(s_{\mathrm{gen}}\,w\,s_{\mathrm{twist}(\mathrm{gen})}\)，是 Tits 扭曲共轭的 Weyl 影子。`twist` 必须是生成元上的对合置换；它与 distinguished involution 的单根作用一致这一要求由调用方保证。stage (b) 使用的长度变化 \(d\in\{0,\pm2\}\) 由调用点对缓存长度作差获得，参见 [[Weyl 元素的扭曲共轭]]。^[weyl-layer.md:80-84]

## 约化词与规范排序

`reduced_word` 通过逐次剥离最小左下降生成约化词，采用从左到右复合的约定。`canonical_word` 则复刻上游 transducer 的规范约化词，在 `WeylInterface` 的**内部生成元顺序**下取字典序最小值，上游称为 “minimal for ShortLex”。能够成为约化表达式首字母的生成元恰为左下降，因此逐次剥离最小内部左下降即可确定规范词；每步长度必须恰减一，否则返回 `WeylElementInvariantViolation`。详见 [[Weyl 元素的规范词]]。^[weyl-layer.md:86-94]

`WeylInterface::new(cartan)` 保存上游构造器的内部生成元重编号：Dynkin 分量按分类顺序排列，各分量的 Bourbaki `position` 对 A/E/F/G 型直接使用，对 B/C/D 型反转。`outward()` 将内部编号映射到 datum 生成元编号。移植保留的可观察效果是规范词选择与 `ParabolicPieces` 的内部序 piece 索引。^[weyl-layer.md:95-99]

`ParabolicPieces::key` 返回按 internal-level 顺序排列的 piece 列表，对应唯一分解 \(w=w_1\cdots w_n\)，其中各 \(w_i\) 是相应右陪集的最小代表元。列表的字典序比较复刻上游 `WeylElt::operator<`，也用于 involution 排序的平局判定，并由 KGB 重编号使用。详见 [[ParabolicPieces 的抛物分解与排序键]]。^[weyl-layer.md:100-105]

## 算术与失败边界

反射矩阵构造采用经过检查的 `i128` 运算，再收窄至 `i32`。简单反射对根使用 `.get`，越界返回 `IndexOutOfRange { upper_bound: semisimple_rank }`；对余根则直接下标访问。来源将根与余根长度不一致时可能发生 panic 记录为阅读观察。^[weyl-layer.md:31-35]

矩阵复合使用 `i64` 累加，随后通过 `sum as i32` 无检查截断；源码注释以 Weyl 矩阵条目受 Cartan 界约束为理由，但该论证未形式化。枚举热循环使用的 crate 内部接口 `compose_fast` 完全无检查，违反前置条件存在 panic 风险；`apply_matrix` 对不规则矩阵行复用 `InvalidRootAutomorphism` 错误变体。^[weyl-layer.md:48-51]

## 测试与证据范围

来源记录了七个测试锚点：格秩 2、半单秩 1 的 A1 全格作用；A2 编织关系值相等；非对称 Cartan 下双作用保持配对；空半单部分的索引越界；A2 枚举预算 6 成功而 5 返回 `ResourceLimitExceeded`；同秩异 datum 返回 `DatumMismatch`；以及 `i32::MAX` 根坐标返回 `ArithmeticOverflow`。这些是结构性阅读记录的覆盖项，并非本次执行测试所得的结果。^[weyl-layer.md:59-62, weyl-layer.md:121-121]

本页依据 `weyl.rs` 与 `weyl_element.rs` 的结构性阅读；2026-10-06 对 `weyl.rs` 的重读记录其字节未变。上游行号转述自源码注释，未独立重读上游，可能随版本变化。来源未执行构建、测试或原版运行，不包含数学验收、性能或并行结论；Weyl 层正确性仍属于其独立的 [[HPC 验收证据链]]，包括 capacity gate 与 Weyl owner 语义线。^[weyl-layer.md:9-15, weyl-layer.md:109-121]

## Sources

- [weyl-layer.md](../../sources/weyl-layer.md) — Weyl 群层：矩阵作用与词级元素的双层结构。
