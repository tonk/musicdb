<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useCollectionStore } from '../stores/collection'
import { useSettingsStore } from '../stores/settings'
import ItemList from '../components/collection/ItemList.vue'
import ItemGrid from '../components/collection/ItemGrid.vue'
import FilterPanel from '../components/collection/FilterPanel.vue'

const { t }      = useI18n()
const collection = useCollectionStore()
const settings   = useSettingsStore()

onMounted(() => collection.fetchItems())

const totalPages = computed(() => Math.max(1, Math.ceil(collection.total / collection.pageSize)))

function prevPage() {
  if (collection.page > 1) { collection.page--; collection.fetchItems() }
}
function nextPage() {
  if (collection.page < totalPages.value) { collection.page++; collection.fetchItems() }
}

function onGridSortFieldChange(event: Event) {
  const target = event.target as HTMLSelectElement
  collection.sortField = target.value
  collection.page = 1
  collection.fetchItems()
}

function onGridSortDirChange(event: Event) {
  const target = event.target as HTMLSelectElement
  collection.sortDir = target.value as 'asc' | 'desc'
  collection.page = 1
  collection.fetchItems()
}

const gridSortField = computed(() => {
  return collection.sortField === 'artist' || collection.sortField === 'artist_names' ? 'artist' : 'album'
})
</script>

<template>
  <div style="display: flex; gap: 20px; align-items: flex-start;">
    <FilterPanel />

    <div style="flex: 1; min-width: 0;">
      <!-- Header row -->
      <div class="flex items-center gap-2" style="margin-bottom: 12px;">
        <span class="text-muted text-sm">{{ collection.total }} {{ t('collection.title') }}</span>
        <div v-if="collection.loading" class="spinner" style="width: 16px; height: 16px; margin-left: 8px;" />
        <div
          v-if="settings.defaultView !== 'list'"
          class="flex items-center gap-2"
          style="margin-left: auto;"
        >
          <label for="grid-sort-field" class="text-muted text-sm">{{ t('collection.sortBy') }}</label>
          <select
            id="grid-sort-field"
            class="input"
            :value="gridSortField"
            @change="onGridSortFieldChange"
          >
            <option value="artist">{{ t('item.artist') }}</option>
            <option value="album">{{ t('collection.sortAlbum') }}</option>
          </select>
          <select
            id="grid-sort-dir"
            class="input"
            :value="collection.sortDir"
            @change="onGridSortDirChange"
          >
            <option value="asc">{{ t('collection.sortAscending') }}</option>
            <option value="desc">{{ t('collection.sortDescending') }}</option>
          </select>
        </div>
      </div>

      <div v-if="!collection.loading && collection.items.length === 0"
           style="padding: 48px; text-align: center; color: var(--color-text-muted);">
        {{ t('collection.empty') }}
      </div>

      <ItemList v-else-if="settings.defaultView === 'list'" :items="collection.items" />
      <ItemGrid v-else :items="collection.items" />

      <!-- Pagination -->
      <div v-if="totalPages > 1" class="flex items-center gap-2" style="margin-top: 16px;">
        <button class="btn btn-ghost" :disabled="collection.page <= 1" @click="prevPage">← Prev</button>
        <span class="text-muted text-sm">Page {{ collection.page }} / {{ totalPages }}</span>
        <button class="btn btn-ghost" :disabled="collection.page >= totalPages" @click="nextPage">Next →</button>
      </div>
    </div>
  </div>
</template>
