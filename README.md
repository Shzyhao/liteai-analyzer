# LiteAI Analyzer · 轻析

> Windows 右键菜单 AI 文件分析助手 — 选中文件，右键一键分析，AI 流式输出结果。

## 功能特性

- 🖱️ **右键一键分析**：选中单个/多个文件 → 右键 →「AI 分析」→ 应用自动弹出并流式生成结果
- ⚡ **流式输出**：结果边生成边显示在界面，无需等待（带打字机光标）
- 📄 **双通道呈现**：UI 实时预览 + 可选保存为 `.ai.md` / `.docx` / `.xlsx`
- 🎛️ **多套 API 配置**：配置多套模型（DeepSeek / OpenAI / Kimi…），一键切换使用
- 🤖 **AI 生成导出技能**：一句话让 AI 生成自定义导出脚本（严格安全约束），或手动添加脚本
- 📚 **文件解析**：支持 txt / md / pdf / xlsx / docx / csv / json / 代码文件 等
- 🗂️ **分析历史记录**：自动保存每次分析，可回看 / 重新分析 / 定位文件
- 🌙 **深色模式**：跟随系统 / 浅色 / 深色切换
- 🖱️ **拖拽分析**：把文件直接拖进窗口即可分析
- 🔐 **隐私优先**：每套配置的 Key 独立存 Windows 凭据管理器，一键清除；数据不出域
- 📦 **极致轻量**：Tauri 2.0 构建，单文件 exe 约 17MB，无后台驻留

📋 完整版本更新见 [CHANGELOG.md](CHANGELOG.md)。

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

详见 [开发运行指南.md](开发运行指南.md)。English version: [README.en.md](README.en.md).

## 下载

[![GitHub Release](https://img.shields.io/github/v/release/Shzyhao/liteai-analyzer)](https://github.com/Shzyhao/liteai-analyzer/releases/latest)

- 📦 **便携版**（单文件 exe，免安装，直接运行）：[liteai-app.exe](https://github.com/Shzyhao/liteai-analyzer/releases/latest)
- 🖥️ **安装版**（NSIS 安装包，装到 Program Files）：[LiteAI_*_x64-setup.exe](https://github.com/Shzyhao/liteai-analyzer/releases/latest)

> 提示：Win10 用户若运行 exe 提示缺 WebView2，请先安装 [Microsoft Edge WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/)。Win11 自带。

## 许可证

MIT
