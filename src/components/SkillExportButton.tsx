// 结果卡片上的「自定义导出」按钮：选择导出技能并运行。

import { useEffect, useRef, useState } from 'react';
import { useSkills } from '../stores/useSkills';
import { runSkill } from '../api/tauri';
import type { SkillRunResult } from '../types/shared';

export default function SkillExportButton({ sourceFile, analysisText }: { sourceFile: string; analysisText: string }) {
  const { skills, load } = useSkills();
  const [open, setOpen] = useState(false);
  const [running, setRunning] = useState<string | null>(null);
  const [result, setResult] = useState<SkillRunResult | null>(null);
  const boxRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    load();
  }, [load]);

  useEffect(() => {
    const onClick = (e: MouseEvent) => {
      if (boxRef.current && !boxRef.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener('mousedown', onClick);
    return () => document.removeEventListener('mousedown', onClick);
  }, []);

  const onPick = async (id: string) => {
    setOpen(false);
    setRunning(id);
    setResult(null);
    try {
      setResult(await runSkill(id, sourceFile, analysisText));
    } catch (e) {
      setResult({ success: false, exit_code: null, stdout: '', stderr: String(e), analysis_file: '', output_dir: '' });
    } finally {
      setRunning(null);
    }
  };

  return (
    <div className="skill-export" ref={boxRef}>
      <button className="btn small" onClick={() => setOpen((o) => !o)} disabled={!skills.length || !!running}>
        {running ? '运行中…' : '自定义导出 ▾'}
      </button>
      {open && (
        <div className="skill-dropdown">
          {skills.length === 0 && <div className="empty">还没有导出技能，请到「导出技能」页添加</div>}
          {skills.map((s) => (
            <div key={s.id} className="skill-item" onClick={() => onPick(s.id)}>
              {s.name || s.command}
            </div>
          ))}
        </div>
      )}
      {result && (
        <div className={`skill-result ${result.success ? 'ok' : 'err'}`}>
          {result.success ? '✔ 导出完成' : '✖ 导出失败'}
          <span className="skill-result-dir"> → {result.output_dir}</span>
          {result.stdout && <pre className="skill-stdout">{result.stdout.slice(0, 800)}</pre>}
          {result.stderr && <pre className="skill-stdout err">{result.stderr.slice(0, 800)}</pre>}
        </div>
      )}
    </div>
  );
}
