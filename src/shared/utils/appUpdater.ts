import { computed, ref } from "vue";
import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";
import { getVersion } from "@tauri-apps/api/app";

export type UpdateStatus = "idle" | "checking" | "available" | "downloading" | "ready" | "up-to-date" | "error";

const status = ref<UpdateStatus>("idle");
const progress = ref(0);
const downloadSize = ref(0);
const errorMessage = ref("");
const updateVersion = ref("");
const updateBody = ref("");
const currentVersion = ref("");

async function initCurrentVersion() {
  try {
    currentVersion.value = await getVersion();
  } catch {
    currentVersion.value = "未知版本";
  }
}

initCurrentVersion();

function reset() {
  progress.value = 0;
  downloadSize.value = 0;
  errorMessage.value = "";
  updateVersion.value = "";
  updateBody.value = "";
}

async function checkForUpdate() {
  reset();
  status.value = "checking";
  try {
    const update = await check();
    if (!update) {
      status.value = "up-to-date";
      return false;
    }
    updateVersion.value = update.version;
    updateBody.value = update.body ?? "";
    downloadSize.value = (update as any).contentLength ?? 0;
    status.value = "available";
    return true;
  } catch (error) {
    status.value = "error";
    errorMessage.value = error instanceof Error ? error.message : String(error);
    return false;
  }
}

async function downloadAndInstall() {
  status.value = "downloading";
  progress.value = 0;
  try {
    let downloadedSize = 0;
    const update = await check();
    if (!update) {
      status.value = "error";
      errorMessage.value = "未检测到可用更新";
      return;
    }
    await update.downloadAndInstall((event) => {
      if (event.event === "Started") {
        downloadSize.value = event.data.contentLength ?? 0;
        downloadedSize = 0;
      } else if (event.event === "Progress") {
        if (downloadSize.value > 0) {
          downloadedSize += event.data.chunkLength;
          progress.value = Math.min(99, Math.round((downloadedSize / downloadSize.value) * 100));
        }
      } else if (event.event === "Finished") {
        progress.value = 100;
      }
    });
    status.value = "ready";
    await relaunch();
  } catch (error) {
    status.value = "error";
    errorMessage.value = error instanceof Error ? error.message : String(error);
  }
}

const statusLabel = computed(() => {
  switch (status.value) {
    case "idle":
      return "检查更新";
    case "checking":
      return "正在检查...";
    case "available":
      return `发现新版本 ${updateVersion.value}`;
    case "downloading":
      return `下载中 ${progress.value}%`;
    case "ready":
      return "更新完成，正在重启";
    case "up-to-date":
      return "当前已是最新版本";
    case "error":
      return errorMessage.value || "检查更新失败";
  }
});

export function useAppUpdater() {
  return {
    status,
    progress,
    updateVersion,
    updateBody,
    currentVersion,
    statusLabel,
    checkForUpdate,
    downloadAndInstall,
  };
}
