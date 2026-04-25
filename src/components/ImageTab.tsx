import { useState, useEffect, useCallback } from "react";
import type { ImageConfig, ImageFormat, ImageContentType } from "../types";

interface Props {
  config: ImageConfig;
  savePath: string | undefined;
  onConfigChange: (cfg: Partial<ImageConfig>) => void;
  onGenerate: () => void;
  onEstimate: (cfg: Record<string, unknown>) => Promise<string>;
  generating: boolean;
  disabled?: boolean;
}

const FORMAT_OPTIONS: ImageFormat[] = [
  "PNG",
  "JPG",
  "JPEG",
  "WEBP",
  "GIF",
  "BMP",
  "TIFF",
];
const CONTENT_OPTIONS: { value: ImageContentType; label: string }[] = [
  { value: "noise", label: "随机噪声" },
  { value: "solid", label: "纯色" },
  { value: "gradient", label: "渐变" },
  { value: "pattern", label: "图案(彩条)" },
];

const ASPECT_RATIO = 16 / 9; // height / width for 9:16

export default function ImageTab({
  config,
  savePath,
  onConfigChange,
  onGenerate,
  onEstimate,
  generating,
  disabled = false,
}: Props) {
  const [estimate, setEstimate] = useState("");
  const [lockAspect, setLockAspect] = useState(true);

  useEffect(() => {
    onEstimate({
      format: config.format,
      width: config.width,
      height: config.height,
      count: config.count,
    }).then(setEstimate);
  }, [config, onEstimate]);

  const handleWidthChange = useCallback((w: number) => {
    if (lockAspect) {
      const h = Math.round(w * ASPECT_RATIO);
      onConfigChange({ width: w, height: h });
    } else {
      onConfigChange({ width: w });
    }
  }, [lockAspect, onConfigChange]);

  const handleHeightChange = useCallback((h: number) => {
    if (lockAspect) {
      const w = Math.round(h / ASPECT_RATIO);
      onConfigChange({ width: w, height: h });
    } else {
      onConfigChange({ height: h });
    }
  }, [lockAspect, onConfigChange]);

  const handleStart = useCallback(() => {
    if (!savePath) {
      alert("请先选择保存路径");
      return;
    }
    onGenerate();
  }, [savePath, onGenerate]);

  return (
    <div className="tab-panel">
      <div className="form-row">
        <label>格式</label>
        <select
          value={config.format}
          onChange={(e) => onConfigChange({ format: e.target.value as ImageFormat })}
          disabled={disabled || generating}
        >
          {FORMAT_OPTIONS.map((f) => (
            <option key={f} value={f}>{f}</option>
          ))}
        </select>
      </div>
      <div className="form-row">
        <label>分辨率</label>
        <div className="resolution-row">
          <input
            type="number"
            value={config.width}
            min={1}
            onChange={(e) => handleWidthChange(parseInt(e.target.value) || 1)}
            disabled={disabled || generating}
          />
          <span>x</span>
          <input
            type="number"
            value={config.height}
            min={1}
            onChange={(e) => handleHeightChange(parseInt(e.target.value) || 1)}
            disabled={disabled || generating}
          />
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={lockAspect}
              onChange={(e) => setLockAspect(e.target.checked)}
              disabled={disabled || generating}
            />
            锁定9:16
          </label>
        </div>
      </div>
      <div className="form-row">
        <label>内容类型</label>
        <select
          value={config.contentType}
          onChange={(e) => onConfigChange({ contentType: e.target.value as ImageContentType })}
          disabled={disabled || generating}
        >
          {CONTENT_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>{opt.label}</option>
          ))}
        </select>
      </div>
      <div className="form-row">
        <label>文件数量</label>
        <input
          type="number"
          value={config.count}
          min={1}
          onChange={(e) => onConfigChange({ count: parseInt(e.target.value) || 1 })}
          disabled={disabled || generating}
        />
      </div>
      <div className="form-row">
        <label>文件名前缀</label>
        <input
          type="text"
          value={config.prefix}
          onChange={(e) => onConfigChange({ prefix: e.target.value })}
          disabled={disabled || generating}
        />
      </div>
      <div className="estimate-row">
        <span>预计体积: {estimate}</span>
        <span>{config.count} 个文件</span>
      </div>
      <button
        className="btn-primary"
        onClick={handleStart}
        disabled={disabled || generating}
      >
        {generating ? "生成中..." : "开始生成"}
      </button>
    </div>
  );
}