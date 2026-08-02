// 批量分析队列状态：pending 文件 + 流式结果累积。

import { create } from 'zustand';
import { analyzeFiles, cancelAll, getPending } from '../api/tauri';
import type { PipelineEvent } from '../types/shared';

export interface QueueItem {
  index: number;
  name: string;
  status: 'queued' | 'parsing' | 'streaming' | 'done' | 'error';
  text: string;
  outputPath?: string;
  error?: string;
}

interface QueueState {
  pending: string[];
  items: QueueItem[];
  activeIndex: number | null;
  running: boolean;
  summary: string;
  loadPending: () => Promise<void>;
  startAnalysis: (paths?: string[]) => Promise<void>;
  cancel: () => void;
  handleEvent: (ev: PipelineEvent) => void;
}

function basename(p: string): string {
  return p.split(/[\\/]/).pop() ?? p;
}

export const useQueue = create<QueueState>((set, get) => ({
  pending: [],
  items: [],
  activeIndex: null,
  running: false,
  summary: '',

  loadPending: async () => {
    const pending = await getPending().catch(() => []);
    set({ pending });
  },

  startAnalysis: async (paths) => {
    const pathsToRun = paths ?? get().pending;
    if (pathsToRun.length === 0 || get().running) return;
    set((s) => ({
      running: true,
      activeIndex: null,
      summary: '',
      items: pathsToRun.map((p, i) => ({ index: i, name: basename(p), status: 'queued', text: '' })),
      pending: s.pending.filter((p) => !pathsToRun.includes(p)),
    }));
    try {
      await analyzeFiles(pathsToRun, (ev) => get().handleEvent(ev));
    } catch (e) {
      set((s) => ({ summary: String(e), running: false, items: s.items.map((it) => it.status === 'queued' ? { ...it, status: 'error', error: String(e) } : it) }));
    }
    set({ running: false });
  },

  cancel: () => cancelAll().catch(() => {}),

  handleEvent: (ev) => {
    switch (ev.type) {
      case 'Parsing':
        set({ activeIndex: ev.data.index });
        set((s) => ({
          items: s.items.map((it) => (it.index === ev.data.index ? { ...it, status: 'parsing' } : it)),
        }));
        break;
      case 'Tokens':
        set((s) => {
          const idx = s.activeIndex;
          if (idx === null) return {};
          return {
            items: s.items.map((it) =>
              it.index === idx ? { ...it, text: it.text + ev.data.text, status: 'streaming' } : it,
            ),
          };
        });
        break;
      case 'FileDone':
        set((s) => ({
          items: s.items.map((it) =>
            it.index === ev.data.index
              ? { ...it, status: 'done', outputPath: ev.data.output_path ?? undefined }
              : it,
          ),
        }));
        break;
      case 'Error':
        set((s) => ({
          items: s.items.map((it) =>
            it.index === s.activeIndex ? { ...it, status: 'error', error: ev.data.message } : it,
          ),
        }));
        break;
      case 'Done':
        set({ summary: ev.data.summary });
        break;
      case 'Cancelled':
        set({ summary: '已取消', running: false });
        break;
      default:
        break;
    }
  },
}));
