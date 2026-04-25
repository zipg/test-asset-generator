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
      <div className="form-row gain-row">
        <label>音量增益</label>
        <input
          type="range"
          value={config.gainDb ?? 0}
          min={0}
          max={10}
          step={0.5}
          onChange={(e) => onConfigChange({ gainDb: parseFloat(e.target.value) })}
          disabled={disabled || generating}
        />
        <span className="gain-tag">{config.gainDb ?? 0} dB</span>
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
        <label>音色引擎</label>
        <select
          value={config.soundEngine ?? "fluidsynth"}
          onChange={(e) => onConfigChange({ soundEngine: e.target.value as "fluidsynth" | "simple" })}
          disabled={disabled || generating}
        >
          <option value="fluidsynth">真实乐器 (FluidSynth, 较慢)</option>
          <option value="simple">简易合成 (速度快)</option>
        </select>
        {(config.soundEngine ?? "fluidsynth") === "fluidsynth"
          ? (soundfontReady
            ? <span className="status-ok">✓ 音色库已就绪</span>
            : <span className="status-warn">⚠ 音色库未内置</span>)
          : <span className="status-ok">✓ 无需音色库</span>
        }
      </div>
      {(config.soundEngine ?? "fluidsynth") === "fluidsynth" && (
        <>
          <div className="form-row">
            <label>乐器选择</label>
            <select
              value={config.instrument ?? "random"}
              onChange={(e) => onConfigChange({ instrument: e.target.value })}
              disabled={disabled || generating}
            >
              <option value="random">随机乐器</option>
              <option value="0">Acoustic Grand Piano (大钢琴)</option>
              <option value="1">Bright Acoustic Piano (亮音钢琴)</option>
              <option value="6">Harpsichord (羽管键琴)</option>
              <option value="8">Celesta (钢片琴)</option>
              <option value="11">Vibraphone (颤音琴)</option>
              <option value="13">Marimba (马林巴)</option>
              <option value="15">Dulcimer (扬琴)</option>
              <option value="20">Reed Organ (簧风琴)</option>
              <option value="22">Accordion (手风琴)</option>
              <option value="25">Acoustic Guitar nylon (尼龙吉他)</option>
              <option value="26">Acoustic Guitar steel (钢弦吉他)</option>
              <option value="41">Violin (小提琴)</option>
              <option value="42">Viola (中提琴)</option>
              <option value="43">Cello (大提琴)</option>
              <option value="47">Harp (竖琴)</option>
              <option value="57">Trumpet (小号)</option>
              <option value="67">Tenor Sax (次中音萨克斯)</option>
              <option value="69">Oboe (双簧管)</option>
              <option value="72">Clarinet (单簧管)</option>
              <option value="74">Flute (长笛)</option>
              <option value="76">Pan Flute (排箫)</option>
              <option value="80">Ocarina (陶笛)</option>
            </select>
          </div>
          <div className="form-row">
            <label>
              <input
                type="checkbox"
                checked={config.enableHarmony ?? true}
                onChange={(e) => onConfigChange({ enableHarmony: e.target.checked })}
                disabled={disabled || generating}
              />
              {" "}多乐器和声
            </label>
          </div>
          <div className="form-row">
            <label>
              <input
                type="checkbox"
                checked={config.enableDrums ?? true}
                onChange={(e) => onConfigChange({ enableDrums: e.target.checked })}
                disabled={disabled || generating}
              />
              {" "}添加鼓点伴奏
            </label>
          </div>
        </>
      )}
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
