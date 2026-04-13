<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { useCollectionStore } from '../../stores/collection'

const { t } = useI18n()
const collection = useCollectionStore()
const router = useRouter()

interface DatabaseEntry { name: string; path: string }

const databases  = ref<DatabaseEntry[]>([])
const currentDb  = ref('')
const switching  = ref(false)

async function loadDatabases() {
  try {
    databases.value = await invoke<DatabaseEntry[]>('list_databases')
    currentDb.value = await invoke<string>('current_database')
  } catch { /* ignore on first load */ }
}

async function onSwitchDb() {
  if (switching.value) return
  switching.value = true
  try {
    await invoke('switch_database', { name: currentDb.value })
    await collection.fetchItems()
    router.push({ name: 'collection' })
  } catch (e) {
    console.error('Failed to switch database:', e)
  } finally {
    switching.value = false
  }
}

onMounted(loadDatabases)
</script>

<template>
  <nav class="app-sidebar">
    <div style="padding: 16px 16px 4px; font-weight: 700; font-size: 16px; color: var(--color-text);">
      MusicDB
    </div>

    <!-- Database picker -->
    <div style="padding: 4px 12px 8px;">
      <select
        v-model="currentDb"
        class="form-control"
        style="font-size: 12px; padding: 3px 6px; width: 100%;"
        :disabled="switching || databases.length <= 1"
        @change="onSwitchDb"
      >
        <option v-for="db in databases" :key="db.name" :value="db.name">{{ db.name }}</option>
      </select>
    </div>

    <RouterLink class="nav-item" :to="{ name: 'collection' }">
      <span>💿</span><span>{{ t('nav.collection') }}</span>
    </RouterLink>
    <RouterLink class="nav-item" :to="{ name: 'search' }">
      <span>🔍</span><span>{{ t('nav.search') }}</span>
    </RouterLink>
    <RouterLink class="nav-item" :to="{ name: 'statistics' }">
      <span>📊</span><span>{{ t('nav.statistics') }}</span>
    </RouterLink>
    <RouterLink class="nav-item" :to="{ name: 'import' }">
      <span>📥</span><span>{{ t('nav.import') }}</span>
    </RouterLink>
    <div style="flex: 1;" />
    <RouterLink class="nav-item" :to="{ name: 'settings' }">
      <span>⚙️</span><span>{{ t('nav.settings') }}</span>
    </RouterLink>
    <div style="padding: 8px 16px; font-size: 11px; color: var(--color-text-faint);">
      {{ collection.total }} items
    </div>
  </nav>
</template>
