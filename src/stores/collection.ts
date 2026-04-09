import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ItemSummary, ItemsPage, ItemWithArtists, ListItemsParams } from '../types'

export const useCollectionStore = defineStore('collection', () => {
  const items = ref<ItemSummary[]>([])
  const total = ref(0)
  const page = ref(1)
  const pageSize = ref(50)
  const sortField = ref('date_added')
  const sortDir = ref<'asc' | 'desc'>('desc')
  const filters = ref<Partial<ListItemsParams>>({})
  const loading = ref(false)

  const undoItem = ref<ItemWithArtists | null>(null)

  async function fetchItems() {
    loading.value = true
    try {
      const result: ItemsPage = await invoke('list_items', {
        params: {
          page: page.value,
          page_size: pageSize.value,
          sort_field: sortField.value,
          sort_dir: sortDir.value,
          ...filters.value,
        },
      })
      items.value = result.items
      total.value = result.total
    } finally {
      loading.value = false
    }
  }

  async function deleteItem(id: number) {
    await invoke('delete_item', { id })
    const restored: ItemWithArtists | null = null
    undoItem.value = restored
    await fetchItems()
  }

  async function undoDelete() {
    const result: ItemWithArtists | null = await invoke('undo_delete')
    undoItem.value = null
    if (result) await fetchItems()
    return result
  }

  function setSort(field: string) {
    if (sortField.value === field) {
      sortDir.value = sortDir.value === 'asc' ? 'desc' : 'asc'
    } else {
      sortField.value = field
      sortDir.value = 'asc'
    }
    page.value = 1
    fetchItems()
  }

  function setFilter(key: keyof ListItemsParams, value: unknown) {
    if (value == null || value === '') {
      delete (filters.value as Record<string, unknown>)[key]
    } else {
      (filters.value as Record<string, unknown>)[key] = value
    }
    page.value = 1
    fetchItems()
  }

  function clearFilters() {
    filters.value = {}
    page.value = 1
    fetchItems()
  }

  return {
    items, total, page, pageSize, sortField, sortDir,
    filters, loading, undoItem,
    fetchItems, deleteItem, undoDelete, setSort, setFilter, clearFilters,
  }
})
