<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '../../stores/settings'

const { t } = useI18n()
const router = useRouter()
const settings = useSettingsStore()

const searchInput = ref('')

function submitSearch() {
  const q = searchInput.value.trim()
  if (!q) return
  router.push({ name: 'search', query: { q } })
}

function toggleView() {
  settings.defaultView = settings.defaultView === 'list' ? 'grid' : 'list'
}
</script>

<template>
  <header class="app-header">
    <form class="header-search" @submit.prevent="submitSearch">
      <input
        v-model="searchInput"
        class="form-control"
        :placeholder="t('search.placeholder')"
        style="max-width: 360px;"
      />
    </form>
    <div class="ml-auto flex items-center gap-2">
      <button class="btn btn-ghost" @click="toggleView" :title="t('settings.defaultView')">
        {{ settings.defaultView === 'list' ? '⊞' : '☰' }}
      </button>
      <button class="btn btn-primary" @click="router.push({ name: 'add-item' })">
        + {{ t('collection.addItem') }}
      </button>
    </div>
  </header>
</template>
