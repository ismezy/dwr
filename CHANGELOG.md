# Changelog

本项目的所有重要变更都记录在此文件中。格式参考 [Keep a Changelog](https://keepachangelog.com/)。

All notable changes to this project are documented in this file.

---

## [v0.2.0] - 2026-08-28

### 中文

#### 新增

- **项目树型结构**：项目（仅需名称 + 编号）下可挂载多个项目目录，左侧项目面板改为两级树；目录可随时修改所属项目
- **docs 型项目目录**：支持文档目录作为日报来源——扫描当天修改的文件（纯文本直读，docx / xlsx / pptx 自动提取文本），与快照做 diff 生成文档工作日报
- **按分支生成日报**：代码目录可限定一个或多个本地分支（可搜索的多选弹层），区分同一仓库不同分支上的工作；git log 自动去重多分支共有的提交
- **AI 微调**：对生成的日/周报输入修改要求（如量化工作、调整语气），由 AI 定向改写内容
- **按项目批量生成**：选中项目时一次性为其下所有目录生成日报，报告列表按目录分节展示
- **版本化数据迁移**：引入 `DATA_VERSION` + `PRAGMA user_version` 迁移机制，旧版平铺项目数据启动时自动升级为 项目 → 目录 两级结构

#### 变更

- **生成模式默认值**：模式切换顺序改为 汇总 / 按项目 / 全部，默认选中「汇总」，并通过 localStorage 记住上次选择
- **删除保护**：删除包含目录的项目时会被拦截并提示，需先删除其下所有目录
- **周总结生成逻辑重构**：移除对项目状态的依赖

#### 修复

- 修复 `loadReportsForDirs` 在 `$effect` 中读写同一状态导致的无限循环（`effect_update_depth_exceeded`）

### English

#### Added

- **Project tree structure**: a project (name + code only) can hold multiple directories; the left panel is now a two-level tree, and a directory's parent project can be changed at any time
- **Document directories**: docs folders as report sources — scans files modified on the day (plain text read directly; text extraction for docx / xlsx / pptx) and generates reports from snapshot diffs
- **Branch-filtered reports**: code directories can be restricted to one or more local branches via a searchable multi-select popover, separating work on different branches of the same repository; commits shared by multiple branches are deduplicated
- **AI Refine**: give custom instructions (e.g. quantify work, adjust tone) for targeted AI rewrites of generated reports
- **Batch per-project generation**: selecting a project generates reports for all its directories at once, with the report list grouped by directory
- **Versioned data migration**: `DATA_VERSION` + `PRAGMA user_version` migration mechanism; legacy flat project data is automatically upgraded to the two-level tree on startup

#### Changed

- **Generation mode defaults**: mode order is now Summary / By Project / All, defaulting to Summary, with the last choice remembered via localStorage
- **Deletion guard**: deleting a project that still has directories is blocked with a prompt
- **Weekly summary generation refactored**: no longer depends on project state

#### Fixed

- Fixed an infinite update loop (`effect_update_depth_exceeded`) caused by `loadReportsForDirs` reading and writing the same state inside a `$effect`

---

[v0.1.1]: 初始版本：日报/周报生成、AI 润色、多语言、历史记录管理、自定义 Git 路径等。
