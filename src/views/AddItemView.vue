<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import ItemForm from '../components/item/ItemForm.vue'
import type { ItemWithArtists } from '../types'

const router = useRouter()
const { t } = useI18n()

function onSaved(item: ItemWithArtists) {
  router.push({ name: 'item-detail', params: { id: item.id } })
}
</script>

<template>
  <div>
    <div class="flex items-center gap-2" style="margin-bottom: 16px;">
      <button class="btn btn-ghost" @click="router.back()">← Back</button>
      <h2 style="margin: 0; font-size: 18px;">{{ t('collection.addItem') }}</h2>
    </div>
    <ItemForm @saved="onSaved" @cancel="router.back()" />
  </div>
</template>
