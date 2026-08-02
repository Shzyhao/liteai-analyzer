// 设置面板：API 配置 / 输出方式 / 文件白名单 / 右键菜单。

import { useEffect, useState } from 'react';
import { useConfig } from '../stores/useConfig';
import { testConnection } from '../api/tauri';
import type { OutputMode } from '../types/shared';

const modeLabels: Record<OutputMode, string> = {
  ui_only: '仅 UI 显示',
  file_only: '仅保存文件',
  both: 'UI + 保存文件（双开）',
};

export default function SettingsPanel() {
  const { config, keyConfigured, load, update, saveKey, clearKey } = useConfig();
  const [apiKey, setApiKey] = useState('');
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);
  const [cleared, setCleared] = useState(false);

  useEffect(() => {
    load();
  }, [load]);

  if (!config) return <div className="panel">加载中…</div>;

  const onTest = async () => {
    setTesting(true);
    setTestResult(null);
    try {
      const balance: any = await testConnection(config.api.base_url, config.api.model);
      const info = balance?.balance_infos?.[0];
      setTestResult(
        info
          ? `✔ 连接成功，余额 ${info.total_balance} ${info.currency}`
          : '✔ 连接成功',
      );
    } catch (e) {
      setTestResult(`✖ 连接失败：${e}`);
    } finally {
      setTesting(false);
    }
  };

  const onSave = async () => {
    if (apiKey.trim()) {
      await saveKey(apiKey.trim());
      setApiKey('');
    }
    setSaved(true);
    setTimeout(() => setSaved(false), 1500);
  };

  const onClearKey = async () => {
    if (!window.confirm('确定清除已保存的 API Key 吗？此操作不可撤销。')) return;
    await clearKey();
    setCleared(true);
    setTimeout(() => setCleared(false), 2000);
  };

  return (
    <div className="panel">
      <h2>API 配置</h2>
      <div className="form-row">
        <label>Base URL</label>
        <input
          value={config.api.base_url}
          onChange={(e) => update({ api: { ...config.api, base_url: e.target.value } })}
        />
      </div>
      <div className="form-row">
        <label>模型</label>
        <input
          value={config.api.model}
          onChange={(e) => update({ api: { ...config.api, model: e.target.value } })}
        />
      </div>
      <div className="form-row">
        <label>API Key {keyConfigured && <span className="hint">（已配置）</span>}</label>
        <input
          type="password"
          placeholder={keyConfigured ? '已保存，输入以替换' : 'sk-...'}
          value={apiKey}
          onChange={(e) => setApiKey(e.target.value)}
        />
      </div>
      <div className="form-actions">
        <button className="btn" onClick={onTest} disabled={testing || (!apiKey && !keyConfigured)}>
          {testing ? '测试中…' : '测试连接 & 查余额'}
        </button>
        <button className="btn primary" onClick={onSave}>
          保存设置
        </button>
        {keyConfigured && (
          <button className="btn danger" onClick={onClearKey}>
            清除 API Key
          </button>
        )}
        {saved && <span className="hint">已保存 ✓</span>}
        {cleared && <span className="hint">已清除 ✓</span>}
      </div>
      {testResult && <div className="test-result">{testResult}</div>}

      <h2>输出方式</h2>
      <div className="form-row">
        {(['ui_only', 'file_only', 'both'] as OutputMode[]).map((m) => (
          <label key={m} className="radio">
            <input
              type="radio"
              checked={config.prefs.output_mode === m}
              onChange={() => update({ prefs: { ...config.prefs, output_mode: m } })}
            />
            {modeLabels[m]}
          </label>
        ))}
      </div>
      <div className="form-row">
        <label className="checkbox">
          <input
            type="checkbox"
            checked={config.prefs.export_docx}
            onChange={(e) => update({ prefs: { ...config.prefs, export_docx: e.target.checked } })}
          />
          额外导出 Word (.docx)
        </label>
      </div>

      <h2>文件类型白名单</h2>
      <div className="form-row">
        <input
          value={config.prefs.whitelist.join(', ')}
          onChange={(e) =>
            update({
              prefs: {
                ...config.prefs,
                whitelist: e.target.value.split(',').map((s) => s.trim()).filter(Boolean),
              },
            })
          }
        />
      </div>
      <div className="hint">逗号分隔后缀，留空表示允许全部</div>
    </div>
  );
}
