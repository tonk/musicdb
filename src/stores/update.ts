import { defineStore } from 'pinia'
import { ref } from 'vue'
import { fetch } from '@tauri-apps/plugin-http'

export const useUpdateStore = defineStore('update', () => {
  const currentVersion = __APP_VERSION__
  const latestVersion = ref<string | null>(null)
  const isUpdateAvailable = ref(false)
  const checking = ref(false)
  const error = ref<string | null>(null)

  function compareVersions(vLatest: string, vCurrent: string): number {
    const p1 = vLatest.replace(/^v/, '').split('.').map(Number)
    const p2 = vCurrent.replace(/^v/, '').split('.').map(Number)
    for (let i = 0; i < Math.max(p1.length, p2.length); i++) {
      const a = p1[i] || 0
      const b = p2[i] || 0
      if (a > b) return 1
      if (a < b) return -1
    }
    return 0
  }

  async function checkUpdate() {
    if (checking.value) return
    checking.value = true
    error.value = null
    try {
      const res = await fetch('https://api.github.com/repos/tonk/musicdb/releases/latest', {
        method: 'GET',
        headers: {
          'User-Agent': 'MusicDB'
        }
      })
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
      const data = await res.json() as { tag_name: string }
      if (data && data.tag_name) {
        latestVersion.value = data.tag_name
        isUpdateAvailable.value = compareVersions(data.tag_name, currentVersion) > 0
      }
    } catch (e) {
      console.error('Update check failed:', e)
      error.value = String(e)
    } finally {
      checking.value = false
    }
  }

  return { currentVersion, latestVersion, isUpdateAvailable, checking, error, checkUpdate }
})
