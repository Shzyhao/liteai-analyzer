//! liteai-cli：端到端冒烟工具。
//!
//! 用法：
//! ```sh
//! LITEAI_API_KEY=sk-xxx cargo run -p liteai-cli -- analyze <文件...> [--model deepseek-chat] [--base-url https://api.deepseek.com] [--template summary] [--output-dir DIR]
//! ```

use liteai_core::*;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 || args[1] != "analyze" {
        eprintln!("用法: liteai-cli analyze <文件...> [--model M] [--base-url U] [--template id] [--output-dir D] [--docx]");
        std::process::exit(2);
    }

    let mut model = "deepseek-chat".to_string();
    let mut base_url = "https://api.deepseek.com".to_string();
    let mut template_id = "summary".to_string();
    let mut out_dir: Option<std::path::PathBuf> = None;
    let mut export_docx = false;
    let mut export_xlsx = false;
    let mut files = Vec::new();

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--model" => { i += 1; model = args[i].clone(); }
            "--base-url" => { i += 1; base_url = args[i].clone(); }
            "--template" => { i += 1; template_id = args[i].clone(); }
            "--output-dir" => { i += 1; out_dir = Some(args[i].clone().into()); }
            "--docx" => export_docx = true,
            "--xlsx" => export_xlsx = true,
            s if s.starts_with('-') => { eprintln!("未知参数: {s}"); std::process::exit(2); }
            _ => files.push(args[i].clone()),
        }
        i += 1;
    }

    let key = match std::env::var("LITEAI_API_KEY") {
        Ok(k) if !k.is_empty() => k,
        _ => {
            eprintln!("错误：未设置 LITEAI_API_KEY 环境变量");
            std::process::exit(1);
        }
    };

    let mut files_meta = Vec::new();
    for f in &files {
        match FileMeta::from_path(Path::new(f)) {
            Ok(m) => files_meta.push(m),
            Err(e) => eprintln!("跳过 {f}: {e}"),
        }
    }
    if files_meta.is_empty() {
        eprintln!("没有可分析的文件");
        std::process::exit(1);
    }

    let config_dir = liteai_config::default_config_dir();
    let templates = liteai_config::all_templates(&config_dir);
    let tpl = templates
        .iter()
        .find(|t| t.id == template_id)
        .cloned()
        .unwrap_or_else(|| templates[0].clone());
    println!("模板: {}（{}）", tpl.name, tpl.id);

    let pipeline = AnalysisPipeline {
        parsers: liteai_parsers::default_registry(),
        model: Box::new(liteai_model::OpenAiClient::new(key, base_url.clone())),
        md_serializer: Box::new(liteai_output::MarkdownSerializer),
        docx_serializer: Some(Box::new(liteai_output::DocxSerializer)),
        xlsx_serializer: Some(Box::new(liteai_output::XlsxSerializer)),
        prompt: PromptBuilder::new(tpl.system, tpl.prompt),
    };

    let out_cfg = OutputConfig {
        mode: OutputMode::Both,
        out_dir,
        export_docx,
        export_xlsx,
    };
    let cancel = Arc::new(AtomicBool::new(false));

    let outcome = pipeline
        .analyze_batch(
            files_meta,
            &ModelConfig { base_url, model },
            &out_cfg,
            &mut |ev| {
                let mut out = std::io::stdout();
                match ev {
                    PipelineEvent::Started { total } => {
                        println!("\n===== 开始分析 {total} 个文件 =====")
                    }
                    PipelineEvent::Parsing { file, .. } => println!("▶ 解析 [{file}]"),
                    PipelineEvent::Tokens { text } => {
                        print!("{text}");
                        let _ = out.flush();
                    }
                    PipelineEvent::FileDone { output_path, .. } => {
                        println!("\n✔ 已保存: {}", output_path.as_deref().unwrap_or("(未保存)"))
                    }
                    PipelineEvent::Error { file, message } => {
                        println!("\n✖ [{}]: {message}", file.as_deref().unwrap_or("?"))
                    }
                    PipelineEvent::Done { summary } => println!("\n{summary}"),
                    PipelineEvent::Cancelled => println!("\n已取消"),
                }
                Ok(())
            },
            Some(&cancel),
        )
        .await;

    match outcome {
        Ok(o) => {
            println!("\n成功 {} 个文件，取消: {}", o.succeeded(), o.cancelled);
        }
        Err(e) => {
            eprintln!("\n分析失败: {e}");
            std::process::exit(1);
        }
    }
}
