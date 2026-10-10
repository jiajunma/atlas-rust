---
title: specialGrading 的 Bourbaki 序拉回与因子切片
summary: form_type_name 以 pulled[k] = grading[perm[k]] 拉回 Bourbaki 序并逐因子消费位集，复型条目消费两个因子，环面因子不消费 grading 位。
sources:
  - topology-form-name.md
kind: concept
createdAt: "2026-10-09T15:14:08.068Z"
updatedAt: "2026-10-10T00:53:51.972Z"
tags:
  - 实形式
  - 分级
  - Bourbaki
aliases:
  - specialgrading-的-bourbaki-序拉回与因子切片
  - S的B序
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: specialGrading 的 Bourbaki 序拉回与因子切片
summary: form_type_name 将 datum 单根上的 grading 拉回 Bourbaki 序并逐因子处理切片；复型条目消费两个因子，环面因子不消费 grading 位。
sources:
  - topology-form-name.md
kind: concept
tags:
  - 实形命名
  - 分级
  - Bourbaki编号
aliases:
  - specialgrading-的-bourbaki-序拉回与因子切片
---

# specialGrading 的 Bourbaki 序拉回与因子切片

`form_name.rs` 使用 `specialGrading` 划分重载提供的 grading，为实形式生成李代数名称。输入是按 datum 单根编号排列的位集，`1` 表示非紧虚根；命名时先按 Bourbaki 序读取这些位，再逐因子处理。相关背景见 [[specialGrading 的分区代表与位集编码]] 与 [[实形式的李代数命名规则]]。^[topology-form-name.md:48-55]

## Bourbaki 序拉回

拉回规则为 `pulled[k] = grading[perm[k]]`：Bourbaki 序的输出位置 `k` 读取 datum 序输入位置 `perm[k]` 的位。置换在这里指定的是每个输出位置的输入来源，必须保留这一方向；参见 [[Bourbaki 顶点排序与置换语义]]。^[topology-form-name.md:51-55]

## 逐因子处理

`form_type_name(layout, grading)` 按布局中的内类字母逐项处理。普通因子经 `perm` 拉回对应切片后调用 `factor_name`；复型字母 `'C'` 消费两个同构因子与两段切片；环面因子不消费 grading 位。这一处理对应来源所述上游逐因子执行 `gr >>= rank` 的循环。^[topology-form-name.md:50-55, topology-form-name.md:71-73]

复型名称由 `complex_name` 生成，传入同构因子对中的第一个因子。例如 A 型生成 `sl(n+1,C)`，T 型生成 `gl(1,C)`。普通环面因子则按内类生成紧型 `u(1)` 或非紧型 `gl(1,R)`，可结合 [[复型因子对与环面的名称表示]] 阅读。^[topology-form-name.md:59-61, topology-form-name.md:69-73]

## 切片与名称分派

`factor_name(bits, letter, rank, ic)` 将最低置位的位置加一记为 `m`，全零切片取 `m = 0`，再结合类型、秩及内类分派名称。例如 A1 的全零 grading 对应 `su(2)`，非零 grading 对应 `sl(2,R)`。^[topology-form-name.md:62-69]

来源列出的七个测试锚点涵盖 A1 紧／分裂名称、A2 和 A3 的内类分派、复型因子对的 `sl(2,C)`、B2 的三种名称、D4 不等秩分支及环面名称。其中 B2 案例明确标注：datum 序中的 bit 0 对应前面的长根，并得到 `so(3,2)`，提供了输入编号与名称对应关系的具体检查点。^[topology-form-name.md:75-78]

## 位宽限制与证据边界

`form_type_name` 会拒绝大于等于 128 的 `perm` 位置，以符合 `u128` 位集宽度。下层 `factor_name` 使用 `bits: u32`，假定单因子切片不超过 32 位，但没有相应防护；这是来源记录的实现限制。^[topology-form-name.md:71-73, topology-form-name.md:85-87]

某些对偶实形的对偶侧命名使用折叠小写字母 `'f'`／`'g'`，本移植未覆盖。E/F/G 与 `so*` 分支没有测试锚点，因此上述测试覆盖不包含这些分支。^[topology-form-name.md:70-70, topology-form-name.md:85-87]

本页依据结构性源码阅读，不表示数学验收。来源中的上游位置转录自代码注释，未核对上游字节；该次知识维护未执行 Atlas、Cargo、测试或 benchmark。进一步的范围说明见 [[拓扑计算与实形命名的实现及验证边界]]。^[topology-form-name.md:9-14, topology-form-name.md:91-95]

## Sources

- [topology-form-name.md](../../sources/topology-form-name.md) — 对偶分量群平凡性与实形命名（topology.rs / form_name.rs）。
