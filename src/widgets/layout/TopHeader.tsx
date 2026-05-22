import type { ReactNode } from "react";

interface TopHeaderProps {
  breadcrumb: string;
  title: string;
  summary?: string;
  compact?: boolean;
  actions?: ReactNode;
}

export default function TopHeader({ breadcrumb, title, summary, compact = false, actions }: TopHeaderProps) {
  return (
    <header className={`header ${compact ? "compact" : ""}`}>
      <div className="left">
        <p className="crumb">{breadcrumb}</p>
        <h1 className="title">{title}</h1>
        {summary ? <p className="summary">{summary}</p> : null}
      </div>
      <div className="right">{actions}</div>
    </header>
  );
}
