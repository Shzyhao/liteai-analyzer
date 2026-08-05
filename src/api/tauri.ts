// Tauri invoke 封装：camelCase 参数自动映射到 Rust snake_case。

import { invoke, Channel } from '@tauri-apps/api/core';
import type { AppConfig, ExportSkill, HistoryEntry, PipelineEvent, SkillRunResult, Template } from '../types/shared';

export async function analyzeFiles(paths: string[], onMessage: (m: PipelineEvent) => void) {
  const ch = new Channel<PipelineEvent>();
  ch.onmessage = onMessage;
  await invoke('analyze_files', { paths, onEvent: ch });
}

export const cancelAll = () => invoke('cancel_all');
export const getPending = (): Promise<string[]> => invoke('get_pending');
export const getConfig = (): Promise<AppConfig> => invoke('get_config');
export const saveConfig = (config: AppConfig) => invoke('save_config', { config });
export const setApiKey = (profileId: string, key: string) => invoke('set_api_key', { profileId, key });
export const deleteApiKey = (profileId: string) => invoke('delete_api_key', { profileId });
export const hasApiKey = (profileId: string): Promise<boolean> => invoke('has_api_key', { profileId });
export const testConnection = (profileId: string, baseUrl: string, model: string) =>
  invoke('test_connection', { profileId, baseUrl, model });
export const getTemplates = (): Promise<Template[]> => invoke('get_templates');
export const saveTemplates = (templates: Template[]) => invoke('save_templates', { templates });
export const importTemplates = (json: string): Promise<Template[]> => invoke('import_templates', { json });
export const registerShellMenu = () => invoke('register_shell_menu');
export const unregisterShellMenu = () => invoke('unregister_shell_menu');
export const shellMenuRegistered = (): Promise<boolean> => invoke('shell_menu_registered');

export const getSkills = (): Promise<ExportSkill[]> => invoke('get_skills');
export const saveSkills = (skills: ExportSkill[]) => invoke('save_skills', { skills });
export const runSkill = (skillId: string, sourceFile: string, analysisText: string): Promise<SkillRunResult> =>
  invoke('run_skill', { skillId, sourceFile, analysisText });
export const generateSkill = (description: string): Promise<{ skill: ExportSkill; script_path: string; skills_dir: string }> =>
  invoke('generate_skill', { description });
export const getSkillsDir = (): Promise<string> => invoke('get_skills_dir');
export const checkPathExists = (path: string): Promise<boolean> => invoke('check_path_exists', { path });
export const openPath = (path: string) => invoke('open_path', { path });
export const getHistory = (): Promise<HistoryEntry[]> => invoke('get_history');
export const deleteHistoryEntry = (id: string) => invoke('delete_history_entry', { id });
export const clearHistory = () => invoke('clear_history');
