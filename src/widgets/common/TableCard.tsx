import type { ReactNode } from "react";

interface TableCardProps {
  title: string;
  meta?: string;
  description?: ReactNode;
  actions?: ReactNode;
  children?: ReactNode;
}

export default function TableCard({ title, meta, description, actions, children }: TableCardProps) {
  return (
    <section className="table-card card-shell">
      <header className="head">
        <div className="copy">
          <h3>{title}</h3>
          {description}
        </div>
        <div className="meta-wrap">
          {actions}
          {meta ? <p>{meta}</p> : null}
        </div>
      </header>
      <div className="content">{children}</div>
    </section>
  );
}
