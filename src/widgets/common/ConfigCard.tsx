import type { ReactNode } from "react";

interface ConfigCardProps {
  title?: string;
  description?: string;
  className?: string;
  children?: ReactNode;
}

export default function ConfigCard({ title, description, className = "", children }: ConfigCardProps) {
  return (
    <section className={`config-card card-shell ${className}`.trim()}>
      {title ? <h3>{title}</h3> : null}
      {description ? <p>{description}</p> : null}
      <div className="body">{children}</div>
    </section>
  );
}
