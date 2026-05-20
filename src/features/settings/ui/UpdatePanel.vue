<template>
  <section
    class="settings-card card-shell update-card"
    :class="{ 'state-highlight': isHighlightState }"
  >
    <!-- 状态 1：默认状态 (空闲 / 待检查 / 检查中 / 错误) -->
    <template v-if="updaterStatus === 'idle' || updaterStatus === 'checking' || updaterStatus === 'error'">
      <div class="card-left">
        <div class="icon-box icon-default">
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="2" y="3" width="20" height="14" rx="2" ry="2"></rect>
            <line x1="8" y1="21" x2="16" y2="21"></line>
            <line x1="12" y1="17" x2="12" y2="21"></line>
          </svg>
        </div>
        <div class="text-content">
          <h2 class="main-title">系统版本与更新</h2>
          <p class="sub-text">
            当前版本：<span class="version-tag">{{ currentVersion || "加载中..." }}</span>
            <span v-if="updaterStatus === 'error'" class="error-badge" :title="errorMessage">
              (检查失败: {{ errorMessage }})
            </span>
          </p>
        </div>
      </div>
      <button class="btn btn-primary" @click="handleCheckUpdate" :disabled="isChecking">
        <svg :class="{ 'rotating': isChecking }" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="23 4 23 10 17 10"></polyline>
          <polyline points="1 20 1 14 7 14"></polyline>
          <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
        </svg>
        {{ isChecking ? "正在检查..." : "检查更新" }}
      </button>
    </template>

    <!-- 状态 2：已是最新状态 (安全 / 无需操作) -->
    <template v-else-if="updaterStatus === 'up-to-date'">
      <div class="card-left">
        <div class="icon-box icon-success">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
            <polyline points="22 4 12 14.01 9 11.01"></polyline>
          </svg>
        </div>
        <div class="text-content">
          <h2 class="main-title">当前已是最新版本</h2>
          <p class="sub-text">系统版本 <span class="version-tag">{{ currentVersion }}</span></p>
        </div>
      </div>
      <button class="btn btn-ghost" @click="handleCheckUpdate" :disabled="isChecking">
        <svg :class="{ 'rotating': isChecking }" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="23 4 23 10 17 10"></polyline>
          <polyline points="1 20 1 14 7 14"></polyline>
          <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
        </svg>
        {{ isChecking ? "正在检查..." : "再次检查" }}
      </button>
    </template>

    <!-- 状态 3：发现全新版本 (强引导 / 全新设计 / 包括下载中与准备就绪) -->
    <template v-else-if="updaterStatus === 'available' || updaterStatus === 'downloading' || updaterStatus === 'ready'">
      <div class="card-left">
        <div class="icon-box icon-highlight">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.71-2.13 0-2.97a2.121 2.121 0 0 0-2.97 0z"></path>
            <path d="M15 15c1.46 3 3 3 3 3"></path>
            <path d="M9 9c3-1.46 3-3 3-3"></path>
            <path d="M12 12c4 4 10 4 10 4s0-6-4-10-10-4-10-4 0 6 4 10z"></path>
          </svg>
        </div>
        <div class="text-content">
          <h2 class="main-title">
            <span v-if="updaterStatus === 'available'">发现全新版本！</span>
            <span v-else-if="updaterStatus === 'downloading'">正在下载全新版本...</span>
            <span v-else-if="updaterStatus === 'ready'">更新已准备就绪！</span>
            <span class="version-tag new-version-tag">{{ updateVersion }}</span>
          </h2>
          <div v-if="updaterStatus === 'downloading'" class="download-progress-area">
            <div class="progress-bar">
              <div class="progress-fill" :style="{ width: `${progress}%` }"></div>
            </div>
            <p class="progress-info">正在下载更新，请稍候... {{ progress }}%</p>
          </div>
          <p v-else class="sub-text">
            <template v-if="updaterStatus === 'available'">
              当前版本 <span class="version-tag old-version">{{ currentVersion }}</span>
            </template>
            <template v-else-if="updaterStatus === 'ready'">
              新版本已下载完成，即将自动重启应用以应用更新
            </template>
          </p>
        </div>
      </div>
      <div class="card-right">
        <button v-if="updaterStatus === 'available'" class="btn btn-gradient" @click="handleInstall">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
            <polyline points="7 10 12 15 17 10"></polyline>
            <line x1="12" y1="15" x2="12" y2="3"></line>
          </svg>
          立即更新系统
        </button>
        <button v-else-if="updaterStatus === 'downloading'" class="btn btn-downloading" disabled>
          <span class="btn-progress-fill" :style="{ width: `${progress}%` }"></span>
          <span class="btn-progress-content">
            <svg class="rotating" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="23 4 23 10 17 10"></polyline>
              <polyline points="1 20 1 14 7 14"></polyline>
              <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
            </svg>
            下载中 {{ progress }}%
          </span>
        </button>
        <button v-else-if="updaterStatus === 'ready'" class="btn btn-gradient btn-relaunch" disabled>
          <svg class="rotating" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="23 4 23 10 17 10"></polyline>
            <polyline points="1 20 1 14 7 14"></polyline>
            <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
          </svg>
          正在重启...
        </button>
      </div>
    </template>
  </section>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useAppUpdater } from "../../../shared/utils/appUpdater";

