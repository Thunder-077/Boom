<template>
  <section class="settings-card card-shell">
    <header class="settings-head">
      <div>
        <span class="section-kicker">更新</span>
        <h3>版本与更新</h3>
        <p class="current-version">当前版本：{{ currentVersion || "加载中..." }}</p>
      </div>
      <span class="theme-pill" :class="{ 'status-idle': updaterStatus === 'idle', 'status-available': updaterStatus === 'available', 'status-updating': isUpdating }">{{ statusLabel }}</span>
    </header>

    <div class="update-actions" v-if="updaterStatus === 'idle' || updaterStatus === 'error' || updaterStatus === 'up-to-date'">
      <button type="button" class="action-btn primary" @click="handleCheckUpdate" :disabled="isChecking">
        <span class="material-symbols-rounded">system_update</span>
        {{ isChecking ? "检查中..." : "检查更新" }}
      </button>
    </div>

    <div class="update-available" v-if="updaterStatus === 'available'">
      <div class="update-info">
        <h4>版本 {{ updateVersion }}</h4>
        <pre class="update-body">{{ updateBody }}</pre>
      </div>
      <div class="update-actions">
        <button type="button" class="action-btn primary" @click="handleInstall" :disabled="isUpdating">
          <span class="material-symbols-rounded">download</span>
          {{ isDownloading ? `下载中 ${progress}%` : "下载并安装" }}
        </button>
      </div>
    </div>

    <div class="update-downloading" v-if="updaterStatus === 'downloading'">
      <div class="progress-bar">
        <div class="progress-fill" :style="{ width: `${progress}%` }" />
      </div>
      <p>正在下载更新，请稍候...</p>
    </div>

    <div class="update-ready" v-if="updaterStatus === 'ready'">
      <p>更新已安装完成，正在重启应用...</p>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useAppUpdater } from "../../../shared/utils/appUpdater";

const {
  status: updaterStatus,
  progress,
  updateVersion,
  updateBody,
  currentVersion,
  statusLabel,
  checkForUpdate,
  downloadAndInstall,
} = useAppUpdater();

const isChecking = computed(() => updaterStatus.value === "checking");
const isDownloading = computed(() => updaterStatus.value === "downloading");
const isUpdating = computed(() => ["downloading", "ready"].includes(updaterStatus.value));

async function handleCheckUpdate() {
  await checkForUpdate();
}

async function handleInstall() {
  await downloadAndInstall();
}
</script>

<style scoped>
.settings-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
  padding: var(--space-xl);
  border-radius: var(--radius-card-large);
  background: var(--surface-nav-panel);
}

.settings-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-md);
}

.section-kicker {
  margin: 0;
  color: var(--text-tertiary);
  font-size: var(--font-size-xs);
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.settings-head h3 {
  margin: var(--space-sm) 0 var(--space-xs);
  font-size: var(--font-size-title-md);
  font-weight: 600;
  letter-spacing: -0.02em;
}

.settings-head p {
  margin: 0;
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
  line-height: 1.55;
}

.settings-head .current-version {
  color: var(--text-primary);
  font-weight: 600;
  font-size: var(--font-size-base);
  margin-top: var(--space-sm);
}

.theme-pill {
  display: inline-flex;
  align-items: center;
  min-height: 28px;
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-pill);
  font-size: var(--font-size-sm);
  font-weight: 600;
  background: var(--accent-soft);
  color: var(--accent-primary);
}

.theme-pill.status-available {
  background: var(--color-success-soft);
  color: var(--color-success);
}

.theme-pill.status-updating {
  background: var(--accent-soft);
  color: var(--accent-primary);
}

.update-actions {
  display: flex;
  gap: var(--space-md);
}

.action-btn {
  display: inline-flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-md) var(--space-lg);
  border: none;
  border-radius: var(--radius-sm);
  font-size: var(--font-size-base);
  font-weight: 600;
  cursor: pointer;
  transition: opacity var(--transition-base) var(--transition-ease);
}

.action-btn.primary {
  background: var(--accent-primary);
  color: var(--color-on-primary);
}

.action-btn.primary:hover:not(:disabled) {
  opacity: 0.9;
}

.action-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.action-btn .material-symbols-rounded {
  font-size: var(--font-size-2xl);
}

.update-available {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.update-info {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.update-info h4 {
  margin: 0;
  font-size: var(--font-size-xl);
  font-weight: 600;
}

.update-body {
  margin: 0;
  padding: var(--space-md);
  border-radius: var(--radius-md);
  background: var(--surface-panel);
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
}

.update-downloading {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.progress-bar {
  height: 8px;
  border-radius: var(--radius-xs);
  background: var(--surface-panel);
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  border-radius: var(--radius-xs);
  background: var(--accent-primary);
  transition: width var(--transition-slow) var(--transition-ease);
}

.update-downloading p,
.update-ready p {
  margin: 0;
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
}
</style>
