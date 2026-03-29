<template>
  <header class="window-titlebar" :class="{ unfocused: !isFocused }">
    <div class="drag-zone" data-tauri-drag-region>
      <img :src="appIcon" class="app-icon" alt="应用图标" />
      <span class="app-title">Boom</span>
    </div>
    <div class="window-controls">
      <button class="win-btn" type="button" aria-label="最小化窗口" title="最小化" @click="minimizeWindow">
        <span class="material-symbols-rounded" aria-hidden="true">remove</span>
      </button>
      <button class="win-btn" type="button" :aria-label="isMaximized ? '还原窗口' : '最大化窗口'" :title="isMaximized ? '还原' : '最大化'" @click="toggleMaximize">
        <span class="material-symbols-rounded" aria-hidden="true">{{ isMaximized ? "filter_none" : "crop_square" }}</span>
      </button>
      <button class="win-btn close" type="button" aria-label="关闭窗口" title="关闭" @click="closeWindow">
        <span class="material-symbols-rounded" aria-hidden="true">close</span>
      </button>
    </div>
  </header>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import appIcon from "../../assets/app-icon.png";

const appWindow = getCurrentWindow();
const isFocused = ref(true);
const isMaximized = ref(false);
let unlistenResized: (() => void) | null = null;
let unlistenFocusChanged: (() => void) | null = null;

async function refreshMaxState() {
  try {
    isMaximized.value = await appWindow.isMaximized();
  } catch {
    isMaximized.value = false;
  }
}

async function minimizeWindow() {
  try {
    await appWindow.minimize();
  } catch (error) {
    console.error("[window-titlebar] minimize failed", error);
  }
}

async function toggleMaximize() {
  try {
    await appWindow.toggleMaximize();
    await refreshMaxState();
  } catch (error) {
    console.error("[window-titlebar] toggle maximize failed", error);
  }
}

async function closeWindow() {
  try {
    await appWindow.close();
  } catch (error) {
    console.error("[window-titlebar] close failed", error);
  }
}

onMounted(async () => {
  await refreshMaxState();
  unlistenResized = await appWindow.onResized(() => {
    void refreshMaxState();
  });
  unlistenFocusChanged = await appWindow.onFocusChanged((event) => {
    isFocused.value = event.payload;
  });
});

onBeforeUnmount(() => {
  if (unlistenResized) {
    unlistenResized();
    unlistenResized = null;
  }
  if (unlistenFocusChanged) {
    unlistenFocusChanged();
    unlistenFocusChanged = null;
  }
});
</script>

<style scoped>
.window-titlebar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 200000;
  height: 38px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-left: 12px;
  border-bottom: 1px solid #d7e3f4;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.9), rgba(246, 251, 255, 0.85));
  user-select: none;
}

.window-titlebar.unfocused {
  opacity: 0.96;
}

.drag-zone {
  min-width: 0;
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
  height: 100%;
}

.app-icon {
  width: 24px;
  height: 24px;
  object-fit: contain;
  flex-shrink: 0;
}

.app-title {
  color: #1a2740;
  /* font-family: "Luckiest Guy", "Inter", "Segoe UI Variable", "PingFang SC", "Microsoft YaHei", sans-serif; */
  font-family: "Bangers", "Inter", "Segoe UI Variable", "PingFang SC", "Microsoft YaHei", sans-serif;
  font-size: 18px;
  font-weight: 400;
  letter-spacing: 0.02em;
  line-height: 1;
  white-space: nowrap;
}

.app-subtitle {
  color: #6b7d95;
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
}

.window-controls {
  display: flex;
  align-items: stretch;
  height: 100%;
}

.win-btn {
  width: 46px;
  border: 0;
  padding: 0;
  margin: 0;
  background: transparent;
  color: #4d6079;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.14s ease, color 0.14s ease;
}

.win-btn .material-symbols-rounded {
  font-family: "Material Symbols Rounded";
  font-size: 18px;
}

.win-btn:hover {
  background: rgba(15, 108, 189, 0.12);
  color: #0f6cbd;
}

.win-btn.close:hover {
  background: #d13438;
  color: #fff;
}

.win-btn:active {
  filter: brightness(0.96);
}

@media (max-width: 900px) {
  .app-subtitle {
    display: none;
  }
}
</style>
