import { useState, useCallback, useEffect } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { LogicalSize } from "@tauri-apps/api/dpi";
import Header from "./components/Header";
import TabBar from "./components/TabBar";
import ImageTab from "./components/ImageTab";
import AudioTab from "./components/AudioTab";
import VideoTab from "./components/VideoTab";
import MusicTab from "./components/MusicTab";
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
  const [musicResult, setMusicResult] = useState<TaskResult | null>(null);

  const {
    config,
    updateConfig,
    estimateSize,
    downloading,
    ffmpegReady,
    soundfontReady,
    hostOs,
    downloadFFmpeg,
    generateImages,
    generateAudio,
    generateVideos,
    generateMusic,
    cancelGeneration,
  } = useGenerator();

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    listen<ProgressPayload>("generation-progress", (event) => {
      setProgress(event.payload);
    }).then((fn) => { unlisten = fn; });
    return () => { unlisten?.(); };
  }, []);

  // 自适应窗口高度：1080p@150% 缩放时缩小，其他情况用全高
  useEffect(() => {
    const availH = window.screen.availHeight;
    const targetH = Math.min(700, availH - 48);
    getCurrentWindow().setSize(new LogicalSize(560, targetH));
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

  const handleMusicConfig = useCallback(
    (partial: Record<string, unknown>) => {
      if (!config) return;
      updateConfig({ ...config, musicConfig: { ...config.musicConfig, ...partial } });
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

  const handleGenerateMusic = useCallback(async () => {
    if (!config?.savePath) return;
    setGenerating(true);
    setMusicResult(null);
    setProgress(null);
    try {
      const downloadResult = await downloadFFmpeg();
      if (!downloadResult.success) {
        throw new Error("FFmpeg 下载失败: " + downloadResult.message);
      }
      const res = await generateMusic(
        config.musicConfig as unknown as Record<string, unknown>,
        config.savePath
      );
      setMusicResult(res);
    } catch (e) {
      setMusicResult({ success: 0, failed: 1, errors: [{ file: "unknown", error: String(e) }] });
    } finally {
      setGenerating(false);
    }
  }, [config, generateMusic, downloadFFmpeg]);

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
          {hostOs === "macos" ? (
            <>
              ⚠️ 未检测到可用的 FFmpeg。正式安装包一般已内置；若仍出现本提示，请确认使用完整 DMG 安装，或删除{" "}
              <code>~/Library/Application Support/Muse_Generator/ffmpeg</code> 后重开应用。无内置时可通过 Homebrew 安装：{" "}
              <code>brew install ffmpeg</code>。
            </>
          ) : hostOs === "windows" ? (
            <>
              ⚠️ 未检测到可用的 FFmpeg。请保证用户目录可写且网络可用，以便首次生成时自动下载；或将 FFmpeg 加入系统 PATH。
            </>
          ) : (
            <>⚠️ 未检测到可用的 FFmpeg。请在系统中安装 FFmpeg 并加入 PATH。</>
          )}
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
            soundfontReady={soundfontReady}
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
        {activeTab === "music" && (
          <MusicTab
            config={config.musicConfig}
            savePath={config.savePath ?? undefined}
            onConfigChange={handleMusicConfig}
            onGenerate={handleGenerateMusic}
            onEstimate={(c) => estimateSize("audio", c)}
            generating={generating}
            soundfontReady={soundfontReady}
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
      {activeTab === "music" && musicResult && <ResultSummary {...musicResult} />}
    </div>
  );
}