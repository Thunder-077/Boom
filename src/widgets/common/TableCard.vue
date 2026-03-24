<template>
  <section class="table-card card-shell">
    <header class="head">
      <div class="copy">
        <h3>{{ title }}</h3>
        <slot name="description" />
      </div>
      <div class="meta-wrap">
        <slot name="actions" />
        <p v-if="meta">{{ meta }}</p>
      </div>
    </header>
    <div class="content">
      <slot />
    </div>
  </section>
</template>

<script setup lang="ts">
defineProps<{
  title: string;
  meta?: string;
}>();
</script>

<style scoped>
.table-card {
  padding: 22px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.8);
  border-color: rgba(255, 255, 255, 0.88);
}

.head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.copy {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.meta-wrap {
  display: flex;
  align-items: center;
  gap: 12px;
}

.head h3 {
  margin: 0;
  font-size: 22px;
  font-weight: 600;
}

.head p {
  margin: 0;
  color: var(--color-brand);
  font-size: 13px;
  font-weight: 600;
}

.content {
  overflow: hidden;
  border-radius: 16px;
  border: 1px solid var(--color-border-soft);
  background: #fff;
}
</style>
