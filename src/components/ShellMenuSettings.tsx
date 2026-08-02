// 右键菜单安装/卸载。

import { useEffect, useState } from 'react';
import { registerShellMenu, unregisterShellMenu, shellMenuRegistered } from '../api/tauri';

export default function ShellMenuSettings() {
  const [registered, setRegistered] = useState<boolean | null>(null);
  const [msg, setMsg] = useState<string | null>(null);

  useEffect(() => {
    shellMenuRegistered().then(setRegistered).catch(() => setRegistered(false));
  }, []);

  const onRegister = async () => {
    try {
      await registerShellMenu();
      setRegistered(true);
      setMsg('✔ 已安装右键菜单「AI 分析」。在文件上右键即可看到。');
    } catch (e) {
      setMsg(`✖ 安装失败：${e}`);
    }
  };

  const onUnregister = async () => {
    try {
      await unregisterShellMenu();
      setRegistered(false);
      setMsg('已卸载右键菜单。');
    } catch (e) {
      setMsg(`✖ 卸载失败：${e}`);
    }
  };

  return (
    <div className="panel">
      <h2>右键菜单集成</h2>
      <p className="hint">
        安装后在资源管理器中选中文件 → 右键 → 「AI 分析」即可一键触发分析。
        多文件可批量分析。菜单写入当前用户注册表，无需管理员权限。
      </p>
      <div className="form-actions">
        <button className="btn primary" onClick={onRegister} disabled={registered === true}>
          {registered === true ? '已安装 ✓' : '安装右键菜单'}
        </button>
        <button className="btn danger" onClick={onUnregister} disabled={registered !== true}>
          卸载
        </button>
      </div>
      {msg && <div className="test-result">{msg}</div>}
    </div>
  );
}
