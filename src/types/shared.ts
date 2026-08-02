// 与 Rust 端 serde 序列化对齐的类型定义。

export type OutputMode = 'ui_only' | 'file_only' | 'both';

export interface ApiConfig {
  base_url: string;
  model: string;
}

export interface Prefs {
  output_mode: OutputMode;
  export_docx: boolean;
  output_dir: string | null;
  whitelist: string[];
  incognito: boolean;
}

export interface AppConfig {
  api: ApiConfig;
  prefs: Prefs;
  active_template_id: string;
}

export interface Template {
  id: string;
  name: string;
  system: string;
  prompt: string;
  builtin: boolean;
}

// PipelineEvent（Rust #[serde(tag="type", content="data")]）
export type PipelineEvent =
  | { type: 'Started'; data: { total: number } }
  | { type: 'Parsing'; data: { index: number; file: string } }
  | { type: 'Tokens'; data: { text: string } }
  | { type: 'FileDone'; data: { index: number; output_path: string | null } }
  | { type: 'Done'; data: { summary: string } }
  | { type: 'Error'; data: { file: string | null; message: string } }
  | { type: 'Cancelled' };
