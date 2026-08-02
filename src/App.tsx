import { useEffect, useState } from 'react';
import AnalysisView from './components/AnalysisView';
import SettingsPanel from './components/SettingsPanel';
import PromptEditor from './components/PromptEditor';
import ShellMenuSettings from './components/ShellMenuSettings';
import { useConfig } from './stores/useConfig';
import './App.css';

type Tab = 'analyze' | 'settings' | 'prompt' | 'shell';

const tabs: { key: Tab; label: string }[] = [
  { key: 'analyze', label: '分析' },
  { key: 'settings', label: '设置' },
  { key: 'prompt', label: '模板' },
  { key: 'shell', label: '右键菜单' },
];

function App() {
  const [tab, setTab] = useState<Tab>('analyze');
  const load = useConfig((s) => s.load);
  const keyConfigured = useConfig((s) => s.keyConfigured);

  useEffect(() => {
    load();
  }, [load]);

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
        {!keyConfigured && <span className="warn-dot" title="尚未配置 API Key">⚙</span>}
      </header>

      <main className="app-main">
        {tab === 'analyze' && <AnalysisView />}
        {tab === 'settings' && <SettingsPanel />}
        {tab === 'prompt' && <PromptEditor />}
        {tab === 'shell' && <ShellMenuSettings />}
      </main>
    </div>
  );
}

export default App;
