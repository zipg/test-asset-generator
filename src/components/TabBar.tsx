import type { MediaType } from "../types";

interface Props {
  active: MediaType;
  onChange: (tab: MediaType) => void;
  disabled?: boolean;
}

export default function TabBar({ active, onChange, disabled }: Props) {
  const tabs: { key: MediaType; label: string }[] = [
    { key: "video", label: "视频" },
    { key: "image", label: "图片" },
    { key: "audio", label: "音频" },
  ];

  return (
    <div className="tab-bar">
      {tabs.map((tab) => (
        <button
          key={tab.key}
          className={`tab-btn${active === tab.key ? " active" : ""}${disabled ? " disabled" : ""}`}
          onClick={() => !disabled && onChange(tab.key)}
          disabled={disabled}
        >
          {tab.label}
        </button>
      ))}
    </div>
  );
}