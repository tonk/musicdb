<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import type { Statistics } from '../types'

const { t } = useI18n()

const stats   = ref<Statistics | null>(null)
const loading = ref(true)
const error   = ref<string | null>(null)

onMounted(async () => {
  try {
    stats.value = await invoke<Statistics>('get_statistics')
  } catch (e: unknown) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
})

function barWidth(count: number, max: number): string {
  return max > 0 ? `${Math.round(count / max * 100)}%` : '0%'
}
</script>

<template>
  <div>
    <h2 style="margin: 0 0 20px; font-size: 18px;">{{ t('statistics.title') }}</h2>

    <div v-if="loading" class="flex items-center gap-2"><div class="spinner" /><span class="text-muted">{{ t('common.loading') }}</span></div>
    <div v-else-if="error" style="color: var(--color-danger);">{{ error }}</div>

    <template v-else-if="stats">
      <!-- Total -->
      <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(160px, 1fr)); gap: 16px; margin-bottom: 28px;">
        <div class="card" style="padding: 20px; text-align: center;">
          <div style="font-size: 36px; font-weight: 700; color: var(--color-accent);">{{ stats.total_items }}</div>
          <div class="text-muted text-sm">{{ t('statistics.totalItems') }}</div>
        </div>
      </div>

      <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 24px;">
        <!-- By format -->
        <div class="card" style="padding: 16px;">
          <h3 style="margin: 0 0 12px; font-size: 14px; text-transform: uppercase; letter-spacing: 0.05em; color: var(--color-text-muted);">
            {{ t('statistics.byFormat') }}
          </h3>
          <div v-for="entry in stats.by_format" :key="entry.label" style="margin-bottom: 8px;">
            <div class="flex items-center gap-2" style="margin-bottom: 2px;">
              <span style="width: 100px; font-size: 13px; flex-shrink: 0;">{{ entry.label || '—' }}</span>
              <span class="text-faint text-sm" style="margin-left: auto;">{{ entry.count }}</span>
            </div>
            <div style="height: 6px; background: var(--color-bg-tertiary); border-radius: 3px; overflow: hidden;">
              <div style="height: 100%; background: var(--color-accent); border-radius: 3px;"
                   :style="{ width: barWidth(entry.count, stats.by_format[0]?.count ?? 1) }" />
            </div>
          </div>
        </div>

        <!-- By genre (top 20) -->
        <div class="card" style="padding: 16px;">
          <h3 style="margin: 0 0 12px; font-size: 14px; text-transform: uppercase; letter-spacing: 0.05em; color: var(--color-text-muted);">
            {{ t('statistics.byGenre') }}
          </h3>
          <div v-for="entry in stats.by_genre" :key="entry.label" style="margin-bottom: 8px;">
            <div class="flex items-center gap-2" style="margin-bottom: 2px;">
              <span style="font-size: 13px; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">{{ entry.label }}</span>
              <span class="text-faint text-sm">{{ entry.count }}</span>
            </div>
            <div style="height: 4px; background: var(--color-bg-tertiary); border-radius: 2px; overflow: hidden;">
              <div style="height: 100%; background: var(--color-accent); border-radius: 2px;"
                   :style="{ width: barWidth(entry.count, stats.by_genre[0]?.count ?? 1) }" />
            </div>
          </div>
        </div>

        <!-- By year -->
        <div class="card" style="padding: 16px; grid-column: 1 / -1;">
          <h3 style="margin: 0 0 12px; font-size: 14px; text-transform: uppercase; letter-spacing: 0.05em; color: var(--color-text-muted);">
            {{ t('statistics.byYear') }}
          </h3>
          <div style="display: flex; gap: 4px; align-items: flex-end; height: 80px; overflow-x: auto;">
            <div
              v-for="entry in [...stats.by_year].reverse()"
              :key="entry.label"
              style="flex-shrink: 0; width: 16px; background: var(--color-accent); border-radius: 2px 2px 0 0; cursor: default;"
              :style="{ height: barWidth(entry.count, stats.by_year.reduce((m, e) => Math.max(m, e.count), 0)) }"
              :title="`${entry.label}: ${entry.count}`"
            />
          </div>
          <div style="margin-top: 4px; font-size: 11px; color: var(--color-text-faint);">
            {{ stats.by_year[stats.by_year.length - 1]?.label }} – {{ stats.by_year[0]?.label }}
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
