import { useState, useEffect, useCallback } from "react";
import type { VideoConfig, VideoFormat, Codec, ContentType } from "../types";

interface Props {
  config: VideoConfig;
  savePath: string | undefined;
  onConfigChange: (cfg: Partial<VideoConfig>) => void;
  onGenerate: () => void;
  onEstimate: (cfg: Record<string, unknown>) => Promise<string>;
  generating: boolean;
}

const FORMAT_OPTIONS: VideoFormat[] = ["MP4", "MOV", "WEBM"];
const CODEC_OPTIONS: { value: Codec; label: string }[] = [
  { value: "hevc", label: "H.265" },
  { value: "h264", label: "H.264" },
];
const FPS_OPTIONS = [30, 60];
const CONTENT_OPTIONS: { value: ContentType; label: string }[] = [
  { value: "noise", label: "随机噪声" },
  { value: "solid", label: "纯色" },
  { value: "gradient", label: "渐变" },
  { value: "pattern", label: "图案(彩条)" },
];

export default function VideoTab({
  config,
  savePath,
  onConfigChange,
  onGenerate,
  onEstimate,
  generating,
}: Props) {
  const [estimate, setEstimate] = useState("");

  useEffect(() => {
    onEstimate({
      format: config.format,
      codec: config.codec,
      width: config.width,
      height: config.height,
      fps: config.fps,
      duration: config.duration,
      count: config.count,
    }).then(setEstimate);
  }, [config, onEstimate]);

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
        >
          {FORMAT_OPTIONS.map((f) => (
            <option key={f} value={f}>{f}</option>
          ))}
        </select>
      </div>
      <div className="form-row">
        <label>编码</label>
        <select
          value={config.codec}
          onChange={(e) => onConfigChange({ codec: e.target.value as Codec })}
        >
          {CODEC_OPTIONS.map((opt) => (
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
            onChange={(e) => onConfigChange({ width: parseInt(e.target.value) || 1 })}
          />
          <span>x</span>
          <input
            type="number"
            value={config.height}
            min={1}
            onChange={(e) => onConfigChange({ height: parseInt(e.target.value) || 1 })}
          />
        </div>
      </div>
      <div className="form-row">
        <label>帧率</label>
        <select
          value={config.fps}
          onChange={(e) => onConfigChange({ fps: parseInt(e.target.value) })}
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
        />
      </div>
      <div className="form-row">
        <label>内容类型</label>
        <select
          value={config.contentType}
          onChange={(e) => onConfigChange({ contentType: e.target.value as ContentType })}
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
        />
      </div>
      <div className="form-row">
        <label>前缀</label>
        <input
          type="text"
          value={config.prefix}
          onChange={(e) => onConfigChange({ prefix: e.target.value })}
        />
      </div>
      <div className="estimate-row">
        <span>预计体积: {estimate}</span>
        <span>{config.count} 个文件</span>
      </div>
      <button
        className="btn-primary"
        onClick={handleStart}
        disabled={generating}
      >
        {generating ? "生成中..." : "开始生成"}
      </button>
    </div>
  );
}
