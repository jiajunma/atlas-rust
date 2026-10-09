---
title: 基于图结构的 Dynkin 单分量分类
summary: 分类器根据秩、顶点度数、分叉及有向多重边识别 A/B/C/D/E/F/G 型，并拒绝环、过高度数及不支持的多重边结构。
sources:
  - dynkin.md
kind: concept
createdAt: "2026-10-09T14:45:24.003Z"
updatedAt: "2026-10-09T20:50:50.927Z"
tags:
  - Dynkin分类
  - 根系
  - 图算法
aliases:
  - 基于图结构的-dynkin-单分量分类
  - 基D单
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 基于图结构的 Dynkin 单分量分类
summary: 根据连通分量的秩、顶点度数、分叉点与有向多重边判定 Dynkin 类型，并按各型规则构造 Bourbaki 顶点顺序；证据限于结构性源码阅读。
sources:
  - dynkin.md
kind: concept
tags:
  - Dynkin分类
  - 图算法
  - 根系
---

# 基于图结构的 Dynkin 单分量分类

`dynkin.rs` 的私有函数 `classify_component` 根据 Cartan 矩阵连通分量的秩、顶点度数、多重边与分叉结构，判定 A/B/C/D/E/F/G 类型，并构造 Bourbaki 顶点顺序 `position`。实现意图是保留上游分类与排序的平局选择；“精确复现上游”属于注释意图声明，不代表已完成数学或兼容性验收。^[dynkin.md:16-21, dynkin.md:25-34, dynkin.md:49-70, dynkin.md:100-102]

## 输入与图结构

单分量分类建立在 [[Cartan 矩阵输入契约与连通分量划分]] 之上。输入应满足已检查的 based-datum Cartan 不变量；分类入口检查方形、对角元为 2、非对角元属于 `-3..=0`，以及零模式对称，但不校验非零条目的数值配对。^[dynkin.md:36-41]

邻接集合 `star[v]` 保存顶点 `v` 的全部邻居；`down_edges` 收集矩阵条目小于 −1 的有序对 `(i, j, -entry)`，标号为 2 或 3。顶点按升序处理，通过首次匹配与后续合并形成连通分量，最终分量按各自最小顶点升序排列。^[dynkin.md:43-47]

## 秩一与秩二特判

秩一分量归为 A 型。秩二分量按 `support` 升序取顶点 `(i, j)`，依据乘积 `cartan[i][j] * cartan[j][i]` 判型：乘积为 1 时归为 A；为 2 时，若 `cartan[i][j] == -1` 则归为 C，否则归为 B，且保持给定顺序；为 3 时归为 G，并在 `cartan[i][j] != -1` 时交换位置，使短根在前。^[dynkin.md:51-56]

因此，B2/C2 的标签由有序 Cartan 条目决定，分类器不会为统一标签而重排这两个顶点。这一历史编号约定见 [[秩二 Dynkin 分类的顺序约定]]。^[dynkin.md:51-56]

## 高秩分量的类型判定

秩大于 2 时，算法将度数不超过 1 的顶点视为端点，将度数为 3 的顶点视为分叉点 `fork`。度数至少为 4 时报告错误；端点少于两个时报告“环”。多重边标号为 3 时报告 `oversized type G`，出现第二条多重边时报告 `multiple labelled edges`。^[dynkin.md:58-60]

存在多重边时，若同时存在分叉点则报错；否则，`lower` 属于端点集时判为 B，`upper` 属于端点集时判为 C，其余情况判为 F。不存在多重边时，无分叉点则判为 A；有分叉点时，若与分叉点相邻的端点恰有一个，则判为 E，否则判为 D。^[dynkin.md:60-62]

## Bourbaki 顶点顺序

判型后，算法按类型选择遍历起点。A 型取最小编号端点；B、C 型分别从端点集中移除 `lower`、`upper` 后取最小编号端点；D4 任取端点，更高秩 D 型移除分叉点邻居后取唯一剩余端点，即长臂末端；F 型移除 `lower` 的邻居后取最小编号端点。^[dynkin.md:64-69]

E 型要求端点集与分叉点邻居集的交集恰含一个顶点，以此确定短臂端点。随后选择正交长臂端点：若首次选中的端点离分叉点超过两步，则将其视为最长臂末端，交换为另一条长臂的端点。`position` 先写入正交长臂端点、短臂端点，再从公共邻居继续。^[dynkin.md:66-69]

后续遍历每步选择当前顶点最小编号的未访问邻居。D 型遍历中断时，只允许剩余分叉点短臂顶点，并按升序追加。各分量的 `position` 顺序拼接形成 [[Bourbaki 顶点排序与置换语义|Bourbaki 置换]]，其中 `result[i]` 是占据 Bourbaki 位置 `i` 的 datum 顶点。^[dynkin.md:69-76]

## 测试与证据边界

来源列出的六个测试全部覆盖成功路径，涉及 A1/A2、保持给定顺序的 B2/C2、两个方向的 G2，以及标准 D4、E6、F4。重编号测试涉及 B3、A3 与 D4，其中 B3 的 Bourbaki 置换用于将重编号矩阵重建为标准形；另有块对角 A1+A2 的分量顺序测试和空矩阵返回空置换的测试。^[dynkin.md:87-96]

错误路径、E7/E8 与秩大于 4 的 D 型没有测试锚点，详见 [[Dynkin 分类器的测试覆盖与证据边界]]。来源属于结构性阅读，未核对上游字节，本次知识维护也未执行 Atlas、Cargo、测试或 benchmark；这些材料不能据此证明数学正确性或完整上游兼容性。^[dynkin.md:9-12, dynkin.md:100-102, dynkin.md:111-115]

## Sources

- [dynkin.md](dynkin.md) — Dynkin 分类器：连通分量、Bourbaki 置换与折叠 Cartan（dynkin.rs）
