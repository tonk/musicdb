<script setup lang="ts">
import { onMounted } from 'vue'
import { RouterView } from 'vue-router'
import AppSidebar from './components/layout/AppSidebar.vue'
import AppHeader from './components/layout/AppHeader.vue'
import AppFooter from './components/layout/AppFooter.vue'
import ConfirmDialog from './components/shared/ConfirmDialog.vue'
import UndoToast from './components/shared/UndoToast.vue'
import { useSettingsStore } from './stores/settings'
import { useTheme } from './composables/useTheme'
import { useKeyboard } from './composables/useKeyboard'

const settings = useSettingsStore()
useTheme()
useKeyboard()

onMounted(() => settings.load())
</script>

<template>
  <div class="app-shell">
    <AppSidebar />
    <div class="app-main">
      <AppHeader />
      <div class="app-content">
        <RouterView />
      </div>
      <AppFooter />
    </div>
  </div>
  <ConfirmDialog />
  <UndoToast />
</template>
