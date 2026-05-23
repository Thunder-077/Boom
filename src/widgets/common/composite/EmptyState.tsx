import type { ReactNode } from "react";
import type { LucideIcon } from "lucide-react";
import { Package } from "lucide-react";

interface EmptyStateProps {
  icon?: LucideIcon;
  title: string;
  description?: string;
  children?: ReactNode;
}

export default function EmptyState({ icon: Icon = Package, title, description, children }: EmptyStateProps) {
  return (
    <div className="empty-state">
      <span className="empty-icon" aria-hidden="true">
        <Icon size={48} />
      </span>
      <h4 className="empty-title">{title}</h4>
      {description ? <p className="empty-description">{description}</p> : null}
      {children ? <div className="empty-actions">{children}</div> : null}
    </div>
  );
}
