import { useState, useEffect, useCallback } from "react";
import type {
  AudioConfig,
  AudioFormat,
  SampleRate,
  Channels,
  AudioContentType,
} from "../types";

interface Props {
  config: AudioConfig;
  savePath: string | undefined;
  onConfigChange: (cfg: Partial<AudioConfig>) => void;
  onGenerate: () => void;
  onEstimate: (cfg: Record<string, unknown>) => Promise<string>;
  generating: boolean;
  disabled?: boolean;
}

const FORMAT_OPTIONS: AudioFormat[] = ["MP3", "WAV", "AAC"];
const RATE_OPTIONS: SampleRate[] = [44100, 48000];
const CHANNEL_OPTIONS: { value: Channels; label: string }[] = [
  { value: "mono", label: "单声道" },
  { value: "stereo", label: "立体声" },
];
const AUDIO_CONTENT_OPTIONS: { value: AudioContentType; label: string }[] = [
  { value: "noise", label: "随机噪音" },
  { value: "rhythm", label: "简单节奏" },
  { value: "notes", label: "随机音符" },
];

export default function AudioTab({
  config,
  savePath,
  onConfigChange,
  onGenerate,
  onEstimate,
  generating,
  disabled = false,
}: Props) {
  const [estimate, setEstimate] = useState("");

  useEffect(() => {
    onEstimate({
      format: config.format,
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
          onChange={(e) => onConfigChange({ format: e.target.value as AudioFormat })}
          disabled={disabled || generating}
        >
          {FORMAT_OPTIONS.map((f) => (
            <option key={f} value={f}>{f}</option>
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
        <label>采样率</label>
        <select
          value={config.sampleRate}
          onChange={(e) => onConfigChange({ sampleRate: parseInt(e.target.value) as SampleRate })}
          disabled={disabled || generating}
        >
          {RATE_OPTIONS.map((r) => (
            <option key={r} value={r}>{r} Hz</option>
          ))}
        </select>
      </div>
      <div className="form-row">
        <label>声道</label>
        <select
          value={config.channels}
          onChange={(e) => onConfigChange({ channels: e.target.value as Channels })}
          disabled={disabled || generating}
        >
          {CHANNEL_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>{opt.label}</option>
          ))}
        </select>
      </div>
      <div className="form-row">
        <label>音频内容</label>
        <select
          value={config.audioContent}
          onChange={(e) =>
            onConfigChange({ audioContent: e.target.value as AudioContentType })
          }
          disabled={disabled || generating}
        >
          {AUDIO_CONTENT_OPTIONS.map((opt) => (
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