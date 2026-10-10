---
title: 跨对偶的 Cartan 类对应与扭曲代表元
summary: 通过对偶根像置换定位共轭类且缺类时报错，内部代表元则在对偶 datum 上重放原 Weyl 字，以保留双格作用及对合来源。
sources:
  - root-datum-dual.md
kind: concept
createdAt: "2026-10-09T15:10:30.009Z"
updatedAt: "2026-10-10T00:50:35.623Z"
tags:
  - 对偶
  - Cartan分类
  - 扭曲对合
aliases:
  - 跨对偶的-cartan-类对应与扭曲代表元
  - 跨C类
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 跨对偶的 Cartan 类对应与扭曲代表元
summary: Cartan 类对应以对偶根像置换定位共轭类，缺类视为不变量错误；扭曲代表元通过在对偶根数据上重放原 Weyl 字构造，保留两种格作用及对合来源。
sources:
  - root-datum-dual.md
kind: concept
tags:
  - Cartan分类
  - 对偶性
  - 扭曲对合
aliases:
  - 跨对偶的-cartan-类对应与扭曲代表元
---

# 跨对偶的 Cartan 类对应与扭曲代表元

`dual.rs` 提供两种跨对偶操作：公开函数 `dual_cartan_correspondence` 为原分类中的每个 Cartan 类确定对偶 Cartan 类及其弱实形式计数；crate 内部函数 `dual_twisted_representative` 在对偶根数据上重放原 Weyl 字，构造保留两种格作用与 distinguished involution 来源信息的扭曲代表元。^[root-datum-dual.md:141-164]

## 对偶内类基础

[[对偶根数据与对偶内类构造]]通过转置 Cartan 矩阵、交换简单根与简单余根获得对偶 datum，并调用 `BasedRootDatum::from_simple_data` 复用构造校验。设原 distinguished involution 的权作用为 $q$，最长 Weyl 元作用为 $W_0$，则对偶内类的权作用为 $-(qW_0)^t$，余权作用为 $-(qW_0)$。实现逐元素执行 `checked_neg`，最终经过 `LatticeInvolution::new` 与 `InnerClass::new` 构造。^[root-datum-dual.md:108-112, root-datum-dual.md:129-139]

## Cartan 类对应

`dual_cartan_correspondence` 按 crate 的 Cartan 顺序遍历原分类，为每个类输出 `(对偶 CartanId, 对偶类的 weak-real-form 计数)`。前置校验检查两侧 fundamental 的存在性与 datum 一致性，相关错误为 `DatumMismatch`；主流程通过带来源检查的 `TwistedConjugacyPartition::class_of` 建立 `cartan_of_raw` 反查表。编号背景见 [[CartanId 的 Atlas 编号顺序]]。^[root-datum-dual.md:141-146]

代码注释将该对应解释为上游对 `tw` 与 `tw * w0` 的反序配对：对偶 distinguished involution 为 $-(\delta w_0)^t$，结合转置对应逆步，配对 involution 化简为 $-(w\delta)|_{\mathrm{co}}$。由于对偶 twisted involution 随后会被规范化，存储代表元一般是 `tw*w0` 的共轭，直接比较矩阵并不可靠。因此，实现以 lattice map 在对偶根上诱导的根像置换作为定位键。这一设计解释来自代码注释，来源包未独立验证。^[root-datum-dual.md:147-151]

公开参数 `_weyl_budget` 是遗留的未使用参数；函数中的最长元行走实际以 `dual.root_system().roots().len()`，即已枚举的对偶根数为预算。查不到对应类会触发不变量错误，不表示允许缺失的条目。实现另有三处 `.expect("cartan_ids yields in-range ids")`；`cartan_of_raw[raw]` 的直接索引也依赖构造不变量。^[root-datum-dual.md:152-155, root-datum-dual.md:200-201]

## 扭曲代表元构造

`dual_twisted_representative` 先通过 `WeylElement::from_action` 与 `canonical_word` 提取原代表元的 Weyl 字，所用 `WeylInterface` 由原 datum 的 Cartan 矩阵构造。随后在对偶 datum 上逐生成元重放该字，末步左合成 `longest`，再结合对偶 distinguished involution 构造 `TwistedInvolution`。相关背景见 [[Weyl 元素的规范词]]。^[root-datum-dual.md:157-164]

文档将这一流程描述为 RealWeylContext 对偶 fiber 所用的同一原词重放方式。与用于共轭类定位的裸根置换查找相比，它保留权格与余权格上的作用，以及 distinguished-involution 的来源信息。^[root-datum-dual.md:159-164]

## 测试锚点与证据边界

来源转录的 sc A1 Cartan 对应表为 `[(CartanId(1), 1), (CartanId(0), 2)]`，体现反序配对，并注明 oracle capture `3501500` 锚点。紧 B2 的断言包括：对应表长度为 4、首条计数为 1、全格 $-1$ involution 类计数为 3，以及对偶 CartanId 集合恰为 `{0,1,2,3}`，在该案例中形成双射。^[root-datum-dual.md:179-182]

本页依据结构性源码阅读与测试断言转录，不构成数学验收。来源包未核对引用的上游文件字节，本次知识维护也未执行 Atlas、Cargo、测试或 benchmark；这些测试锚点不能据此推广为一般正确性结论。^[root-datum-dual.md:9-14, root-datum-dual.md:204-216]

## Sources

- [root-datum-dual.md](../../sources/root-datum-dual.md) — BasedRootDatum 与对偶内类构造（root_datum.rs / dual.rs）。
