---
title: 伴随 Cartan 纤维的构建与下降验证
summary: 构建依次检查 datum、源对合及预算，建立伴随纤维后通过 validate_induced_map 验证投影可下降到子商；该来源不构成数学验收。
sources:
  - adjoint-fiber.md
  - cartan-fibers.md
kind: concept
createdAt: "2026-10-09T14:23:47.105Z"
updatedAt: "2026-10-10T00:12:59.775Z"
tags:
  - Cartan纤维
  - 子商
  - 构造校验
aliases:
  - 伴随-cartan-纤维的构建与下降验证
  - 伴C纤
confidence: 1
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 伴随 Cartan 纤维的构建与下降验证
summary: AdjointCartanFiber 按顺序检查 datum、源对合与预算，构造伴随对合作用和有限 F₂ 纤维，再验证投影沿分子与分母关系下降；证据限于结构性源码阅读。
sources:
  - adjoint-fiber.md
  - cartan-fibers.md
kind: concept
tags:
  - Cartan纤维
  - 对合
  - 子商映射
aliases:
  - 伴随-cartan-纤维的构建与下降验证
  - 伴C纤
provenanceState: extracted
---

# 伴随 Cartan 纤维的构建与下降验证

`AdjointCartanFiber::build` 接收已验证的 root-datum 对合与已构建的 ambient `CartanFiber`，在伴随半单商上构造有限 $\mathbb F_2$ Cartan 纤维，并验证源投影能够下降到目标子商。构建成功的对象同时持有源纤维、投影与目标纤维。来源属于结构性源码阅读，不构成数学验收。^[adjoint-fiber.md:9-13, adjoint-fiber.md:25-37]

## 子商模型与伴随投影

`CartanFiber` 采用余特征格 $Y$ 上的子商
$\ker_{\mathbb F_2}(I+\theta_Y)/\operatorname{red}_2\ker_{\mathbb Z}(I+\theta_Y)$，并使用 low-pivot 归约基。文档注释声明它与 $Y^\theta/(I+\theta_Y)Y$ 同构，但自然坐标不同；实现采用前一种坐标约定。参见 [[Cartan fiber 的有限域子商模型]]。^[cartan-fibers.md:16-24]

`AdjointBasedRootDatum` 使用源 datum 的 Cartan 矩阵调用 `BasedRootDatum::standard`。按文档声明，其 character 基是源单根全基，cocharacter 基是对应的基本余权基。投影 $Y\to P^\vee$ 将 $y$ 映为它与各源单根的配对，目标坐标按单根顺序排列；该映射可以有中心核，不是同构。参见 [[伴随根数据与余特征格投影]]。^[cartan-fibers.md:101-112]

## 固定的构建顺序

构建首先检查 `root_system` 与 `root_involution` 的 datum 是否一致，不一致返回 `DatumMismatch`；随后检查源纤维的对合是否与输入对合一致，不一致返回 `CartanFiberInvolutionMismatch`。两项检查通过后，才执行 `validate_adjoint_build_budget`，且预算预检早于任何目标矩阵分配。^[cartan-fibers.md:114-128]

接着构造投影，并由 `root_basis_action` 将每个单根像的单根坐标写为矩阵的一列，得到半单秩方阵。`id_of`、`image` 或 `simple_coordinates` 任一步缺失，均返回 `InvalidRootAutomorphism`。余权作用取根作用矩阵的转置；注释给出的理由是对偶作用应为逆转置，而 `RootInvolutionData` 已验证根作用为对合，因此逆转置等于转置。^[cartan-fibers.md:129-133]

随后调用 `LatticeInvolution::new` 与 `CartanFiber::build_owned` 构造目标纤维。后者遵循“先分母后分子”：先计算整数负特征子格并模二归约，再计算 $\ker_{\mathbb F_2}(I+\theta_Y)$，最后构造 `ModTwoSubquotient`。分子计算使用余特征作用矩阵的行，不再转置；整数格预算在分母构造阶段执行。参见 [[Cartan fiber 的先分母后分子构造]]。^[cartan-fibers.md:72-80, cartan-fibers.md:134-136]

## 下降验证与出处约束

目标纤维构造完成后，最后调用 `source.validate_induced_map(&fiber, &projection)`。投影通过 `ModTwoAmbientMap` 接口参与验证；该钩子一次性检查 ambient 映射是否沿子商的分子、分母两类关系下降。失败返回 `CartanFiberMapDoesNotDescend`，其中 `relation` 分别为 `"numerator"` 或 `"denominator"`，两种失败均有测试锚点。^[cartan-fibers.md:90-92, cartan-fibers.md:110-112, cartan-fibers.md:134-136]

