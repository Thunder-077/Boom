import type { HTMLAttributes, ReactNode } from "react";

interface TagProps extends HTMLAttributes<HTMLSpanElement> {
  variant?: "default" | "primary" | "success" | "warning" | "danger" | "info";
  size?: "sm" | "md" | "lg";
  clickable?: boolean;
  active?: boolean;
  children?: ReactNode;
}

export default function Tag({
  variant = "default",
  size = "md",
  clickable = false,
  active = false,
  className = "",
  children,
  onClick,
  onKeyDown,
  ...props
}: TagProps) {
  return (
    <span
      {...props}
      className={`tag tag-${variant} tag-${size} ${clickable ? "clickable" : ""} ${active ? "active" : ""} ${className}`}
      tabIndex={clickable ? 0 : undefined}
      onClick={onClick}
      onKeyDown={(event) => {
        if (clickable && event.key === "Enter") {
          onClick?.(event as never);
        }
        onKeyDown?.(event);
      }}
    >
      {children}
    </span>
  );
}
