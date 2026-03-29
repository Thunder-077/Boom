<template>
  <div class="app-frame">
    <WindowTitleBar />
    <div class="app-content">
      <RouterView />
    </div>
  </div>
</template>

<script setup lang="ts">
import { RouterView } from "vue-router";
import WindowTitleBar from "../widgets/layout/WindowTitleBar.vue";
</script>

<style scoped>
.app-frame {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

.app-content {
  flex: 1;
  min-height: 0;
  padding-top: 38px;
}
</style>
