import { watch } from 'vue'
import { useSettingsStore } from '../stores/settings'

export function useTheme() {
  const settings = useSettingsStore()

  function applyTheme(theme: string) {
    const root = document.documentElement
    if (theme === 'system') {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
      root.setAttribute('data-theme', prefersDark ? 'dark' : 'light')
    } else {
      root.setAttribute('data-theme', theme)
    }
  }

  watch(() => settings.theme, applyTheme, { immediate: true })

  // React to OS theme changes when set to 'system'
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    if (settings.theme === 'system') applyTheme('system')
  })
}
