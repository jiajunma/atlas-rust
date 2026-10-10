---
title: 求值帧链与共享槽（frames.rs）——闭包捕获、借用纪律与全局 cell
source: atlas-rust/atlas-core-frames
ingestedAt: 2026-10-10T01:10:00Z
---

# 求值帧链与共享槽（frames.rs）

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包覆盖
`crates/atlas-core/src/frames.rs`（310 行，全部）——类型化管线 phase B
的求值帧机器。模块自述移植上游形状（axis-types.w:2370-2400、
2830-2848，行号转述自模块注释，未独立重读上游）。对应
[阅读快照](snapshots/2026-10-10-atlas-core-frames.json)。结构性阅读，
不声称语言验收。

## 帧链与槽

- `Frame { next: Option<Rc<Frame>>, slots: RefCell<Vec<Option<SharedValue>> }`
  ——堆上 `Rc` 链接的帧链；闭包可以共享链尾（shared tail）。
  `SharedValue = Rc<Value>`（上游 `shared_value`）。
- 局部寻址用 `(depth, offset)`，在分析期固定；`frame_at` 从当前头逐跳
  走链。
- **空绑定层没有帧**：分析器计算深度时跳过它们，所以每个实际存在的帧
  至少有一个槽（`with_frame_traced` 有 `debug_assert!(!slots.is_empty())`）。
- 槽是 `Option<SharedValue>`：`None` 表示声明而未初始化（例如被
  `take_local` 搬空后）。

## 借用纪律（模块级设计不变量）

读操作在短借用下**克隆**共享值；写操作在右侧完全求值后才取短借用；
任何借用都不跨嵌套求值持有。这取代了上游依赖 C++ RAII 的跨异常恢复：
本移植用 `Result` 路由控制流，并在作用域函数里恢复上下文。

## 上下文操作

- `capture()`：克隆当前链头 `Rc`——闭包值的捕获点；被捕获的链在帧
  弹出后继续存活。
- `with_frame(slots, body)`：压入新帧运行 `body`，**每一个非 panic
  出口**都恢复旧链（含 `?` 传播的 break/return/运行时错误）。
  `with_frame_traced` 额外交出被压入的帧：错误沿调用展开时，回溯用它
  转储局部槽（axis.w:2896-2909 的 local-variable trace）。
- `slot_snapshot`：在展开**之后**短借用读取——出错前被重新赋值的槽
  打印的是当前值。
- `with_context(captured, body)`：把上下文整体换成闭包捕获的链
  （上游 closure apply），返回时恢复调用方的链。
- `local` / `set_local` / `take_local`：读克隆；写在值完全求值后；
  `take_local` 把值搬出槽、留下未初始化——这是上游 pilfering local
  identifier 的安全 Rust 对应物（配合 typed-eval 包的饥饿目的地取值）。

## 全局 cell

`GlobalCell = Rc<RefCell<Option<SharedValue>>`。**每个 `set` 式定义都
无条件分配新 cell**——转换后的代码持有它在分析期捕获的那个 cell；
只有 `:=` 赋值写入既有 cell。`None` 标记已声明未赋值的全局，读取是
运行时错误。`unset_global`/`global_with` 是两个构造入口。

## 同帧的其他状态

`EvaluationContext` 还携带：打印机内建的中间缓冲（上游
`*output_stream` 的求值中途写；命令层在顶层求值后排空成报告事件；
`printed_buffer` 直供"既打印又抛出"的领域内建，注释引
ext_kl.cpp:945-948 的先打印后抛错）；补全候选子模块（见补全索引包）；
`source_names` 回溯显示名表（buffer.w:694；未命名缓冲区回退为
`<standard input>`）。

## 测试锚点

5 个单测：深度走链读写（含坏 offset/无此深度的否定）、捕获链在弹出后
存活且**共享突变**（上游 shared-tail 语义：经交换上下文写入 9，另一
持有者可见）、`?` 传播错误时两层帧都恢复、全局 cell 区分未赋值与已
赋值、`take_local` 后槽留空且可再赋值。

## 边界声明

本包是对当前工作区字节的结构性阅读（git base 与哈希见快照）；上游
CWEB 行号转述自源码注释。帧链行为的兼容性权威是 HPC 语料门；本包不
声称语言验收。
