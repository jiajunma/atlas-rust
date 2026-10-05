---
title: 只读块拓扑与块修正子（block_access.rs / block_modifier.rs）
source: atlas-rust/block-access-modifier
ingestedAt: 2026-10-06T00:50:00Z
---

# 只读块拓扑与块修正子（block_access.rs / block_modifier.rs）

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草（在期限处截断），维护者对照源码逐条核对并补齐尾部**。本包覆盖
`crates/atlas-real-group/src/` 的 `block_access.rs`（277 行）与
`block_modifier.rs`（617 行）。它是结构性阅读，不声称这两层的数学验收；
上游引用（blocks.cpp、repr.h/cpp 行号）仅转录自代码注释，本包未核对上游
字节。

## block_access.rs：BlockTopology 只读边界

`BlockTopology` 是 KL 实现消费的最小只读块表面（上游
`klsupport::KLSupport` 只消费 `const Block_base&`）。两层无效性约定：外层
`None` = 无效的 element/generator 格子；`cayley`/`inverse_cayley` 的内层
`None` = 未定义的 Cayley 链。密封模式（`pub(crate) mod sealed` +
`BlockTopology: sealed::Sealed`）：允许 crate 内的不变量测试实现该 trait，
禁止下游 crate 实现——因为 KL 算法依赖超出方法签名的结构不变量
（rank ≤ 32、元素按非降长度排序、格子存在、链接目标 < size；KL 构造在
递归前校验它们）。`&T` 与 `Arc<T>` 的 blanket impl 均带 `?Sized`，故
`&dyn`/`Arc<dyn>` 满足约束。

`bruhat_hasse`（blocks.cpp:1576-1656）：第 z 行列出 z 的直接下邻。
流程：取首个 strict good descent（`ComplexDescent | RealTypeI`）：
`ComplexDescent` → 插入 cross 像并对该行 `insert_ascents`；`RealTypeI` →
inverse-Cayley 的第一（必需）与第二（可选）像，再对**第一像的行**
`insert_ascents`；无 strict good 时遍历所有生成元，凡 `RealTypeII` 者插入其
inverse-Cayley 的第一分量（第二分量不取）。`insert_ascents`：
`ComplexAscent`→cross、`ImaginaryTypeI`→cayley 第一分量、
`ImaginaryTypeII`→cayley 两个分量。本文件无 `Result`：多处 `expect`
（`"complex descent cross"` 等）与 `hasse[sz]` 直接索引依赖构造不变量。

两个具体实现者：`BlockGraph` 基本同名委托（`descent` → 固有
`descent_value`）；`PartialBlock` 的 `cross`/`cayley`/`inverse_cayley` 含
**参数交换**（`PartialBlock::cross(self, generator, element)`）与门控：
`cayley` 在 is_descent 时返回 `Some((None, None))`、否则委托；
`inverse_cayley` 恰好相反（非下降时返回 `Some((None, None))`）——「因下降
状态而无 Cayley 链」编码为内层 `(None, None)` 而非外层 `None`。

## block_modifier.rs：块修正子与 Weyl 姿态算术

「nonidentity generator attitude」切片第 2 步（locator 是第 1 步；两文件
均**尚未接线**）：上游 `repr::block_modifier`（repr.h:493-499、
repr.cpp:1401-1419）加 `Rep_context` 的 `transform`（repr.cpp:712-754）、
`shift`（352-356）、`make_diff_integral_orthogonal`（317-329）、
`make_relative_to`（338-350）与带 modifier 的 `sr`（815-823）。

`BlockModifier { locator: BlockLocator, shift: RationalWeight }`：
`trivial(system, simp_int)` 构造恒等 w、恒等 simple_pi、零 shift，
`int_sys` 取 `u32::MAX`（上游 `-1` 哨兵；仅供局部使用、永不读取）；
`from_locator` 无校验包装；`clear` 保留 `int_sys` 与 `simp_int`，重置 w/
simple_pi/shift。

