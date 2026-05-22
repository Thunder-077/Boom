import type { ReactNode } from "react";

interface EmptyStateProps {
  icon?: string;
  title: string;
  description?: string;
  children?: ReactNode;
}

export default function EmptyState({ icon = "inventory_2", title, description, children }: EmptyStateProps) {
  return (
    <div className="empty-state">
      <span className="empty-icon material-symbols-rounded" aria-hidden="true">
        {icon}
      </span>
      <h4 className="empty-title">{title}</h4>
      {description ? <p className="empty-description">{description}</p> : null}
      {children ? <div className="empty-actions">{children}</div> : null}
    </div>
  );
}
