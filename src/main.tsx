import React from "react";
import ReactDOM from "react-dom/client";
import App from "./app/App";
import { appendAppLog } from "./shared/utils/appLog";
import { initializeTheme } from "./shared/theme/theme";
import "./shared/styles/index.css";

initializeTheme();

window.addEventListener("error", (event) => {
  void appendAppLog(
    "error",
    "frontend.window",
    `${event.message} @ ${event.filename || "unknown"}:${event.lineno || 0}:${event.colno || 0}`,
  );
});

window.addEventListener("unhandledrejection", (event) => {
  const reason =
    event.reason instanceof Error
      ? `${event.reason.name}: ${event.reason.message}\n${event.reason.stack || ""}`
      : String(event.reason);
  void appendAppLog("error", "frontend.promise", reason);
});

void appendAppLog("info", "frontend.startup", "react shell boot");

ReactDOM.createRoot(document.getElementById("app") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