一处有意偏差（文档声明对目标域无语义差异）：上游 `transform` 走
`Weyl_group().word(w)`（transducer 随元素存储的词），crate 走
`WeylElement::reduced_word`（典范最左下降约化词）；双向使用同一典范词使
`transform<false>` 成为 `transform<true>` 的逐字母逆——`make_relative_to`
与 `sr` 只依赖这一点。

`RepContext` 扩展方法：

- `transform_srm<LEFT_TO_RIGHT>(w, srm)`：逐字母按 `kgb_status(x, s)` 分派：
  Complex → cross x 且分子简单反射（offset 0）；Real → x 不变、分子取以
  `-ρ_R` 为中心的仿射反射（offset = 分母）；Imaginary* →
  `RepInvariantViolation`（上游抛 `Bad Weyl group element SRM transform`）。
  收尾在最终 x 处 `real_unique` 归一化。`LEFT_TO_RIGHT` 选施加方向。
- `shift_srm`：`gamma_lambda += amount` 后在不变的 involution 处归一化。
- `make_diff_integral_orthogonal`：两代表的差减去其在 `(1-θ)X*` 中的固定
  原像（经 `IntegralSubsystem::integral` + `RepTable::integral_codec` +
  `theta_1_preimage`），使结果与 `srm.gamma_lambda()` 的整根系正交；差为零
  时短路；debug 构建带正交性 `debug_assert`（上游 repr.cpp:326）。
- `make_relative_to(loc, srm0, bm, srm1)`：先 locator 部分的逆合成（见
  locator 包），再以**更新后**的 `bm.w` 做 `transform<true>` 把 srm1 移回
  基姿态，最后把 `bm.shift` 设为两个 gamma_lambda 的整正交差。
- `sr_with_modifier(srm, bm, gamma)`：先加 `bm.shift`，再
  `transform<false>(bm.w)`，再 `to_standard`——即上游 repr.cpp:819-822 的
  读取路径。

`simple_reflect_numerator`（私有）：`v -= alpha_s * (<v, coroot_s> + offset)`
作用于有理权分子（分母不变），全程 checked；对应 rootdata.h:610-611 与
617-618 的 offset 变体。

## 测试锚点

`block_access.rs` 无测试模块。`block_modifier.rs` 两个集成测试：
恒等 block_modifier 使 `sr_with_modifier` 与无 modifier 的 `sr` 一致
（A2 的 SL(3,R) fixture，KGB 尺寸 4）；`make_relative_to` 往返：SL(3,R)
锚点对的定位器在同一典范数据上碰撞、`bm.w` 的词为 `[1]`（oracle 的
`<1>`）、shift 为 `[0,1]/4`，shift+transform 往返逐字母落回查询参数
（精确相等而非仅差根平移），完整 `sr_with_modifier` 恢复 q 本身。两个测试
的注释携带完整手算推导（KGB 布局、墙交集、word 过滤）。

## 限制与未覆盖面

- 不做数学/正确性验收；整个 block_modifier 切片未接线（无现存消费方）。
- `BlockDescent` 的完整变体集与 `is_descent()` 定义在本包外（`block.rs`/
  块图包）。
- `bruhat_hasse` 的 `expect` 面与 `hasse[...]` 直接索引依赖构造不变量，无
  运行时防护。
- `transform_srm` 的分支覆盖经集成测试间接进行；`shift_srm`/
  `make_diff_integral_orthogonal` 的独立单元测试不存在。

## 来源与限制

精确读取身份见
[`2026-10-06-block-access-modifier.json`](snapshots/2026-10-06-block-access-modifier.json)：
绑定 Git base、两文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe
（无工具档案）以两文件完整字节起草，在 480s 期限处截断（运行约 486s 被
SIGTERM）；已覆盖部分由维护者对照源码逐条核对无误，尾部（测试锚点与限制
章节）由维护者按自己的完整阅读补齐。本次知识维护未执行 Atlas、Cargo、
测试或 benchmark。
