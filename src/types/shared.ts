// 与 Rust 端 serde 序列化对齐的类型定义。

export type OutputMode = 'ui_only' | 'file_only' | 'both';
export type Theme = 'light' | 'dark' | 'system';

export interface ApiProfile {
  id: string;
  name: string;
  base_url: string;
  model: string;
}

export interface Prefs {
  output_mode: OutputMode;
  export_docx: boolean;
  export_xlsx: boolean;
  output_dir: string | null;
  whitelist: string[];
  incognito: boolean;
  skills_dir: string | null;
  theme: Theme;
}

export interface AppConfig {
  api_profiles: ApiProfile[];
  active_profile_id: string;
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

// 导出技能：用户自定义的外部脚本
export interface ExportSkill {
  id: string;
  name: string;
  command: string;
  args: string;
  cwd: string | null;
}

export interface SkillRunResult {
  success: boolean;
  exit_code: number | null;
  stdout: string;
  stderr: string;
  analysis_file: string;
  output_dir: string;
}

// PipelineEvent（Rust #[serde(tag="type", content="data")]）
export type PipelineEvent =
  | { type: 'Started'; data: { total: number } }
  | { type: 'Parsing'; data: { index: number; file: string } }
  | { type: 'Tokens'; data: { text: string } }
  | {
      type: 'FileDone';
      data: { index: number; output_path: string | null; usage: { prompt_tokens: number; completion_tokens: number } | null };
    }
  | { type: 'Done'; data: { summary: string } }
  | { type: 'Error'; data: { file: string | null; message: string } }
  | { type: 'Cancelled' };

export interface HistoryEntry {
  id: string;
  timestamp_ms: number;
  source_file: string;
  template: string;
  analysis: string;
  output_files: string[];
  prompt_tokens: number;
  completion_tokens: number;
}
