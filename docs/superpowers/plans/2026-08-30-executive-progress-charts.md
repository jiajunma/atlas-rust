# Executive Progress Charts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the two technical Atlas-Rust progress charts with two 16:9 executive slides that explain project output, recent Codex usage, verified completion, speed improvement, and remaining risk in plain Chinese.

**Architecture:** Keep the existing SVG filenames so report links remain stable, remove the PNG derivatives, and treat each SVG as the only delivered slide asset. Verify XML validity, required text, viewBox dimensions, and visual layout before updating the report wording.

**Tech Stack:** Hand-authored SVG 1.1, `xmllint`, SVG metadata/static checks, Markdown.

---

### Task 1: Redesign the project-output and Codex-usage slide

**Files:**
- Modify: `docs/assets/atlas-rust-slide-01-progress.svg`

- [x] **Step 1: Replace the technical title and subtitle**

Use these exact audience-facing lines:

```text
Atlas-Rust：35 天完成核心功能闭环，进入性能与发布收尾
从零搭建 Rust 版本，并用原版 Atlas 持续做自动对照
```

- [x] **Step 2: Build the “project output” area**

Show three large metrics without internal implementation terminology:

```text
约 11.8 万行代码
891 项测试
1,620 次代码提交
```

Use the secondary explanation `3 个主要代码模块 · 84 个 Rust 文件` rather than `workspace crates`.

- [x] **Step 3: Build the “verified progress” area**

Use plain-language labels and preserve the audited values:

```text
核心语言功能                  13 / 13
功能接口与类型转换            469 项 + 29 项，全部覆盖
自动对照测试                  360 / 360 通过
真实使用脚本                  240 / 240 结果一致
数据文件互通                  183 / 183 结果一致
```

Do not use `builtin`, `coercion`, `corpus`, `clean commit`, `PASS`, or `MATCH` as visible primary labels.

- [x] **Step 4: Add the Codex-usage area**

Show these exact values:

```text
Codex 辅助投入（近期可审计）
6.67 亿 token
4,079 次模型响应
93.3% 输入内容由缓存复用
```

Add this exact footnote at readable size:

```text
统计窗口：2026-08-22 15:17—08-30 14:07；包含开发、审查和本次进展分析，并非整个项目生命周期。
```

- [x] **Step 5: Add the honest current-status footer**

```text
已完成：核心功能与固定版本验证
进行中：最新新增测试、内存优化、交互体验与正式发布
```

- [x] **Step 6: Validate slide 1**

Run:

```bash
xmllint --noout docs/assets/atlas-rust-slide-01-progress.svg
rg -F 'viewBox="0 0 1600 900"' docs/assets/atlas-rust-slide-01-progress.svg
```

Expected: XML validation succeeds and the SVG retains a 1600×900 viewBox. Inspect the SVG preview and confirm no footnote, label, or metric overlaps.

### Task 2: Rewrite the performance slide in plain language

**Files:**
- Modify: `docs/assets/atlas-rust-slide-02-performance.svg`

- [x] **Step 1: Replace the title and axis explanation**

```text
同一批真实任务：速度差距从 29.5 倍降到 3.29 倍
相对原版 Atlas 的耗时倍数；1× 表示速度相当，越低越好
```

- [x] **Step 2: Keep the seven audited curve values**

```text
29.5×, 12.97×, 10.25×, 4.42×, 4.09×, 4.01×, 3.29×
```

Keep the logarithmic y-axis and 1× reference line because they make multiplicative convergence readable.

- [x] **Step 3: Replace the seven technical checkpoint labels**

```text
初始版本
减少重复计算
加快功能查找
优化核心算法
优化数据处理
修复超长文件瓶颈
当前完整基线
```

Remove `overload/type clone`, `variant cache`, `O(1) lookup`, `KGB`, `typed gate`, `LineCursor`, `reflection cache`, and job IDs from the primary visual.

- [x] **Step 4: Rewrite the three right-side conclusions**

```text
整体速度差距缩小约 9 倍
复杂任务：77.2 秒 → 8.4 秒
当前主要短板：典型任务内存约为原版的 12.2 倍
```

Do not use `wall-ratio`, `unipotent`, `RSS`, or `C++ reference` in the main labels. Label the reference line `原版 Atlas = 1×`.

- [x] **Step 5: Add the interpretation footnote**

```text
注：“约 9 倍”指相对耗时倍数从 29.5 降至 3.29，不代表所有程序统一加速 9 倍。
```

- [x] **Step 6: Validate slide 2**

Run:

```bash
xmllint --noout docs/assets/atlas-rust-slide-02-performance.svg
rg -F 'viewBox="0 0 1600 900"' docs/assets/atlas-rust-slide-02-performance.svg
```

Expected: XML validation succeeds and the SVG retains a 1600×900 viewBox. Inspect the SVG preview and confirm the first point, x-axis labels, right-side conclusions, and footnote do not overlap.

### Task 3: Align the progress report with the redesigned slides

**Files:**
- Modify: `docs/ATLAS_RUST_PROGRESS_REPORT_2026-08-30.md`

- [x] **Step 1: Update the slide guidance**

In section 2, describe slide 1 as `项目产出、可验证进展与 Codex 投入`, and slide 2 as `面向非技术听众的性能提升曲线`.

- [x] **Step 2: State the Codex-only scope**

Add this sentence beside the slide links:

```text
图中 AI 投入只展示 Codex；6.67 亿 token 是近期本地可审计记录，不代表项目全周期用量。
```

- [x] **Step 3: Verify report links**

Confirm both asset links resolve:

```text
assets/atlas-rust-slide-01-progress.svg
assets/atlas-rust-slide-02-performance.svg
```

Expected: every linked path exists relative to `docs/`.

### Task 4: Final visual and data verification

**Files:**
- Verify: `docs/assets/atlas-rust-slide-01-progress.svg`
- Verify: `docs/assets/atlas-rust-slide-02-performance.svg`
- Verify: `docs/ATLAS_RUST_PROGRESS_REPORT_2026-08-30.md`

- [x] **Step 1: Check required values and banned primary terminology**

Use `rg` to confirm both SVGs contain all required values and that the removed internal terms do not appear in visible text.

- [x] **Step 2: Confirm PNG derivatives are absent**

Verify that `docs/assets/atlas-rust-slide-01-progress.png` and `docs/assets/atlas-rust-slide-02-performance.png` do not exist.

- [x] **Step 3: Inspect both SVG previews at original aspect ratio**

Expected: titles fit on one line, all Chinese labels are legible, the Codex window disclaimer is visible, and no mark or label is clipped.

- [x] **Step 4: Review the exact file diff**

Run:

```bash
git status --short docs/ATLAS_RUST_PROGRESS_REPORT_2026-08-30.md docs/assets
```

Expected: only the report and the four chart assets are in scope; unrelated user modifications remain untouched.
