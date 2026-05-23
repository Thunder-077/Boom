import { closeAppDialog, useReactAppDialogState } from "../../shared/ui/appDialog";
import { Trash2, Check, X } from "lucide-react";

export default function AppDialogHost() {
  const state = useReactAppDialogState();

  if (!state.visible) {
    return null;
  }

  const DialogIcon = state.icon;

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
          <span className="dialog-icon" aria-hidden="true">
            <DialogIcon size={20} />
          </span>
          <div>
            <h3 id="app-dialog-title">{state.title}</h3>
            <p id="app-dialog-summary">{state.summary}</p>
          </div>
          <button className="dialog-close" type="button" aria-label="关闭弹窗" onClick={() => closeAppDialog(false)}>
            <X size={16} />
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
            {state.tone === "danger" ? <Trash2 size={16} /> : <Check size={16} />}
            {state.confirmText}
          </button>
        </footer>
      </section>
    </div>
  );
}
