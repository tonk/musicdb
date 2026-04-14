<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '../stores/settings'
import { useCollectionStore } from '../stores/collection'

const { t, locale } = useI18n()
const settings = useSettingsStore()
const collection = useCollectionStore()

const dbPath        = ref('')
const moveDbLoading = ref(false)
const moveDbError   = ref<string | null>(null)
const moveDbDone    = ref(false)
const version       = ref(__APP_VERSION__)

const backupLoading = ref(false)
const backupError   = ref<string | null>(null)
const backupDone    = ref<string | null>(null)

const resetConfirm  = ref(false)
const resetLoading  = ref(false)
const resetDone     = ref(false)
const resetError    = ref<string | null>(null)

async function resetDb() {
  resetLoading.value = true
  resetDone.value    = false
  resetError.value   = null
  resetConfirm.value = false
  try {
    await invoke('reset_database')
    resetDone.value = true
    await collection.fetchItems()
  } catch (e: unknown) {
    resetError.value = String(e)
  } finally {
    resetLoading.value = false
  }
}

onMounted(async () => {
  try {
    const v = await invoke<string | null>('get_setting', { key: 'db_path' })
    if (v) dbPath.value = v
  } catch { /* ok */ }
  await loadDatabases()
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

async function backupDb() {
  const now = new Date()
  const pad = (n: number, len = 2) => String(n).padStart(len, '0')
  const ts = `${now.getFullYear()}${pad(now.getMonth() + 1)}${pad(now.getDate())}${pad(now.getHours())}${pad(now.getMinutes())}${pad(now.getSeconds())}`
  const dbName = currentDb.value.replace(/ /g, '_')
  const defaultName = `musicdb_backup_${dbName}_${ts}.sqlite`

  const path = await save({
    filters: [{ name: 'SQLite', extensions: ['sqlite', 'db'] }],
    defaultPath: defaultName,
  })
  if (!path) return

  backupLoading.value = true
  backupError.value   = null
  backupDone.value    = null
  try {
    const result = await invoke<string>('backup_database', { destPath: path })
    backupDone.value = result
  } catch (e: unknown) {
    backupError.value = String(e)
  } finally {
    backupLoading.value = false
  }
}

// sync locale with settings language
watch(() => settings.language, lang => { locale.value = lang }, { immediate: true })

// ── Multi-database management ─────────────────────────────────────────────────
interface DatabaseEntry { name: string; path: string }

const databases      = ref<DatabaseEntry[]>([])
const currentDb      = ref('')
const newDbName      = ref('')
const dbLoading      = ref(false)
const dbError        = ref<string | null>(null)
const renamingDb     = ref<string | null>(null)
const renameValue    = ref('')
const deleteConfirm  = ref<string | null>(null)

async function loadDatabases() {
  try {
    databases.value = await invoke<DatabaseEntry[]>('list_databases')
    currentDb.value = await invoke<string>('current_database')
  } catch { /* ignore */ }
}

async function createDatabase() {
  const name = newDbName.value.trim()
  if (!name) return
  dbLoading.value = true
  dbError.value   = null
  try {
    databases.value = await invoke<DatabaseEntry[]>('create_database', { name })
    newDbName.value = ''
  } catch (e: unknown) {
    dbError.value = String(e)
  } finally {
    dbLoading.value = false
  }
}

function startRename(name: string) {
  renamingDb.value  = name
  renameValue.value = name
}

async function confirmRename() {
  const oldName = renamingDb.value
  const newName = renameValue.value.trim()
  if (!oldName || !newName || oldName === newName) {
    renamingDb.value = null
    return
  }
  dbLoading.value  = true
  dbError.value    = null
  try {
    databases.value = await invoke<DatabaseEntry[]>('rename_database', { oldName, newName })
    if (currentDb.value === oldName) currentDb.value = newName
    renamingDb.value = null
  } catch (e: unknown) {
    dbError.value = String(e)
  } finally {
    dbLoading.value = false
  }
}

async function deleteDatabase(name: string) {
  dbLoading.value = true
  dbError.value   = null
  deleteConfirm.value = null
  try {
    databases.value = await invoke<DatabaseEntry[]>('delete_database', { name })
  } catch (e: unknown) {
    dbError.value = String(e)
  } finally {
    dbLoading.value = false
  }
}
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

    <!-- Databases section -->
    <div class="card" style="padding: 20px; margin-top: 16px;">
      <h3 style="margin: 0 0 12px; font-size: 14px; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.05em;">
        {{ t('settings.databases') }}
      </h3>
      <p class="text-sm text-muted" style="margin: 0 0 12px;">
        {{ t('settings.databasesDesc') }}
      </p>

      <!-- Database list -->
      <div style="display: flex; flex-direction: column; gap: 6px; margin-bottom: 16px;">
        <div
          v-for="db in databases"
          :key="db.name"
          style="display: flex; align-items: center; gap: 8px; padding: 8px 10px;
                 border: 1px solid var(--color-border); border-radius: var(--radius-md);
                 background: var(--color-bg-secondary);"
          :style="db.name === currentDb ? 'border-color: var(--color-accent);' : ''"
        >
          <!-- Rename input or name -->
          <template v-if="renamingDb === db.name">
            <input
              v-model="renameValue"
              class="form-control"
              style="flex: 1; font-size: 13px; padding: 2px 6px;"
              @keydown.enter="confirmRename"
              @keydown.escape="renamingDb = null"
            />
            <button class="btn btn-primary" style="font-size: 12px; padding: 2px 8px;" @click="confirmRename">
              {{ t('item.save') }}
            </button>
            <button class="btn btn-ghost" style="font-size: 12px; padding: 2px 8px;" @click="renamingDb = null">
              {{ t('item.cancel') }}
            </button>
          </template>
          <template v-else>
            <span style="flex: 1; font-size: 13px; font-weight: db.name === currentDb ? 600 : 400;">
              {{ db.name }}
              <span v-if="db.name === currentDb" style="margin-left: 6px; font-size: 11px; color: var(--color-accent);">●</span>
            </span>
            <button class="btn btn-ghost" style="font-size: 11px; padding: 2px 6px;" @click="startRename(db.name)">
              {{ t('settings.rename') }}
            </button>
            <template v-if="deleteConfirm === db.name">
              <button
                class="btn btn-danger"
                style="font-size: 11px; padding: 2px 6px;"
                :disabled="dbLoading"
                @click="deleteDatabase(db.name)"
              >{{ t('common.confirm') }}</button>
              <button class="btn btn-ghost" style="font-size: 11px; padding: 2px 6px;" @click="deleteConfirm = null">
                {{ t('item.cancel') }}
              </button>
            </template>
            <button
              v-else
              class="btn btn-ghost"
              style="font-size: 11px; padding: 2px 6px; color: var(--color-danger);"
              :disabled="db.name === currentDb || databases.length <= 1"
              @click="deleteConfirm = db.name"
            >
              {{ t('settings.deleteDatabase') }}
            </button>
          </template>
        </div>
      </div>

      <!-- Add new database -->
      <div class="flex gap-2" style="align-items: center;">
        <input
          v-model="newDbName"
          class="form-control"
          style="flex: 1; font-size: 13px;"
          :placeholder="t('settings.databaseName')"
          @keydown.enter="createDatabase"
        />
        <button class="btn btn-secondary" :disabled="dbLoading || !newDbName.trim()" @click="createDatabase">
          {{ dbLoading ? '…' : t('settings.addDatabase') }}
        </button>
      </div>
      <p v-if="dbError" class="text-sm" style="margin-top: 8px; color: var(--color-danger);">{{ dbError }}</p>
    </div>

    <!-- Database section -->
    <div class="card" style="padding: 20px; margin-top: 16px;">
      <h3 style="margin: 0 0 12px; font-size: 14px; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.05em;">
        {{ t('settings.database') }}
      </h3>
      <p v-if="dbPath" class="text-faint text-sm" style="margin: 0 0 12px; word-break: break-all;">{{ dbPath }}</p>

      <!-- Backup -->
      <button class="btn btn-secondary" :disabled="backupLoading" @click="backupDb">
        {{ backupLoading ? '…' : t('settings.backupDb') }}
      </button>
      <p v-if="backupDone"  class="text-sm" style="margin-top: 8px; color: var(--color-success);">{{ t('settings.backupDbDone') }} {{ backupDone }}</p>
      <p v-if="backupError" class="text-sm" style="margin-top: 8px; color: var(--color-danger);">{{ backupError }}</p>

      <hr style="border: none; border-top: 1px solid var(--color-border); margin: 12px 0;" />

      <!-- Move -->
      <button class="btn btn-secondary" :disabled="moveDbLoading" @click="moveDb">
        {{ moveDbLoading ? '…' : t('settings.moveDb') }}
      </button>
      <p v-if="moveDbDone"  class="text-sm" style="margin-top: 8px; color: var(--color-success);">Database copied successfully. Restart the app to use the new location.</p>
      <p v-if="moveDbError" class="text-sm" style="margin-top: 8px; color: var(--color-danger);">{{ moveDbError }}</p>

      <hr style="border: none; border-top: 1px solid var(--color-border); margin: 12px 0;" />

      <template v-if="!resetConfirm">
        <button class="btn btn-danger" :disabled="resetLoading" @click="resetConfirm = true">
          {{ t('settings.resetDb') }}
        </button>
      </template>
      <template v-else>
        <p class="text-sm" style="margin: 0 0 10px; color: var(--color-danger);">{{ t('settings.resetDbConfirm') }}</p>
        <div class="flex gap-2">
          <button class="btn btn-danger" :disabled="resetLoading" @click="resetDb">
            {{ resetLoading ? '…' : t('settings.resetDb') }}
          </button>
          <button class="btn btn-secondary" @click="resetConfirm = false">{{ t('item.cancel') }}</button>
        </div>
      </template>
      <p v-if="resetDone"  class="text-sm" style="margin-top: 8px; color: var(--color-success);">{{ t('settings.resetDbDone') }}</p>
      <p v-if="resetError" class="text-sm" style="margin-top: 8px; color: var(--color-danger);">{{ resetError }}</p>
    </div>

    <p class="text-faint text-xs" style="margin-top: 16px;">MusicDB v{{ version }}</p>
  </div>
</template>
