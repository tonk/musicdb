<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '../stores/settings'

const { t, locale } = useI18n()
const settings = useSettingsStore()

const dbPath        = ref('')
const moveDbLoading = ref(false)
const moveDbError   = ref<string | null>(null)
const moveDbDone    = ref(false)
const version       = ref(__APP_VERSION__)

onMounted(async () => {
  try {
    const v = await invoke<string | null>('get_setting', { key: 'db_path' })
    if (v) dbPath.value = v
  } catch { /* ok */ }
})

async function moveDb() {
  const path = await save({
    filters: [{ name: 'SQLite', extensions: ['sqlite', 'db'] }],
    defaultPath: 'musicdb.sqlite',
  })
  if (!path) return
  moveDbLoading.value = true
  moveDbError.value = null
  moveDbDone.value = false
  try {
    await invoke('move_database', { newPath: path })
    dbPath.value = path
    moveDbDone.value = true
  } catch (e: unknown) {
    moveDbError.value = String(e)
  } finally {
    moveDbLoading.value = false
  }
}

// sync locale with settings language
watch(() => settings.language, lang => { locale.value = lang }, { immediate: true })
</script>

<template>
  <div style="max-width: 480px;">
    <h2 style="margin: 0 0 20px; font-size: 18px;">{{ t('settings.title') }}</h2>

    <div class="card" style="padding: 20px; display: flex; flex-direction: column; gap: 16px;">
      <!-- Theme -->
      <div class="form-group" style="margin-bottom: 0;">
        <label class="form-label">{{ t('settings.theme') }}</label>
        <select v-model="settings.theme" class="form-control">
          <option value="system">{{ t('settings.themeSystem') }}</option>
          <option value="light">{{ t('settings.themeLight') }}</option>
          <option value="dark">{{ t('settings.themeDark') }}</option>
        </select>
      </div>

      <!-- Language -->
      <div class="form-group" style="margin-bottom: 0;">
        <label class="form-label">{{ t('settings.language') }}</label>
        <select v-model="settings.language" class="form-control">
          <option value="en">English</option>
          <option value="nl">Nederlands</option>
          <option value="de">Deutsch</option>
          <option value="fr">Français</option>
          <option value="es">Español</option>
        </select>
      </div>

      <!-- Default view -->
      <div class="form-group" style="margin-bottom: 0;">
        <label class="form-label">{{ t('settings.defaultView') }}</label>
        <select v-model="settings.defaultView" class="form-control">
          <option value="list">{{ t('collection.listView') }}</option>
          <option value="grid">{{ t('collection.gridView') }}</option>
        </select>
      </div>

      <!-- Startup view -->
      <div class="form-group" style="margin-bottom: 0;">
        <label class="form-label">{{ t('settings.startupView') }}</label>
        <select v-model="settings.startupView" class="form-control">
          <option value="collection">{{ t('nav.collection') }}</option>
          <option value="statistics">{{ t('nav.statistics') }}</option>
        </select>
      </div>
    </div>

    <!-- Database section -->
    <div class="card" style="padding: 20px; margin-top: 16px;">
      <h3 style="margin: 0 0 12px; font-size: 14px; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.05em;">
        {{ t('settings.database') }}
      </h3>
      <p v-if="dbPath" class="text-faint text-sm" style="margin: 0 0 12px; word-break: break-all;">{{ dbPath }}</p>
      <button class="btn btn-secondary" :disabled="moveDbLoading" @click="moveDb">
        {{ moveDbLoading ? '…' : t('settings.moveDb') }}
      </button>
      <p v-if="moveDbDone"  class="text-sm" style="margin-top: 8px; color: var(--color-success);">Database copied successfully. Restart the app to use the new location.</p>
      <p v-if="moveDbError" class="text-sm" style="margin-top: 8px; color: var(--color-danger);">{{ moveDbError }}</p>
    </div>

    <p class="text-faint text-xs" style="margin-top: 16px;">MusicDB v{{ version }}</p>
  </div>
</template>
