---
title: 整数列阶梯归约与带符号 gcd 扫描
summary: build 自底向上扫描并跟踪列操作；gcd_sweep 选最小绝对值主元、记录负主元取正的符号操作，并使用 div_euclid 保持典范像基定向。
sources:
  - real-projection.md
kind: concept
createdAt: "2026-10-09T15:06:30.859Z"
updatedAt: "2026-10-09T15:06:30.859Z"
tags:
  - 整数矩阵
  - 阶梯归约
  - 符号纪律
aliases:
  - 整数列阶梯归约与带符号-gcd-扫描
  - 整G扫
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 整数列阶梯归约与带符号 gcd 扫描

整数列阶梯归约用于为每个对合 $\theta$ 的 $(1-\theta)X^*$ 构造图像基对。`lift_mat` 是 $1-\theta$ 像的列阶梯基，维数为 $n\times r$；`m_real` 的维数为 $r\times n$，二者满足
$\mathrm{lift\_mat}\,\mathrm{m\_real}=1-\theta$。相关概念见 [[对合的 (1−θ)X* 图像基对]]。^[real-projection.md:19-25]

## 列阶梯归约流程

`build(theta)` 将 `column_echelon` 应用于 $1-\theta$，并增量跟踪列操作矩阵及其逆矩阵。归约自底向上扫描各行，每行执行 `gcd_sweep(row, limit)` 后，将主元置于第 `limit-1` 列。擦除零列时，核列逐列轮转到右端，已经停放的列不再移动；上游随后取列操作矩阵逆矩阵的前 $r$ 行作为 `M_real`。^[real-projection.md:36-39]

这种归约不仅要求得到一个有效图像基，还需要复现上游的具体列操作。图像基并非由 $\theta$ 唯一决定，而选出的 `lambda-rho` 代表元与 `y_lift` 的符号依赖精确的图像基，因此播种端逐操作移植了上游 `matreduc::column_echelon` 及其 gcd 扫描。^[real-projection.md:29-34]

## 带符号 gcd 扫描

`gcd_sweep` 复制局部行后，选择绝对值最小的主元。若主元为负，则将其取正，同时在操作记录矩阵中设置 `ops[mindex][mindex] = −1`。源码注释指出，E6 的 involution-187 分解必须记录这一符号才能成立。^[real-projection.md:41-43]

消元使用 `div_euclid`，以避免负余数导致选出负主元并反转典范图像基的定向。记录的 `ops` 通过 `apply_column_ops` 同时作用于工作矩阵 `a` 和列操作矩阵 `col`；该操作对应上游 `column_apply` 对前 `limit` 列的作用。^[real-projection.md:43-45]

## 幺模逆与分解校验

`invert_integer_matrix` 使用欧几里得行消元计算幺模逆，每次选择当前列中绝对值最小的主元。对角元必须为 $\pm1$，遇到 $-1$ 时将整行取负，最后逐项验证 $M M^{-1}=I$。`build` 收尾时还通过 `check_against` 检查图像基分解，失败时报告 `RepInvariantViolation { invariant: "image basis factorization" }`。相关概念见 [[幺模矩阵求逆与分解自校验]]。^[real-projection.md:46-49]

## 播种与传送的区别

列阶梯归约在 Cartan 轨道的 canonical involution 处用于播种，后续沿 cross-action BFS 传送基对。对单反射 $s$，传送采用 $L'=sL$、$M'=Ms$，保持
$(sL)(Ms)=s(1-\theta)s=1-\theta'$。传送所得基与对 $1-\theta'$ 重新归约所得基可能存在列符号或列次序差异，因此基对保存在记录中随轨道传送。相关概念见 [[图像基的典范播种与轨道传送纪律]]、[[单反射下的图像基传送]]。^[real-projection.md:29-34, real-projection.md:53-58]

## 测试锚点与证据边界

带符号 gcd 扫描具有针对 original3840186 的逐字锚点：输入 `[[8,-12],[4,-6]]`，预期 pivot 为 `2`，image 为 `[[0,4],[0,2]]`，columns 为 `[[-3,2],[-2,1]]`。其他锚点覆盖斜环面与斜乘积的像基字面量，以及恒等对合的零秩像和 $-I$ 的满秩像；后者满足 `lift_mat = 2I`。`transported` 没有测试。^[real-projection.md:73-77]

算术检查存在边界：`invert_integer_matrix` 的消元使用非受检普通算术，与文件其余部分的 checked 风格不一致，分配检查也不统一。源材料仅记录结构性阅读，没有执行构建、测试或原版运行；其中上游行号转述自源码注释，未独立重读上游，因此这些说明不构成新的数学验收或性能结论。^[real-projection.md:69-71, real-projection.md:87-92]

## Sources

- [real-projection.md](real-projection.md)：per-involution (1-θ)X* 图像基对：播种、传送与坐标。
