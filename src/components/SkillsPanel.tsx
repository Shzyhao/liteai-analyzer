// 导出技能管理：增删改 + 测试运行 + AI 生成。

import { useEffect, useState } from 'react';
import { useSkills } from '../stores/useSkills';
import { runSkill, generateSkill, getSkillsDir } from '../api/tauri';
import type { ExportSkill, SkillRunResult } from '../types/shared';

const emptySkill = (): ExportSkill => ({
  id: `skill_${Date.now().toString(36)}`,
  name: '',
  command: 'python',
  args: '',
  cwd: null,
});

export default function SkillsPanel() {
  const { skills, load, add, update, remove } = useSkills();
  const [editing, setEditing] = useState<ExportSkill | null>(null);
  const [isNew, setIsNew] = useState(false);
  const [testOut, setTestOut] = useState<SkillRunResult | null>(null);
  const [aiDesc, setAiDesc] = useState('');
  const [aiBusy, setAiBusy] = useState(false);
  const [aiMsg, setAiMsg] = useState<string | null>(null);
  const [skillsDir, setSkillsDir] = useState('');

  useEffect(() => {
    load();
    getSkillsDir().then(setSkillsDir).catch(() => {});
  }, [load]);

  const onAiGenerate = async () => {
    if (!aiDesc.trim()) {
      alert('请先描述你想要的导出效果');
      return;
    }
    setAiBusy(true);
    setAiMsg(null);
    try {
      const res = await generateSkill(aiDesc.trim());
      await load();
      setAiDesc('');
      setAiMsg(
        `✔ 已生成技能「${res.skill.name}」。脚本已保存到：\n${res.script_path}\n\n` +
          '⚠️ 安全提醒：AI 生成的脚本会在本机以你的权限运行。提示词已禁止网络/删除/系统修改操作，' +
          '但请仍在使用前点开脚本检查一遍内容（应只含「读入结果 + 写一个输出文件」）。',
      );
    } catch (e) {
      setAiMsg(`✖ AI 生成失败：${e}`);
    } finally {
      setAiBusy(false);
    }
  };

  const onSave = () => {
    if (!editing || !editing.name.trim() || !editing.command.trim()) {
      alert('请填写名称和命令');
      return;
    }
    if (isNew) add(editing);
    else update(editing);
    setEditing(null);
    setIsNew(false);
  };

  const onTest = async () => {
    if (!editing) return;
    try {
      setTestOut(
        await runSkill(editing.id, '', `# 测试\n\n这是一条测试分析结果，用来验证导出技能。`),
      );
    } catch (e) {
      setTestOut({ success: false, exit_code: null, stdout: '', stderr: String(e), analysis_file: '', output_dir: '' });
    }
  };

  return (
    <div className="panel">
      <h2>AI 生成导出技能</h2>
      <div className="form-row">
        <textarea
          rows={2}
          placeholder={'用一句话描述想要的导出效果，例如：「把分析结果生成一个带表格的 HTML 报告」'}
          value={aiDesc}
          onChange={(e) => setAiDesc(e.target.value)}
          style={{ flex: 1 }}
        />
      </div>
      <div className="form-actions">
        <button className="btn primary" onClick={onAiGenerate} disabled={aiBusy}>
          {aiBusy ? 'AI 生成中…' : '🤖 AI 生成技能'}
        </button>
        <span className="hint">生成到：{skillsDir || '桌面（默认）'}</span>
      </div>
      {aiMsg && <div className="test-result" style={{ whiteSpace: 'pre-wrap' }}>{aiMsg}</div>}

      <h2>已保存的技能</h2>
      <div className="editor-layout">
      <div className="template-list">
        <h2>技能列表</h2>
        {skills.map((s) => (
          <div
            key={s.id}
            className={`template-item ${editing?.id === s.id ? 'active' : ''}`}
            onClick={() => { setEditing({ ...s }); setIsNew(false); setTestOut(null); }}
          >
            <span>{s.name || s.command}</span>
          </div>
        ))}
        <button
          className="btn small"
          onClick={() => { setEditing(emptySkill()); setIsNew(true); setTestOut(null); }}
        >
          + 新建技能
        </button>
      </div>

      <div className="template-editor">
        {editing ? (
          <>
            <div className="form-row">
              <label>名称</label>
              <input value={editing.name} onChange={(e) => setEditing({ ...editing, name: e.target.value })} placeholder="如：生成 HTML 报告" />
            </div>
            <div className="form-row">
              <label>命令</label>
              <input value={editing.command} onChange={(e) => setEditing({ ...editing, command: e.target.value })} placeholder="python / node / powershell / 脚本路径" />
            </div>
            <div className="form-row">
              <label>参数</label>
              <input value={editing.args} onChange={(e) => setEditing({ ...editing, args: e.target.value })} placeholder='如：-File "C:\scripts\report.py"' />
            </div>
            <div className="form-row">
              <label>工作目录（可选）</label>
              <input value={editing.cwd ?? ''} onChange={(e) => setEditing({ ...editing, cwd: e.target.value || null })} />
            </div>
            <div className="form-actions">
              <button className="btn primary" onClick={onSave}>{isNew ? '添加' : '保存'}</button>
              <button className="btn" onClick={onTest}>测试运行</button>
              {!isNew && (
                <button className="btn danger" onClick={() => { remove(editing.id); setEditing(null); setTestOut(null); }}>
                  删除
                </button>
              )}
            </div>
            <p className="hint">
              脚本会收到：分析结果文件路径作为最后一个参数；环境变量
              LITEAI_SOURCE_FILE（源文件）、LITEAI_ANALYSIS_FILE（结果文件）、LITEAI_OUTPUT_DIR（输出目录）。
              请把生成的文件写到 LITEAI_OUTPUT_DIR。
            </p>
          </>
        ) : (
          <div className="empty">选择左侧技能进行编辑，或新建一个</div>
        )}

        {testOut && (
          <div className="test-result" style={{ whiteSpace: 'pre-wrap' }}>
            {testOut.success ? `✔ 运行成功（退出码 ${testOut.exit_code ?? '?'}）` : `✖ 运行失败（退出码 ${testOut.exit_code ?? '?'}）`}
            {testOut.stdout && `\n--- stdout ---\n${testOut.stdout}`}
            {testOut.stderr && `\n--- stderr ---\n${testOut.stderr}`}
          </div>
        )}
      </div>
      </div>
    </div>
  );
}
