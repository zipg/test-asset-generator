interface Props {
  current: number;
  total: number;
  currentFile: string;
  estimatedRemainingSecs: number;
  onCancel: () => void;
}

function formatTime(secs: number): string {
  if (secs < 60) return `00:${String(Math.floor(secs)).padStart(2, "0")}`;
  const m = Math.floor(secs / 60);
  const s = Math.floor(secs % 60);
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

export default function ProgressPanel({
  current,
  total,
  currentFile,
  estimatedRemainingSecs,
  onCancel,
}: Props) {
  const pct = total > 0 ? (current / total) * 100 : 0;

  return (
    <div className="progress-panel">
      <div className="progress-header">
        <span>进度: {current} / {total}</span>
        <span>剩余 {formatTime(estimatedRemainingSecs)}</span>
      </div>
      <div className="progress-bar">
        <div className="progress-fill" style={{ width: `${pct}%` }} />
      </div>
      <div className="progress-file">{currentFile}</div>
      <button className="btn-cancel" onClick={onCancel}>取消</button>
    </div>
  );
}
