import type { ReactNode } from "react";

interface ToolbarItem {
  label: string;
  value: string;
}

interface FilterToolbarProps {
  items?: ToolbarItem[];
  children?: ReactNode;
}

export default function FilterToolbar({ items = [], children }: FilterToolbarProps) {
  return (
    <section className="toolbar card-shell">
      {children ?? items.map((item) => (
        <div key={item.label} className="chip">
          <span>{item.label}</span>
          <span className="value">{item.value}</span>
        </div>
      ))}
    </section>
  );
}
