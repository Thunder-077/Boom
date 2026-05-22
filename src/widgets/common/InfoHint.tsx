interface InfoHintProps {
  text?: string;
  type?: "info" | "success" | "warning" | "error";
  linkLabel?: string;
  suffix?: string;
  className?: string;
  onClickLink?: () => void;
}

export default function InfoHint({
  text,
  type = "info",
  linkLabel,
  suffix,
  className = "",
  onClickLink,
}: InfoHintProps) {
  return (
    <section className={`hint ${type} ${className}`.trim()}>
      <span className="icon-wrap" aria-hidden="true">
        <svg className="dot" viewBox="0 0 24 24">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z" />
        </svg>
      </span>
      <p className="hint-text">
        {text ? <span>{text}</span> : null}
        {linkLabel ? (
          <a
            className="hint-link"
            href="#"
            onClick={(event) => {
              event.preventDefault();
              onClickLink?.();
            }}
          >
            {linkLabel}
          </a>
        ) : null}
        {suffix ? <span>{suffix}</span> : null}
      </p>
    </section>
  );
}