下降验证绑定到确切的源纤维模型。`CartanFiberElement` 的等值判定要求模型满足 `Arc::ptr_eq` 且坐标相等，独立构建的同坐标元素不因此相等；`ambient_fiber()` 所对应的精确源实例才是下降验证覆盖的来源。`AdjointCartanFiber` 不实现 `Eq`，因为值相等不足以表达这种出处约束。参见 [[Fiber 元素的模型来源绑定与规范代表]]。^[cartan-fibers.md:82-89, cartan-fibers.md:118-121, adjoint-fiber.md:76-77]

整数坐标另有投影出处绑定：`AmbientCoweight` 与 `AdjointCoweight` 持有 `Arc<AdjointProjectionModel>` 和裸 `Coweight`，等值同样要求模型指针与坐标一致。两次独立构建的投影会以 `DatumMismatch` 拒绝对方的绑定坐标，参见 [[余权坐标的投影出处绑定]]。^[adjoint-fiber.md:17-23]

## 验证后的按需应用

`fiber_map()` 克隆所持有的源纤维、目标纤维和投影，构造字段全私有的 `FiberToAdjoint`。其 `apply` 固定执行三步：取得源元素的 `canonical_representative`，应用 mod-2 投影，再调用目标纤维的 `element_from_ambient`。实现不保存稠密 mod-2 映射矩阵或缓存像，每次调用都按需投影。参见 [[FiberToAdjoint 的按需投影]]。^[adjoint-fiber.md:35-37, cartan-fibers.md:138-141]

源元素的典范代表由 low-pivot 坐标约定确定：商坐标第 $j$ 位选择 `basis_representatives()[j]`，所选代表的 XOR 即典范 ambient 代表。mod-2 投影按根系数的奇性逐坐标翻转位；其 `bit(i) == None` 分支按非 1 处理。^[cartan-fibers.md:85-89, adjoint-fiber.md:45-48]

## 资源预算与失败边界

设半单秩为 $r$、源格秩为 $n$。构建预检依次限制半单秩、持久存储估算 $16r^2+rn$，以及投影操作估算 $2n^2r$，超限分别返回带 `"semisimple rank"`、`"persistent entries"` 或 `"projection operations"` 的 `AdjointFiberResourceLimit`。投影工作量估算依据是每个源坐标至多贡献一个分子与一个分母基向量；系数 16 的逐项构成未文档化。参见 [[伴随 fiber 的分配前资源预算]]。^[cartan-fibers.md:122-128]

运行期 `check_projection_work` 按“源秩 × 目标秩 × 向量数”逐次检查，没有跨调用累计。预算乘加使用 `checked_product` 和 `checked_sum`，溢出返回 `ArithmeticOverflow`；分配使用 `try_reserve_exact`，失败返回 `AllocationFailed`。其他错误包括 `RankMismatch` 与转置输入非方阵时的 `InvalidInvolution`；非测试代码无 panic 或 assert。^[adjoint-fiber.md:52-62]

## 测试锚点与证据范围

伴随纤维的九个测试覆盖 A1 恒等投影及两条映射路径的一致性、中心余方向入核与可加性、A2 twisted 作用矩阵、源纤维对合不匹配和跨投影坐标拒绝。此外，A1+A2 非对称例逐坐标基验证 `map(θ·y) == θ_adjoint·map(y)`，rank-33 例覆盖超过历史打包上限的动态秩，两条预算测试锚定持久存储与投影操作的提前拒绝。^[adjoint-fiber.md:64-72, cartan-fibers.md:143-149]

未覆盖的错误分支包括 `AllocationFailed`、`ArithmeticOverflow`、`InvalidRootAutomorphism`、`InvalidInvolution` 和构建入口的 `DatumMismatch`；`coordinates`、`same_class`、`basis_representatives` 与 `identity` 的直接调用也未覆盖。来源另指出，投影构造未显式校验单根数量与目标秩的一致性，而是依赖构造不变量。参见 [[伴随纤维映射的测试证据与覆盖边界]]。^[adjoint-fiber.md:72-75, cartan-fibers.md:159-163]

两份来源均经维护者对照源码核对，并以快照记录 Git base、文件字节 SHA-256 与草案调用记录；本次知识维护未执行 Atlas、Cargo、测试或 benchmark。子商同构与逆转置论证属于代码注释声明，测试锚点也不代表本次执行或数学验收结果。^[adjoint-fiber.md:81-85, cartan-fibers.md:151-158, cartan-fibers.md:165-171]

## Sources

- [adjoint-fiber.md](../../sources/adjoint-fiber.md)：伴随 Cartan 纤维：构建、投影与 mod-2 商（adjoint_fiber.rs）。
- [cartan-fibers.md](../../sources/cartan-fibers.md)：Cartan fiber 与伴随 Cartan fiber（cartan_fiber.rs / adjoint_fiber.rs）。
