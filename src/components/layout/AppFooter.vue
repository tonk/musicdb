<script setup lang="ts">
import { useCollectionStore } from '../../stores/collection'

const collection = useCollectionStore()
const version = __APP_VERSION__
</script>

<template>
  <footer class="app-footer">
    <span>MusicDB {{ version }}</span>
    <span>{{ collection.total }} items</span>
  </footer>
</template>
