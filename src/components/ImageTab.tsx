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
  { value: "network", label: "云端生成" },
  { value: "anime", label: "二次元" },
  { value: "boudoir", label: "其它" },
];
const CONTENT_OPTIONS: { value: ImageContentType; label: string }[] = [
  { value: "noise", label: "随机噪声" },
  { value: "solid", label: "纯色" },
  { value: "gradient", label: "渐变" },
  { value: "pattern", label: "图案(彩条)" },
];

const ASPECT_RATIO = 16 / 9;
const BOUDOIR_OVERLAY_KEY = "muse_boudoir_dontshow_v2";

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
  const [formatExpanded, setFormatExpanded] = useState(false);
  const [boudoirOverlayVisible, setBoudoirOverlayVisible] = useState(false);
  const [boudoirDontShow, setBoudoirDontShow] = useState(() => {
    try {
      return localStorage.getItem(BOUDOIR_OVERLAY_KEY) === "true";
    } catch {
      return false;
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
    const val = w || 2;
    if (lockAspect) {
      const h = Math.round(val * ASPECT_RATIO);
      onConfigChange({ width: val, height: h });
    } else {
      onConfigChange({ width: val });
    }
  }, [lockAspect, onConfigChange]);

  const handleHeightChange = useCallback((h: number) => {
    const val = h || 2;
    if (lockAspect) {
      const w = Math.round(val / ASPECT_RATIO);
      onConfigChange({ width: w, height: val });
    } else {
      onConfigChange({ height: val });
    }
  }, [lockAspect, onConfigChange]);

  const handleSourceChange = useCallback((source: ImageSource) => {
    const updates: Partial<ImageConfig> = { imageSource: source };
    if (source === "network") {
      updates.prefix = "云端图片";
      updates.crop = false;
    } else if (source === "anime") {
      updates.prefix = "二次元";
      updates.crop = false;
    } else if (source === "boudoir") {
      updates.prefix = "NSFW";
      updates.crop = false;
      if (!boudoirDontShow) {
        setBoudoirOverlayVisible(true);
      }
    } else {
      updates.prefix = "测试图片";
    }
    setFormatExpanded(false);
    onConfigChange(updates);
  }, [onConfigChange, boudoirDontShow]);

  const dismissBoudoirOverlay = useCallback(() => {
    setBoudoirOverlayVisible(false);
  }, []);

  const dismissBoudoirForever = useCallback(() => {
    setBoudoirOverlayVisible(false);
    setBoudoirDontShow(true);
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
  const isCloudOrOther = imageSource === "network" || imageSource === "boudoir" || imageSource === "anime";

  return (
    <div className="tab-panel">
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
      {!isCloudOrOther ? (
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
      ) : (
        <>
          {!formatExpanded ? (
            <div className="form-row">
              <label></label>
              <span
                className="expand-link"
                onClick={() => setFormatExpanded(true)}
              >
                指定格式
              </span>
              <span className="hint-text">保持原始格式</span>
            </div>
          ) : (
            <div className="form-row">
              <label>
                <span
                  className="expand-link collapse"
                  onClick={() => setFormatExpanded(false)}
                >
                  收起
                </span>
              </label>
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
          )}
        </>
      )}
      {isCloudOrOther ? (
        config.crop ? (
          <div className="form-row">
            <label>
              <span
                className="expand-link collapse"
                onClick={() => onConfigChange({ crop: false })}
              >
                收起
              </span>
              分辨率
            </label>
            <div className="resolution-row">
              <input
                type="number"
                value={config.width}
                min={2}
                onChange={(e) => handleWidthChange(parseInt(e.target.value) || 2)}
                disabled={disabled || generating}
              />
              <span>x</span>
              <input
                type="number"
                value={config.height}
                min={2}
                onChange={(e) => handleHeightChange(parseInt(e.target.value) || 2)}
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
        ) : (
          <div className="form-row">
            <label></label>
            <span
              className="expand-link"
              onClick={() => onConfigChange({ crop: true })}
            >
              指定分辨率
            </span>
            <span className="hint-text">保持原始分辨率</span>
          </div>
        )
      ) : (
        <div className="form-row">
          <label>分辨率</label>
          <div className="resolution-row">
            <input
              type="number"
              value={config.width}
              min={2}
              onChange={(e) => handleWidthChange(parseInt(e.target.value) || 2)}
              disabled={disabled || generating}
            />
            <span>x</span>
            <input
              type="number"
              value={config.height}
              min={2}
              onChange={(e) => handleHeightChange(parseInt(e.target.value) || 2)}
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
      )}
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
              ⚠️ 内容警告<br /><br />
              此功能可能生成违规内容素材，<br />
              仅用于图片审核机制测试，<br />
              严禁传播获取的内容。
            </p>
            <button className="boudoir-overlay-btn" onClick={dismissBoudoirOverlay}>
              我已知晓，继续
            </button>
            <div
              className="boudoir-overlay-dontshow"
              onClick={dismissBoudoirForever}
            >
              不再提示
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
