// Tauri invoke 封装：camelCase 参数自动映射到 Rust snake_case。

import { invoke, Channel } from '@tauri-apps/api/core';
import type { AppConfig, PipelineEvent, Template } from '../types/shared';

export async function analyzeFiles(paths: string[], onMessage: (m: PipelineEvent) => void) {
  const ch = new Channel<PipelineEvent>();
  ch.onmessage = onMessage;
  await invoke('analyze_files', { paths, onEvent: ch });
}

export const cancelAll = () => invoke('cancel_all');
export const getPending = (): Promise<string[]> => invoke('get_pending');
export const getConfig = (): Promise<AppConfig> => invoke('get_config');
export const saveConfig = (config: AppConfig) => invoke('save_config', { config });
export const setApiKey = (key: string) => invoke('set_api_key', { key });
export const deleteApiKey = () => invoke('delete_api_key');
export const hasApiKey = (): Promise<boolean> => invoke('has_api_key');
export const testConnection = (baseUrl: string, model: string) =>
  invoke('test_connection', { baseUrl, model });
export const getTemplates = (): Promise<Template[]> => invoke('get_templates');
export const saveTemplates = (templates: Template[]) => invoke('save_templates', { templates });
export const importTemplates = (json: string): Promise<Template[]> => invoke('import_templates', { json });
export const registerShellMenu = () => invoke('register_shell_menu');
export const unregisterShellMenu = () => invoke('unregister_shell_menu');
export const shellMenuRegistered = (): Promise<boolean> => invoke('shell_menu_registered');
