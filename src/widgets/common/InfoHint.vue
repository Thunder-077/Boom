<template>
  <section class="hint" :class="type || 'info'">
    <span class="icon-wrap" aria-hidden="true">
      <svg class="dot" viewBox="0 0 24 24">
        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 
                 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/>
      </svg>
    </span>
    <p>{{ text }}</p>
  </section>
</template>

<script setup lang="ts">
defineProps<{ 
  text: string;
  type?: 'info' | 'success' | 'warning' | 'error';
}>();
</script>

<style scoped>
.hint {
  display: flex;
  align-items: center;
  gap: 10px;
  min-height: 48px;
  padding: 10px 14px;
  border-radius: 16px;
  border: 1px solid transparent;
  background: rgba(255, 255, 255, 0.68);
  font-size: 13px;
  line-height: 1.45;
}

.hint.info { 
  color: var(--accent-primary);
  border-color: rgba(var(--accent-rgb), 0.16);
  background: rgba(var(--accent-rgb), 0.12);
}
.hint.info .dot { fill: var(--accent-primary); }

.hint.success { 
  color: var(--color-success);
  border-color: rgba(24, 121, 78, 0.14);
  background: var(--color-success-soft);
}
.hint.success .dot { fill: var(--color-success); }

.hint.error { 
  color: var(--color-danger);
  border-color: rgba(182, 68, 68, 0.14);
  background: var(--color-danger-soft);
}
.hint.error .dot { fill: var(--color-danger); }

.hint.warning { 
  color: var(--color-warning);
  border-color: rgba(168, 103, 18, 0.14);
  background: var(--color-warning-soft);
}
.hint.warning .dot { fill: var(--color-warning); }

.icon-wrap {
  width: 24px;
  height: 24px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.66);
  flex-shrink: 0;
}

.dot {
  width: 16px;
  height: 16px;
}

.hint p {
  margin: 0;
}
</style>
