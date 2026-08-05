import { useEffect, useState } from 'react';
import AnalysisView from './components/AnalysisView';
import SettingsPanel from './components/SettingsPanel';
import PromptEditor from './components/PromptEditor';
import ShellMenuSettings from './components/ShellMenuSettings';
import SkillsPanel from './components/SkillsPanel';
import HistoryPanel from './components/HistoryPanel';
import { useConfig } from './stores/useConfig';
import type { Theme } from './types/shared';
import './App.css';

type Tab = 'analyze' | 'history' | 'settings' | 'prompt' | 'skills' | 'shell';

const tabs: { key: Tab; label: string }[] = [
  { key: 'analyze', label: '分析' },
  { key: 'history', label: '历史' },
  { key: 'settings', label: '设置' },
  { key: 'prompt', label: '模板' },
  { key: 'skills', label: '导出技能' },
  { key: 'shell', label: '右键菜单' },
];

const themeLabel: Record<Theme, string> = { light: '☀', dark: '🌙', system: '🖥' };

function App() {
  const [tab, setTab] = useState<Tab>('analyze');
  const load = useConfig((s) => s.load);
  const config = useConfig((s) => s.config);
  const keyConfigured = useConfig((s) => s.keyConfigured);

  useEffect(() => {
    load();
  }, [load]);

  // 应用主题
  useEffect(() => {
    const apply = () => {
      const theme = config?.prefs.theme ?? 'system';
      const dark =
        theme === 'dark' || (theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);
      document.documentElement.dataset.theme = dark ? 'dark' : 'light';
    };
    apply();
    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    mq.addEventListener('change', apply);
    return () => mq.removeEventListener('change', apply);
  }, [config?.prefs.theme]);

  const cycleTheme = () => {
    const order: Theme[] = ['system', 'light', 'dark'];
    const cur = config?.prefs.theme ?? 'system';
    const next = order[(order.indexOf(cur) + 1) % order.length];
    if (config) useConfig.getState().update({ prefs: { ...config.prefs, theme: next } });
  };

  return (
    <div className="app">
      <header className="app-header">
        <div className="brand">
          <span className="logo">轻析</span>
          <span className="subtitle">AI 文件分析助手</span>
        </div>
        <nav className="tabs">
          {tabs.map((t) => (
            <button
              key={t.key}
              className={`tab ${tab === t.key ? 'active' : ''}`}
              onClick={() => setTab(t.key)}
            >
              {t.label}
            </button>
          ))}
        </nav>
        <button className="theme-toggle" onClick={cycleTheme} title="切换主题（浅色/深色/跟随系统）">
          {themeLabel[config?.prefs.theme ?? 'system']}
        </button>
        {!keyConfigured && <span className="warn-dot" title="尚未配置 API Key">⚙</span>}
      </header>

      <main className="app-main">
        {tab === 'analyze' && <AnalysisView />}
        {tab === 'history' && <HistoryPanel />}
        {tab === 'settings' && <SettingsPanel />}
        {tab === 'prompt' && <PromptEditor />}
        {tab === 'skills' && <SkillsPanel />}
        {tab === 'shell' && <ShellMenuSettings />}
      </main>
    </div>
  );
}

export default App;