const {
  status: updaterStatus,
  progress,
  updateVersion,
  currentVersion,
  errorMessage,
  checkForUpdate,
  downloadAndInstall,
} = useAppUpdater();

const isChecking = computed(() => updaterStatus.value === "checking");
const isHighlightState = computed(() =>
  ["available", "downloading", "ready"].includes(updaterStatus.value)
);

async function handleCheckUpdate() {
  await checkForUpdate();
}

async function handleInstall() {
  await downloadAndInstall();
}
</script>

<style scoped>
/* ================= 核心卡片通用样式 ================= */
.update-card {
  width: 100%;
  background-color: var(--surface-panel);
  border-radius: var(--radius-card-large);
  padding: var(--space-xl) var(--space-2xl);
  display: flex;
  justify-content: space-between;
  align-items: center;
  box-shadow: var(--shadow-soft);
  border: 1px solid var(--border-default);
  box-sizing: border-box;
  transition: all 0.3s var(--transition-ease);
}

.update-card:not(.state-highlight):hover {
  box-shadow: var(--shadow-medium);
  border-color: var(--border-strong);
  transform: translateY(-1px);
}

/* ================= 左侧信息区通用样式 ================= */
.card-left {
  display: flex;
  align-items: center;
  gap: var(--space-xl);
}

.icon-box {
  width: 52px;
  height: 52px;
  border-radius: var(--radius-lg);
  display: flex;
  justify-content: center;
  align-items: center;
  flex-shrink: 0;
}

.icon-default {
  background-color: var(--accent-fill-soft);
  color: var(--text-secondary);
}

