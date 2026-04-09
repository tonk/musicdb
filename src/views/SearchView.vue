<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import ItemList from '../components/collection/ItemList.vue'
import type { ItemSummary } from '../types'

const route  = useRoute()
const router = useRouter()
const { t }  = useI18n()

const queryInput = ref(String(route.query.q ?? ''))
const results    = ref<ItemSummary[]>([])
const loading    = ref(false)
const searched   = ref(false)

watch(() => route.query.q, q => {
  queryInput.value = String(q ?? '')
  if (queryInput.value) doSearch()
}, { immediate: true })

async function doSearch() {
  if (!queryInput.value.trim()) return
  loading.value = true
  searched.value = false
  try {
    results.value = await invoke<ItemSummary[]>('search_items', { query: queryInput.value.trim() })
  } finally {
    loading.value = false
    searched.value = true
  }
}

function submit() {
  router.push({ name: 'search', query: { q: queryInput.value.trim() } })
}
</script>

<template>
  <div>
    <form @submit.prevent="submit" style="display: flex; gap: 8px; margin-bottom: 20px;">
      <input
        v-model="queryInput"
        class="form-control"
        :placeholder="t('search.placeholder')"
        style="max-width: 480px;"
        autofocus
      />
      <button type="submit" class="btn btn-primary">{{ t('nav.search') }}</button>
    </form>

    <div v-if="loading" class="flex items-center gap-2">
      <div class="spinner" />
      <span class="text-muted">{{ t('common.loading') }}</span>
    </div>

    <template v-else-if="searched">
      <p v-if="results.length === 0" class="text-muted">
        {{ t('search.noResults', { query: queryInput }) }}
      </p>
      <template v-else>
        <p class="text-muted text-sm" style="margin-bottom: 12px;">
          {{ t('search.results', { count: results.length }) }}
        </p>
        <ItemList :items="results" />
      </template>
    </template>
  </div>
</template>
