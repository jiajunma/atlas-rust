---
title: mod-2 投影与纤维商上的诱导映射
summary: mod-2 投影利用根系数的奇性翻转坐标位，FiberToAdjoint::apply 依次取规范代表、执行投影并构造目标纤维元素，每次应用现算而不缓存稠密矩阵。
sources:
  - adjoint-fiber.md
kind: concept
createdAt: "2026-10-09T14:23:41.841Z"
updatedAt: "2026-10-09T14:23:41.841Z"
tags:
  - 模二运算
  - 商空间
  - 诱导映射
aliases:
  - mod-2-投影与纤维商上的诱导映射
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# mod-2 投影与纤维商上的诱导映射

mod-2 投影将源余特征格中的奇偶坐标映到伴随半单商，并经下降验证得到 Cartan 纤维之间的诱导映射。`AdjointCartanFiber::build` 在构造目标纤维后，调用 `source.validate_induced_map(&fiber, &projection)` 验证该投影可下降到纤维商；投影通过 `ModTwoAmbientMap` 接口参与验证。^[adjoint-fiber.md:25-37]

## 整数投影与模二实现

整数投影 `map_coweight` 实现限制映射 \(Y \to P^\vee\)：对源余特征 \(y\)，按源单根顺序计算配对 \(\langle\alpha_i,y\rangle\)，以这些整数作为目标坐标。伴随 datum 的 character 基为源单根全基，cocharacter 基为对应的基本余权基。这一投影可以具有中心核，不要求为同构；相关构造见 [[伴随根数据与余特征格投影]]。^[adjoint-fiber.md:17-19, adjoint-fiber.md:39-44]

私有方法 `apply_mod_two` 按根系数的奇性逐坐标翻转奇偶位，可经 `ModTwoAmbientMap` 或 `FiberToAdjoint` 调用。在其实现中，`bit(i) == None` 的分支按非 1 处理。^[adjoint-fiber.md:45-46]

## 从 ambient 映射下降到纤维商

构建链先检查 datum 一致性与源纤维对合一致性，再检查预算、构造投影和伴随纤维，最后执行诱导映射验证。伴随余权作用由根作用矩阵的转置给出：通常的对偶作用为逆转置，而已验证的根作用是对合，因此逆转置等于转置。该流程把对合作用构造与 [[Ambient 映射的子商下降验证]] 连接起来。^[adjoint-fiber.md:25-35]

`FiberToAdjoint::apply` 将源纤维元素依次经过 `canonical_representative`、mod-2 投影和目标纤维的 `element_from_ambient`，得到目标纤维元素。其计算路径可与 [[F₂ 商空间的确定性代表元与陪集判定]] 对照理解。^[adjoint-fiber.md:47-48]

`AdjointCartanFiber` 保存源纤维克隆、投影和目标纤维。每次调用 `fiber_map()` 都克隆这三者以构造 `FiberToAdjoint`；实现不缓存稠密 mod-2 矩阵，而是在每次 `apply` 时计算投影，详见 [[FiberToAdjoint 的按需投影]]。^[adjoint-fiber.md:35-37]

## 出处约束与资源边界

`AmbientCoweight` 与 `AdjointCoweight` 均携带 `Arc<AdjointProjectionModel>`。其相等判断同时要求模型指针相同与坐标相等；整数投影也先检查来源。这种 [[余权坐标的投影出处绑定]] 防止跨投影复用坐标，两次独立构建的投影会以 `DatumMismatch` 拒绝彼此的坐标。^[adjoint-fiber.md:19-23, adjoint-fiber.md:41-42]

投影工作量在构建期按 \(2\,lr^2\,ss\) 预检，在运行期按 `lr·target·vector_count` 检查，其中 `lr` 为格秩、`ss` 为半单秩。运行期预算逐次调用独立检查，不跨调用累计；预算乘加使用受检算术，溢出返回 `ArithmeticOverflow`。^[adjoint-fiber.md:52-58]

## 测试证据与限制

测试锚点包括 A1 恒等对合下的生成元投影与规范代表元、中心余权入核及投影可加性，以及 A1+A2 非对称作用下逐坐标基的交织关系
\(\operatorname{map}(\theta y)=\theta_{\mathrm{adjoint}}\operatorname{map}(y)\)。
另有跨投影坐标拒绝、rank-33 动态秩和两条预算拒绝路径的测试。^[adjoint-fiber.md:64-72]

这些证据仍有明确边界：若干分配、算术及非法作用错误分支未覆盖，`coordinates`、`same_class`、`basis_representatives` 和 `identity` 的直接调用也未覆盖。源材料属于结构性阅读，不声称数学验收；本次知识维护未执行测试或 benchmark。^[adjoint-fiber.md:10-13, adjoint-fiber.md:72-75, adjoint-fiber.md:81-85]

## Sources

- [adjoint-fiber.md](../../sources/adjoint-fiber.md) — 伴随 Cartan 纤维：构建、投影与 mod-2 商（adjoint_fiber.rs）。
