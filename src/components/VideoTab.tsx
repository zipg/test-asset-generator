import { useState, useEffect, useCallback } from "react";
import type {
  VideoConfig,
  VideoFormat,
  Codec,
  VideoContentType,
} from "../types";

interface Props {
  config: VideoConfig;
  savePath: string | undefined;
  onConfigChange: (cfg: Partial<VideoConfig>) => void;
  onGenerate: () => void;
  onEstimate: (cfg: Record<string, unknown>) => Promise<string>;
  generating: boolean;
  soundfontReady: boolean;
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
const AUDIO_ENGINE_OPTIONS: { value: string; label: string }[] = [
  { value: "none", label: "无" },
  { value: "simple", label: "简易合成 (速度快)" },
  { value: "fluidsynth", label: "真实乐器 (FluidSynth, 较慢)" },
];
const CODEC_OPTIONS: { value: Codec; label: string }[] = [
  { value: "h264", label: "H.264" },
  { value: "hevc", label: "H.265" },
];
const FPS_OPTIONS = [30, 60];
const CONTENT_OPTIONS: { value: VideoContentType; label: string }[] = [
  { value: "gradient", label: "随机渐变" },
  { value: "pattern", label: "彩条图案" },
  { value: "noise", label: "元胞噪声 (生成慢)" },
  { value: "plasma", label: "等离子动态 (生成慢)" },
  { value: "waves", label: "波纹律动 (生成慢)" },
  { value: "kaleidoscope", label: "万花筒 (生成慢)" },
  { value: "audioviz", label: "音频可视化 (生成慢)" },
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
  soundfontReady,
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
          onChange={(e) => onConfigChange({ contentType: e.target.value as VideoContentType })}
          disabled={disabled || generating}
        >
          {CONTENT_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>{opt.label}</option>
          ))}
        </select>
      </div>
      <div className="form-row">
        <label>画面动态</label>
        <div className="range-with-value">
          <input
            type="range"
            value={config.dynamics ?? 5}
            min={1}
            max={10}
            step={1}
            onChange={(e) => onConfigChange({ dynamics: parseInt(e.target.value) })}
            disabled={disabled || generating}
          />
          <span className="range-value">{config.dynamics ?? 5}</span>
        </div>
      </div>
      <div className="form-row">
        <label>增加音频</label>
        <select
          value={config.audioEngine ?? (config.addAudioTrack ? "fluidsynth" : "none")}
          onChange={(e) => {
            const v = e.target.value;
            onConfigChange({
              audioEngine: v as VideoConfig["audioEngine"],
              addAudioTrack: v !== "none",
            });
          }}
          disabled={disabled || generating}
        >
          {AUDIO_ENGINE_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>{opt.label}</option>
          ))}
        </select>
        {(config.audioEngine ?? "fluidsynth") === "fluidsynth" && (
          config.addAudioTrack !== false
            ? (soundfontReady
              ? <span className="status-ok">✓ 音色库已就绪</span>
              : <span className="status-warn">⚠ 音色库未内置</span>)
            : null
        )}
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
