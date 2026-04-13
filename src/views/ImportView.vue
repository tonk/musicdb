<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open, save } from '@tauri-apps/plugin-dialog'
import { useI18n } from 'vue-i18n'
import type { ImportSummary } from '../types'

const { t } = useI18n()

// ── Audio import ─────────────────────────────────────────────────────────────
interface AudioImportSummary { total_files: number; total_albums: number; imported: number; skipped: number }
const audioLoading  = ref(false)
const audioProgress = ref<ProgressEvent | null>(null)
const audioResult   = ref<AudioImportSummary | null>(null)
const audioError    = ref<string | null>(null)

async function importAudio() {
  const selected = await open({ directory: true, multiple: true })
  if (!selected || (Array.isArray(selected) && selected.length === 0)) return
  const folders = Array.isArray(selected) ? selected : [selected]

  audioLoading.value = true
  audioProgress.value = null
  audioResult.value = null
  audioError.value = null

  const unlisten = await listen<ProgressEvent>('import-progress', e => {
    audioProgress.value = e.payload
  })

  try {
    audioResult.value = await invoke<AudioImportSummary>('import_audio_folder', { folders })
  } catch (e: unknown) {
    audioError.value = String(e)
  } finally {
    unlisten()
    audioLoading.value = false
  }
}

// ── TXT import ──────────────────────────────────────────────────────────────
interface ProgressEvent { done: number; total: number; current: string }
const txtLoading  = ref(false)
const txtProgress = ref<ProgressEvent | null>(null)
const txtResult   = ref<ImportSummary | null>(null)
const txtError    = ref<string | null>(null)

async function importTxt() {
  const file = await open({ filters: [{ name: 'Text files', extensions: ['txt'] }], multiple: false })
  if (!file || Array.isArray(file)) return

  txtLoading.value = true
  txtProgress.value = null
  txtResult.value = null
  txtError.value = null

  const unlisten = await listen<ProgressEvent>('import-progress', e => {
    txtProgress.value = e.payload
  })

  try {
    txtResult.value = await invoke<ImportSummary>('import_txt_file', { path: file })
  } catch (e: unknown) {
    txtError.value = String(e)
  } finally {
    unlisten()
    txtLoading.value = false
  }
}

// ── Export ──────────────────────────────────────────────────────────────────
const exportLoading = ref(false)
const exportError   = ref<string | null>(null)

async function exportCsv() {
  const path = await save({ filters: [{ name: 'CSV', extensions: ['csv'] }], defaultPath: 'musicdb_export.csv' })
  if (!path) return
  exportLoading.value = true
  exportError.value = null
  try {
    await invoke('export_csv', { path })
  } catch (e: unknown) {
    exportError.value = String(e)
  } finally {
    exportLoading.value = false
  }
}

async function exportJson() {
  const path = await save({ filters: [{ name: 'JSON', extensions: ['json'] }], defaultPath: 'musicdb_export.json' })
  if (!path) return
  exportLoading.value = true
  exportError.value = null
  try {
    await invoke('export_json', { path })
  } catch (e: unknown) {
    exportError.value = String(e)
  } finally {
    exportLoading.value = false
  }
}

// ── CSV import ───────────────────────────────────────────────────────────────
interface CsvPreview { headers: string[]; rows: string[][] }
interface CsvColumnMapping { csv_column: string; field: string }

const csvFile       = ref<string | null>(null)
const csvPreview    = ref<CsvPreview | null>(null)
const csvMapping    = ref<CsvColumnMapping[]>([])
const csvLoading    = ref(false)
const csvResult     = ref<ImportSummary | null>(null)
const csvError      = ref<string | null>(null)

const TARGET_FIELDS = ['title', 'artist', 'format', 'year', 'label', 'publisher',
                       'catalogue_number', 'condition', 'genre', 'notes',
                       'total_time', 'archive_number']

async function pickCsv() {
  const file = await open({ filters: [{ name: 'CSV', extensions: ['csv'] }], multiple: false })
  if (!file || Array.isArray(file)) return
  csvFile.value = file
  csvLoading.value = true
  try {
    csvPreview.value = await invoke<CsvPreview>('preview_csv', { path: file })
    // Auto-map columns where header name matches target field
    csvMapping.value = csvPreview.value.headers.map(h => ({
      csv_column: h,
      field: TARGET_FIELDS.find(f => h.toLowerCase().includes(f)) ?? '',
    }))
  } catch (e: unknown) {
    csvError.value = String(e)
  } finally {
    csvLoading.value = false
  }
}

async function runCsvImport() {
  if (!csvFile.value) return
  csvLoading.value = true
  csvResult.value = null
  csvError.value = null
  const active = csvMapping.value.filter(m => m.field)
  try {
    csvResult.value = await invoke<ImportSummary>('import_csv', { path: csvFile.value, mapping: active })
  } catch (e: unknown) {
    csvError.value = String(e)
  } finally {
    csvLoading.value = false
  }
}
</script>

