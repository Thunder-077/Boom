import type { SecondaryNavItem } from "./types";

interface SecondaryNavProps {
  title: string;
  description?: string;
  items: SecondaryNavItem[];
  activeKey: string;
  onSelect?: (key: string) => void;
}

export default function SecondaryNav({ title, description: _description, items, activeKey, onSelect }: SecondaryNavProps) {
  return (
    <aside className="secondary-nav">
      <div className="nav-content">
        <div className="nav-head">
          <h2 className="title">{title}</h2>
        </div>
        <div className="list">
          {items.map((item) => (
            <button
              key={item.key}
              type="button"
              className={`nav-item ${item.key === activeKey ? "active" : ""}`}
              onClick={() => onSelect?.(item.key)}
            >
              {item.icon ? (
                <span className="nav-icon material-symbols-rounded" aria-hidden="true">
                  {item.icon}
                </span>
              ) : (
                <span className="nav-icon placeholder" aria-hidden="true" />
              )}
              {item.label}
            </button>
          ))}
        </div>
      </div>
    </aside>
  );
}
