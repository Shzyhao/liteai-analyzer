// 配置与模板状态。

import { create } from 'zustand';
import { getConfig, saveConfig as apiSave, setApiKey, deleteApiKey, getTemplates, saveTemplates as apiSaveTemplates, hasApiKey } from '../api/tauri';
import type { AppConfig, Template } from '../types/shared';

interface ConfigState {
  config: AppConfig | null;
  templates: Template[];
  keyConfigured: boolean;
  loading: boolean;
  load: () => Promise<void>;
  update: (patch: Partial<AppConfig>) => Promise<void>;
  saveKey: (key: string) => Promise<void>;
  clearKey: () => Promise<void>;
  saveTemplates: (templates: Template[]) => Promise<void>;
}

export const useConfig = create<ConfigState>((set, get) => ({
  config: null,
  templates: [],
  keyConfigured: false,
  loading: false,

  load: async () => {
    set({ loading: true });
    const [config, templates, keyConfigured] = await Promise.all([
      getConfig().catch(() => null),
      getTemplates().catch(() => []),
      hasApiKey().catch(() => false),
    ]);
    set({ config, templates, keyConfigured, loading: false });
  },

  update: async (patch) => {
    const cur = get().config;
    if (!cur) return;
    const next = { ...cur, ...patch, api: patch.api ?? cur.api, prefs: patch.prefs ?? cur.prefs };
    set({ config: next });
    await apiSave(next).catch(() => {});
  },

  saveKey: async (key) => {
    await setApiKey(key);
    set({ keyConfigured: true });
  },

  clearKey: async () => {
    await deleteApiKey();
    set({ keyConfigured: false });
  },

  saveTemplates: async (templates) => {
    await apiSaveTemplates(templates);
    set({ templates });
  },
}));
