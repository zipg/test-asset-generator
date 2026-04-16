import { useState, useCallback, useEffect } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import Header from "./components/Header";
import TabBar from "./components/TabBar";
import ImageTab from "./components/ImageTab";
import AudioTab from "./components/AudioTab";
import VideoTab from "./components/VideoTab";
import ProgressPanel from "./components/ProgressPanel";
import ResultSummary from "./components/ResultSummary";
import { useGenerator } from "./hooks/useGenerator";
import type { MediaType, ProgressPayload, TaskResult } from "./types";

export default function App() {
  const [activeTab, setActiveTab] = useState<MediaType>("video");
  const [generating, setGenerating] = useState(false);
  const [progress, setProgress] = useState<ProgressPayload | null>(null);
  const [imageResult, setImageResult] = useState<TaskResult | null>(null);
  const [audioResult, setAudioResult] = useState<TaskResult | null>(null);
  const [videoResult, setVideoResult] = useState<TaskResult | null>(null);

  const {
    config,
    updateConfig,
    estimateSize,
    downloading,
    ffmpegReady,
    downloadFFmpeg,
    generateImages,
    generateAudio,
    generateVideos,
    cancelGeneration,
  } = useGenerator();

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    listen<ProgressPayload>("generation-progress", (event) => {
      setProgress(event.payload);
    }).then((fn) => { unlisten = fn; });
    return () => { unlisten?.(); };
  }, []);

  const handleTabChange = useCallback((tab: MediaType) => {
    if (!generating && !downloading) {
      setActiveTab(tab);
    }
  }, [generating, downloading]);

  const handlePathChange = useCallback(
    (path: string) => {
      if (config) {
        updateConfig({ ...config, savePath: path });
      }
    },
    [config, updateConfig]
  );

  const handleImageConfig = useCallback(
    (partial: Record<string, unknown>) => {
      if (!config) return;
      updateConfig({ ...config, imageConfig: { ...config.imageConfig, ...partial } });
    },
    [config, updateConfig]
  );

  const handleAudioConfig = useCallback(
    (partial: Record<string, unknown>) => {
      if (!config) return;
      updateConfig({ ...config, audioConfig: { ...config.audioConfig, ...partial } });
    },
    [config, updateConfig]
  );

  const handleVideoConfig = useCallback(
    (partial: Record<string, unknown>) => {
      if (!config) return;
      updateConfig({ ...config, videoConfig: { ...config.videoConfig, ...partial } });
    },
    [config, updateConfig]
  );

  const handleGenerateImages = useCallback(async () => {
    if (!config?.savePath) return;
    setGenerating(true);
    setImageResult(null);
    setProgress(null);
    try {
      const downloadResult = await downloadFFmpeg();
      if (!downloadResult.success) {
        throw new Error("FFmpeg 下载失败: " + downloadResult.message);
      }
      const res = await generateImages(
        config.imageConfig as unknown as Record<string, unknown>,
        config.savePath
      );
      setImageResult(res);
    } catch (e) {
      setImageResult({ success: 0, failed: 1, errors: [{ file: "unknown", error: String(e) }] });
    } finally {
      setGenerating(false);
    }
  }, [config, generateImages, downloadFFmpeg]);

  const handleGenerateAudio = useCallback(async () => {
    if (!config?.savePath) return;
    setGenerating(true);
    setAudioResult(null);
    setProgress(null);
    try {
      const downloadResult = await downloadFFmpeg();
      if (!downloadResult.success) {
        throw new Error("FFmpeg 下载失败: " + downloadResult.message);
      }
      const res = await generateAudio(
        config.audioConfig as unknown as Record<string, unknown>,
        config.savePath
      );
      setAudioResult(res);
    } catch (e) {
      setAudioResult({ success: 0, failed: 1, errors: [{ file: "unknown", error: String(e) }] });
    } finally {
      setGenerating(false);
    }
  }, [config, generateAudio, downloadFFmpeg]);

  const handleGenerateVideos = useCallback(async () => {
    if (!config?.savePath) return;
    setGenerating(true);
    setVideoResult(null);
    setProgress(null);
    try {
      const downloadResult = await downloadFFmpeg();
      if (!downloadResult.success) {
        throw new Error("FFmpeg 下载失败: " + downloadResult.message);
      }
      const res = await generateVideos(
        config.videoConfig as unknown as Record<string, unknown>,
        config.savePath
      );
      setVideoResult(res);
    } catch (e) {
      setVideoResult({ success: 0, failed: 1, errors: [{ file: "unknown", error: String(e) }] });
    } finally {
      setGenerating(false);
    }
  }, [config, generateVideos, downloadFFmpeg]);

  if (!config) {
    return <div className="app-container"><div style={{ padding: "24px" }}>加载中...</div></div>;
  }

  const isDisabled = generating || downloading || !ffmpegReady;

  return (
    <div className="app-container">
      <Header savePath={config.savePath ?? undefined} onPathChange={handlePathChange} />
      <TabBar active={activeTab} onChange={handleTabChange} disabled={isDisabled} />
      {!ffmpegReady && (
        <div className="ffmpeg-warning">
          ⚠️ 未检测到可用的 FFmpeg。macOS 可安装 Homebrew 后执行 <code>brew install ffmpeg</code>；Windows 需可写入用户目录且联网，以便首次生成时自动下载。
        </div>
      )}
      <div className="tab-content">
        {activeTab === "video" && (
          <VideoTab
            config={config.videoConfig}
            savePath={config.savePath ?? undefined}
            onConfigChange={handleVideoConfig}
            onGenerate={handleGenerateVideos}
            onEstimate={(c) => estimateSize("video", c)}
            generating={generating}
            disabled={!ffmpegReady || downloading}
          />
        )}
        {activeTab === "image" && (
          <ImageTab
            config={config.imageConfig}
            savePath={config.savePath ?? undefined}
            onConfigChange={handleImageConfig}
            onGenerate={handleGenerateImages}
            onEstimate={(c) => estimateSize("image", c)}
            generating={generating}
            disabled={!ffmpegReady || downloading}
          />
        )}
        {activeTab === "audio" && (
          <AudioTab
            config={config.audioConfig}
            savePath={config.savePath ?? undefined}
            onConfigChange={handleAudioConfig}
            onGenerate={handleGenerateAudio}
            onEstimate={(c) => estimateSize("audio", c)}
            generating={generating}
            disabled={!ffmpegReady || downloading}
          />
        )}
      </div>
      {generating && progress && (
        <ProgressPanel
          current={progress.current}
          total={progress.total}
          currentFile={progress.currentFile}
          estimatedRemainingSecs={progress.estimatedRemainingSecs}
          onCancel={cancelGeneration}
        />
      )}
      {activeTab === "video" && videoResult && <ResultSummary {...videoResult} />}
      {activeTab === "image" && imageResult && <ResultSummary {...imageResult} />}
      {activeTab === "audio" && audioResult && <ResultSummary {...audioResult} />}
    </div>
  );
}