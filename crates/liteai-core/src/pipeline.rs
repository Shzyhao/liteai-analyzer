//! 分析编排管线：GUI 与 CLI 共用的唯一入口。

use crate::domain::*;
use crate::prompt::PromptBuilder;
use crate::registry::ParserRegistry;
use crate::traits::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// 模型连接配置。
#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub base_url: String,
    pub model: String,
}

/// 输出配置：模式（仅UI/仅文件/双开）+ 输出目录（None = 源文件旁）。
#[derive(Debug, Clone, Default)]
pub struct OutputConfig {
    pub mode: OutputMode,
    pub out_dir: Option<PathBuf>,
    pub export_docx: bool,
}

pub struct AnalysisPipeline {
    pub parsers: ParserRegistry,
    pub model: Box<dyn ModelClient>,
    pub md_serializer: Box<dyn Serializer>,
    pub docx_serializer: Option<Box<dyn Serializer>>,
    pub prompt: PromptBuilder,
}

impl AnalysisPipeline {
    /// 批量分析。
    ///
    /// - `on_event` 返回 `Err(())` 表示请求取消（Tauri 里 Channel 关闭/前端取消）。要求 `Send`。
    /// - `cancel` 为可选取消标志（`cancel_all` 命令置位后，下一文件边界停止）。
    pub async fn analyze_batch(
        &self,
        files: Vec<FileMeta>,
        model_cfg: &ModelConfig,
        out_cfg: &OutputConfig,
        on_event: &mut (dyn FnMut(PipelineEvent) -> Result<(), ()> + Send),
        cancel: Option<&AtomicBool>,
    ) -> Result<BatchOutcome, PipelineError> {
        if files.is_empty() {
            return Err(PipelineError::NoFiles);
        }
        let is_cancelled = |c: Option<&AtomicBool>| c.map(|x| x.load(Ordering::Relaxed)).unwrap_or(false);

        let total = files.len();
        let mut failed: usize = 0;
        let mut outcome = BatchOutcome::default();
        on_event(PipelineEvent::Started { total }).map_err(|_| PipelineError::Cancelled)?;

        for (index, file) in files.into_iter().enumerate() {
            if is_cancelled(cancel) {
                on_event(PipelineEvent::Cancelled).ok();
                outcome.cancelled = true;
                break;
            }
            let mut emit = |ev: PipelineEvent| on_event(ev).map_err(|_| PipelineError::Cancelled);

            emit(PipelineEvent::Parsing { index, file: file.file_name.clone() })?;

            // 1) 解析
            let doc = match self.parsers.get(&file.path) {
                Some(parser) => match parser.parse(&file.path) {
                    Ok(d) => d,
                    Err(e) => {
                        failed += 1;
                        emit(PipelineEvent::Error { file: Some(file.file_name.clone()), message: e.to_string() })?;
                        continue;
                    }
                },
                None => {
                    failed += 1;
                    emit(PipelineEvent::Error { file: Some(file.file_name.clone()), message: "不支持的文件类型".into() })?;
                    continue;
                }
            };

            // 2) 组装请求并流式调用
            let req = self.prompt.build_request(&model_cfg.base_url, &model_cfg.model, &doc);
            let mut full = String::new();
            let usage = match self
                .model
                .stream_chat(&req, &mut |tok: String| {
                    full.push_str(&tok);
                    emit(PipelineEvent::Tokens { text: tok }).map_err(|_| ModelError::Cancelled)
                })
                .await
            {
                Ok(u) => Some(u),
                Err(e) => {
                    failed += 1;
                    emit(PipelineEvent::Error { file: Some(file.file_name.clone()), message: e.to_string() })?;
                    continue;
                }
            };

            // 3) 落盘（仅文件 / 双开模式）
            let out_doc = OutputDocument {
                source_name: file.file_name.clone(),
                source_text: doc.text.clone(),
                truncated: doc.truncated,
                analysis: full.clone(),
            };
            let output_path = self.write_output(&out_doc, &file, out_cfg, &mut emit)?;

            outcome.results.push(FileResult {
                index,
                file,
                analysis: full,
                usage,
                output_path: output_path.clone(),
            });
            emit(PipelineEvent::FileDone { index, output_path: output_path.map(|p| p.display().to_string()) })?;
        }

        let summary = if outcome.cancelled {
            format!("已取消：成功 {} 个，失败 {} 个（共 {total} 个）", outcome.succeeded(), failed)
        } else {
            format!("分析完成：成功 {} 个，失败 {} 个（共 {total} 个）", outcome.succeeded(), failed)
        };
        on_event(PipelineEvent::Done { summary }).map_err(|_| PipelineError::Cancelled)?;
        Ok(outcome)
    }

