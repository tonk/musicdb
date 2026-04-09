<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { MbRelease } from '../../types'

const props = defineProps<{
  initialTitle: string
  initialArtist: string
}>()

const emit = defineEmits<{
  selected: [release: MbRelease]
  close: []
}>()

const title    = ref(props.initialTitle)
const artist   = ref(props.initialArtist)
const results  = ref<MbRelease[]>([])
const loading  = ref(false)
const searched = ref(false)
const error    = ref<string | null>(null)

async function search() {
  loading.value = true
  error.value = null
  searched.value = false
  try {
    results.value = await invoke<MbRelease[]>('lookup_release', { title: title.value, artist: artist.value })
    searched.value = true
  } catch (e: unknown) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <Teleport to="body">
    <div class="modal-overlay" @click.self="emit('close')">
      <div class="modal" style="width: 620px; max-width: 95vw;">
        <div class="flex items-center gap-2" style="margin-bottom: 16px;">
          <h3 style="margin: 0; font-size: 16px; flex: 1;">MusicBrainz Lookup</h3>
          <button class="btn btn-ghost" style="padding: 4px 8px;" @click="emit('close')">✕</button>
        </div>

        <div style="display: flex; gap: 8px; margin-bottom: 12px;">
          <input v-model="title"  class="form-control" placeholder="Title"  style="flex: 2;" />
          <input v-model="artist" class="form-control" placeholder="Artist" style="flex: 2;" />
          <button class="btn btn-primary" :disabled="loading" @click="search">
            {{ loading ? '…' : 'Search' }}
          </button>
        </div>

        <p v-if="error" style="color: var(--color-danger); font-size: 13px;">{{ error }}</p>

        <div v-if="loading" class="flex items-center gap-2">
          <div class="spinner" />
          <span class="text-muted text-sm">Searching MusicBrainz…</span>
        </div>

        <p v-else-if="searched && results.length === 0" class="text-muted text-sm">No results found.</p>

        <table v-else-if="results.length" class="data-table" style="font-size: 13px;">
          <thead>
            <tr>
              <th>Title</th>
              <th>Artist</th>
              <th>Year</th>
              <th>Label</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="r in results" :key="r.id">
              <td class="truncate" style="max-width: 160px;">{{ r.title }}</td>
              <td class="truncate text-muted" style="max-width: 120px;">{{ r.artist }}</td>
              <td class="text-faint">{{ r.date?.slice(0, 4) }}</td>
              <td class="truncate text-faint" style="max-width: 120px;">{{ r.label }}</td>
              <td>
                <button class="btn btn-primary" style="padding: 3px 10px; font-size: 12px;" @click="emit('selected', r)">
                  Use
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </Teleport>
</template>
