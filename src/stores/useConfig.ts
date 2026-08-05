// 配置与模板状态（含多套 API 配置管理）。

import { create } from 'zustand';
import { getConfig, saveConfig as apiSave, setApiKey, deleteApiKey, getTemplates, saveTemplates as apiSaveTemplates, hasApiKey } from '../api/tauri';
import type { AppConfig, ApiProfile, Template } from '../types/shared';

interface ConfigState {
  config: AppConfig | null;
  templates: Template[];
  keyConfigured: boolean;
  loading: boolean;
  load: () => Promise<void>;
  update: (patch: Partial<AppConfig>) => Promise<void>;
  setActiveProfile: (id: string) => Promise<void>;
  saveKey: (profileId: string, key: string) => Promise<void>;
  clearKey: (profileId: string) => Promise<void>;
  saveTemplates: (templates: Template[]) => Promise<void>;
}

function genId(): string {
  return 'p' + Math.random().toString(36).slice(2, 10) + Date.now().toString(36);
}

export const useConfig = create<ConfigState>((set, get) => ({
  config: null,
  templates: [],
  keyConfigured: false,
  loading: false,

  load: async () => {
    set({ loading: true });
    const config = await getConfig().catch(() => null);
    const templates = await getTemplates().catch(() => []);
    let keyConfigured = false;
    if (config) {
      const active = config.api_profiles.find((p) => p.id === config.active_profile_id);
      if (active) keyConfigured = await hasApiKey(active.id).catch(() => false);
    }
    set({ config, templates, keyConfigured, loading: false });
  },

  update: async (patch) => {
    const cur = get().config;
    if (!cur) return;
    const next = { ...cur, ...patch, prefs: patch.prefs ?? cur.prefs, api_profiles: patch.api_profiles ?? cur.api_profiles };
    set({ config: next });
    await apiSave(next).catch(() => {});
  },

  setActiveProfile: async (id) => {
    const cur = get().config;
    if (!cur) return;
    const next = { ...cur, active_profile_id: id };
    set({ config: next });
    await apiSave(next).catch(() => {});
    // 刷新当前配置的 Key 状态
    const active = next.api_profiles.find((p) => p.id === id);
    const keyConfigured = active ? await hasApiKey(active.id).catch(() => false) : false;
    set({ keyConfigured });
  },

  saveKey: async (profileId, key) => {
    await setApiKey(profileId, key);
    if (get().config?.active_profile_id === profileId) set({ keyConfigured: true });
  },

  clearKey: async (profileId) => {
    await deleteApiKey(profileId);
    if (get().config?.active_profile_id === profileId) set({ keyConfigured: false });
  },

  saveTemplates: async (templates) => {
    await apiSaveTemplates(templates);
    set({ templates });
  },
}));

export const profileHelpers = {
  addProfile(config: AppConfig, profile: Omit<ApiProfile, 'id'>): AppConfig {
    return { ...config, api_profiles: [...config.api_profiles, { ...profile, id: genId() }] };
  },
  removeProfile(config: AppConfig, id: string): AppConfig {
    const profiles = config.api_profiles.filter((p) => p.id !== id);
    return {
      ...config,
      api_profiles: profiles,
      active_profile_id: config.active_profile_id === id ? profiles[0]?.id ?? '' : config.active_profile_id,
    };
  },
  updateProfile(config: AppConfig, id: string, patch: Partial<ApiProfile>): AppConfig {
    return { ...config, api_profiles: config.api_profiles.map((p) => (p.id === id ? { ...p, ...patch } : p)) };
  },
};
