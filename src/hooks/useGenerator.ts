import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppConfig, TaskResult } from "../types";

export function useGenerator() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [loading, setLoading] = useState(false);
  const [downloading, setDownloading] = useState(false);
  const [ffmpegReady, setFfmpegReady] = useState(true);
  const [soundfontReady, setSoundfontReady] = useState(false);
  const [hostOs, setHostOs] = useState<string>("unknown");

  useEffect(() => {
    Promise.all([
      invoke<string>("host_os").catch(() => "unknown"),
      invoke<string>("check_ffmpeg").catch(() => "not_found"),
      invoke<string>("check_soundfont").catch(() => "not_found"),
    ]).then(([os, ffmpegStatus, soundfontStatus]) => {
      setHostOs(os);
      setFfmpegReady(ffmpegStatus === "found");
      setSoundfontReady(soundfontStatus === "found");
    });

    invoke<AppConfig>("get_config")
      .then((cfg) => setConfig(cfg))
      .catch(console.error);
  }, []);

  const updateConfig = useCallback(
    (updated: AppConfig) => {
      setConfig(updated);
      invoke("save_config", { cfg: updated }).catch(console.error);
    },
    []
  );

  const estimateSize = useCallback(
    async (
      mediaType: "image" | "audio" | "video",
      cfg: Record<string, unknown>
    ): Promise<string> => {
      return invoke<string>("estimate_size", { mediaType, cfg });
    },
    []
  );

  const selectPath = useCallback(async (): Promise<string | null> => {
    return invoke<string | null>("select_save_path");
  }, []);

  const downloadFFmpeg = useCallback(async (): Promise<{ success: boolean; message: string }> => {
    setDownloading(true);
    try {
      const result = await invoke<string>("download_ffmpeg");
      return { success: true, message: result };
    } catch (e) {
      return { success: false, message: String(e) };
    } finally {
      setDownloading(false);
    }
  }, []);

  const downloadSoundfont = useCallback(async (): Promise<{ success: boolean; message: string }> => {
    setDownloading(true);
    try {
      const result = await invoke<string>("download_soundfont");
      setSoundfontReady(true);
      return { success: true, message: result };
    } catch (e) {
      return { success: false, message: String(e) };
    } finally {
      setDownloading(false);
    }
  }, []);

  const generateImages = useCallback(
    async (
      imageConfig: Record<string, unknown>,
      savePath: string
    ): Promise<TaskResult> => {
      setLoading(true);
      try {
        const result = await invoke<TaskResult>("generate_images", {
          config: imageConfig,
          savePath,
        });
        return result;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  const generateAudio = useCallback(
    async (
      audioConfig: Record<string, unknown>,
      savePath: string
    ): Promise<TaskResult> => {
      setLoading(true);
      try {
        const result = await invoke<TaskResult>("generate_audio", {
          config: audioConfig,
          savePath,
        });
        return result;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  const generateVideos = useCallback(
    async (
      videoConfig: Record<string, unknown>,
      savePath: string
    ): Promise<TaskResult> => {
      setLoading(true);
      try {
        const result = await invoke<TaskResult>("generate_videos", {
          config: videoConfig,
          savePath,
        });
        return result;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  const cancelGeneration = useCallback(async () => {
    await invoke("set_cancelled", { val: true });
  }, []);

  const generateMusic = useCallback(
    async (
      musicConfig: Record<string, unknown>,
      savePath: string
    ): Promise<TaskResult> => {
      setLoading(true);
      try {
        const result = await invoke<TaskResult>("generate_music", {
          config: musicConfig,
          savePath,
        });
        return result;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  return {
    config,
    updateConfig,
    loading,
    downloading,
    ffmpegReady,
    soundfontReady,
    hostOs,
    estimateSize,
    selectPath,
    downloadFFmpeg,
    downloadSoundfont,
    generateImages,
    generateAudio,
    generateVideos,
    generateMusic,
    cancelGeneration,
  };
}