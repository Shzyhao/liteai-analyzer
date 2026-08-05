// 导出技能状态。

import { create } from 'zustand';
import { getSkills, saveSkills as apiSave } from '../api/tauri';
import type { ExportSkill } from '../types/shared';

interface SkillsState {
  skills: ExportSkill[];
  load: () => Promise<void>;
  add: (s: ExportSkill) => Promise<void>;
  update: (s: ExportSkill) => Promise<void>;
  remove: (id: string) => Promise<void>;
}

export const useSkills = create<SkillsState>((set, get) => ({
  skills: [],

  load: async () => {
    const skills = await getSkills().catch(() => []);
    set({ skills });
  },

  add: async (skill) => {
    await apiSave([...get().skills, skill]);
    set({ skills: [...get().skills, skill] });
  },

  update: async (skill) => {
    await apiSave(get().skills.map((s) => (s.id === skill.id ? skill : s)));
    set({ skills: get().skills.map((s) => (s.id === skill.id ? skill : s)) });
  },

  remove: async (id) => {
    await apiSave(get().skills.filter((s) => s.id !== id));
    set({ skills: get().skills.filter((s) => s.id !== id) });
  },
}));
