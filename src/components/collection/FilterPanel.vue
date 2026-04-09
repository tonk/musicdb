<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { useCollectionStore } from '../../stores/collection'
import type { Genre } from '../../types'
import { FORMATS, CONDITIONS } from '../../types'

const { t }      = useI18n()
const collection = useCollectionStore()

const genres  = ref<Genre[]>([])
const format    = ref('')
const condition = ref('')
const yearFrom  = ref('')
const yearTo    = ref('')
const genreId   = ref('')

onMounted(async () => {
  genres.value = await invoke<Genre[]>('list_genres')
})

function apply() {
  if (format.value)    collection.setFilter('format',     format.value)
  else                 delete (collection.filters as Record<string, unknown>)['format']
  if (condition.value) collection.setFilter('condition',  condition.value)
  else                 delete (collection.filters as Record<string, unknown>)['condition']
  if (yearFrom.value)  collection.setFilter('year_from',  Number(yearFrom.value))
  else                 delete (collection.filters as Record<string, unknown>)['year_from']
  if (yearTo.value)    collection.setFilter('year_to',    Number(yearTo.value))
  else                 delete (collection.filters as Record<string, unknown>)['year_to']
  if (genreId.value)   collection.setFilter('genre_id',   Number(genreId.value))
  else                 delete (collection.filters as Record<string, unknown>)['genre_id']
  collection.fetchItems()
}

function clear() {
  format.value = condition.value = yearFrom.value = yearTo.value = genreId.value = ''
  collection.clearFilters()
}
</script>

<template>
  <aside style="width: 200px; flex-shrink: 0; display: flex; flex-direction: column; gap: 12px;">
    <div style="font-weight: 600; font-size: 13px;">{{ t('collection.filters') }}</div>

    <div class="form-group" style="margin-bottom: 0;">
      <label class="form-label">{{ t('item.format') }}</label>
      <select v-model="format" class="form-control" style="font-size: 13px;">
        <option value="">{{ t('common.noData') }}</option>
        <option v-for="f in FORMATS" :key="f" :value="f">{{ f }}</option>
      </select>
    </div>

    <div class="form-group" style="margin-bottom: 0;">
      <label class="form-label">{{ t('item.condition') }}</label>
      <select v-model="condition" class="form-control" style="font-size: 13px;">
        <option value="">{{ t('common.noData') }}</option>
        <option v-for="c in CONDITIONS" :key="c" :value="c">{{ c }}</option>
      </select>
    </div>

    <div class="form-group" style="margin-bottom: 0;">
      <label class="form-label">{{ t('item.year') }}</label>
      <div style="display: flex; gap: 4px;">
        <input v-model="yearFrom" class="form-control" type="number" placeholder="from" style="font-size: 12px;" />
        <input v-model="yearTo"   class="form-control" type="number" placeholder="to"   style="font-size: 12px;" />
      </div>
    </div>

    <div class="form-group" style="margin-bottom: 0;">
      <label class="form-label">{{ t('item.genre') }}</label>
      <select v-model="genreId" class="form-control" style="font-size: 13px;">
        <option value="">{{ t('common.noData') }}</option>
        <option v-for="g in genres" :key="g.id" :value="g.id">{{ g.name }}</option>
      </select>
    </div>

    <div style="display: flex; flex-direction: column; gap: 6px;">
      <button class="btn btn-primary" style="font-size: 13px; padding: 6px 12px;" @click="apply">Apply</button>
      <button class="btn btn-ghost"   style="font-size: 13px; padding: 6px 12px;" @click="clear">
        {{ t('collection.clearFilters') }}
      </button>
    </div>
  </aside>
</template>
