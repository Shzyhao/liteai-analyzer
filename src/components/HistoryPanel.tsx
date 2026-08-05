// 分析历史：查看 / 重新分析 / 删除。

import { useEffect, useState } from 'react';
import { getHistory, deleteHistoryEntry, clearHistory, openPath } from '../api/tauri';
import { useQueue } from '../stores/useQueue';
import type { HistoryEntry } from '../types/shared';
import MarkdownRenderer from './MarkdownRenderer';

function fmtTime(ms: number): string {
  return new Date(ms).toLocaleString('zh-CN', {
    month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit',
  });
}

function basename(p: string): string {
  return p.split(/[\\/]/).pop() ?? p;
}

export default function HistoryPanel() {
  const [entries, setEntries] = useState<HistoryEntry[]>([]);
  const [expanded, setExpanded] = useState<string | null>(null);
  const startAnalysis = useQueue((s) => s.startAnalysis);

  const load = () => getHistory().then(setEntries).catch(() => setEntries([]));
  useEffect(() => {
    load();
  }, []);

  const onDelete = async (id: string) => {
    await deleteHistoryEntry(id);
    load();
  };

  const onClear = async () => {
    if (!window.confirm('确定清空全部历史记录吗？此操作不可恢复。')) return;
    await clearHistory();
    load();
  };

  if (entries.length === 0) {
    return (
      <div className="empty-state">
        <div className="empty-icon">🗂️</div>
        <div className="empty-title">暂无历史记录</div>
        <div className="empty-sub">完成一次分析后会自动保存到这里，方便随时回看和重新分析</div>
      </div>
    );
  }

  return (
    <div className="panel">
      <div className="pending-header">
        <h2 style={{ margin: 0 }}>历史记录（{entries.length} 条）</h2>
        <button className="btn danger small" onClick={onClear}>清空历史</button>
      </div>

      {entries.map((e) => (
        <div key={e.id} className="history-item">
          <div className="history-head" onClick={() => setExpanded(expanded === e.id ? null : e.id)}>
            <span className="history-name">{basename(e.source_file)}</span>
            <span className="history-meta">
              {fmtTime(e.timestamp_ms)} · {e.template}
              {e.prompt_tokens > 0 && ` · ↑${e.prompt_tokens}/↓${e.completion_tokens}t`}
            </span>
            <span className="history-actions">
              <button className="btn small" onClick={(ev) => { ev.stopPropagation(); startAnalysis([e.source_file]); }}>
                重新分析
              </button>
              <button className="btn small" onClick={(ev) => { ev.stopPropagation(); openPath(e.source_file); }}>
                定位文件
              </button>
              <button className="btn small danger" onClick={(ev) => { ev.stopPropagation(); onDelete(e.id); }}>
                删除
              </button>
            </span>
          </div>
          {expanded === e.id && (
            <div className="history-body">
              <MarkdownRenderer content={e.analysis} />
              {e.output_files.length > 0 && (
                <div className="history-files">
                  {e.output_files.map((f, i) => (
                    <button key={i} className="btn small" onClick={() => openPath(f)}>
                      打开结果文件 {i + 1}
                    </button>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
