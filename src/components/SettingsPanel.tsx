// 设置面板：多套 API 配置管理 + 主题 + 输出方式 + 技能目录 + 白名单。

import { useEffect, useState } from 'react';
import { useConfig, profileHelpers } from '../stores/useConfig';
import { testConnection, checkPathExists } from '../api/tauri';
import type { ApiProfile, OutputMode } from '../types/shared';

const modeLabels: Record<OutputMode, string> = {
  ui_only: '仅 UI 显示',
  file_only: '仅保存文件',
  both: 'UI + 保存文件（双开）',
};

export default function SettingsPanel() {
  const { config, keyConfigured, load, update, setActiveProfile, saveKey, clearKey } = useConfig();
  const [editingId, setEditingId] = useState<string | null>(null);
  const [apiKey, setApiKey] = useState('');
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);
  const [cleared, setCleared] = useState(false);

  useEffect(() => {
    load().then(() => {
      if (config) setEditingId((cur) => cur ?? config.active_profile_id);
    });
  }, [load]);

  if (!config) return <div className="panel">加载中…</div>;

  const profiles = config.api_profiles;
  const selected = profiles.find((p) => p.id === editingId) ?? profiles.find((p) => p.id === config.active_profile_id) ?? profiles[0];
  const isActive = selected?.id === config.active_profile_id;

  const editSelected = (patch: Partial<ApiProfile>) => {
    if (!selected) return;
    update(profileHelpers.updateProfile(config, selected.id, patch));
  };

  const onTest = async () => {
    if (!selected) return;
    setTesting(true);
    setTestResult(null);
    try {
      const balance: any = await testConnection(selected.id, selected.base_url, selected.model);
      const info = balance?.balance_infos?.[0];
      if (info) setTestResult(`✔ 连接成功，余额 ${info.total_balance} ${info.currency}`);
      else if (balance?.is_available === false) setTestResult('✔ 连接成功（该平台不支持余额查询）');
      else setTestResult('✔ 连接成功');
    } catch (e) {
      setTestResult(`✖ 连接失败：${e}`);
    } finally {
      setTesting(false);
    }
  };

  const onSave = async () => {
    // 自定义技能目录须已存在
    if (config.prefs.skills_dir?.trim()) {
      const ok = await checkPathExists(config.prefs.skills_dir.trim());
      if (!ok) {
        alert(`技能目录不存在：${config.prefs.skills_dir}\n请确认路径正确（留空则使用默认桌面\\liteai-skills，会自动创建）。`);
        return;
      }
    }
    if (selected && apiKey.trim()) {
      await saveKey(selected.id, apiKey.trim());
      setApiKey('');
    }
    setSaved(true);
    setTimeout(() => setSaved(false), 1500);
  };

  const onClearKey = async () => {
    if (!selected) return;
    if (!window.confirm('确定清除该配置已保存的 API Key 吗？此操作不可撤销。')) return;
    await clearKey(selected.id);
    setCleared(true);
    setTimeout(() => setCleared(false), 2000);
  };

  const onAdd = () => {
    const next = profileHelpers.addProfile(config, {
      name: '新配置',
      base_url: 'https://api.deepseek.com',
      model: 'deepseek-chat',
    });
    update(next);
    setEditingId(next.api_profiles[next.api_profiles.length - 1].id);
  };

  const onDelete = async () => {
    if (!selected) return;
    if (!window.confirm(`确定删除配置「${selected.name}」吗？其保存的 Key 也会一并删除。`)) return;
    const next = profileHelpers.removeProfile(config, selected.id);
    await update(next);
    setEditingId(next.active_profile_id || null);
    if (selected.id === config.active_profile_id) {
      // 删除的是当前配置 → 刷新 Key 状态
      load();
    }
  };

  return (
    <div className="panel">
      <h2>API 配置（可多套，切换使用）</h2>
      <div className="profile-list">
        {profiles.map((p) => (
          <div
            key={p.id}
            className={`profile-item ${p.id === editingId ? 'editing' : ''} ${p.id === config.active_profile_id ? 'active' : ''}`}
            onClick={() => setEditingId(p.id)}
          >
            <span className="profile-name">{p.name}</span>
            <span className="profile-model">{p.model}</span>
            {p.id === config.active_profile_id && <span className="badge done">当前使用</span>}
            {p.id !== config.active_profile_id && (
              <button
                className="btn small"
                onClick={(ev) => { ev.stopPropagation(); setActiveProfile(p.id); }}
                title="切换为当前使用的配置"
              >
                设为当前
              </button>
            )}
          </div>
        ))}
        <button className="btn small" onClick={onAdd}>+ 添加配置</button>
      </div>

      {selected && (
        <div className="profile-editor">
          <div className="form-row">
            <label>名称</label>
            <input value={selected.name} onChange={(e) => editSelected({ name: e.target.value })} />
          </div>
          <div className="form-row">
            <label>Base URL</label>
            <input value={selected.base_url} onChange={(e) => editSelected({ base_url: e.target.value })} />
          </div>
          <div className="form-row">
            <label>模型</label>
            <input value={selected.model} onChange={(e) => editSelected({ model: e.target.value })} />
          </div>
          <div className="form-row">
            <label>API Key {isActive && keyConfigured && <span className="hint">（已配置）</span>}</label>
            <input
              type="password"
              placeholder={keyConfigured && isActive ? '已保存，输入以替换' : 'sk-...'}
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
            />
          </div>
          <div className="form-actions">
            <button className="btn" onClick={onTest} disabled={testing}>
              {testing ? '测试中…' : '测试连接 & 查余额'}
            </button>
            <button className="btn primary" onClick={onSave}>保存设置</button>
            {isActive && keyConfigured && (
              <button className="btn danger" onClick={onClearKey}>清除 Key</button>
            )}
            {profiles.length > 1 && (
              <button className="btn danger" onClick={onDelete}>删除此配置</button>
            )}
            {saved && <span className="hint">已保存 ✓</span>}
            {cleared && <span className="hint">已清除 ✓</span>}
          </div>
          {testResult && <div className="test-result">{testResult}</div>}
          <div className="hint">
            {isActive ? '此配置为当前使用，分析将用它。' : '此配置当前未启用；点列表里的「设为当前」切换。'}
          </div>
        </div>
      )}

      <h2>外观主题</h2>
      <div className="form-row">
        {(['system', 'light', 'dark'] as const).map((t) => (
          <label key={t} className="radio">
            <input
              type="radio"
              checked={(config.prefs.theme ?? 'system') === t}
              onChange={() => update({ prefs: { ...config.prefs, theme: t } })}
            />
            {{ system: '跟随系统', light: '浅色', dark: '深色' }[t]}
          </label>
        ))}
      </div>

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
      <div className="form-row">
        <label className="checkbox">
          <input
            type="checkbox"
            checked={config.prefs.export_xlsx}
            onChange={(e) => update({ prefs: { ...config.prefs, export_xlsx: e.target.checked } })}
          />
          额外导出 Excel (.xlsx)
        </label>
      </div>

      <h2>AI 技能目录</h2>
      <div className="form-row">
        <input
          placeholder="留空 = 默认桌面\liteai-skills"
          value={config.prefs.skills_dir ?? ''}
          onChange={(e) =>
            update({ prefs: { ...config.prefs, skills_dir: e.target.value.trim() || null } })
          }
        />
      </div>
      <div className="hint">
        留空则使用「桌面\liteai-skills」（自动创建）；自定义路径必须已存在，否则无法保存/生成。
        每个技能会存到其下的独立子文件夹。
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
