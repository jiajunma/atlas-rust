---
title: Twisted involution 表与 Cartan 轨道存储
summary: KGB stage b 按 Cartan 添加顺序连续存储 twisted involution 轨道，轨道内采用 external-order BFS，并为记录分配全局连续的 InvolutionId。
sources:
  - involution-table.md
kind: concept
createdAt: "2026-10-09T14:52:18.743Z"
updatedAt: "2026-10-09T14:52:18.743Z"
tags:
  - KGB
  - 数据结构
  - Cartan分类
aliases:
  - twisted-involution-表与-cartan-轨道存储
  - TI表C轨
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Twisted involution 表与 Cartan 轨道存储

Twisted involution 表是 KGB 构建流水线的 stage b，按 Cartan 类连续存储 twisted involution 轨道。每条记录同时保存词级 Weyl 元素、矩阵级 `TwistedInvolution`、模二去重子空间、$(1+\theta)\rho$、对合长度与 Weyl 长度，以及 $(1-\theta)X^*$ 的图像基对 `lift_mat`、`M_real`。^[involution-table.md:20-24]

## 编号与连续存储

表的全局编号由调用方添加 Cartan 类的顺序决定；文档规定按升序 `CartanId` 添加。每条轨道内部按外部生成元顺序进行 BFS，`InvolutionId` 跨 Cartan 类连续递增。`orbit_slice(cartan)` 返回该类的连续轨道切片及起始编号，因此 Cartan 类的分组与全局元素编号共同构成表的存储布局。^[involution-table.md:33-35]

## 初始化与轨道添加

`InvolutionTable::new` 为一个 inner class 创建空表。该 inner class 已同时拥有 datum、根系和 distinguished involution，无需额外的跨输入门控；初始化还一次性导出经过验证的 simple twist、每个简单生成元对应的反射元素，以及由 positivity slice 得到的 $2\rho$。BFS 遍历边时复用这些反射元素。^[involution-table.md:39-42]

`add_cartan(classification, cartan)` 将一个 Cartan 类的轨道添加为连续切片，重复添加则返回已有切片。轨道种子和期望大小均来自 classification，实际生成的轨道必须恰好达到期望大小，否则报告 `"orbit size"` 不变量错误。种子通过 `WeylElement::from_action` 从矩阵级代表元转换一次，其对合长度按 $(W\_length+\#Cayley)/2$ 计算；分子为奇数时报 `"length parity"`。`CayleyCrossDecomposition` 按类使用，不为每条记录重建。相关构造约束见 [[Cartan 轨道的幂等添加与容量约束]]。^[involution-table.md:44-49]

## BFS、去重与 cross 链接

BFS 的邻居为 $s_g\cdot w\cdot s_{\mathrm{twist}(g)}$，分别在词级通过两次 `multiply`、在作用级通过两次 `compose` 构造。去重键是前向根置换。对于未被去重消费的边，Weyl 长度差必须为 $\pm2$，对应对合长度变化为 $\mp1$，否则报告 `"twisted length step"`；长度差为零表示该边固定当前对合，已由去重处理。^[involution-table.md:51-54]

遍历每个节点时，表保存一条 `cross_links` 记录，供构建后的 `cross(generator, id)` 直接查询。该接口表示 $s\cdot w\cdot\mathrm{twist}(s)$，来源将其描述为构建后 O(1) 的存储直查；这属于访问结构说明，不是实测性能结论。^[involution-table.md:55-57, involution-table.md:71-72]

种子插入 `index_by_permutation` 时没有碰撞检查：`BTreeMap::insert` 会静默覆盖相同键。因此，不同 Cartan 轨道的键互不重叠仍依赖调用纪律。容量检查在 `records.len() == max_involutions` 时拒绝继续插入，并返回 `InvolutionTableResourceLimit { resource: "involutions" }`。^[involution-table.md:60-63]

## 记录字段与图像基传送

除图像基对外，记录字段均在入表时由 $\theta$ 典范导出。图像基对在轨道的典范 involution 处，通过 $1-\theta$ 的 echelon reduction 播种，再沿 cross-action BFS 传送；它具有路径依赖性，且 `y_lift` 的符号依赖于所选基。相关机制见 [[实投影像基的播种与路径依赖传送]]。^[involution-table.md:26-29]

投影传送使用普通生成元 $s$ 的矩阵，而非 $\mathrm{twist}(s)$，因为 $\delta$ 已并入 $\theta$。每条传送后的投影都先通过 `check_against` 与新记录重新推导的 $\theta$ 核对，种子则使用 `RealProjection::build`。此外，`push_record` 按 $(2\rho+\theta\cdot2\rho)/2$ 计算 $(1+\theta)\rho$，出现奇坐标时报 `"theta rho parity"`。^[involution-table.md:55-59]

## 查询语义与调用方契约

`lookup(&WeylElement)` 使用前向根置换查找。stage a 已将该置换确定为完整相等性键，但同基数外部系统的兼容性仍属于调用方契约。表还提供 `record(id)`、`involution_count()`、`root_system()`、`inner_class()`，以及通过扫描轨道切片确定归属的 `cartan_of(id)`。^[involution-table.md:67-70]

`cayley(generator, id)` 通过反射乘积和置换查表计算邻居 $s\cdot w$。目标 Cartan 类尚未添加时返回 `None`；stage e 要求预先添加该实形式向上封闭的 Cartan 集合，在此契约下出现 `None` 表示调用方违反不变量。`simple_root_kind` 则统一覆盖上游三个 `is_*_simple` 测试。相关接口边界见 [[Cayley 邻居查询与向上封闭 Cartan 集合]]。^[involution-table.md:73-77]

## 测试与证据边界

来源列出的测试锚点涵盖 A1 字段与幂等性、B2 全表与分类一致性及独立建表可复现性、记录的 $\theta$ 与长度公式、Cartan 添加前后的 Cayley 查询、扭转 A2 的轨道大小，以及容量和越界守卫。其中 B2 投影测试明确区分传送基与现场重算基：指定 $\theta$ 下，传送得到的 `lift_mat` 为 `[[2],[-2]]`，重算则为 `[[-2],[2]]`，并以 `assert_ne` 锚定这一差异。^[involution-table.md:79-87]

这些测试锚点不覆盖四个不变量错误字面量、`AllocationFailed` 和 `lookup` 的 `Some` 命中直断。来源包本身仅进行了结构性阅读，未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；上游行号也仅转述自源码注释，未独立重读上游。进一步的覆盖说明见 [[Involution 表的测试锚点与证据边界]]。^[involution-table.md:87-88, involution-table.md:98-103]

## Sources

- [involution-table.md](involution-table.md) — Twisted involution 表（KGB stage b）。
