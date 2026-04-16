import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppConfig, TaskResult } from "../types";

const CONTENT_TYPES = ["noise", "solid", "gradient", "pattern"] as const;

function randomContentType() {
  return CONTENT_TYPES[Math.floor(Math.random() * CONTENT_TYPES.length)];
}

export function useGenerator() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [loading, setLoading] = useState(false);
  const [downloading, setDownloading] = useState(false);
  const [ffmpegReady, setFfmpegReady] = useState(true);

  useEffect(() => {
    // Check FFmpeg status at startup
    invoke<string>("check_ffmpeg").then((status) => {
      setFfmpegReady(status === "found");
    }).catch(() => setFfmpegReady(false));

    invoke<AppConfig>("get_config")
      .then((cfg) => {
        // Randomize content type on startup for all media types
        setConfig({
          ...cfg,
          imageConfig: { ...cfg.imageConfig, contentType: randomContentType() },
          videoConfig: { ...cfg.videoConfig, contentType: randomContentType() },
        });
      })
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

  return {
    config,
    updateConfig,
    loading,
    downloading,
    ffmpegReady,
    estimateSize,
    selectPath,
    downloadFFmpeg,
    generateImages,
    generateAudio,
    generateVideos,
    cancelGeneration,
  };
}