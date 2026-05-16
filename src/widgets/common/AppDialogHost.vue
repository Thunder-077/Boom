<template>
  <Teleport to="body">
    <Transition name="dialog-fade">
      <div v-if="state.visible" class="app-dialog-mask" @click.self="closeAppDialog(false)" @keydown.esc="closeAppDialog(false)">
        <section
          class="app-dialog"
          :class="`tone-${state.tone}`"
          role="dialog"
          aria-modal="true"
          aria-labelledby="app-dialog-title"
          aria-describedby="app-dialog-summary"
          tabindex="-1"
        >
          <header class="app-dialog-head">
            <span class="dialog-icon material-symbols-rounded" aria-hidden="true">{{ state.icon }}</span>
            <div>
              <h3 id="app-dialog-title">{{ state.title }}</h3>
              <p id="app-dialog-summary">{{ state.summary }}</p>
            </div>
            <button class="dialog-close" type="button" aria-label="关闭弹窗" @click="closeAppDialog(false)">
              <span class="material-symbols-rounded" aria-hidden="true">close</span>
            </button>
          </header>

          <ul v-if="state.details.length > 0" class="dialog-details">
            <li v-for="(line, index) in state.details" :key="index">{{ line }}</li>
          </ul>

          <footer class="dialog-actions">
            <button v-if="state.kind === 'confirm'" class="dialog-btn secondary" type="button" @click="closeAppDialog(false)">
              {{ state.cancelText }}
            </button>
            <button class="dialog-btn primary" type="button" autofocus @click="closeAppDialog(true)">
              <span class="material-symbols-rounded" aria-hidden="true">{{ state.tone === "danger" ? "delete" : "check" }}</span>
              {{ state.confirmText }}
            </button>
          </footer>
        </section>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { appDialogState as state, closeAppDialog } from "../../shared/ui/appDialog";
</script>

<style scoped>
.app-dialog-mask {
  position: fixed;
  inset: 0;
  z-index: 3000;
  display: grid;
  place-items: start center;
  padding: 10vh 24px 24px;
  background:
    radial-gradient(circle at 50% 18%, rgba(255, 255, 255, 0.32), transparent 34%),
    var(--surface-overlay);
  backdrop-filter: blur(10px);
}

.app-dialog {
  width: min(480px, calc(100vw - 48px));
  overflow: hidden;
  border: 1px solid var(--border-default);
  border-radius: 20px;
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.72), rgba(255, 255, 255, 0)),
    var(--surface-panel-strong);
  box-shadow: var(--shadow-strong);
  outline: none;
}

.app-dialog-head {
  position: relative;
  display: grid;
  grid-template-columns: 44px 1fr 34px;
  gap: 14px;
  align-items: start;
  padding: 22px 22px 16px;
}

.dialog-icon {
  width: 44px;
  height: 44px;
  border-radius: 14px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--accent-primary);
  background: var(--accent-soft);
  box-shadow: var(--shadow-inset);
  font-size: 24px;
}

.tone-danger .dialog-icon {
  color: var(--color-danger);
  background: var(--color-danger-soft);
}

.tone-success .dialog-icon {
  color: var(--color-success);
  background: var(--color-success-soft);
}

.tone-warning .dialog-icon {
  color: var(--color-warning);
  background: var(--color-warning-soft);
}

.app-dialog h3 {
  margin: 1px 0 8px;
  color: var(--text-primary);
  font-size: 18px;
  line-height: 1.25;
}

.app-dialog p {
  margin: 0;
  color: var(--text-secondary);
  font-size: 14px;
  line-height: 1.65;
}

.dialog-close {
  width: 34px;
  height: 34px;
  border: 1px solid transparent;
  border-radius: 10px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--text-tertiary);
  background: transparent;
  cursor: pointer;
}

.dialog-close:hover {
  color: var(--text-primary);
  border-color: var(--border-default);
  background: var(--surface-panel);
}

.dialog-details {
  display: grid;
  gap: 8px;
  margin: 0 22px 2px 80px;
  padding: 12px 14px;
  border: 1px solid var(--border-default);
  border-radius: 14px;
  color: var(--text-secondary);
  background: var(--surface-input);
  font-size: 13px;
  line-height: 1.55;
  list-style-position: inside;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 18px 22px 22px;
}

.dialog-btn {
  min-width: 94px;
  height: 40px;
  border-radius: 12px;
  border: 1px solid transparent;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 0 16px;
  font-size: 14px;
  font-weight: 700;
  cursor: pointer;
  transition:
    transform 0.16s ease,
    box-shadow 0.16s ease,
    border-color 0.16s ease,
    background-color 0.16s ease;
}

.dialog-btn .material-symbols-rounded {
  font-size: 18px;
}

.dialog-btn.primary {
  color: var(--text-on-dark);
  background: linear-gradient(135deg, var(--accent-primary-strong), var(--accent-primary));
  box-shadow: 0 14px 26px rgba(var(--accent-rgb), 0.22);
}

.tone-danger .dialog-btn.primary {
  background: linear-gradient(135deg, #9f3030, var(--color-danger));
  box-shadow: 0 14px 26px rgba(182, 68, 68, 0.22);
}

.dialog-btn.secondary {
  color: var(--text-secondary);
  border-color: var(--border-default);
  background: var(--surface-panel);
}

.dialog-btn:hover {
  transform: translateY(-1px);
}

.dialog-btn.secondary:hover {
  border-color: var(--border-strong);
  background: var(--surface-panel-strong);
}

.dialog-fade-enter-active,
.dialog-fade-leave-active {
  transition: opacity 0.18s ease;
}

.dialog-fade-enter-active .app-dialog,
.dialog-fade-leave-active .app-dialog {
  transition:
    transform 0.18s ease,
    opacity 0.18s ease;
}

.dialog-fade-enter-from,
.dialog-fade-leave-to {
  opacity: 0;
}

.dialog-fade-enter-from .app-dialog,
.dialog-fade-leave-to .app-dialog {
  opacity: 0;
  transform: translateY(-8px) scale(0.98);
}

@media (max-width: 640px) {
  .app-dialog-mask {
    place-items: end center;
    padding: 16px;
  }

  .app-dialog {
    width: 100%;
  }

  .app-dialog-head {
    grid-template-columns: 40px 1fr 32px;
    gap: 12px;
    padding: 20px 18px 14px;
  }

  .dialog-icon {
    width: 40px;
    height: 40px;
  }

  .dialog-details {
    margin: 0 18px 2px;
  }

  .dialog-actions {
    padding: 16px 18px 18px;
  }
}
</style>
