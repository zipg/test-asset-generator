import { useState, useEffect, useCallback } from "react";
import type { ImageConfig, ImageFormat, ImageContentType, ImageSource } from "../types";

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
const SOURCE_OPTIONS: { value: ImageSource; label: string }[] = [
  { value: "generated", label: "程序生成" },
  { value: "network", label: "网络获取" },
  { value: "boudoir", label: "其它" },
];
const CONTENT_OPTIONS: { value: ImageContentType; label: string }[] = [
  { value: "noise", label: "随机噪声" },
  { value: "solid", label: "纯色" },
  { value: "gradient", label: "渐变" },
  { value: "pattern", label: "图案(彩条)" },
];

const ASPECT_RATIO = 16 / 9; // height / width for 9:16
const BOUDOIR_OVERLAY_KEY = "muse_boudoir_overlay_dismissed";

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
  const [boudoirOverlayVisible, setBoudoirOverlayVisible] = useState(() => {
    try {
      return localStorage.getItem(BOUDOIR_OVERLAY_KEY) !== "true";
    } catch {
      return true;
    }
  });

  useEffect(() => {
    onEstimate({
      format: config.format,
      width: config.width,
      height: config.height,
      count: config.count,
      imageSource: config.imageSource ?? "generated",
      crop: config.crop ?? true,
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

  const handleSourceChange = useCallback((source: ImageSource) => {
    const updates: Partial<ImageConfig> = { imageSource: source };
    if (source === "network") {
      updates.prefix = "网络图片";
    } else if (source === "boudoir") {
      updates.prefix = "NSFW";
      try {
        if (localStorage.getItem(BOUDOIR_OVERLAY_KEY) !== "true") {
          setBoudoirOverlayVisible(true);
        }
      } catch {}
    } else {
      updates.prefix = "测试图片";
    }
    onConfigChange(updates);
  }, [onConfigChange]);

  const dismissBoudoirOverlay = useCallback(() => {
    setBoudoirOverlayVisible(false);
    try {
      localStorage.setItem(BOUDOIR_OVERLAY_KEY, "true");
    } catch {}
  }, []);

  const handleStart = useCallback(() => {
    if (!savePath) {
      alert("请先选择保存路径");
      return;
    }
    onGenerate();
  }, [savePath, onGenerate]);

  const imageSource = config.imageSource ?? "generated";
  const isRemote = imageSource !== "generated";

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
        <label>图片来源</label>
        <select
          value={imageSource}
          onChange={(e) => handleSourceChange(e.target.value as ImageSource)}
          disabled={disabled || generating}
        >
          {SOURCE_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>{opt.label}</option>
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
      {!isRemote && (
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
      )}
      {isRemote && (
        <div className="form-row">
          <label>裁剪填充</label>
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={config.crop ?? true}
              onChange={(e) => onConfigChange({ crop: e.target.checked })}
              disabled={disabled || generating}
            />
            {config.crop ? "裁剪到精确尺寸" : "保持原始比例"}
          </label>
        </div>
      )}
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
      {boudoirOverlayVisible && imageSource === "boudoir" && (
        <div className="boudoir-overlay">
          <div className="boudoir-overlay-card">
            <p className="boudoir-overlay-text">
              ⚠️ 成人内容警告<br /><br />
              此功能将获取成人内容素材，<br />
              未满18岁禁止使用。<br />
              请勿传播获取的内容。
            </p>
            <button className="boudoir-overlay-btn" onClick={dismissBoudoirOverlay}>
              我已知晓，继续
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
