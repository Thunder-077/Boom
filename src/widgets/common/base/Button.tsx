import type { ButtonHTMLAttributes, ReactNode } from "react";

type ButtonVariant = "primary" | "secondary" | "danger" | "ghost";
type ButtonSize = "sm" | "md" | "lg";

interface ButtonProps extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, "disabled"> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  disabled?: boolean;
  loading?: boolean;
  children?: ReactNode;
}

export default function Button({
  variant = "primary",
  size = "md",
  disabled = false,
  loading = false,
  className = "",
  children,
  type = "button",
  ...props
}: ButtonProps) {
  return (
    <button
      {...props}
      type={type}
      className={`btn btn-${variant} btn-${size} ${disabled ? "disabled" : ""} ${loading ? "loading" : ""} ${className}`}
      disabled={disabled || loading}
    >
      {loading ? <span className="btn-icon btn-spinner" aria-hidden="true" /> : null}
      <span className="btn-content">{children}</span>
    </button>
  );
}
