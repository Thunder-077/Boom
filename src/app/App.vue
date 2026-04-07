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
  position: relative;
}

.app-content {
  flex: 1;
  min-height: 0;
  padding-top: 38px;
  position: relative;
}

.app-content::before {
  content: "";
  position: absolute;
  inset: 0;
  pointer-events: none;
  background:
    radial-gradient(circle at 18% 10%, rgba(255, 255, 255, 0.36), transparent 24%),
    radial-gradient(circle at 78% 16%, rgba(255, 255, 255, 0.2), transparent 18%);
  opacity: 0.58;
}
</style>
