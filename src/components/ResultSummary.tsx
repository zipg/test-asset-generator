interface Props {
  success: number;
  failed: number;
  errors: Array<{ file: string; error: string }>;
}

export default function ResultSummary({ success, failed, errors }: Props) {
  return (
    <div className="result-summary">
      <div className="result-counts">
        <span className="success">成功 {success} 个</span>
        {failed > 0 && <span className="failed">失败 {failed} 个</span>}
      </div>
      {errors.length > 0 && (
        <div className="error-list">
          {errors.map((e, i) => (
            <div key={i} className="error-item">
              <strong>{e.file}:</strong> {e.error.slice(0, 200)}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
