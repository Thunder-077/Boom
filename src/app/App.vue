<template>
  <div class="app-frame">
    <WindowTitleBar />
    <div class="app-content">
      <RouterView />
    </div>
    <AppDialogHost />
  </div>
</template>

<script setup lang="ts">
import { RouterView } from "vue-router";
import AppDialogHost from "../widgets/common/AppDialogHost.vue";
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
  display: none;
}
</style>
