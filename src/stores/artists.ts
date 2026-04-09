import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Artist } from '../types'

export const useArtistsStore = defineStore('artists', () => {
  const cache = ref<Map<string, Artist[]>>(new Map())

  async function autocomplete(query: string): Promise<Artist[]> {
    if (query.length < 1) return []
    const key = query.toLowerCase()
    if (cache.value.has(key)) return cache.value.get(key)!
    const results: Artist[] = await invoke('autocomplete_artists', { query })
    cache.value.set(key, results)
    return results
  }

  async function createArtist(name: string, sortName?: string): Promise<Artist> {
    const artist: Artist = await invoke('create_artist', { name, sortName: sortName ?? null })
    // Invalidate cache entries that could include this artist
    cache.value.clear()
    return artist
  }

  return { autocomplete, createArtist }
})
