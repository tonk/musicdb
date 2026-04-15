import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export const useSettingsStore = defineStore('settings', () => {
  const theme = ref<'light' | 'dark' | 'system'>('system')
  const language = ref('en')
  const defaultView = ref<'list' | 'grid'>('list')
  const startupView = ref<'collection' | 'statistics'>('collection')
  const dateFormat = ref('YYYY-MM-DD')
  const pageSize = ref(50)

  async function load() {
    const entries: { key: string; value: string | null }[] = await invoke('get_all_settings')
    for (const e of entries) {
      if (e.key === 'theme' && e.value) theme.value = e.value as typeof theme.value
      if (e.key === 'language' && e.value) language.value = e.value
      if (e.key === 'default_view' && e.value) defaultView.value = e.value as typeof defaultView.value
      if (e.key === 'startup_view' && e.value) startupView.value = e.value as typeof startupView.value
      if (e.key === 'date_format' && e.value) dateFormat.value = e.value
      if (e.key === 'page_size' && e.value) {
        const v = parseInt(e.value)
        if (!isNaN(v) && v > 0) pageSize.value = v
      }
    }
  }

  async function save(key: string, value: string | null) {
    await invoke('set_setting', { key, value })
  }

  watch(theme,       v => save('theme', v))
  watch(language,    v => save('language', v))
  watch(defaultView, v => save('default_view', v))
  watch(startupView, v => save('startup_view', v))
  watch(dateFormat,  v => save('date_format', v))
  watch(pageSize,    v => save('page_size', String(v)))

  return { theme, language, defaultView, startupView, dateFormat, pageSize, load }
})
