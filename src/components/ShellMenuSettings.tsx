// 右键菜单安装 / 卸载 / 重新安装。

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
    if (!window.confirm('确定卸载右键菜单吗？卸载后选中文件右键将不再出现「AI 分析」。')) return;
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
        菜单写入当前用户注册表（HKCU），无需管理员权限。
      </p>

      <div className={`status-banner ${registered ? 'ok' : 'off'}`}>
        {registered === null
          ? '正在检测…'
          : registered
            ? '● 当前状态：右键菜单【已安装】'
            : '○ 当前状态：右键菜单【未安装】'}
      </div>

      <div className="form-actions">
        {registered === true ? (
          <>
            <button className="btn danger" onClick={onUnregister}>
              卸载右键菜单
            </button>
            <button className="btn" onClick={onRegister}>
              重新安装
            </button>
          </>
        ) : (
          <button className="btn primary" onClick={onRegister}>
            安装右键菜单
          </button>
        )}
      </div>

      {msg && <div className="test-result">{msg}</div>}
    </div>
  );
}
