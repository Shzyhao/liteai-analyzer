# LiteAI Analyzer · 轻析

> Windows 右键菜单 AI 文件分析助手 — 选中文件，右键一键分析，AI 流式输出结果。

## 功能特性

- 🖱️ **右键一键分析**：选中单个/多个文件 → 右键 →「AI 分析」→ 应用自动弹出并流式生成结果
- ⚡ **流式输出**：结果边生成边显示在界面，无需等待
- 📄 **双通道呈现**：UI 实时预览 + 可选保存为 `.ai.md`（默认）或额外导出 `.docx`
- 📚 **文件解析**：支持 txt / md / pdf / xlsx / docx / csv / json / 代码文件 等
- 🎛️ **可视化配置**：API 配置、5 个内置 Prompt 模板 + 自定义模板、JSON 模板包导入导出、文件类型白名单
- 🔐 **隐私优先**：API Key 存 Windows 凭据管理器，一键清除；本地配置加密，数据不出域
- 📦 **极致轻量**：Tauri 2.0 构建，单文件 exe 约 16MB，无后台驻留

## 快速开始

### 1. 配置模型
打开应用 →「设置」→ 填入 Base URL、模型、API Key → 测试连接。

默认适配 DeepSeek（OpenAI 兼容）：
- Base URL：`https://api.deepseek.com`
- 模型：`deepseek-chat`

### 2. 安装右键菜单
「右键菜单」页 → 点「安装右键菜单」。

### 3. 使用
在资源管理器中选中文件 → 右键 →「AI 分析」。

## 技术架构

```
crates/
  liteai-core    领域模型 + 分析编排管线（GUI/CLI 共用，零依赖 tauri）
  liteai-parsers 文件解析器（txt/xlsx/docx/pdf）
  liteai-model   OpenAI 兼容流式客户端（SSE/余额/连通测试）
  liteai-output  Markdown / Word 序列化器
  liteai-config  配置存储 + 密钥安全存储 + 模板
  liteai-cli     命令行冒烟工具
src-tauri/       Tauri 壳（命令层 / 右键集成 / 单实例）
src/             React 前端（流式渲染 / 设置 / 模板 / 队列）
```

技术栈：**Tauri 2.0**（Rust 后端 + React/Vite 前端）、DeepSeek（OpenAI 兼容 SSE）。

## 开发

```bash
# 核心库测试
cargo test -p liteai-core -p liteai-parsers -p liteai-model -p liteai-output -p liteai-config

# CLI 冒烟（需环境变量 LITEAI_API_KEY）
LITEAI_API_KEY=sk-xxx cargo run -p liteai-cli -- analyze samples/项目周报.txt --docx

# 开发模式（GUI）
npm run tauri dev

# 打包（NSIS 安装包）
npm run tauri build
```

详见 [开发运行指南.md](开发运行指南.md)。

## 许可证

MIT
