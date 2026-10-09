---
title: Twisted cross-action 的 BFS 轨道构建
summary: 通过 s_g·w·s_twist(g) 生成邻居，以前向根置换去重并保存 cross_links；简单反射预先构造，Cayley/Cross 分解仅按 Cartan 类计算。
sources:
  - involution-table.md
kind: concept
createdAt: "2026-10-09T14:52:20.246Z"
updatedAt: "2026-10-09T20:55:58.104Z"
tags:
  - 轨道枚举
  - 广度优先搜索
  - Weyl群
aliases:
  - twisted-cross-action-的-bfs-轨道构建
  - TC的B轨
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Twisted cross-action 的 BFS 轨道构建
summary: 按 external-order BFS 生成 twisted involution 轨道，以前向根置换去重，沿遍历路径传送图像基，并保存 cross-action 链接供直接查询。
sources:
  - involution-table.md
kind: concept
tags:
  - 扭曲对合
  - 广度优先搜索
  - Weyl群
---

# Twisted cross-action 的 BFS 轨道构建

Twisted cross-action 的 BFS 轨道构建属于 KGB 构建流水线的 stage b：`InvolutionTable` 按 Cartan 类生成 twisted involution 轨道，每条轨道存为连续切片。轨道内部采用 external-order BFS，`InvolutionId` 跨 Cartan 全局连续递增；Cartan 类的添加顺序由调用方决定，文档纪律要求按 `CartanId` 升序添加。整体布局见 [[Twisted involution 表与 Cartan 轨道存储]]。^[involution-table.md:20-24, involution-table.md:33-35]

## 初始化与播种

`InvolutionTable::new` 为一个 inner class 创建空表，一次性导出经过验证的 simple twist、rank 个简单反射元素，以及来自 positivity slice 的 \(2\rho\)。BFS 边复用这些简单反射，不在每次遍历时重新构造。^[involution-table.md:39-42]

`add_cartan(classification, cartan)` 从 classification 取得种子与预期轨道大小；重复添加同一 Cartan 类会返回已有切片。种子通过 `WeylElement::from_action` 从矩阵级代表元转换为词级元素，对合长度按 \((W_{\mathrm{length}}+\#\mathrm{Cayley})/2\) 计算，分子为奇数时报 `"length parity"`。`CayleyCrossDecomposition` 按 Cartan 类使用，不在每个条目上重建。^[involution-table.md:44-49]

## 邻居生成、去重与长度更新

对当前元素 \(w\) 和简单生成元 \(g\)，BFS 邻居为 \(s_g\,w\,s_{\mathrm{twist}(g)}\)。实现分别在词级执行两次 `multiply`、在作用级执行两次 `compose`，以前向根置换作为去重键。该键也是 `lookup` 使用的完整相等性键；同基数外部系统的兼容性仍属于调用方契约，参见 [[前向根置换索引及其调用方契约]]。^[involution-table.md:51-54, involution-table.md:67-68]

新发现邻居的对合长度由 `stepped_length` 更新。来源记载，Weyl 长度差恰为 \(\pm2\) 时，对合长度相应变化为 \(\mp1\)，否则报 `"twisted length step"`；这里沿用来源的差值表述。长度差为零意味着边固定该对合，已由先行去重处理。每访问一个节点，构建过程都会存入一条 `cross_links`，供后续 `cross(generator, id)` 直接查询。^[involution-table.md:51-57]

## 沿 BFS 传送图像基

每条记录携带 \((1-\theta)X^*\) 的图像基对 `lift_mat`、`M_real`。除这对图像基外，所有记录字段都在入表时由 \(\theta\) 典范导出；图像基则在轨道典范 involution 处通过 \(1-\theta\) 的阶梯归约播种，再沿 cross-action BFS 传送。该基具有路径依赖性，且 `y_lift` 的符号依赖于它，因此不能以逐记录现场重算替代既定的传送过程。相关纪律见 [[图像基的典范播种与轨道传送纪律]]。^[involution-table.md:23-29]

投影传送使用普通生成元 \(s\) 的矩阵，而非 \(\mathrm{twist}(s)\)，因为 \(\delta\) 已并入 \(\theta\)。种子处调用 `RealProjection::build`；后续传送所得投影须先通过 `check_against`，与本记录新鲜推导的 \(\theta\) 核对后才采用。`push_record` 还以 \((2\rho+\theta\cdot2\rho)/2\) 计算 \((1+\theta)\rho\)，出现奇坐标则报 `"theta rho parity"`。^[involution-table.md:55-59]

## 构建不变量与访问契约

生成轨道必须恰好达到 classification 给出的预期大小，否则触发 `InvolutionTableInvariantViolation { invariant: "orbit size" }`。记录数已达到 `max_involutions` 时，继续插入会被拒绝，并返回 `InvolutionTableResourceLimit { resource: "involutions" }`。相关主题见 [[Cartan 轨道的幂等添加与容量约束]]。^[involution-table.md:44-47, involution-table.md:62-63]

种子写入 `index_by_permutation` 时没有碰撞检查：`BTreeMap::insert` 会静默覆盖同键值。因此，构建依赖不同 Cartan 轨道的键不重叠这一调用纪律。^[involution-table.md:60-61]

构建后，`orbit_slice(cartan)` 返回连续轨道切片及起始编号；`cross` 直接读取已存储的链接。来源将构建后的 `cross` 查询描述为 \(O(1)\)，这属于接口文档陈述，不是实测性能结论。^[involution-table.md:33-35, involution-table.md:71-72]

`cayley(generator, id)` 则通过反射乘积与置换查表计算邻居 \(s\cdot w\)；目标 Cartan 类尚未添加时返回 `None`。stage e 要求预先加入该实形式的向上封闭 Cartan 集合，此后出现 `None` 即违反调用方不变量，参见 [[Cayley 邻居查询与向上封闭 Cartan 集合]]。^[involution-table.md:73-75]

## 测试锚点与证据边界

来源列出的相关测试包括 A1 分裂两单元轨道与幂等添加、B2 全表与分类对账、两次独立建表逐切片相等、B2 每条记录的 \(\theta\) 典范性与长度公式，以及扭转 A2 的轨道大小排序后为 `[1, 3]`。这些是源码中的测试锚点。^[involution-table.md:79-84]

B2 投影传送测试包含 arm64 oracle 锚点：当 \(\theta=\begin{pmatrix}-1&0\\2&1\end{pmatrix}\) 时，记录中的 `lift_mat` 为 \(\begin{pmatrix}2\\-2\end{pmatrix}\)，现场重算则得到 \(\begin{pmatrix}-2\\2\end{pmatrix}\)，测试以 `assert_ne` 明确区分两者。这一案例保留了图像基传送与重新构造之间可观察的符号差异。^[involution-table.md:84-86]

来源还列出容量、越界和外来系统查询守卫；尚未触及四个不变量错误字面量、`AllocationFailed` 及 `lookup` 的 `Some` 命中直断。参见 [[Involution 表的测试锚点与证据边界]]。^[involution-table.md:86-88]

本页依据的来源属于结构性阅读，未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。上游行号仅转述自源码注释，未独立重读上游，可能随版本演进漂移。^[involution-table.md:9-16, involution-table.md:98-103]

## Sources

- [Twisted involution 表（KGB stage b）](involution-table.md)
