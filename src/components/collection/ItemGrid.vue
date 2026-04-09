<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useUiStore } from '../../stores/ui'
import type { ItemSummary } from '../../types'

defineProps<{ items: ItemSummary[] }>()

const router = useRouter()
const ui     = useUiStore()
</script>

<template>
  <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(160px, 1fr)); gap: 16px;">
    <div
      v-for="item in items"
      :key="item.id"
      class="card"
      style="cursor: pointer; padding: 0; overflow: hidden;"
      @click="router.push({ name: 'item-detail', params: { id: item.id } })"
    >
      <!-- Cover -->
      <div style="aspect-ratio: 1; background: var(--color-bg-tertiary); display: flex;
                  align-items: center; justify-content: center; font-size: 48px; overflow: hidden;">
        <img
          v-if="item.cover_art_path"
          :src="item.cover_art_path"
          alt=""
          style="width: 100%; height: 100%; object-fit: cover;"
        />
        <span v-else>💿</span>
      </div>
      <!-- Info -->
      <div style="padding: 8px 10px;">
        <div class="truncate" style="font-weight: 500; font-size: 13px;">{{ item.title }}</div>
        <div class="truncate text-muted text-sm">{{ item.artist_names }}</div>
        <div class="flex items-center gap-2" style="margin-top: 4px;">
          <span class="text-faint text-xs">{{ item.format }}</span>
          <span v-if="item.year" class="text-faint text-xs">{{ item.year }}</span>
          <span
            class="text-faint text-xs"
            style="margin-left: auto; cursor: pointer;"
            @click.stop="ui.requestDelete(item.id)"
          >✕</span>
        </div>
      </div>
    </div>
  </div>
</template>
