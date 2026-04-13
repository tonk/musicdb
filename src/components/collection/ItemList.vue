<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { convertFileSrc } from '@tauri-apps/api/core'
import { useCollectionStore } from '../../stores/collection'
import { useUiStore } from '../../stores/ui'
import type { ItemSummary } from '../../types'

defineProps<{ items: ItemSummary[] }>()

const router     = useRouter()
const { t }      = useI18n()
const collection = useCollectionStore()
const ui         = useUiStore()

const columns = [
  { key: 'title',            label: () => t('item.title') },
  { key: 'artist_names',     label: () => t('item.artist') },
  { key: 'format',           label: () => t('item.format') },
  { key: 'year',             label: () => t('item.year') },
  { key: 'label',            label: () => t('item.label') },
  { key: 'catalogue_number', label: () => t('item.catalogueNumber') },
] as const

function sortArrow(key: string): string {
  if (collection.sortField !== key) return ''
  return collection.sortDir === 'asc' ? ' ▲' : ' ▼'
}
</script>

<template>
  <table class="data-table">
    <thead>
      <tr>
        <th style="width: 48px;"></th>
        <th
          v-for="col in columns"
          :key="col.key"
          @click="collection.setSort(col.key)"
        >
          {{ col.label() }}{{ sortArrow(col.key) }}
        </th>
        <th style="width: 48px;"></th>
      </tr>
    </thead>
    <tbody>
      <tr
        v-for="item in items"
        :key="item.id"
        style="cursor: pointer;"
        @click="router.push({ name: 'item-detail', params: { id: item.id } })"
      >
        <td>
          <div style="width: 32px; height: 32px; border-radius: 4px; overflow: hidden;
                      background: var(--color-bg-tertiary); flex-shrink: 0; display: flex;
                      align-items: center; justify-content: center; font-size: 16px;">
            <img
              v-if="item.cover_art_path"
              :src="convertFileSrc(item.cover_art_path)"
              alt=""
              style="width: 100%; height: 100%; object-fit: cover;"
            />
            <span v-else>💿</span>
          </div>
        </td>
        <td class="truncate" style="max-width: 200px;">{{ item.title }}</td>
        <td class="truncate text-muted" style="max-width: 160px;">{{ item.artist_names }}</td>
        <td class="text-sm">{{ item.format }}</td>
        <td class="text-faint text-sm">{{ item.year }}</td>
        <td class="truncate text-faint text-sm" style="max-width: 140px;">{{ item.label }}</td>
        <td class="text-faint text-sm">{{ item.catalogue_number }}</td>
        <td @click.stop>
          <button
            class="btn btn-ghost"
            style="padding: 2px 8px; font-size: 12px;"
            @click="ui.requestDelete(item.id)"
          >✕</button>
        </td>
      </tr>
    </tbody>
  </table>
</template>
