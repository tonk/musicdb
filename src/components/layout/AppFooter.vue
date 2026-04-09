<script setup lang="ts">
import { useCollectionStore } from '../../stores/collection'

const collection = useCollectionStore()
</script>

<template>
  <footer class="app-footer">
    <span>MusicDB</span>
    <span>{{ collection.total }} items</span>
  </footer>
</template>
