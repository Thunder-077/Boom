import { invoke } from "@tauri-apps/api/core";

export async function appendAppLog(level: string, scope: string, message: string) {
  try {
    await invoke("append_app_log", { level, scope, message });
  } catch {
    // Ignore logging failures to avoid recursive noise.
  }
}

export async function getAppLogPath() {
  try {
    return await invoke<string>("get_app_log_path");
  } catch {
    return "";
  }
}

export async function revealInExplorer(path: string) {
  await invoke("reveal_in_explorer", { path });
}
