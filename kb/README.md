# Atlas 知识库

本知识库与 Atlas Rust 源码保存在同一个 Git 仓库，在同一项开发变更中更新、审查和提交。主线是 Rust 版本的数学知识、计算算法、系统设计和持续演进。C++ 是兼容性 baseline，只在对齐行为、追溯算法来源或解释差异时重点比较，不要求每篇笔记都写两版对照。

从 [知识索引](index.md) 开始阅读。新增页面遵循 [页面约定](schema.md)，开发 agent 先读 [维护规则](AGENTS.md)。

## 在 Obsidian 中阅读

选择“打开本地仓库”，打开当前仓库的 `kb` 目录即可。这里的“仓库”指 Obsidian vault；不要在 `kb` 内再次执行 `git init`。第一版只用 Markdown、LaTeX 和普通相对链接，不要求社区插件或在线服务。

KB 内部链接可以在 Obsidian 与 Git 网页中阅读。指向上层 `docs`、`crates`、`tests` 的相对链接供 Git 网页和代码编辑器使用；独立 vault 可能无法打开这些外部文件。需要浏览整个项目时，也可以将整个 Git 仓库作为 vault 打开。个人 Obsidian 配置和缓存不进入 Git。

## 与项目一起迭代

开始任务时找到有关主题和来源版本；结束任务时根据实际 diff 更新受影响的页面、[索引](index.md)和 [变更日志](log.md)。没有知识变化的任务无需产生笔记。当前实现解释可以更新，历史观察、反例和设计决策保留来源及被替代关系。

现有 `docs` 继续保存兼容性契约、设计文档和操作记录；测试及 HPC 报告仍在原位置。KB 通过 [来源索引](sources/index.md)引用它们，不另建一份验收账本。理论依据、源码判断和执行证据分别说明。

第一批四篇主题页来自当前工作区的只读阅读，其精确来源列在 [初始来源快照](sources/snapshots/2026-10-01-initial.json)。工作区包含未提交改动，快照不是构建产物、独立复核或数学验收。它们保留为人工维护的起始笔记，尚未迁移到编译器的原生页面格式。

## llm-wiki-compiler

已选定 [llm-wiki-compiler](https://github.com/atomicstrata/llm-wiki-compiler)，在本目录本地安装，版本固定为 **1.4.0-rc.2**。这是候选发行版，包含 1.3.0 之后的代码与链接保护修复；版本和传递依赖分别锁定在 package.json 与 pnpm-lock.yaml。运行环境为 Node.js 24+，包管理器为 pnpm 11.19.0。

从仓库根目录使用：

```sh
./kb/llmwiki --version
./kb/llmwiki --help
./kb/llmwiki compile --review --instructions AGENTS.md
./kb/llmwiki review list
./kb/llmwiki review show <id>
./kb/llmwiki review approve <id>
```

启动器会切换到 kb/，因此上面的 AGENTS.md 指的是本目录的维护规则。它优先使用 PATH 中的 Node，可用 ATLAS_KB_NODE 指定绝对路径；本机也支持现有桌面运行时的 Node。默认接入已有的 Codex CLI 登录、输出中文，并关闭 embedding 更新。可通过 LLMWIKI_PROVIDER、LLMWIKI_MODEL、LLMWIKI_EMBEDDINGS 等环境变量调整；不在 Git 中保存凭据。安装不等于已经验证模型登录或完成第一次知识编译。

在另一台机器恢复依赖：安装 Node.js 24+ 和 pnpm 11.19.0，然后在 kb/ 运行 `pnpm install --frozen-lockfile --ignore-scripts`。无需安装全局 llmwiki，也不要使用会漂移版本的 npx latest。

原生配置 [.llmwiki/config.json](.llmwiki/config.json)强制所有编译结果进入待审稿。编译输入是 sources/ 下的 Markdown 来源包，排除来源索引与历史快照；工具不会自行发现来源包背后的 Rust 文件已经变化。新生成页使用 wiki/concepts/，通过标签和正文区分数学、算法、系统设计及 baseline 对齐。现有四篇笔记仍由顶层 index.md 导航，不会被初始化覆盖。

审核时检查来源、当前目标页及批准后的完整 Git diff，保留手工补充。工具的 fresh、confidence 和批准状态只用于知识编辑，不能代替 Atlas 的数学验收。公式、代码、来源版本和逐页更新范围的具体要求见 [AGENTS.md](AGENTS.md)。
