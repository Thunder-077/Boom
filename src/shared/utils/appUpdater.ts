import { useSyncExternalStore } from "react";
import { createStore } from "zustand/vanilla";
import { Channel, invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";

export type UpdateStatus = "idle" | "checking" | "available" | "downloading" | "ready" | "up-to-date" | "error";
export type UpdateChannel = "stable" | "canary";

interface UpdateMetadata {
  version: string;
  currentVersion: string;
  channel: UpdateChannel;
}

type DownloadEvent =
  | { event: "Started"; data: { contentLength?: number } }
  | { event: "Progress"; data: { chunkLength: number } }
  | { event: "Finished" };

interface AppUpdaterState {
  status: UpdateStatus;
  progress: number;
  downloadSize: number;
  errorMessage: string;
  updateVersion: string;
  currentVersion: string;
  channel: UpdateChannel;
}

const updaterStore = createStore<AppUpdaterState>(() => ({
  status: "idle",
  progress: 0,
  downloadSize: 0,
  errorMessage: "",
  updateVersion: "",
  currentVersion: "",
  channel: "stable",
}));

function detectChannel(version: string): UpdateChannel {
  return version.includes("-canary.") ? "canary" : "stable";
}

function statusLabel(state: AppUpdaterState) {
  switch (state.status) {
    case "idle":
      return "检查更新";
    case "checking":
      return "正在检查...";
    case "available":
      return `发现新版本 ${state.updateVersion}`;
    case "downloading":
      return `下载中 ${state.progress}%`;
    case "ready":
      return "更新完成，正在重启";
    case "up-to-date":
      return "当前已是最新版本";
    case "error":
      return state.errorMessage || "检查更新失败";
  }
}

async function initCurrentVersion() {
  try {
    const currentVersion = await getVersion();
    updaterStore.setState({
      currentVersion,
      channel: detectChannel(currentVersion),
    });
  } catch {
    updaterStore.setState({ currentVersion: "未知版本" });
  }
}

initCurrentVersion();

function reset() {
  updaterStore.setState({
    progress: 0,
    downloadSize: 0,
    errorMessage: "",
    updateVersion: "",
  });
}

async function checkForUpdate() {
  reset();
  updaterStore.setState({ status: "checking" });
  try {
    const update = await invoke<UpdateMetadata | null>("fetch_update");
    if (!update) {
      updaterStore.setState({ status: "up-to-date" });
      return false;
    }
    updaterStore.setState({
      updateVersion: update.version,
      currentVersion: update.currentVersion,
      channel: update.channel,
      status: "available",
    });
    return true;
  } catch (error) {
    updaterStore.setState({
      status: "error",
      errorMessage: error instanceof Error ? error.message : String(error),
    });
    return false;
  }
}

async function downloadAndInstall() {
  updaterStore.setState({ status: "downloading", progress: 0 });
  try {
    let downloadedSize = 0;
    const onEvent = new Channel<DownloadEvent>();
    onEvent.onmessage = (event) => {
      if (event.event === "Started") {
        updaterStore.setState({
          downloadSize: event.data.contentLength ?? 0,
          progress: 0,
        });
        downloadedSize = 0;
      } else if (event.event === "Progress") {
        const { downloadSize } = updaterStore.getState();
        if (downloadSize > 0) {
          downloadedSize += event.data.chunkLength;
          updaterStore.setState({
            progress: Math.min(99, Math.round((downloadedSize / downloadSize) * 100)),
          });
        }
      } else if (event.event === "Finished") {
        updaterStore.setState({ progress: 100 });
      }
    };

    await invoke("install_update", { onEvent });
    updaterStore.setState({ status: "ready" });
  } catch (error) {
    updaterStore.setState({
      status: "error",
      errorMessage: error instanceof Error ? error.message : String(error),
    });
  }
}

export function useReactAppUpdater() {
  const state = useSyncExternalStore(updaterStore.subscribe, updaterStore.getState, updaterStore.getInitialState);
  return {
    ...state,
    statusLabel: statusLabel(state),
    checkForUpdate,
    downloadAndInstall,
  };
}
