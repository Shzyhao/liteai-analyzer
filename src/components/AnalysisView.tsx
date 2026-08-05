// 主分析视图：待分析队列 + 流式结果展示。

import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useQueue } from '../stores/useQueue';
import MarkdownRenderer from './MarkdownRenderer';

const statusLabel: Record<string, string> = {
  queued: '排队中',
  parsing: '解析中',
  streaming: '生成中',
  done: '完成',
  error: '失败',
};

function basename(p: string): string {
  return p.split(/[\\/]/).pop() ?? p;
}

export default function AnalysisView() {
  const { pending, items, running, summary, activeIndex, lastPaths, loadPending, startAnalysis, cancel } = useQueue();

  useEffect(() => {
    loadPending();
    const un = listen('queue-updated', () => loadPending());
    return () => {
      un.then((f) => f());
    };
  }, [loadPending]);

  return (
    <div className="analysis-view">
      {pending.length > 0 && !running && (
        <div className="pending-card">
          <div className="pending-header">
            <span>已接收 {pending.length} 个文件</span>
            <button className="btn primary" onClick={() => startAnalysis()} disabled={running}>
              开始分析
            </button>
          </div>
          <ul className="pending-list">
            {pending.map((p, i) => (
              <li key={i}>{basename(p)}</li>
            ))}
          </ul>
        </div>
      )}

      {items.length > 0 && (
        <div className="results">
          <div className="results-toolbar">
            <span>
              {running && activeIndex !== null ? (
                <span className="status streaming">▶ 正在分析「{items[activeIndex]?.name}」…</span>
              ) : (
                <span className="status">{summary || '就绪'}</span>
              )}
            </span>
            <div className="toolbar-actions">
              {running ? (
                <button className="btn danger" onClick={cancel}>
                  取消
                </button>
              ) : (
                <button
                  className="btn"
                  onClick={() => startAnalysis(lastPaths.length ? lastPaths : undefined)}
                  disabled={running}
                >
                  重新分析
                </button>
              )}
            </div>
          </div>

          {items.map((item) => (
            <div key={item.index} className={`result-card status-${item.status}`}>
              <div className="result-card-header">
                <span className="file-name">{item.name}</span>
                <span className={`badge ${item.status}`}>{statusLabel[item.status]}</span>
                {item.outputPath && <span className="output-path">→ {item.outputPath}</span>}
              </div>
              {item.error && <div className="error-box">{item.error}</div>}
              {item.text ? (
                <MarkdownRenderer content={item.text} />
              ) : (
                item.status === 'done' && <div className="empty">（无输出）</div>
              )}
              {item.status === 'done' && (
                <div className="card-actions">
                  <button
                    className="btn small"
                    onClick={() => navigator.clipboard.writeText(item.text)}
                  >
                    复制结果
                  </button>
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
