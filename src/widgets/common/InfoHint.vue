<template>
  <section class="hint">
    <svg class="dot" viewBox="0 0 24 24" aria-hidden="true">
      <path d="M11 7h2V5h-2v2Zm0 12h2v-8h-2v8Zm1-17C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2Z" />
    </svg>
    <p>{{ text }}</p>
  </section>
</template>

<script setup lang="ts">
defineProps<{
  text: string;
}>();
</script>

<style scoped>
.hint {
  min-height: 34px;
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--color-text-muted);
  font-size: 13px;
  padding: 0 4px;
}

.dot {
  width: 16px;
  height: 16px;
  fill: currentColor;
}

.hint p {
  margin: 0;
}
</style>
