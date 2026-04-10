<script setup lang="ts">
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import ArtistField from './ArtistField.vue'
import type { ItemWithArtists, Genre } from '../../types'
import { FORMATS, CONDITIONS } from '../../types'

interface ArtistEntry { artist_id: number; name: string; role: string }

const props = defineProps<{
  item?: ItemWithArtists
}>()

const emit = defineEmits<{
  saved: [item: ItemWithArtists]
  cancel: []
}>()

const { t } = useI18n()

// ── form state ────────────────────────────────────────────────────────────────
const title          = ref(props.item?.title ?? '')
const format         = ref(props.item?.format ?? 'CD')
const year           = ref<number | ''>(props.item?.year ?? '')
const label          = ref(props.item?.label ?? '')
const publisher      = ref(props.item?.publisher ?? '')
const catalogueNum   = ref(props.item?.catalogue_number ?? '')
const condition      = ref(props.item?.condition ?? '')
const notes          = ref(props.item?.notes ?? '')
const totalTime      = ref(props.item?.total_time ?? '')
const archiveNumber  = ref(props.item?.archive_number ?? '')

const artists = ref<ArtistEntry[]>(
  props.item?.artists.map(a => ({ artist_id: a.id, name: a.name, role: a.role })) ?? []
)

const allGenres   = ref<Genre[]>([])
const genreIds    = ref<number[]>(props.item?.genres.map(g => g.id) ?? [])
const genreInput  = ref('')

const saving  = ref(false)
const error   = ref<string | null>(null)

// ── load genres ───────────────────────────────────────────────────────────────
invoke<Genre[]>('list_genres').then(g => { allGenres.value = g })

function toggleGenre(id: number) {
  const idx = genreIds.value.indexOf(id)
  if (idx >= 0) genreIds.value.splice(idx, 1)
  else genreIds.value.push(id)
}

async function addGenreByName() {
  const name = genreInput.value.trim()
  if (!name) return
  const genre = await invoke<Genre>('create_genre', { name })
  if (!allGenres.value.find(g => g.id === genre.id)) allGenres.value.push(genre)
  if (!genreIds.value.includes(genre.id)) genreIds.value.push(genre.id)
  genreInput.value = ''
}

// ── submit ────────────────────────────────────────────────────────────────────
async function submit() {
  if (!title.value.trim()) { error.value = 'Title is required'; return }
  saving.value = true
  error.value = null
  try {
    const payload = {
      title: title.value.trim(),
      format: format.value,
      year: year.value === '' ? null : Number(year.value),
      label: label.value.trim() || null,
      publisher: publisher.value.trim() || null,
      catalogue_number: catalogueNum.value.trim() || null,
      condition: condition.value || null,
      notes: notes.value.trim() || null,
      musicbrainz_id: props.item?.musicbrainz_id ?? null,
      total_time: totalTime.value.trim() || null,
      archive_number: archiveNumber.value.trim() || null,
      artist_ids: artists.value.map(a => ({ artist_id: a.artist_id, role: a.role })),
      genre_ids: genreIds.value,
    }

    let saved: ItemWithArtists
    if (props.item) {
      saved = await invoke('update_item', { id: props.item.id, input: payload })
    } else {
      saved = await invoke('create_item', { input: payload })
    }
    emit('saved', saved)
  } catch (e: unknown) {
    error.value = String(e)
  } finally {
    saving.value = false
  }
}

// Sync from props when item changes (used when switching between edit mode)
watch(() => props.item, (item) => {
  if (!item) return
  title.value = item.title
  format.value = item.format
  year.value = item.year ?? ''
  label.value = item.label ?? ''
  publisher.value = item.publisher ?? ''
  catalogueNum.value = item.catalogue_number ?? ''
  condition.value = item.condition ?? ''
  notes.value = item.notes ?? ''
  totalTime.value = item.total_time ?? ''
  archiveNumber.value = item.archive_number ?? ''
  artists.value = item.artists.map(a => ({ artist_id: a.id, name: a.name, role: a.role }))
  genreIds.value = item.genres.map(g => g.id)
})
</script>

