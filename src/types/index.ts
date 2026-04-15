export type ImageFormat = "PNG" | "JPG" | "WEBP";
export type AudioFormat = "MP3" | "WAV" | "AAC";
export type VideoFormat = "MP4" | "MOV" | "WEBM";
export type ContentType = "solid" | "gradient" | "pattern" | "noise";
export type Codec = "h264" | "hevc";
export type SampleRate = 44100 | 48000;
export type Channels = "mono" | "stereo";
export type MediaType = "image" | "audio" | "video";

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
}

export interface AppConfig {
  savePath: string | null;
  imageConfig: ImageConfig;
  audioConfig: AudioConfig;
  videoConfig: VideoConfig;
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