.text-content {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.main-title {
  font-size: var(--font-size-2xl);
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
  display: flex;
  align-items: center;
  gap: var(--space-md);
  line-height: 1.2;
}

.sub-text {
  font-size: var(--font-size-base);
  color: var(--text-secondary);
  margin: 0;
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  line-height: 1.4;
}

.version-tag {
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
  background-color: var(--accent-fill-soft);
  color: var(--text-primary);
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  font-weight: 600;
  letter-spacing: 0.5px;
}

/* ================= 右侧按钮通用样式 ================= */
.btn {
  border-radius: var(--radius-md);
  padding: 10px 24px;
  font-size: var(--font-size-base);
  font-weight: 600;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  transition: all 0.2s var(--transition-ease);
  border: 1px solid transparent;
  white-space: nowrap;
  user-select: none;
}

.btn:active:not(:disabled) {
  transform: scale(0.97);
}

.btn:disabled {
  cursor: not-allowed;
  opacity: 0.8;
}

/* --- 按钮变体：主操作按钮 (Primary) --- */
.btn-primary {
  background-color: var(--accent-primary);
  color: var(--color-on-primary);
  box-shadow: 0 2px 6px rgba(var(--accent-rgb), 0.2);
}

.btn-primary:hover:not(:disabled) {
  background-color: var(--accent-primary-strong);
  box-shadow: 0 4px 12px rgba(var(--accent-strong-rgb), 0.25);
}

/* --- 按钮变体：次操作按钮 (Ghost) --- */
.btn-ghost {
  background-color: var(--surface-panel);
  color: var(--text-secondary);
  border-color: var(--border-default);
}

.btn-ghost:hover:not(:disabled) {
  background-color: var(--accent-fill-soft);
  color: var(--text-primary);
  border-color: var(--border-strong);
}

/* ================= 状态 2: 成功特有样式 ================= */
.icon-success {
  background-color: var(--color-success-soft);
  color: var(--color-success);
}

/* ================= 状态 3: 发现新版特有样式 ================= */
.update-card.state-highlight {
  border-color: var(--accent-border-strong);
  background: radial-gradient(circle at top left, var(--surface-panel) 0%, var(--accent-soft) 100%);
  box-shadow: 0 4px 20px -4px rgba(var(--accent-rgb), 0.15);
}

.update-card.state-highlight:hover {
  box-shadow: 0 10px 30px -4px rgba(var(--accent-rgb), 0.25);
  transform: translateY(-2px);
}

.icon-highlight {
  background: linear-gradient(135deg, var(--accent-softer) 0%, var(--accent-soft) 100%);
  color: var(--accent-primary);
  border: 1px solid var(--accent-border-soft);
  position: relative;
}

/* 让火箭图标产生轻微的悬浮动效 */
@keyframes float {
  0%, 100% { transform: translateY(0px) rotate(0deg); }
  50% { transform: translateY(-3px) rotate(2deg); }
}

.icon-highlight svg {
  animation: float 3s ease-in-out infinite;
}

/* 新版本号标签的高亮发光设计 */
.new-version-tag {
  background: linear-gradient(135deg, var(--accent-primary) 0%, var(--accent-primary-strong) 100%);
  color: var(--color-on-primary);
  border: none;
  box-shadow: 0 2px 8px rgba(var(--accent-rgb), 0.3);
  padding: 4px 10px;
  position: relative;
  overflow: hidden;
}

/* 新版本标签的一道扫光动画 */
.new-version-tag::after {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 50%;
  height: 100%;
  background: linear-gradient(
    to right,
    rgba(255, 255, 255, 0) 0%,
    rgba(255, 255, 255, 0.3) 50%,
    rgba(255, 255, 255, 0) 100%
  );
  animation: shimmer 3s infinite;
}

@keyframes shimmer {
  0% { left: -100%; }
  20% { left: 200%; }
  100% { left: 200%; }
}

/* 立体渐变操作按钮 */
.btn-gradient {
  background: linear-gradient(135deg, var(--accent-primary) 0%, var(--accent-primary-strong) 100%);
  color: var(--color-on-primary);
  border: none;
  box-shadow: 0 4px 14px 0 rgba(var(--accent-rgb), 0.3), inset 0 1px 0 0 rgba(255, 255, 255, 0.2);
}

.btn-gradient:hover:not(:disabled) {
  background: linear-gradient(135deg, var(--accent-primary-strong) 0%, var(--accent-primary) 100%);
  box-shadow: 0 6px 20px 0 rgba(var(--accent-rgb), 0.4), inset 0 1px 0 0 rgba(255, 255, 255, 0.3);
  transform: translateY(-1px);
}

.arrow-icon {
  color: var(--border-strong);
  stroke-width: 3px;
}

.old-version {
  color: var(--text-secondary);
  background-color: transparent;
  padding: 0;
}

/* 旋转动画 */
@keyframes rotate {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.rotating {
  animation: rotate 1.5s linear infinite;
  transform-origin: center;
}

/* 下载中按钮样式（带局部百分比填充效果） */
.btn-downloading {
  position: relative;
  background: var(--accent-fill-soft);
  color: var(--text-secondary);
  border: 1px solid var(--accent-border-soft);
  box-shadow: none;
  overflow: hidden;
}

.btn-progress-fill {
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
  background: var(--accent-primary);
  opacity: 0.15;
  transition: width 0.3s ease;
}

.btn-progress-content {
  position: relative;
  z-index: 1;
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

/* 左侧文本区域下方的进度条 */
.download-progress-area {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
  margin-top: var(--space-xs);
  width: 240px;
}

.download-progress-area .progress-bar {
  height: 6px;
  border-radius: var(--radius-pill);
  background: var(--accent-fill-soft);
  overflow: hidden;
}

.download-progress-area .progress-fill {
  height: 100%;
  border-radius: var(--radius-pill);
  background: var(--accent-primary);
  transition: width 0.3s var(--transition-ease);
}

.download-progress-area .progress-info {
  font-size: var(--font-size-sm);
  margin: 0;
  color: var(--text-secondary);
}

.error-badge {
  color: var(--color-danger);
  font-size: var(--font-size-sm);
  margin-left: var(--space-sm);
  font-weight: 500;
}
</style>

