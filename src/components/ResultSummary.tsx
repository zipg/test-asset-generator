import { useState } from "react";

interface Props {
  success: number;
  failed: number;
  errors: Array<{ file: string; error: string }>;
}

export default function ResultSummary({ success, failed, errors }: Props) {
  const [showErrors, setShowErrors] = useState(false);

  return (
    <div className="result-summary">
      <div className="result-counts">
        <span className="success">成功 {success} 个</span>
        {failed > 0 && <span className="failed">失败 {failed} 个</span>}
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
                  <strong>{e.file}:</strong> {e.error.slice(0, 200)}
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
