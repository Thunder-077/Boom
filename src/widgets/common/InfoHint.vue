<template>
  <section class="hint" :class="type || 'info'">
    <svg class="dot" viewBox="0 0 24 24">
      <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 
               10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/>
    </svg>
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
  gap: 8px;
  font-size: 13px;
}

.hint.info { 
  color: #409eff; 
}
.hint.info .dot { fill: #409eff; }

.hint.success { 
  color: #67c23a; 
}
.hint.success .dot { fill: #67c23a; }

.hint.error { 
  color: #f56c6c; 
}
.hint.error .dot { fill: #f56c6c; }

.hint.warning { 
  color: #e6a23c; 
}
.hint.warning .dot { fill: #e6a23c; }

.dot {
  width: 16px;
  height: 16px;
}

.hint p {
  margin: 0;
}
</style>