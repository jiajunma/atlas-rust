---
title: specialGrading 的 Bourbaki 序拉回与因子切片
summary: form_type_name 通过 pulled[k] = grading[perm[k]] 拉回 Bourbaki 序并逐因子消费位集，其中复型条目消费两个因子，环面因子不消费 grading 位。
sources:
  - topology-form-name.md
kind: concept
createdAt: "2026-10-09T15:14:08.068Z"
updatedAt: "2026-10-09T21:12:00.405Z"
tags:
  - 实形式
  - Bourbaki编号
  - 分级
aliases:
  - specialgrading-的-bourbaki-序拉回与因子切片
  - S的B序
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: specialGrading 的 Bourbaki 序拉回与因子切片
summary: form_type_name 通过 pulled[k] = grading[perm[k]] 将 datum 单根上的 grading 拉回 Bourbaki 序，再逐因子切片；复型条目消费两个因子，环面因子不消费 grading 位。
sources:
  - topology-form-name.md
kind: concept
tags:
  - 实形式
  - 根系
  - 位集
aliases:
  - specialgrading-的-bourbaki-序拉回与因子切片
---

# specialGrading 的 Bourbaki 序拉回与因子切片

`form_name.rs` 使用 `specialGrading` 划分重载提供的 grading，为实形式生成李代数名称。输入位集按 datum 单根编号排列，`1` 表示非紧虚根；命名前先拉回到 Bourbaki 序，再逐因子消费相应切片。相关概念见 [[specialGrading 的分区代表与位集编码]] 与 [[实形式的李代数命名规则]]。^[topology-form-name.md:48-55]

## Bourbaki 序拉回

拉回遵循 `pulled[k] = grading[perm[k]]`：Bourbaki 序的输出位置 `k` 读取 datum 序位置 `perm[k]` 的输入位。这里的置换用于确定取值来源，其方向可结合 [[Bourbaki 顶点排序与置换语义]] 理解。^[topology-form-name.md:51-55]

## 逐因子切片

`form_type_name(layout, grading)` 按内类字母逐项处理布局。普通因子经 `perm` 拉回切片后调用 `factor_name`；复型字母 `'C'` 一次消费两个同构因子与两段切片；环面因子不消费 grading 位。整体消费方式对应上游逐因子执行 `gr >>= rank` 的循环。^[topology-form-name.md:50-55, topology-form-name.md:71-73]

复型名称由 `complex_name` 生成，传入同构因子对中的第一个因子。例如 A 型得到 `sl(n+1,C)`，T 型得到 `gl(1,C)`。普通环面因子则按内类命名为紧型 `u(1)` 或非紧型 `gl(1,R)`，不需要消费单根 grading 位。^[topology-form-name.md:59-61, topology-form-name.md:69-73]

## 切片对命名的影响

`factor_name(bits, letter, rank, ic)` 将切片的最低置位位置加一记为 `m`，全零切片取 `m = 0`，再据此参与各类型的名称分派。例如 A1 的全零 grading 给出 `su(2)`，非零 grading 给出 `sl(2,R)`。^[topology-form-name.md:62-69]

来源记录的测试锚点包括 A1 紧／分裂名称、A2 与 A3 的内类分派、复型因子对的 `sl(2,C)`、B2 的三种名称、D4 不等秩分支以及环面名称。其中 B2 案例明确指出，datum 序的 bit 0 对应前面的长根，并得到 `so(3,2)`，体现了输入编号对名称分派的影响。^[topology-form-name.md:75-78]

## 位宽与证据边界

`form_type_name` 会拒绝大于等于 128 的 `perm` 位置，以符合 `u128` 位集宽度。下层 `factor_name` 使用 `bits: u32`，假定单因子切片不超过 32 位，但没有相应防护；这是来源记录的实现限制。^[topology-form-name.md:71-73, topology-form-name.md:85-87]

折叠小写字母 `'f'`／`'g'` 仅用于某些对偶实形的对偶侧命名，本移植未覆盖。E/F/G 与 `so*` 分支也没有测试锚点，不能由已有案例推定这些分支已获验证。^[topology-form-name.md:70-70, topology-form-name.md:85-87]

本页依据结构性源码阅读整理，不表示数学验收。来源中的上游位置转录自代码注释，未核对上游字节；本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。参见 [[拓扑计算与实形命名的实现及验证边界]]。^[topology-form-name.md:9-14, topology-form-name.md:91-95]

## Sources

- [topology-form-name.md](topology-form-name.md) — 对偶分量群平凡性与实形命名（topology.rs / form_name.rs）。
