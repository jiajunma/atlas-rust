---
title: 对偶根映射与 Cartan 矩阵转置
summary: 对偶根通过 primal 余根向量映回 primal RootId，而实根及实紧根子系统使用转置 Cartan 矩阵确定类型，由此体现 B/C 类型互换。
sources:
  - real-weyl.md
kind: concept
createdAt: "2026-10-09T15:07:38.595Z"
updatedAt: "2026-10-09T15:07:38.595Z"
tags:
  - 根系
  - 对偶性
  - Cartan矩阵
aliases:
  - 对偶根映射与-cartan-矩阵转置
  - 对C矩
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 对偶根映射与 Cartan 矩阵转置

在实 Weyl 群构造中，对偶侧根列表需要映回原侧（primal）的 `RootId`，而对偶子系统的类型需要通过转置 Cartan 矩阵计算。这两项处理分别决定根的存储标识与子系统的类型判定。^[real-weyl.md:58-70]

## 对偶根映回原侧

`RealWeyl` 的根列表统一保存 primal `RootId`。对偶侧产生的 `real_compact` 与 `real_orth` 利用余根向量映回 primal：对应依据是，对偶根向量就是 primal 的余根向量。^[real-weyl.md:58-62]

构造时，两侧分别执行 `fiber_side`，随后通过 `primal_roots_of_dual` 转换对偶侧根。若无法找到对应根，构造返回带有 `"dual root correspondence"` 标识的不变量错误。^[real-weyl.md:94-97]

根列表的顺序也属于表示约定：`imaginary` 与 `real` 按上游 `RootNbr` 键排序，即先比较高度，再比较简单根坐标的反向字典序；`complex` 则保留 `makeSimpleComplex` 的输出顺序。相关编号约定可参见 [[RootNumbering 根编号与 RootNbr 顺序]]。^[real-weyl.md:58-62]

## Cartan 矩阵转置与类型判定

虽然对偶侧根映回了 primal 标识，`real_type` 与 `real_compact_type` 仍需按对偶子系统计算。实现为这两个类型调用 `subsystem_cartan(..., transposed=true)`，体现对偶子系统 Cartan 矩阵是 primal 子系统 Cartan 矩阵的转置；其余三个类型使用 `transposed=false`。^[real-weyl.md:66-70]

B/C 类型互换正是在这一转置步骤进入结果。例如，在 C2 datum 上，`Sp(4,R)` 的 Cartan #3 打印 `W^R is a Weyl group of type B2`，测试锚点为 `(2,3)`。^[real-weyl.md:66-70]

## 对偶侧构造上下文

上述映射依赖临时重建的对偶 fiber 链。实现不能直接复用对偶分类中存储的 fiber，因为所需的对偶 Cartan 对合 `−θ` 一般只是典范对偶 Cartan 代表元的共轭。因此，每次调用都会从 `dual_twisted_representative` 开始重建对偶 fiber、grading、弱实形式分区及标签，且不使用缓存；各阶段预算来自 `RealWeylContext.budget`。详见 [[对偶 Cartan fiber 链的临时重建]]。^[real-weyl.md:104-116]

对偶来源还体现在 R-群数据的归属上：`real_r` 保存对偶侧的 R-群向量，`imaginary_r` 保存 primal 侧向量。理解这些字段时，需要同时区分根标识所在的一侧与数据计算的来源侧。^[real-weyl.md:72-73]

## 测试证据与边界

来源记录的测试注释声明，输出来自固定上游构建 `rev 4d3e9449`，于 2026-08-11 重新生成并逐字节复制到断言；其中包含 `Sp(4,R)` 的 B/C 互换锚点。这是 [[实 Weyl 层的 oracle 测试与证据边界]] 中的具体覆盖案例。^[real-weyl.md:159-168]

该来源属于结构性源码阅读，不构成数学或正确性验收；上游文件与行号来自代码注释，未核对上游字节。预算耗尽路径及非 quasisplit 对偶形式的行为也没有测试锚点，因此不能将上述案例扩展为这些路径的验证结论。^[real-weyl.md:177-188]

## Sources

- [real-weyl.md](real-weyl.md) — 实 Weyl 群与块稳定子：real_weyl.rs 的构造、对偶 fiber 重放与打印层。
