<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import ItemList from '../components/collection/ItemList.vue'
import type { Artist, ItemSummary } from '../types'

const route  = useRoute()
const router = useRouter()

const artist  = ref<Artist | null>(null)
const items   = ref<ItemSummary[]>([])
const loading = ref(true)
const error   = ref<string | null>(null)

onMounted(async () => {
  const id = Number(route.params.id)
  try {
    const [allArtists, artistItems] = await Promise.all([
      invoke<Artist[]>('list_artists'),
      invoke<ItemSummary[]>('get_artist_items', { artistId: id }),
    ])
    artist.value = allArtists.find(a => a.id === id) ?? null
    items.value = artistItems
  } catch (e: unknown) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div>
    <div class="flex items-center gap-2" style="margin-bottom: 16px;">
      <button class="btn btn-ghost" @click="router.back()">← Back</button>
    </div>

    <div v-if="loading" class="text-muted">Loading…</div>
    <div v-else-if="error" style="color: var(--color-danger);">{{ error }}</div>

    <template v-else>
      <h1 style="margin: 0 0 4px; font-size: 22px;">{{ artist?.name ?? `Artist #${route.params.id}` }}</h1>
      <p v-if="artist?.sort_name && artist.sort_name !== artist.name"
         class="text-muted text-sm" style="margin: 0 0 20px;">
        {{ artist.sort_name }}
      </p>
      <p v-else style="margin: 0 0 20px;" />

      <h3 style="margin: 0 0 8px; font-size: 14px; text-transform: uppercase; letter-spacing: 0.05em; color: var(--color-text-muted);">
        {{ items.length }} release{{ items.length !== 1 ? 's' : '' }}
      </h3>
      <ItemList v-if="items.length" :items="items" />
      <p v-else class="text-muted">No releases found.</p>
    </template>
  </div>
</template>
