import { closeAppDialog, useReactAppDialogState } from "../../shared/ui/appDialog";

export default function AppDialogHost() {
  const state = useReactAppDialogState();

  if (!state.visible) {
    return null;
  }

  return (
    <div
      className="app-dialog-mask"
      onClick={(event) => {
        if (event.target === event.currentTarget) {
          closeAppDialog(false);
        }
      }}
    >
      <section
        className={`app-dialog tone-${state.tone}`}
        role="dialog"
        aria-modal="true"
        aria-labelledby="app-dialog-title"
        aria-describedby="app-dialog-summary"
        tabIndex={-1}
      >
        <header className="app-dialog-head">
          <span className="dialog-icon material-symbols-rounded" aria-hidden="true">{state.icon}</span>
          <div>
            <h3 id="app-dialog-title">{state.title}</h3>
            <p id="app-dialog-summary">{state.summary}</p>
          </div>
          <button className="dialog-close" type="button" aria-label="关闭弹窗" onClick={() => closeAppDialog(false)}>
            <span className="material-symbols-rounded" aria-hidden="true">close</span>
          </button>
        </header>

        {state.details.length > 0 ? (
          <ul className="dialog-details">
            {state.details.map((line, index) => (
              <li key={`${index}-${line}`}>{line}</li>
            ))}
          </ul>
        ) : null}

        <footer className="dialog-actions">
          {state.kind === "confirm" ? (
            <button className="dialog-btn secondary" type="button" onClick={() => closeAppDialog(false)}>
              {state.cancelText}
            </button>
          ) : null}
          <button className="dialog-btn primary" type="button" autoFocus onClick={() => closeAppDialog(true)}>
            <span className="material-symbols-rounded" aria-hidden="true">{state.tone === "danger" ? "delete" : "check"}</span>
            {state.confirmText}
          </button>
        </footer>
      </section>
    </div>
  );
}
