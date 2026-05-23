import { useEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { ChevronDown } from "lucide-react";

interface FluentSelectOption<T extends string | number> {
  label: string;
  value: T | "";
}

interface FluentSelectProps<T extends string | number> {
  modelValue: T | "";
  options: Array<FluentSelectOption<T>>;
  placeholder?: string;
  disabled?: boolean;
  searchable?: boolean;
  size?: "sm" | "md" | "lg";
  className?: string;
  onUpdateModelValue?: (value: T | "") => void;
  onChange?: (value: T | "") => void;
}

export default function FluentSelect<T extends string | number>({
  modelValue,
  options,
  placeholder = "请选择",
  disabled = false,
  searchable = false,
  size = "md",
  className = "",
  onUpdateModelValue,
  onChange,
}: FluentSelectProps<T>) {
  const [isOpen, setIsOpen] = useState(false);
  const [searchKeyword, setSearchKeyword] = useState("");
  const [highlightIndex, setHighlightIndex] = useState(-1);
  const [menuStyle, setMenuStyle] = useState<{ top: string; left: string; width: string }>({
    top: "0px",
    left: "0px",
    width: "auto",
  });
  const comboRef = useRef<HTMLDivElement | null>(null);
  const triggerRef = useRef<HTMLDivElement | null>(null);
  const menuRef = useRef<HTMLDivElement | null>(null);
  const searchInputRef = useRef<HTMLInputElement | null>(null);

  const filteredOptions = useMemo(() => {
    if (!searchable || !searchKeyword.trim()) {
      return options;
    }
    const keyword = searchKeyword.trim().toLowerCase();
    return options.filter((option) => option.label.toLowerCase().includes(keyword));
  }, [options, searchKeyword, searchable]);

  const displayLabel = useMemo(() => {
    if (modelValue === "" || modelValue === null || modelValue === undefined) {
      const defaultPlaceholderOption = options.find((option) => option.value === "");
      return defaultPlaceholderOption ? defaultPlaceholderOption.label : placeholder;
    }
    const selected = options.find((option) => option.value === modelValue);
    return selected ? selected.label : placeholder;
  }, [modelValue, options, placeholder]);

  const isPlaceholder = useMemo(() => {
    if (modelValue === "" || modelValue === null || modelValue === undefined) {
      return !options.find((option) => option.value === "");
    }
    return false;
  }, [modelValue, options]);

  useEffect(() => {
    if (searchable && modelValue !== "" && modelValue !== null && modelValue !== undefined) {
      const selected = options.find((option) => option.value === modelValue);
      if (selected) {
        setSearchKeyword(selected.label);
      }
      return;
    }
    if (searchable) {
      setSearchKeyword("");
    }
  }, [modelValue, options, searchable]);

  useEffect(() => {
    if (!isOpen) {
      setHighlightIndex(-1);
      if (!searchable) {
        setSearchKeyword("");
      }
      return;
    }

    function handleScrollOrResize(event: Event) {
      if (event.type === "scroll" && event.target === menuRef.current) {
        return;
      }
      setIsOpen(false);
    }

    function handleClickOutside(event: MouseEvent) {
      const target = event.target as Node;
      if (comboRef.current?.contains(target) || menuRef.current?.contains(target)) {
        return;
      }
      setIsOpen(false);
    }

    window.addEventListener("scroll", handleScrollOrResize, true);
    window.addEventListener("resize", handleScrollOrResize);
    document.addEventListener("mousedown", handleClickOutside);

    return () => {
      window.removeEventListener("scroll", handleScrollOrResize, true);
      window.removeEventListener("resize", handleScrollOrResize);
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [isOpen, searchable]);

  function updatePosition() {
    if (!triggerRef.current) {
      return;
    }
    const rect = triggerRef.current.getBoundingClientRect();
    const viewportHeight = window.innerHeight;
    const spaceBelow = viewportHeight - rect.bottom;
    const spaceAbove = rect.top;
    const estimatedHeight = Math.min(240, Math.max(spaceBelow, spaceAbove) - 12);

    setMenuStyle({
      top: spaceBelow < 240 && spaceAbove > spaceBelow
        ? `${rect.top - estimatedHeight - 6}px`
        : `${rect.bottom + 6}px`,
      left: `${rect.left}px`,
      width: `${rect.width}px`,
    });
  }

  function openCombo() {
    if (disabled || isOpen) {
      return;
    }
    updatePosition();
    setHighlightIndex(-1);
    setIsOpen(true);
    if (searchable) {
      window.setTimeout(() => {
        const input = searchInputRef.current;
        if (!input) {
          return;
        }
        input.focus();
        const len = input.value.length;
        input.setSelectionRange(len, len);
      }, 0);
    } else {
      comboRef.current?.focus();
    }
  }

  function closeCombo() {
    setIsOpen(false);
  }

  function toggleCombo() {
    if (disabled) {
      return;
    }
    if (isOpen) {
      closeCombo();
    } else {
      openCombo();
    }
  }

  function selectOption(value: T | "") {
    if (disabled) {
      return;
    }
    if (searchable) {
      const selected = options.find((option) => option.value === value);
      setSearchKeyword(selected ? selected.label : "");
    }
    onUpdateModelValue?.(value);
    onChange?.(value);
    closeCombo();
  }

  function scrollHighlightedIntoView() {
    window.setTimeout(() => {
      const highlighted = menuRef.current?.querySelector(".fluent-option.highlighted") as HTMLElement | null;
      highlighted?.scrollIntoView({ block: "nearest" });
    }, 0);
  }

  function navigateOptions(direction: number) {
    const len = filteredOptions.length;
    if (len === 0) {
      return;
    }
    setHighlightIndex((current) => {
      const next = current === -1 ? (direction > 0 ? 0 : len - 1) : (current + direction + len) % len;
      window.setTimeout(scrollHighlightedIntoView, 0);
      return next;
    });
  }

  function selectHighlighted() {
    if (highlightIndex >= 0 && highlightIndex < filteredOptions.length) {
      selectOption(filteredOptions[highlightIndex].value);
    }
  }

  const comboClasses = `fluent-combo ${isOpen ? "open" : ""} ${disabled ? "disabled" : ""} ${size !== "md" ? `size-${size}` : ""} ${className}`.trim();

  return (
    <div
      className={comboClasses}
      tabIndex={searchable ? -1 : 0}
      onKeyDown={(event) => {
        if (event.key === "Escape") {
          event.preventDefault();
          closeCombo();
        }
      }}
      ref={comboRef}
    >
      {searchable ? (
        <div
          className="fluent-trigger fluent-searchable-trigger"
          ref={triggerRef}
          onMouseDown={(event) => {
            event.preventDefault();
            if (!isOpen) {
              openCombo();
              return;
            }
            const input = searchInputRef.current;
            if (!input) {
              return;
            }
            input.focus();
            const len = input.value.length;
            input.setSelectionRange(len, len);
          }}
        >
          <input
            ref={searchInputRef}
            className="fluent-searchable-input"
            value={searchKeyword}
            placeholder={isPlaceholder ? placeholder : ""}
            onChange={(event) => {
              setSearchKeyword(event.target.value);
              setHighlightIndex(-1);
              if (!isOpen) {
                openCombo();
              }
            }}
            onKeyDown={(event) => {
              if (event.key === "Escape") {
                event.preventDefault();
                closeCombo();
              }
              if (event.key === "ArrowDown") {
                event.preventDefault();
                event.stopPropagation();
                navigateOptions(1);
              }
              if (event.key === "ArrowUp") {
                event.preventDefault();
                event.stopPropagation();
                navigateOptions(-1);
              }
              if (event.key === "Enter") {
                event.preventDefault();
                event.stopPropagation();
                selectHighlighted();
              }
            }}
          />
          <span className="combo-icon" aria-hidden="true">
            <ChevronDown size={18} />
          </span>
        </div>
      ) : (
        <div className="fluent-trigger" ref={triggerRef} onMouseDown={(event) => {
          event.preventDefault();
          toggleCombo();
        }}>
          <span className={`fluent-value ${isPlaceholder ? "placeholder" : ""}`.trim()}>
            {displayLabel}
          </span>
          <span className="combo-icon" aria-hidden="true">
            <ChevronDown size={18} />
          </span>
        </div>
      )}

      {createPortal(
        isOpen ? (
          <div className="teleported-fluent-menu" style={menuStyle} ref={menuRef}>
            {filteredOptions.map((option, index) => (
              <button
                key={String(option.value)}
                type="button"
                className={`fluent-option ${option.value === modelValue ? "selected" : ""} ${index === highlightIndex ? "highlighted" : ""}`.trim()}
                onClick={() => selectOption(option.value)}
              >
                {option.label}
              </button>
            ))}
            {filteredOptions.length === 0 ? (
              <div className="menu-empty">无匹配选项</div>
            ) : null}
          </div>
        ) : null,
        document.body,
      )}
    </div>
  );
}
