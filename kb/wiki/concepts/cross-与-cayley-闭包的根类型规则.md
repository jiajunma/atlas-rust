---
title: cross 与 Cayley 闭包的根类型规则
summary: 闭包按复根、虚根与实根处理 cross 并检查包及 Cartan 一致性；Cayley 仅用于非紧虚根，克隆环面部分且按写入顺序保存逆像。
sources:
  - global-kgb.md
kind: concept
createdAt: "2026-10-09T14:49:36.658Z"
updatedAt: "2026-10-09T20:53:32.403Z"
tags:
  - KGB
  - 交叉作用
  - Cayley变换
aliases:
  - cross-与-cayley-闭包的根类型规则
  - C与C闭
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: cross 与 Cayley 闭包的根类型规则
summary: GlobalKgb 按根类型构造 cross/Cayley 闭包，检查长度奇偶、包边界与 Cartan 类一致性；Cayley 仅用于非紧虚根，保留环面部分并按写入顺序记录逆像。
sources:
  - global-kgb.md
kind: concept
tags:
  - cross作用
  - Cayley变换
  - 根类型
  - 算法不变量
---

# cross 与 Cayley 闭包的根类型规则

`GlobalKgb::build` 在基本纤维播种后，以包区间广度优先搜索（BFS）构造 cross/Cayley 闭包。该阶段根据长度变化、包编号与下降判定处理复根、虚根和实根，更新环面部分、根类型状态及链接，属于 [[GlobalKgb 的分阶段广度优先构造]]。^[global-kgb.md:63-74]

## 根类型与 cross 规则

闭包阶段要求长度差 `Δ` 为偶数，否则报告 `"cross length parity"`。当 `d = Δ/2 ≠ 0` 时，执行 `simple_reflect`，并将状态设为 `Complex`。^[global-kgb.md:67-69]

`simple_reflect` 在对偶侧预根 datum 上执行 `numer −= ⟨numer,α⟩·α∨`，故意不在反射后约化分子。因此，环面表示保留算术历史，打印结果可以出现负分子；SC B2 的 `[0,-1]/2` 是这一行为的测试锚点。参见 [[全局环面元素的算术历史表示]]。^[global-kgb.md:19-25]

虚根分支由 `new_number == index && !has_descent` 识别，使用 `imaginary_cross_act`，并通过 `negative_at` 判定紧性。`negative_at` 要求根配对为整数，否则报告 `KgbInvariantViolation "integral root evaluation"`；配对为奇数时判为紧。非紧时，`imaginary_cross_act` 通过 `coroot·(denominator − remainder)` 对应的 `exp_2pi` 加数平移环面部分。^[global-kgb.md:26-29, global-kgb.md:69-70]

实根分支要求 cross 像等于元素自身，否则报告 `"real cross image"`。^[global-kgb.md:70-70]

## Cayley 链接与逆像顺序

Cayley 链接仅为 `ImaginaryNoncompact` 元素建立，环面部分原样克隆。`inverse_cayley` 槽按写入顺序填充：首个写入者占据 `.0`，第二个写入者占据 `.1`。相关链接结构见 [[Cross、Cayley 与逆 Cayley 链接]]。^[global-kgb.md:72-74]

## 包边界与一致性约束

新指纹只允许落入当前正在开启的新包，否则报告 `"cross image inside closed packet"`；cross 两端的 Cartan 类号必须一致，否则报告 `"cross Cartan class"`。相关去重机制见 [[基于饱和像适应基的 KGB 去重指纹]]。^[global-kgb.md:33-36, global-kgb.md:71-72]

构造收尾时扫描每个 `(element, generator)`，确保状态已经写入，否则报告 `"element status"`。源材料据写入顺序进一步推断：由于每次迭代先写 cross 槽、后写状态，通过该扫描后，cross 槽不再残留构造期哨兵 `usize::MAX`。这是来源明确标注的推断。^[global-kgb.md:78-80]

## 测试与证据边界

现有四个测试包括 SC A1、adjoint A1 和 SC B2 的逐字节打印匹配，以及 B2 结构不变量检查。后者覆盖 17 个元素、包大小 `[8,2,2,2,2,1]`、包字、cross 对合性和 Cayley 配对；没有错误分支测试。参见 [[GlobalKgb 的回归测试与证据边界]]。^[global-kgb.md:94-101]

半单秩 0 的情形有意未测，因为共享内类机制在空生成元集合上会 panic。维度匹配、非零分母等隐式前置条件违反时也可能 panic；部分 `2 * denominator` 运算采用普通乘法，存在 debug 溢出 panic 与 release 回绕的边界。^[global-kgb.md:101-105]

源包属于结构性源码阅读记录，上游位置仅转录自代码注释，未核对上游字节，不声称数学验收。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[global-kgb.md:9-15, global-kgb.md:109-113]

## Sources

- [global-kgb.md](../../sources/global-kgb.md)：内类范围 KGB 图与 print_X 布局（global_kgb.rs）。
