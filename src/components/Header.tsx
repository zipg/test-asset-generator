import { useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

interface Props {
  savePath: string | undefined;
  onPathChange: (path: string) => void;
}

export default function Header({ savePath, onPathChange }: Props) {
  const handleSelect = useCallback(async () => {
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      onPathChange(selected as string);
    }
  }, [onPathChange]);

  const handleGoToFolder = useCallback(async () => {
    if (!savePath) return;
    try {
      await invoke("open_folder", { path: savePath });
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      window.alert(msg);
    }
  }, [savePath]);

  return (
    <header className="header">
      <div className="path-row">
        <span className="path-label">保存路径:</span>
        <span className="path-value" title={savePath}>
          {savePath || "未设置"}
        </span>
        <button className="btn-small" onClick={handleSelect}>
          选择
        </button>
        <button
          type="button"
          className="btn-small"
          disabled={!savePath}
          onClick={handleGoToFolder}
        >
          前往
        </button>
      </div>
    </header>
  );
}
