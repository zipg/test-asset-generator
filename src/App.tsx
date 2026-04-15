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
  const [activeTab, setActiveTab] = useState<MediaType>("image");
  const [generating, setGenerating] = useState(false);
  const [progress, setProgress] = useState<ProgressPayload | null>(null);
  const [result, setResult] = useState<TaskResult | null>(null);

  const {
    config,
    updateConfig,
    estimateSize,
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
    setResult(null);
    setProgress(null);
    try {
      await downloadFFmpeg();
      const res = await generateImages(
        config.imageConfig as unknown as Record<string, unknown>,
        config.savePath
      );
      setResult(res);
    } catch (e) {
      setResult({ success: 0, failed: 1, errors: [{ file: "unknown", error: String(e) }] });
    } finally {
      setGenerating(false);
    }
  }, [config, generateImages, downloadFFmpeg]);

  const handleGenerateAudio = useCallback(async () => {
    if (!config?.savePath) return;
    setGenerating(true);
    setResult(null);
    setProgress(null);
    try {
      await downloadFFmpeg();
      const res = await generateAudio(
        config.audioConfig as unknown as Record<string, unknown>,
        config.savePath
      );
      setResult(res);
    } catch (e) {
      setResult({ success: 0, failed: 1, errors: [{ file: "unknown", error: String(e) }] });
    } finally {
      setGenerating(false);
    }
  }, [config, generateAudio, downloadFFmpeg]);

  const handleGenerateVideos = useCallback(async () => {
    if (!config?.savePath) return;
    setGenerating(true);
    setResult(null);
    setProgress(null);
    try {
      await downloadFFmpeg();
      const res = await generateVideos(
        config.videoConfig as unknown as Record<string, unknown>,
        config.savePath
      );
      setResult(res);
    } catch (e) {
      setResult({ success: 0, failed: 1, errors: [{ file: "unknown", error: String(e) }] });
    } finally {
      setGenerating(false);
    }
  }, [config, generateVideos, downloadFFmpeg]);

  if (!config) {
    return <div className="app-container"><div style={{ padding: "24px" }}>加载中...</div></div>;
  }

  return (
    <div className="app-container">
      <Header savePath={config.savePath ?? undefined} onPathChange={handlePathChange} />
      <TabBar active={activeTab} onChange={setActiveTab} />
      <div className="tab-content">
        {activeTab === "image" && (
          <ImageTab
            config={config.imageConfig}
            savePath={config.savePath ?? undefined}
            onConfigChange={handleImageConfig}
            onGenerate={handleGenerateImages}
            onEstimate={(c) => estimateSize("image", c)}
            generating={generating}
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
          />
        )}
        {activeTab === "video" && (
          <VideoTab
            config={config.videoConfig}
            savePath={config.savePath ?? undefined}
            onConfigChange={handleVideoConfig}
            onGenerate={handleGenerateVideos}
            onEstimate={(c) => estimateSize("video", c)}
            generating={generating}
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
      {result && <ResultSummary {...result} />}
    </div>
  );
}