<template>
  <form @submit.prevent="submit" style="max-width: 640px;">
    <div v-if="error" style="padding: 10px 14px; background: var(--color-danger); color: var(--color-accent-text);
                              border-radius: var(--radius-sm); margin-bottom: 16px; font-size: 13px;">
      {{ error }}
    </div>

    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 0 16px;">
      <!-- Title -->
      <div class="form-group" style="grid-column: 1 / -1;">
        <label class="form-label">{{ t('item.title') }} *</label>
        <input v-model="title" class="form-control" required />
      </div>

      <!-- Format -->
      <div class="form-group">
        <label class="form-label">{{ t('item.format') }}</label>
        <select v-model="format" class="form-control">
          <option v-for="f in FORMATS" :key="f" :value="f">{{ f }}</option>
        </select>
      </div>

      <!-- Year -->
      <div class="form-group">
        <label class="form-label">{{ t('item.year') }}</label>
        <input v-model="year" class="form-control" type="number" min="1800" max="2100" placeholder="e.g. 1998" />
      </div>

      <!-- Label -->
      <div class="form-group">
        <label class="form-label">{{ t('item.label') }}</label>
        <input v-model="label" class="form-control" />
      </div>

      <!-- Publisher -->
      <div class="form-group">
        <label class="form-label">{{ t('item.publisher') }}</label>
        <input v-model="publisher" class="form-control" />
      </div>

      <!-- Catalogue # -->
      <div class="form-group">
        <label class="form-label">{{ t('item.catalogueNumber') }}</label>
        <input v-model="catalogueNum" class="form-control" />
      </div>

      <!-- Condition -->
      <div class="form-group">
        <label class="form-label">{{ t('item.condition') }}</label>
        <select v-model="condition" class="form-control">
          <option value="">—</option>
          <option v-for="c in CONDITIONS" :key="c" :value="c">{{ c }}</option>
        </select>
      </div>

      <!-- Total time -->
      <div class="form-group">
        <label class="form-label">{{ t('item.totalTime') }}</label>
        <input v-model="totalTime" class="form-control" placeholder="e.g. 45:30" />
      </div>

      <!-- Archive number -->
      <div class="form-group">
        <label class="form-label">{{ t('item.archiveNumber') }}</label>
        <input v-model="archiveNumber" class="form-control" />
      </div>

      <!-- Notes -->
      <div class="form-group" style="grid-column: 1 / -1;">
        <label class="form-label">{{ t('item.notes') }}</label>
        <textarea v-model="notes" class="form-control" rows="3" style="resize: vertical;" />
      </div>

      <!-- Artists -->
      <div class="form-group" style="grid-column: 1 / -1;">
        <label class="form-label">{{ t('item.artist') }}</label>
        <ArtistField v-model="artists" />
      </div>

      <!-- Genres -->
      <div class="form-group" style="grid-column: 1 / -1;">
        <label class="form-label">{{ t('item.genre') }}</label>
        <div style="display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 8px;">
          <button
            v-for="g in allGenres"
            :key="g.id"
            type="button"
            class="chip"
            :style="genreIds.includes(g.id) ? 'background: var(--color-accent); color: var(--color-accent-text);' : ''"
            @click="toggleGenre(g.id)"
          >
            {{ g.name }}
          </button>
        </div>
        <div style="display: flex; gap: 8px;">
          <input v-model="genreInput" class="form-control" placeholder="Add genre…"
                 @keydown.enter.prevent="addGenreByName" style="max-width: 200px;" />
          <button type="button" class="btn btn-ghost" @click="addGenreByName">+</button>
        </div>
      </div>
    </div>

    <div class="flex gap-2" style="margin-top: 8px;">
      <button type="submit" class="btn btn-primary" :disabled="saving">
        {{ saving ? t('common.loading') : t('item.save') }}
      </button>
      <button type="button" class="btn btn-secondary" @click="emit('cancel')">
        {{ t('item.cancel') }}
      </button>
    </div>
  </form>
</template>
