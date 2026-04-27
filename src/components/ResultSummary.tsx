import { useState } from "react";

interface Props {
  success: number;
  failed: number;
  errors: Array<{ file: string; error: string }>;
  elapsedSecs?: number;
}

function formatElapsed(secs: number): string {
  if (secs >= 60) {
    const min = Math.floor(secs / 60);
    const s = Math.round(secs % 60);
    return `${min}分${s}秒`;
  }
  return `${secs.toFixed(1)}秒`;
}

export default function ResultSummary({ success, failed, errors, elapsedSecs }: Props) {
  const [showErrors, setShowErrors] = useState(false);

  return (
    <div className="result-summary">
      <div className="result-counts">
        <span className="success">成功 {success} 个</span>
        {failed > 0 && <span className="failed">失败 {failed} 个</span>}
        {elapsedSecs !== undefined && (
          <span style={{ color: "#86868b", fontSize: "13px", fontWeight: 400 }}>
            耗时 {formatElapsed(elapsedSecs)}
          </span>
        )}
      </div>
      {errors.length > 0 && (
        <div className="error-section">
          <button className="error-toggle" onClick={() => setShowErrors(!showErrors)}>
            {showErrors ? "▼ 收起错误信息" : `▶ 查看 ${errors.length} 个错误`}
          </button>
          {showErrors && (
            <div className="error-list">
              {errors.map((e, i) => (
                <div key={i} className="error-item">
                  <strong>{e.file}:</strong> {e.error}
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
