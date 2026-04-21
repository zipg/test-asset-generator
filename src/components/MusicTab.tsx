import { useState, useEffect, useCallback } from "react";
import type {
  MusicConfig,
  AudioFormat,
  MelodyTemplate,
} from "../types";

interface Props {
  config: MusicConfig;
  savePath: string | undefined;
  onConfigChange: (cfg: Partial<MusicConfig>) => void;
  onGenerate: () => void;
  onEstimate: (cfg: Record<string, unknown>) => Promise<string>;
  generating: boolean;
  soundfontReady: boolean;
  disabled?: boolean;
}

const FORMAT_OPTIONS: AudioFormat[] = ["MP3", "WAV", "AAC"];
const MELODY_OPTIONS: { value: MelodyTemplate; label: string }[] = [
  { value: "scale", label: "音阶练习" },
  { value: "arpeggio", label: "琶音" },
  { value: "folk", label: "民谣旋律" },
  { value: "twinkle", label: "小星星" },
  { value: "ode_to_joy", label: "欢乐颂" },
  { value: "canon", label: "卡农" },
  { value: "castle_sky", label: "天空之城" },
  { value: "jasmine", label: "茉莉花" },
  { value: "birthday", label: "生日快乐" },
  { value: "random", label: "随机旋律" },
];

export default function MusicTab({
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
      <div className="experimental-notice">
        ⚠️ 实验性功能：使用 FFmpeg sine 滤镜生成简单旋律，音色为纯正弦波（类似 8-bit 游戏音乐）
      </div>
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
          min={5}
          max={120}
          onChange={(e) => onConfigChange({ duration: parseFloat(e.target.value) || 30 })}
          disabled={disabled || generating}
        />
      </div>
      <div className="form-row">
        <label>BPM (节奏)</label>
        <input
          type="number"
          value={config.bpm}
          min={60}
          max={180}
          onChange={(e) => onConfigChange({ bpm: parseInt(e.target.value) || 120 })}
          disabled={disabled || generating}
        />
      </div>
      <div className="form-row">
        <label>旋律模板</label>
        <select
          value={config.melody}
          onChange={(e) => onConfigChange({ melody: e.target.value as MelodyTemplate })}
          disabled={disabled || generating}
        >
          {MELODY_OPTIONS.map((opt) => (
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
          max={50}
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
      <div className="form-row">
        <label>
          <input
            type="checkbox"
            checked={config.useFluidsynth}
            onChange={(e) => onConfigChange({ useFluidsynth: e.target.checked })}
            disabled={disabled || generating}
          />
          {" "}使用 FluidSynth（真实乐器音色）
        </label>
        {soundfontReady ? (
          <span className="status-ok">✓ 音色库已就绪</span>
        ) : (
          <span className="status-warn">⚠ 音色库未内置</span>
        )}
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
