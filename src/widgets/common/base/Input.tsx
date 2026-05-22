import type { InputHTMLAttributes, ReactNode } from "react";
import { useState } from "react";

interface InputProps extends Omit<InputHTMLAttributes<HTMLInputElement>, "size" | "prefix" | "onChange"> {
  value?: string;
  label?: string;
  prefix?: ReactNode;
  suffix?: ReactNode;
  size?: "sm" | "md" | "lg";
  error?: string;
  helpText?: string;
  onValueChange?: (value: string) => void;
}

export default function Input({
  value = "",
  label,
  prefix,
  suffix,
  size = "md",
  error = "",
  helpText = "",
  disabled = false,
  className = "",
  onValueChange,
  onFocus,
  onBlur,
  ...props
}: InputProps) {
  const [focused, setFocused] = useState(false);

  return (
    <label className={`input-wrapper input-${size} ${focused ? "focused" : ""} ${disabled ? "disabled" : ""} ${className}`}>
      {label ? <span className="input-label">{label}</span> : null}
      <div className="input-field-wrap">
        {prefix ? <span className="input-prefix">{prefix}</span> : null}
        <input
          {...props}
          className="input-field"
          value={value}
          disabled={disabled}
          onChange={(event) => onValueChange?.(event.target.value)}
          onFocus={(event) => {
            setFocused(true);
            onFocus?.(event);
          }}
          onBlur={(event) => {
            setFocused(false);
            onBlur?.(event);
          }}
        />
        {suffix ? <span className="input-suffix">{suffix}</span> : null}
      </div>
      {error ? <p className="input-error">{error}</p> : helpText ? <p className="input-help">{helpText}</p> : null}
    </label>
  );
}
