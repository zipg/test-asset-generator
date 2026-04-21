export type ImageFormat =
  | "PNG"
  | "JPG"
  | "JPEG"
  | "WEBP"
  | "GIF"
  | "BMP"
  | "TIFF";
export type AudioFormat = "MP3" | "WAV" | "AAC";
export type VideoFormat =
  | "MP4"
  | "MOV"
  | "WEBM"
  | "AVI"
  | "FLV"
  | "MKV"
  | "3GP";
export type ContentType = "solid" | "gradient" | "pattern" | "noise";
/** 音频内容：随机噪音 / 简单节奏 / 随机音符 / 随机音乐（古典动机循环） */
export type AudioContentType =
  | "noise"
  | "rhythm"
  | "notes"
  | "random_music";
export type Codec = "h264" | "hevc";
export type SampleRate = 44100 | 48000;
export type Channels = "mono" | "stereo";
export type MediaType = "image" | "audio" | "video" | "music";

export interface ImageConfig {
  format: ImageFormat;
  width: number;
  height: number;
  contentType: ContentType;
  count: number;
  prefix: string;
}

export interface AudioConfig {
  format: AudioFormat;
  duration: number;
  sampleRate: SampleRate;
  channels: Channels;
  count: number;
  prefix: string;
  audioContent: AudioContentType;
}

export interface VideoConfig {
  format: VideoFormat;
  codec: Codec;
  width: number;
  height: number;
  fps: number;
  duration: number;
  contentType: ContentType;
  count: number;
  prefix: string;
  addAudioTrack: boolean;
  audioContent: AudioContentType;
}

export type MelodyTemplate = "scale" | "arpeggio" | "folk" | "random";

export interface MusicConfig {
  format: AudioFormat;
  duration: number;
  bpm: number;
  melody: MelodyTemplate;
  count: number;
  prefix: string;
}

export interface AppConfig {
  /** Persisted by Rust; omit on old clients — do not strip when saving. */
  schemaVersion?: number;
  savePath: string | null;
  imageConfig: ImageConfig;
  audioConfig: AudioConfig;
  videoConfig: VideoConfig;
  musicConfig: MusicConfig;
}

export interface ProgressPayload {
  current: number;
  total: number;
  currentFile: string;
  estimatedRemainingSecs: number;
}

export interface TaskResult {
  success: number;
  failed: number;
  errors: Array<{ file: string; error: string }>;
}
