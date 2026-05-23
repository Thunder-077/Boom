import React from "react";
import { Menu, Settings } from "lucide-react";
import type { RailItem } from "./types";
import { RAIL_ICONS } from "./types";

interface PrimaryRailProps {
  items: RailItem[];
  activeKey: string;
  isSecondaryNavVisible: boolean;
  isSettingsActive?: boolean;
  onSelect?: (key: string) => void;
  onToggleSecondaryNav?: () => void;
  onOpenSettings?: () => void;
}

export default function PrimaryRail({
  items,
  activeKey,
  isSecondaryNavVisible,
  isSettingsActive = false,
  onSelect,
  onToggleSecondaryNav,
  onOpenSettings,
}: PrimaryRailProps) {
  const toggleLabel = isSecondaryNavVisible ? "收起二级菜单" : "展开二级菜单";

  return (
    <aside className="primary-rail">
      <div className="rail-top">
        <button
          type="button"
          className="toggle-btn"
          aria-label={toggleLabel}
          aria-pressed={isSecondaryNavVisible}
          data-tooltip={toggleLabel}
          onClick={onToggleSecondaryNav}
        >
          <Menu size={20} />
        </button>
      </div>
      <div className="nav-group">
        {items.map((item) => (
          <button
            key={item.key}
            type="button"
            className={`rail-btn ${item.key === activeKey ? "active" : ""}`}
            aria-label={item.label}
            data-tooltip={item.label}
            onClick={() => onSelect?.(item.key)}
          >
            {React.createElement(RAIL_ICONS[item.icon], { size: 20 })}
            <span className="sr-only">{item.label}</span>
          </button>
        ))}
      </div>
      <div className="nav-bottom">
        <button
          type="button"
          className={`rail-btn ${isSettingsActive ? "active" : ""}`}
          aria-label="打开系统设置"
          data-tooltip="系统设置"
          onClick={onOpenSettings}
        >
          <Settings size={20} />
          <span className="sr-only">系统设置</span>
        </button>
      </div>
    </aside>
  );
}