    fn write_output(
        &self,
        doc: &OutputDocument,
        file: &FileMeta,
        out_cfg: &OutputConfig,
        emit: &mut dyn FnMut(PipelineEvent) -> Result<(), PipelineError>,
    ) -> Result<Option<PathBuf>, PipelineError> {
        if out_cfg.mode == OutputMode::UiOnly {
            return Ok(None);
        }
        let dir = out_cfg
            .out_dir
            .clone()
            .unwrap_or_else(|| file.path.parent().unwrap_or(Path::new(".")).to_path_buf());

        // 主序列化器（Markdown）
        let md_path = match self.md_serializer.serialize(doc, &dir) {
            Ok(p) => p,
            Err(e) => {
                emit(PipelineEvent::Error { file: Some(doc.source_name.clone()), message: format!("保存文件失败: {e}") })?;
                return Ok(None);
            }
        };
        // 可选额外导出 docx
        if out_cfg.export_docx {
            if let Some(docx) = &self.docx_serializer {
                if let Err(e) = docx.serialize(doc, &dir) {
                    emit(PipelineEvent::Error { file: Some(doc.source_name.clone()), message: format!("导出 Word 失败: {e}") })?;
                }
            }
        }
        Ok(Some(md_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::OutputMode;
    use crate::traits::{Parser, Serializer};
    use async_trait::async_trait;
    use std::sync::Arc;

    struct FakeParser;
    impl Parser for FakeParser {
        fn extensions(&self) -> &'static [&'static str] {
            &["txt"]
        }
        fn parse(&self, path: &Path) -> Result<ExtractedDocument, ParseError> {
            Ok(ExtractedDocument::new(
                FileMeta::from_path(path).unwrap(),
                "这是测试文件内容。".into(),
                "text/plain",
            ))
        }
    }

    struct FakeModel;
    #[async_trait]
    impl ModelClient for FakeModel {
        async fn stream_chat(
            &self,
            _req: &ChatRequest,
            on_token: &mut (dyn FnMut(String) -> Result<(), ModelError> + Send),
        ) -> Result<ChatUsage, ModelError> {
            for part in ["# 摘要\n", "第一条要点。\n", "第二条要点。\n"] {
                on_token(part.to_string())?;
            }
            Ok(ChatUsage { prompt_tokens: 10, completion_tokens: 5 })
        }
        async fn check_balance(&self) -> Result<BalanceInfo, ModelError> {
            Ok(BalanceInfo { is_available: true, balance_infos: vec![] })
        }
        async fn ping(&self, _b: &str, _m: &str) -> Result<(), ModelError> {
            Ok(())
        }
    }

    struct MdSerializer;
    impl Serializer for MdSerializer {
        fn extension(&self) -> &'static str {
            "md"
        }
        fn serialize(&self, doc: &OutputDocument, out_dir: &Path) -> Result<PathBuf, OutputError> {
            let name = format!("{}.ai.md", doc.source_name);
            std::fs::write(out_dir.join(&name), &doc.analysis)?;
            Ok(out_dir.join(name))
        }
    }

    fn pipeline() -> AnalysisPipeline {
        let mut parsers = ParserRegistry::new();
        parsers.register(Arc::new(FakeParser));
        AnalysisPipeline {
            parsers,
            model: Box::new(FakeModel),
            md_serializer: Box::new(MdSerializer),
            docx_serializer: None,
            prompt: PromptBuilder::default(),
        }
    }

    #[tokio::test]
    async fn batch_runs_and_emits_events_in_order() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("a.txt");
        std::fs::write(&f, "hi").unwrap();
        let files = vec![FileMeta::from_path(&f).unwrap()];

        let mut events = vec![];
        let outcome = pipeline()
            .analyze_batch(
                files,
                &ModelConfig { base_url: "https://api.deepseek.com".into(), model: "deepseek-chat".into() },
                &OutputConfig { mode: OutputMode::Both, out_dir: Some(dir.path().to_path_buf()), export_docx: false },
                &mut |ev| {
                    events.push(ev);
                    Ok(())
                },
                None,
            )
            .await
            .unwrap();

        assert_eq!(outcome.succeeded(), 1);
        assert_eq!(outcome.results[0].analysis, "# 摘要\n第一条要点。\n第二条要点。\n");
        assert!(dir.path().join("a.txt.ai.md").exists());
        assert!(events.iter().any(|e| matches!(e, PipelineEvent::Started { .. })));
        assert!(events.iter().any(|e| matches!(e, PipelineEvent::Done { .. })));
        // 流式 token 顺序
        let tokens: String = events
            .iter()
            .filter_map(|e| match e {
                PipelineEvent::Tokens { text } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(tokens, "# 摘要\n第一条要点。\n第二条要点。\n");
    }

    #[tokio::test]
    async fn empty_files_errors() {
        let err = pipeline()
            .analyze_batch(
                vec![],
                &ModelConfig { base_url: "x".into(), model: "m".into() },
                &OutputConfig::default(),
                &mut |_| Ok(()),
                None,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, PipelineError::NoFiles));
    }

    #[tokio::test]
    async fn cancel_at_boundary_stops() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("b.txt");
        std::fs::write(&f, "hi").unwrap();
        let files = vec![FileMeta::from_path(&f).unwrap(), FileMeta::from_path(&f).unwrap()];
        let flag = AtomicBool::new(true); // 立即取消
        let mut cancelled_seen = false;
        let outcome = pipeline()
            .analyze_batch(
                files,
                &ModelConfig { base_url: "x".into(), model: "m".into() },
                &OutputConfig::default(),
                &mut |ev| {
                    if matches!(ev, PipelineEvent::Cancelled) {
                        cancelled_seen = true;
                    }
                    Ok(())
                },
                Some(&flag),
            )
            .await
            .unwrap();
        assert!(outcome.cancelled);
        assert!(cancelled_seen);
    }
}
