# DWR - 开发代理指南

## 项目概述

DWR（Daily Work Report）是一款桌面端工具软件，帮助程序员根据项目的 Git 提交记录自动生成工作日报。用户选择本地 Git 仓库后，软件解析指定日期范围内的 commit 信息，汇总并生成结构化日报内容，支持复制或导出。

## 技术栈

- **桌面框架**: [Tauri v2](https://v2.tauri.app/) — Rust 后端 + Web 前端
- **前端框架**: [SvelteKit](https://svelte.dev/) + [Svelte 5 (Runes)](https://svelte.dev/docs/svelte/what-are-runes)
- **样式系统**: [Tailwind CSS v4](https://tailwindcss.com/) + [shadcn-svelte](https://shadcn-svelte.com/)
- **国际化**: 自定义轻量级 i18n（基于 Svelte 5 Runes，`src/lib/i18n/`）
- **构建工具**: Vite 6
- **包管理器**: Yarn
- **语言**: TypeScript（前端）+ Rust（后端）

## 项目结构

```
dwr/
├── src/                    # 前端源码（SvelteKit）
│   ├── routes/             # 页面路由
│   │   ├── +page.svelte    # 主页面
│   │   ├── +layout.ts      # 根布局（SSR 关闭，SPA 模式）
│   │   └── +layout.svelte  # 根布局组件（引入全局样式）
│   ├── lib/                # 库代码
│   │   ├── components/     # 组件目录（含 shadcn/ui）
│   │   │   └── ui/         # shadcn 组件存放位置
│   │   ├── i18n/           # 国际化
│   │   │   ├── zh.json     # 中文语言包
│   │   │   ├── en.json     # 英文语言包
│   │   │   └── index.svelte.ts  # i18n 核心逻辑
│   │   ├── hooks/          # 自定义 hooks
│   │   └── utils.ts        # 工具函数（cn 等）
│   └── app.html            # HTML 模板
├── src-tauri/              # Tauri 后端（Rust）
│   ├── src/
│   │   ├── main.rs         # 程序入口
│   │   └── lib.rs          # 核心逻辑、Command 定义
│   ├── capabilities/       # 权限配置
│   ├── icons/              # 应用图标
│   ├── Cargo.toml          # Rust 依赖
│   └── tauri.conf.json     # Tauri 配置
├── static/                 # 静态资源
├── components.json         # shadcn-svelte 配置文件
├── package.json            # Node 依赖与脚本
├── svelte.config.js        # SvelteKit 配置（adapter-static）
├── vite.config.js          # Vite 配置
├── tsconfig.json           # TypeScript 配置
└── src/app.css             # 全局样式入口（Tailwind + shadcn 主题）
```

## 常用命令

```bash
# 开发模式（同时启动 Vite 前端和 Tauri 桌面窗口）
yarn tauri dev

# 构建前端
yarn build

# 构建桌面应用安装包
yarn tauri build

# TypeScript 类型检查
yarn check

# 添加 shadcn-svelte 组件（示例：button、dialog）
npx shadcn-svelte add button dialog

# 添加所有 shadcn-svelte 组件
npx shadcn-svelte add -a
```

## 架构约定

### 前端（SvelteKit）

- **SPA 模式**: 使用 `@sveltejs/adapter-static`，关闭 SSR（`src/routes/+layout.ts` 中 `export const ssr = false`）。
- **状态管理**: 优先使用 Svelte 5 的 Runes（`$state`, `$derived`, `$effect`），避免遗留的 `let` 响应式语法。
- **样式系统**: 全局样式通过 `src/app.css` 引入 Tailwind CSS v4 和 shadcn 主题变量。UI 组件以 **shadcn-svelte** 为主，辅以 Tailwind utility class 进行微调。不要大面积手写 `<style>` 块。
- **Tauri API**: 通过 `@tauri-apps/api/core` 的 `invoke()` 调用 Rust 命令。

### 后端（Rust）

- **Command 模式**: 所有暴露给前端的能力以 `#[tauri::command]` 函数定义在 `src-tauri/src/lib.rs` 中，并通过 `tauri::generate_handler![]` 注册。
- **错误处理**: Rust 函数返回 `Result<T, String>`，将错误信息字符串传递给前端展示。
- **Git 操作**: 调用系统 Git 命令或使用 Rust Git 库（如 `git2`）读取仓库日志，**禁止**修改用户仓库内容。

### 前后端通信示例

```rust
// src-tauri/src/lib.rs
#[tauri::command]
fn get_commits(repo_path: &str, since: &str, until: &str) -> Result<Vec<Commit>, String> {
    // 解析 Git 日志...
}
```

```typescript
// 前端调用
import { invoke } from "@tauri-apps/api/core";
const commits = await invoke<Commit[]>("get_commits", { repo_path: path, since, until });
```

## 国际化（i18n）规范

项目使用自定义轻量级 i18n 方案（`src/lib/i18n/index.svelte.ts`），基于 Svelte 5 Runes 实现响应式语言切换。当前支持 **中文（zh）** 和 **英文（en）**。

### 使用方式

```svelte
<script lang="ts">
  import { i18n } from '$lib/i18n';
</script>

<span>{i18n.t('project.title')}</span>
<button>{i18n.t('common.save')}</button>
```

### 开发要求

- **禁止硬编码文本**：所有用户可见的界面文本（按钮、标签、提示、空状态、错误信息等）**必须**通过 `i18n.t('key')` 获取，禁止直接写死中文或英文。
- **同步更新双语**：新增或修改文案时，必须同时在 `src/lib/i18n/zh.json` 和 `src/lib/i18n/en.json` 中添加对应的键值对。
- **键名命名规范**：采用 `模块.子模块.键名` 的层级结构，使用 camelCase：
  - `app.name`、`app.subtitle` — 应用级文本
  - `project.title`、`project.add`、`project.emptyHint` — 项目模块
  - `dailyReport.title`、`dailyReport.emptyHint` — 日报模块
  - `content.title`、`content.placeholder` — 内容展示模块
  - `config.title`、`config.workDir`、`config.language` — 配置模块
  - `common.save`、`common.cancel`、`common.required` — 通用操作
- **类型安全**：`t()` 函数的 `key` 参数类型为 `keyof Messages`，TS 会自动提示可用的 key，不要绕过类型检查传动态字符串（除非确有必要）。
- **fallback 处理**：如需提供默认值，使用 `i18n.t('key', '默认文本')`。

### 语言切换

```typescript
import { i18n } from '$lib/i18n';
i18n.setLocale('en');  // 切换为英文
i18n.setLocale('zh');  // 切换为中文
```

语言偏好由 `configStore` 持久化保存，应用启动时自动恢复。

## 核心功能规范

1. **仓库选择**: 通过 Tauri 的对话框 API 让用户选择本地文件夹路径。
2. **提交解析**: 读取 `git log`，提取 commit message、author、date、changed files、diff stats。
3. **日报生成**: 按日期/项目聚合 commit，过滤合并提交（merge commits），去重并生成可读的工作日报文本。
4. **输出格式**: 支持纯文本、Markdown，未来可扩展为直接复制到剪贴板或导出 `.md`/`.txt` 文件。
5. **日期范围**: 默认今天，支持自定义起止日期。

## Tailwind CSS 与 shadcn-svelte 规范

### Tailwind CSS v4

- **配置方式**: Tailwind v4 不再使用 `tailwind.config.js`，而是通过 `src/app.css` 中的 `@import "tailwindcss"` 和 `@theme` 指令配置。
- **自定义样式**: 如需扩展主题，在 `src/app.css` 中使用 `@theme` 块添加 CSS 自定义属性，或在组件中使用 `class="..."` 组合 Tailwind utility class。
- **Dark Mode**: 通过 `@custom-variant dark (&:is(.dark *));` 支持，切换 `html` 或 `body` 的 `.dark` 类即可。

### shadcn-svelte 组件

- **组件存放位置**: 所有 shadcn 组件安装在 `src/lib/components/ui/` 目录下。
- **使用规范**:
  - 优先使用 shadcn-svelte 提供的现成组件，保持界面风格一致。
  - **如果项目没有所需的 shadcn 组件，必须先下载再使用**，不要手写替代：
    ```bash
    npx shadcn-svelte add <组件名>
    # 例如：npx shadcn-svelte add button card dialog
    ```
  - 自定义组件应放在 `src/lib/components/` 下的其他子目录（如 `src/lib/components/custom/`），与 `ui/` 区分开。
- **常用组件**: button、card、dialog、input、label、select、table、textarea、tooltip 等。
- **图标库**: shadcn-svelte 默认使用 `@lucide/svelte`，图标从该包导入。
- **工具函数**: 样式合并统一使用 `src/lib/utils.ts` 中的 `cn()` 函数（基于 `clsx` + `tailwind-merge`）。

## 开发注意事项

- **权限**: Tauri v2 使用 capabilities 文件管理权限。如需文件系统访问、对话框、剪贴板等新能力，需在 `src-tauri/capabilities/` 中声明对应权限。
- **跨平台**: 支持 Windows / macOS / Linux，路径处理使用 Rust 的 `std::path::PathBuf`，避免硬编码分隔符。
- **性能**: Git 日志解析在 Rust 端完成，前端只负责展示，避免大仓库数据在前端处理导致卡顿。
- **安全性**: 仅读取用户指定的 Git 仓库路径，不向外发送任何代码或日志信息。

## 依赖管理

- 前端依赖通过 `yarn add` / `yarn add -D` 管理。
- Rust 依赖通过修改 `src-tauri/Cargo.toml`，Tauri 插件需在 `lib.rs` 中 `.plugin(...)` 注册。

## 命名规范

- Rust: `snake_case`
- TypeScript / Svelte: `camelCase`（变量/函数）、`PascalCase`（类型/组件名）
- 前端调用 Rust command 的字符串名称使用 `snake_case`
