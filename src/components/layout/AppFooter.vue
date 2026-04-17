<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useCollectionStore } from '../../stores/collection'
import { useUpdateStore } from '../../stores/update'
import { openUrl } from '@tauri-apps/plugin-opener'

const { t } = useI18n()
const collection = useCollectionStore()
const update = useUpdateStore()
const version = __APP_VERSION__

async function openReleases() {
  await openUrl('https://github.com/tonk/musicdb/releases/latest')
}
</script>

<template>
  <footer class="app-footer">
    <div class="flex items-center gap-4">
      <span>MusicDB {{ version }}</span>
      <span
        v-if="update.isUpdateAvailable"
        class="update-link"
        @click="openReleases"
        :title="t('settings.checkUpdate')"
      >
        {{ t('settings.updateAvailable', { version: update.latestVersion }) }}
      </span>
    </div>
    <span>{{ collection.total }} items</span>
  </footer>
</template>

<style scoped>
.update-link {
  color: var(--color-primary);
  font-weight: 500;
  cursor: pointer;
  text-decoration: underline;
}
.update-link:hover {
  opacity: 0.8;
}
</style>
