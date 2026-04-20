import { useState, useEffect, useCallback } from "react";
import type {
  VideoConfig,
  VideoFormat,
  Codec,
  ContentType,
  AudioContentType,
} from "../types";

interface Props {
  config: VideoConfig;
  savePath: string | undefined;
  onConfigChange: (cfg: Partial<VideoConfig>) => void;
  onGenerate: () => void;
  onEstimate: (cfg: Record<string, unknown>) => Promise<string>;
  generating: boolean;
  disabled?: boolean;
}

const FORMAT_OPTIONS: VideoFormat[] = [
  "MP4",
  "MOV",
  "WEBM",
  "AVI",
  "FLV",
  "MKV",
  "3GP",
];
/** 无 = 不混音；其余与音频 Tab 三种内容一致 */
const EMBEDDED_AUDIO_OPTIONS: { value: "none" | AudioContentType; label: string }[] = [
  { value: "none", label: "无" },
  { value: "noise", label: "随机噪音" },
  { value: "rhythm", label: "简单节奏" },
  { value: "notes", label: "随机音符" },
];
const CODEC_OPTIONS: { value: Codec; label: string }[] = [
  { value: "h264", label: "H.264" },
  { value: "hevc", label: "H.265" },
];
const FPS_OPTIONS = [30, 60];
const CONTENT_OPTIONS: { value: ContentType; label: string }[] = [
  { value: "noise", label: "随机噪声" },
  { value: "solid", label: "纯色" },
  { value: "gradient", label: "渐变" },
  { value: "pattern", label: "图案(彩条)" },
];

const ASPECT_RATIO = 16 / 9; // height / width for 9:16

/** FLV / 3GP 等容器通常只搭配 H.264，不支持在 UI 中选 H.265 */
const CODEC_HEVC_UNSUPPORTED: VideoFormat[] = ["FLV", "3GP"];

export default function VideoTab({
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

  const hevcDisabled = CODEC_HEVC_UNSUPPORTED.includes(config.format);

  useEffect(() => {
    if (hevcDisabled && config.codec === "hevc") {
      onConfigChange({ codec: "h264" });
    }
  }, [hevcDisabled, config.codec, config.format, onConfigChange]);

  useEffect(() => {
    onEstimate({
      format: config.format,
      codec: config.codec,
      width: config.width,
      height: config.height,
      fps: config.fps,
      duration: config.duration,
      count: config.count,
      addAudioTrack: config.addAudioTrack,
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
          onChange={(e) => onConfigChange({ format: e.target.value as VideoFormat })}
          disabled={disabled || generating}
        >
          {FORMAT_OPTIONS.map((f) => (
            <option key={f} value={f}>{f}</option>
          ))}
        </select>
      </div>
      <div className="form-row">
        <label>编码</label>
        <select
          value={hevcDisabled ? "h264" : config.codec}
          onChange={(e) => onConfigChange({ codec: e.target.value as Codec })}
          disabled={disabled || generating}
        >
          {CODEC_OPTIONS.map((opt) => (
            <option
              key={opt.value}
              value={opt.value}
              disabled={opt.value === "hevc" && hevcDisabled}
            >
              {opt.label}
              {opt.value === "hevc" && hevcDisabled ? "（当前格式不支持）" : ""}
            </option>
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
        <label>帧率</label>
        <select
          value={config.fps}
          onChange={(e) => onConfigChange({ fps: parseInt(e.target.value) })}
          disabled={disabled || generating}
        >
          {FPS_OPTIONS.map((f) => (
            <option key={f} value={f}>{f} fps</option>
          ))}
        </select>
      </div>
      <div className="form-row">
        <label>时长 (秒)</label>
        <input
          type="number"
          value={config.duration}
          min={1}
          onChange={(e) => onConfigChange({ duration: parseFloat(e.target.value) || 1 })}
          disabled={disabled || generating}
        />
      </div>
      <div className="form-row">
        <label>内容类型</label>
        <select
          value={config.contentType}
          onChange={(e) => onConfigChange({ contentType: e.target.value as ContentType })}
          disabled={disabled || generating}
        >
          {CONTENT_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>{opt.label}</option>
          ))}
        </select>
      </div>
      <div className="form-row">
        <label>增加音频</label>
        <select
          value={config.addAudioTrack ? config.audioContent : "none"}
          onChange={(e) => {
            const v = e.target.value;
            if (v === "none") {
              onConfigChange({ addAudioTrack: false });
            } else {
              onConfigChange({
                addAudioTrack: true,
                audioContent: v as AudioContentType,
              });
            }
          }}
          disabled={disabled || generating}
        >
          {EMBEDDED_AUDIO_OPTIONS.map((opt) => (
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
