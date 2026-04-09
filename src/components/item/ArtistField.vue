<script setup lang="ts">
import { ref, watch } from 'vue'
import { useArtistsStore } from '../../stores/artists'
import type { Artist } from '../../types'

interface ArtistEntry {
  artist_id: number
  name: string
  role: string
}

const props = defineProps<{
  modelValue: ArtistEntry[]
}>()

const emit = defineEmits<{
  'update:modelValue': [value: ArtistEntry[]]
}>()

const artistsStore = useArtistsStore()

const inputText = ref('')
const suggestions = ref<Artist[]>([])
const showSuggestions = ref(false)
const activeRole = ref('artist')
let debounceTimer: ReturnType<typeof setTimeout> | null = null

watch(inputText, (val) => {
  if (debounceTimer) clearTimeout(debounceTimer)
  if (!val.trim()) { suggestions.value = []; return }
  debounceTimer = setTimeout(async () => {
    suggestions.value = await artistsStore.autocomplete(val)
    showSuggestions.value = suggestions.value.length > 0
  }, 200)
})

function selectSuggestion(artist: Artist) {
  addEntry(artist.id, artist.name)
}

async function addNew() {
  const name = inputText.value.trim()
  if (!name) return
  const existing = suggestions.value.find(a => a.name.toLowerCase() === name.toLowerCase())
  if (existing) {
    addEntry(existing.id, existing.name)
    return
  }
  const artist = await artistsStore.createArtist(name)
  addEntry(artist.id, artist.name)
}

function addEntry(id: number, name: string) {
  if (props.modelValue.some(e => e.artist_id === id && e.role === activeRole.value)) return
  emit('update:modelValue', [...props.modelValue, { artist_id: id, name, role: activeRole.value }])
  inputText.value = ''
  suggestions.value = []
  showSuggestions.value = false
}

function remove(index: number) {
  const updated = [...props.modelValue]
  updated.splice(index, 1)
  emit('update:modelValue', updated)
}

function onBlur() {
  setTimeout(() => { showSuggestions.value = false }, 150)
}

function updateRole(index: number, role: string) {
  const updated = props.modelValue.map((e, i) => i === index ? { ...e, role } : e)
  emit('update:modelValue', updated)
}
</script>

<template>
  <div class="artist-field">
    <div v-if="modelValue.length" style="display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 8px;">
      <div v-for="(entry, i) in modelValue" :key="i" class="chip" style="gap: 6px; align-items: center;">
        <span>{{ entry.name }}</span>
        <select
          :value="entry.role"
          @change="updateRole(i, ($event.target as HTMLSelectElement).value)"
          style="border: none; background: transparent; font-size: 11px; color: var(--color-text-muted); padding: 0; cursor: pointer;"
        >
          <option value="artist">artist</option>
          <option value="composer">composer</option>
          <option value="arranger">arranger</option>
        </select>
        <span class="chip-dismiss" @click="remove(i)">✕</span>
      </div>
    </div>

    <div style="display: flex; gap: 8px; align-items: center; position: relative;">
      <select v-model="activeRole" class="form-control" style="width: auto; flex-shrink: 0;">
        <option value="artist">artist</option>
        <option value="composer">composer</option>
        <option value="arranger">arranger</option>
      </select>
      <div style="position: relative; flex: 1;">
        <input
          v-model="inputText"
          class="form-control"
          placeholder="Search or add artist…"
          @keydown.enter.prevent="addNew"
          @keydown.escape="showSuggestions = false"
          @focus="showSuggestions = suggestions.length > 0"
          @blur="onBlur"
        />
        <ul
          v-if="showSuggestions"
          style="position: absolute; top: 100%; left: 0; right: 0; z-index: 50;
                 background: var(--color-surface); border: 1px solid var(--color-border);
                 border-radius: var(--radius-sm); margin: 2px 0; padding: 4px 0; list-style: none;
                 box-shadow: var(--shadow-md); max-height: 200px; overflow-y: auto;"
        >
          <li
            v-for="artist in suggestions"
            :key="artist.id"
            @mousedown.prevent="selectSuggestion(artist)"
            style="padding: 8px 12px; cursor: pointer;"
            onmouseover="this.style.background='var(--color-bg-secondary)'"
            onmouseout="this.style.background=''"
          >
            {{ artist.name }}
            <span style="font-size: 11px; color: var(--color-text-faint);">{{ artist.sort_name }}</span>
          </li>
          <li
            v-if="inputText.trim() && !suggestions.some(s => s.name.toLowerCase() === inputText.trim().toLowerCase())"
            @mousedown.prevent="addNew"
            style="padding: 8px 12px; cursor: pointer; color: var(--color-accent);"
            onmouseover="this.style.background='var(--color-bg-secondary)'"
            onmouseout="this.style.background=''"
          >
            + Create "{{ inputText.trim() }}"
          </li>
        </ul>
      </div>
    </div>
  </div>
</template>
