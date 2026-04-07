import { createApp } from "vue";
import App from "./app/App.vue";
import { router } from "./app/router";
import { appendAppLog } from "./shared/utils/appLog";
import { initializeTheme } from "./shared/theme/theme";
import "./shared/styles/index.css";

const app = createApp(App);
initializeTheme();

app.config.errorHandler = (error, instance, info) => {
  const componentType = (instance as { type?: { name?: string } } | null)?.type;
  const componentName = componentType?.name ? String(componentType.name) : "unknown-component";
  void appendAppLog("error", "frontend.vue", `${info} | ${componentName} | ${String(error)}`);
};

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

void appendAppLog("info", "frontend.startup", "application boot");

app.use(router).mount("#app");
