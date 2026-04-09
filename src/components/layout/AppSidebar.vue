<script setup lang="ts">
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useCollectionStore } from '../../stores/collection'

const { t } = useI18n()
const collection = useCollectionStore()
</script>

<template>
  <nav class="app-sidebar">
    <div style="padding: 16px 16px 8px; font-weight: 700; font-size: 16px; color: var(--color-text);">
      MusicDB
    </div>
    <RouterLink class="nav-item" :to="{ name: 'collection' }">
      <span>💿</span><span>{{ t('nav.collection') }}</span>
    </RouterLink>
    <RouterLink class="nav-item" :to="{ name: 'search' }">
      <span>🔍</span><span>{{ t('nav.search') }}</span>
    </RouterLink>
    <RouterLink class="nav-item" :to="{ name: 'statistics' }">
      <span>📊</span><span>{{ t('nav.statistics') }}</span>
    </RouterLink>
    <RouterLink class="nav-item" :to="{ name: 'import' }">
      <span>📥</span><span>{{ t('nav.import') }}</span>
    </RouterLink>
    <div style="flex: 1;" />
    <RouterLink class="nav-item" :to="{ name: 'settings' }">
      <span>⚙️</span><span>{{ t('nav.settings') }}</span>
    </RouterLink>
    <div style="padding: 8px 16px; font-size: 11px; color: var(--color-text-faint);">
      {{ collection.total }} items
    </div>
  </nav>
</template>
