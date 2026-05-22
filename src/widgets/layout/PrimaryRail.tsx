import type { RailItem } from "./types";

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
          <span className="material-symbols-rounded" aria-hidden="true">
            menu
          </span>
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
            <span className="icon material-symbols-rounded" aria-hidden="true">
              {item.icon}
            </span>
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
          <span className="icon material-symbols-rounded" aria-hidden="true">
            settings
          </span>
          <span className="sr-only">系统设置</span>
        </button>
      </div>
    </aside>
  );
}
