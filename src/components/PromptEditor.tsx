// 模板编辑器：内置模板（只读）+ 自定义模板 + JSON 导入导出。

import { useState } from 'react';
import { useConfig } from '../stores/useConfig';
import { importTemplates } from '../api/tauri';
import type { Template } from '../types/shared';

export default function PromptEditor() {
  const { templates, saveTemplates, load } = useConfig();
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [importText, setImportText] = useState('');

  const selected = templates.find((t) => t.id === selectedId) ?? null;

  const upsert = (patch: Partial<Template>) => {
    if (!selected) return;
    saveTemplates(templates.map((t) => (t.id === selected.id ? { ...t, ...patch } : t)));
  };

  const addCustom = () => {
    const t: Template = {
      id: `custom_${Date.now().toString(36)}`,
      name: '新模板',
      system: '你是一个专业的文件分析助手。',
      prompt: '请分析以下文件：\n\n文件名：{filename}\n\n文件内容：\n{content}',
      builtin: false,
    };
    saveTemplates([...templates, t]);
    setSelectedId(t.id);
  };

  const remove = () => {
    if (!selected || selected.builtin) return;
    saveTemplates(templates.filter((t) => t.id !== selected.id));
    setSelectedId(null);
  };

  const onImport = async () => {
    try {
      const imported = await importTemplates(importText);
      saveTemplates([...templates, ...imported]);
      setImportText('');
      load();
    } catch (e) {
      alert(`导入失败：${e}`);
    }
  };

  const onExport = () => {
    const blob = new Blob([JSON.stringify(templates.filter((t) => !t.builtin), null, 2)], {
      type: 'application/json',
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'liteai-templates.json';
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="panel editor-layout">
      <div className="template-list">
        <h2>模板库</h2>
        {templates.map((t) => (
          <div
            key={t.id}
            className={`template-item ${t.id === selectedId ? 'active' : ''}`}
            onClick={() => setSelectedId(t.id)}
          >
            <span>{t.name}</span>
            {t.builtin ? <span className="badge builtin">内置</span> : <span className="badge custom">自定义</span>}
          </div>
        ))}
        <button className="btn small" onClick={addCustom}>
          + 新建自定义模板
        </button>
      </div>

      <div className="template-editor">
        {selected ? (
          <>
            <div className="form-row">
              <label>名称</label>
              <input
                value={selected.name}
                disabled={selected.builtin}
                onChange={(e) => upsert({ name: e.target.value })}
              />
            </div>
            <div className="form-row">
              <label>System 指令</label>
              <textarea
                value={selected.system}
                disabled={selected.builtin}
                onChange={(e) => upsert({ system: e.target.value })}
              />
            </div>
            <div className="form-row">
              <label>Prompt（可用变量 {`{filename} {path} {content}`}）</label>
              <textarea
                rows={8}
                value={selected.prompt}
                disabled={selected.builtin}
                onChange={(e) => upsert({ prompt: e.target.value })}
              />
            </div>
            {!selected.builtin && (
              <button className="btn danger small" onClick={remove}>
                删除此模板
              </button>
            )}
          </>
        ) : (
          <div className="empty">选择左侧模板进行编辑</div>
        )}
      </div>

      <div className="template-io">
        <h2>导入 / 导出</h2>
        <textarea
          rows={5}
          placeholder={'粘贴 JSON 模板包…'}
          value={importText}
          onChange={(e) => setImportText(e.target.value)}
        />
        <div className="form-actions">
          <button className="btn small" onClick={onImport}>
            导入 JSON
          </button>
          <button className="btn small" onClick={onExport}>
            导出 JSON
          </button>
        </div>
      </div>
    </div>
  );
}
