import type { MediaType } from "../types";

interface Props {
  active: MediaType;
  onChange: (tab: MediaType) => void;
}

export default function TabBar({ active, onChange }: Props) {
  const tabs: { key: MediaType; label: string }[] = [
    { key: "image", label: "图片" },
    { key: "audio", label: "音频" },
    { key: "video", label: "视频" },
  ];

  return (
    <div className="tab-bar">
      {tabs.map((tab) => (
        <button
          key={tab.key}
          className={`tab-btn${active === tab.key ? " active" : ""}`}
          onClick={() => onChange(tab.key)}
        >
          {tab.label}
        </button>
      ))}
    </div>
  );
}
