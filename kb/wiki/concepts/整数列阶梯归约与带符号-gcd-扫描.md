---
title: 整数列阶梯归约与带符号 gcd 扫描
summary: 自底向上扫描行，以最小绝对值主元和 div_euclid 消元，记录负主元取正的列操作符号，并将核列逐列轮转至右端。
sources:
  - real-projection.md
kind: concept
createdAt: "2026-10-09T15:06:30.859Z"
updatedAt: "2026-10-10T03:33:38.645Z"
tags:
  - 整数线性代数
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
---

# 整数列阶梯归约与带符号 gcd 扫描

整数列阶梯归约用于构造对合 $\theta$ 的 $(1-\theta)X^*$ 图像基对：`lift_mat` 是大小为 $n\times r$ 的像的列阶梯基，`m_real` 是大小为 $r\times n$ 的坐标矩阵，满足 $\mathrm{lift\_mat}\,\mathrm{m\_real}=1-\theta$。相关定义见 [[对合的 (1−θ)X* 图像基对]]。^[real-projection.md:21-25]

## 归约流程与基选择

图像基并非由 $\theta$ 唯一决定，而 `lambda-rho` 代表元与 `y_lift` 的符号依赖精确的图像基。因此，播种端逐操作复现上游 `matreduc::column_echelon` 及其 gcd 扫描，保留具体的基选择与符号约定。^[real-projection.md:29-34]

`build(theta)` 将列阶梯归约应用于 $1-\theta$，增量跟踪列操作矩阵及其逆。算法**自底向上**扫描各行，每行执行 `gcd_sweep(row, limit)` 后，主元落在下标为 `limit-1` 的列。擦除零列时，核列逐列向右端轮转，已经停放的列不再移动；上游随后取列操作矩阵之逆的前 $r$ 行作为 `M_real`。^[real-projection.md:36-39]

## 带符号 gcd 扫描

`gcd_sweep` 复制局部行后，选择绝对值最小的主元。若主元为负，则将其取正，同时在操作记录矩阵中设置 `ops[mindex][mindex] = −1`。源码注释指出，E6 的 involution-187 分解只有记录这一符号才成立。^[real-projection.md:41-43]

消元使用 `div_euclid`，避免负余数导致选出负主元并反转典范像基的定向。记录的 `ops` 经 `apply_column_ops` 同时作用于工作矩阵 `a` 和列操作矩阵 `col`；该操作对应上游 `column_apply` 对前 `limit` 列的作用。^[real-projection.md:43-45]

## 幺模逆与分解自校验

`invert_integer_matrix` 使用欧几里得行消元求幺模逆，主元选择当前列中绝对值最小者。对角元必须为 $\pm1$；遇到 $-1$ 时将整行取负，末尾逐项验证 $MM^{-1}=I$。`build` 收尾时通过 `check_against` 校验图像基分解，失败时报 `RepInvariantViolation { invariant: "image basis factorization" }`。^[real-projection.md:46-49]

## 播种与轨道传送

列阶梯归约在 Cartan 轨道的 canonical involution 处用于播种，随后沿 cross-action BFS 传送基对。这是 [[图像基的典范播种与轨道传送纪律]] 的核心约定。^[real-projection.md:29-34]

对单反射 $s$，传送采用 $L'=sL$、$M'=Ms$，保持分解不变式 $(sL)(Ms)=s(1-\theta)s=1-\theta'$。传送所得基与对 $1-\theta'$ 重新归约所得的基存在列符号或列次序差异，因此上游将基保存在记录中携带，而非重新计算。详见 [[单反射下的图像基传送]]。^[real-projection.md:53-58]

## 测试锚点

带符号 gcd 扫描具有针对 original3840186 的逐字锚点：输入矩阵 $\begin{pmatrix}8&-12\\4&-6\end{pmatrix}$，预期 pivot 为 $2$，image 为 $\begin{pmatrix}0&4\\0&2\end{pmatrix}$，columns 为 $\begin{pmatrix}-3&2\\-2&1\end{pmatrix}$。该锚点固定了归约结果及列操作记录的具体形式。^[real-projection.md:79-81]

其他锚点覆盖斜环面对合 $\theta=\begin{pmatrix}-7&12\\-4&7\end{pmatrix}$ 与斜乘积的像基字面量，包括 `lift_mat` 为 $\begin{pmatrix}4\\2\end{pmatrix}$、`m_real` 为 $\begin{pmatrix}2&-3\end{pmatrix}$；边界测试覆盖恒等对合的零秩像，以及 $-I$ 的满秩像，后者满足 `lift_mat = 2I`。来源记录 `transported` 没有测试，参见 [[图像基算法的测试锚点与证据范围]]。^[real-projection.md:81-83]

## 实现限制与证据边界

算术与分配检查存在不一致：`invert_integer_matrix` 的消元使用非受检普通算术，与文件其余部分的 checked 风格不同；分配同时使用 `try_reserve_exact` 和 `vec!`、`to_vec`、`collect`。相关限制见 [[图像基接口的维度与算术安全边界]]。^[real-projection.md:75-77]

来源属于结构性阅读，本说明不扩展该实现已有的 signed-projection、fundamental-lattice gate 等 HPC 证据链。^[real-projection.md:9-17]

来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。上游行号转述自源码注释，未独立重读上游，可能随版本演进漂移。^[real-projection.md:93-98]

## Sources

- [real-projection.md](../../sources/real-projection.md) — per-involution (1-θ)X* 图像基对：播种、传送与坐标。
