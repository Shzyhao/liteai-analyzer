# LiteAI Analyzer

> Windows right-click AI file analyzer — select a file, right-click, and get instant AI analysis with streaming output.

## Features

- 🖱️ **One-click right-click analysis**: Select one or more files → right-click → "AI Analyze" → the app opens and streams the result live
- ⚡ **Streaming output**: Results render in real time as the model generates
- 📄 **Dual output channels**: Live UI preview + optional save as `.ai.md` (default) or extra `.docx` export
- 📚 **File parsing**: txt / md / pdf / xlsx / docx / csv / json / source code files, and more
- 🎛️ **Visual configuration**: API settings, 5 built-in prompt templates + custom templates, JSON template import/export, file-type whitelist
- 🔐 **Privacy-first**: API key stored in Windows Credential Manager with one-click clear; local config encrypted; data never leaves your machine
- 📦 **Lightweight**: Built with Tauri 2.0, single-file exe ~16MB, no background processes

## Quick Start

### 1. Configure the model
Open the app → "Settings" → fill in Base URL, model, and API key → Test connection.

Optimized for DeepSeek (OpenAI-compatible):
- Base URL: `https://api.deepseek.com`
- Model: `deepseek-chat`

### 2. Install the right-click menu
Go to the "Context Menu" tab → click "Install right-click menu".

### 3. Use it
Select a file in File Explorer → right-click → "AI Analyze".

## Architecture

```
crates/
  liteai-core    Domain model + analysis pipeline (shared by GUI/CLI, no tauri dependency)
  liteai-parsers File parsers (txt/xlsx/docx/pdf)
  liteai-model   OpenAI-compatible streaming client (SSE/balance/connectivity)
  liteai-output  Markdown / Word serializers
  liteai-config  Config storage + secure secret storage + templates
  liteai-cli     CLI smoke-test tool
src-tauri/       Tauri shell (commands / context-menu / single instance)
src/             React frontend (streaming viewer / settings / templates / queue)
```

Tech stack: **Tauri 2.0** (Rust backend + React/Vite frontend), DeepSeek (OpenAI-compatible SSE).

## Development

```bash
# Core library tests
cargo test -p liteai-core -p liteai-parsers -p liteai-model -p liteai-output -p liteai-config

# CLI smoke test (requires LITEAI_API_KEY env var)
LITEAI_API_KEY=sk-xxx cargo run -p liteai-cli -- analyze samples/项目周报.txt --docx

# GUI dev mode
npm run tauri dev

# Package (NSIS installer)
npm run tauri build
```

## Download

See the [Releases](https://github.com/Shzyhao/liteai-analyzer/releases) page for prebuilt executables and installers.

## License

MIT