<template>
  <div style="max-width: 720px; display: flex; flex-direction: column; gap: 24px;">
    <h2 style="margin: 0; font-size: 18px;">{{ t('import.title') }}</h2>

    <!-- Audio Import -->
    <div class="card" style="padding: 20px;">
      <h3 style="margin: 0 0 12px; font-size: 15px;">{{ t('import.audioImport') }}</h3>
      <p class="text-muted text-sm" style="margin: 0 0 12px;">
        {{ t('import.audioImportDesc') }}
      </p>
      <button class="btn btn-primary" :disabled="audioLoading" @click="importAudio">
        {{ audioLoading ? t('import.importing') : t('import.selectFolder') }}
      </button>

      <div v-if="audioProgress && audioLoading" style="margin-top: 12px;">
        <div style="font-size: 12px; color: var(--color-text-muted); margin-bottom: 4px;">
          {{ audioProgress.done }} / {{ audioProgress.total }} — {{ audioProgress.current }}
        </div>
        <div style="height: 6px; background: var(--color-bg-tertiary); border-radius: 3px; overflow: hidden;">
          <div
            style="height: 100%; background: var(--color-accent); transition: width 0.2s;"
            :style="{ width: audioProgress.total ? `${Math.round(audioProgress.done / audioProgress.total * 100)}%` : '0%' }"
          />
        </div>
      </div>

      <div v-if="audioResult" style="margin-top: 12px; font-size: 13px; color: var(--color-success);">
        {{ t('import.done') }} — {{ t('import.audioSummary', { imported: audioResult.imported, total: audioResult.total_albums, totalFiles: audioResult.total_files, skipped: audioResult.skipped }) }}
      </div>
      <div v-if="audioError" style="margin-top: 12px; font-size: 13px; color: var(--color-danger);">{{ audioError }}</div>
    </div>

    <!-- TXT Import -->
    <div class="card" style="padding: 20px;">
      <h3 style="margin: 0 0 12px; font-size: 15px;">{{ t('import.txtImport') }}</h3>
      <p class="text-muted text-sm" style="margin: 0 0 12px;">
        Import from the legacy .txt export format (CDN database).
      </p>
      <button class="btn btn-primary" :disabled="txtLoading" @click="importTxt">
        {{ txtLoading ? t('import.importing') : t('import.selectFile') }}
      </button>

      <!-- Progress bar -->
      <div v-if="txtProgress && txtLoading" style="margin-top: 12px;">
        <div style="font-size: 12px; color: var(--color-text-muted); margin-bottom: 4px;">
          {{ txtProgress.done }} / {{ txtProgress.total }} — {{ txtProgress.current }}
        </div>
        <div style="height: 6px; background: var(--color-bg-tertiary); border-radius: 3px; overflow: hidden;">
          <div
            style="height: 100%; background: var(--color-accent); transition: width 0.2s;"
            :style="{ width: txtProgress.total ? `${Math.round(txtProgress.done / txtProgress.total * 100)}%` : '0%' }"
          />
        </div>
      </div>

      <div v-if="txtResult" style="margin-top: 12px; font-size: 13px; color: var(--color-success);">
        {{ t('import.done') }} — {{ t('import.summary', { imported: txtResult.imported, total: txtResult.total, skipped: txtResult.skipped }) }}
      </div>
      <div v-if="txtError" style="margin-top: 12px; font-size: 13px; color: var(--color-danger);">{{ txtError }}</div>
    </div>

    <!-- CSV Import -->
    <div class="card" style="padding: 20px;">
      <h3 style="margin: 0 0 12px; font-size: 15px;">{{ t('import.csvImport') }}</h3>
      <button class="btn btn-secondary" @click="pickCsv">{{ t('import.selectFile') }}</button>

      <div v-if="csvPreview" style="margin-top: 16px;">
        <p class="text-sm text-muted" style="margin: 0 0 8px;">{{ t('import.mapColumns') }}</p>
        <table style="font-size: 13px; border-collapse: collapse; width: 100%;">
          <thead>
            <tr>
              <th style="text-align: left; padding: 4px 8px; border-bottom: 1px solid var(--color-border);">CSV column</th>
              <th style="text-align: left; padding: 4px 8px; border-bottom: 1px solid var(--color-border);">Maps to</th>
              <th style="text-align: left; padding: 4px 8px; border-bottom: 1px solid var(--color-border);">Preview</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(m, i) in csvMapping" :key="m.csv_column">
              <td style="padding: 4px 8px;">{{ m.csv_column }}</td>
              <td style="padding: 4px 8px;">
                <select v-model="csvMapping[i].field" class="form-control" style="font-size: 12px; padding: 2px 6px;">
                  <option value="">— skip —</option>
                  <option v-for="f in TARGET_FIELDS" :key="f" :value="f">{{ f }}</option>
                </select>
              </td>
              <td style="padding: 4px 8px; color: var(--color-text-faint); font-size: 12px;" class="truncate">
                {{ csvPreview.rows[0]?.[csvPreview.headers.indexOf(m.csv_column)] }}
              </td>
            </tr>
          </tbody>
        </table>
        <button class="btn btn-primary" style="margin-top: 12px;" :disabled="csvLoading" @click="runCsvImport">
          {{ csvLoading ? t('import.importing') : t('import.confirm') }}
        </button>
      </div>

      <div v-if="csvResult" style="margin-top: 12px; font-size: 13px; color: var(--color-success);">
        {{ t('import.done') }} — {{ t('import.summary', { imported: csvResult.imported, total: csvResult.total, skipped: csvResult.skipped }) }}
      </div>
      <div v-if="csvError" style="margin-top: 12px; font-size: 13px; color: var(--color-danger);">{{ csvError }}</div>
    </div>

    <!-- Export -->
    <div class="card" style="padding: 20px;">
      <h3 style="margin: 0 0 12px; font-size: 15px;">Export</h3>
      <div class="flex gap-2">
        <button class="btn btn-secondary" :disabled="exportLoading" @click="exportCsv">
          {{ exportLoading ? '…' : 'Export CSV' }}
        </button>
        <button class="btn btn-secondary" :disabled="exportLoading" @click="exportJson">
          {{ exportLoading ? '…' : 'Export JSON' }}
        </button>
      </div>
      <div v-if="exportError" style="margin-top: 8px; font-size: 13px; color: var(--color-danger);">{{ exportError }}</div>
    </div>
  </div>
</template>
