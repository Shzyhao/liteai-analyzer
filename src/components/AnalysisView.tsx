// 主分析视图：待分析队列 + 流式结果 + 拖拽添加 + 首次引导。

import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { useQueue, statusLabel } from '../stores/useQueue';
import { useConfig } from '../stores/useConfig';
import { openPath } from '../api/tauri';
import MarkdownRenderer from './MarkdownRenderer';
import SkillExportButton from './SkillExportButton';

function basename(p: string): string {
  return p.split(/[\\/]/).pop() ?? p;
}

function parentOf(p: string): string {
  return p.replace(/[\\/][^\\/]*$/, '') || p;
}

function fmtTime(ms: number): string {
  return new Date(ms).toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
}

export default function AnalysisView() {
  const { pending, items, running, summary, activeIndex, lastPaths, loadPending, addPaths, startAnalysis, retryItem, cancel } = useQueue();
  const keyConfigured = useConfig((s) => s.keyConfigured);
  const [dragOver, setDragOver] = useState(false);

  useEffect(() => {
    loadPending();
    const un = listen('queue-updated', () => loadPending());
    // 拖拽文件进窗口
    const unDrop = getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === 'over') setDragOver(true);
      else if (event.payload.type === 'drop') {
        setDragOver(false);
        addPaths(event.payload.paths);
      } else if (event.payload.type === 'leave') setDragOver(false);
    });
    return () => {
      un.then((f) => f());
      unDrop.then((f) => f());
    };
  }, [loadPending, addPaths]);

  const hasContent = pending.length > 0 || items.length > 0 || running;
  const showGuide = !keyConfigured && !hasContent;
  const showEmpty = keyConfigured && !hasContent;

  return (
    <div className={`analysis-view${dragOver ? ' drag-over' : ''}`}>
      {dragOver && <div className="drag-hint">松开鼠标添加文件分析</div>}

      {showGuide && (
        <div className="guide-card">
          <h3>👋 欢迎使用轻析 AI 文件分析助手</h3>
          <ol>
            <li>到「设置」填入 API Key（DeepSeek 等）并测试连接</li>
            <li>到「右键菜单」安装「AI 分析」菜单项</li>
            <li>在资源管理器选中文件 → 右键 → AI 分析；<b>也可以直接把文件拖进这个窗口</b></li>
          </ol>
        </div>
      )}

      {showEmpty && (
        <div className="empty-state">
          <div className="empty-icon">📄</div>
          <div className="empty-title">暂无分析内容</div>
          <div className="empty-sub">在资源管理器选中文件 → 右键「AI 分析」，<br />或直接把文件拖进这个窗口开始分析</div>
        </div>
      )}

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
                {item.usage && item.status === 'done' && (
                  <span className="usage-hint" title="本次分析 Token 消耗">
                    ↑{item.usage.prompt_tokens} / ↓{item.usage.completion_tokens} tokens
                  </span>
                )}
                {item.outputPath && <span className="output-path">→ {item.outputPath}</span>}
              </div>
              {item.error && <div className="error-box">{item.error}</div>}
              {item.text ? (
                <MarkdownRenderer content={item.text} />
              ) : (
                item.status === 'done' && <div className="empty">（无输出）</div>
              )}
              {(item.status === 'done' || item.status === 'error' || item.status === 'cancelled') && item.text && (
                <div className="card-actions">
                  <button className="btn small" onClick={() => navigator.clipboard.writeText(item.text)}>
                    复制结果
                  </button>
                  <button className="btn small" onClick={() => openPath(parentOf(item.path))}>
                    打开文件夹
                  </button>
                  {item.outputPath && (
                    <button className="btn small" onClick={() => openPath(item.outputPath!)}>
                      打开结果
                    </button>
                  )}
                  {(item.status === 'error' || item.status === 'cancelled') && (
                    <button className="btn small" onClick={() => retryItem(item)}>
                      重新分析
                    </button>
                  )}
                  {item.status === 'done' && <SkillExportButton sourceFile={item.path} analysisText={item.text} />}
                </div>
              )}
              {item.status === 'done' && (
                <div className="meta-line">
                  {item.outputPath ? `保存于 ${fmtTime(Date.now())}` : '仅 UI 显示，未保存文件'}
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
