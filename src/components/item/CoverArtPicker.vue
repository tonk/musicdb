<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { convertFileSrc } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

const props = defineProps<{
  itemId: number
  coverPath: string | null
}>()

const emit = defineEmits<{
  updated: [path: string]
}>()

const loading = ref(false)
const error   = ref<string | null>(null)

function currentSrc(): string | null {
  return props.coverPath ? convertFileSrc(props.coverPath) : null
}

async function pickFile() {
  const selected = await open({
    filters: [{ name: 'Images', extensions: ['jpg', 'jpeg', 'png', 'webp'] }],
    multiple: false,
  })
  if (!selected || Array.isArray(selected)) return

  loading.value = true
  error.value = null
  try {
    // Read file as base64 via fetch on asset protocol
    const src = convertFileSrc(selected)
    const resp = await fetch(src)
    const blob = await resp.blob()
    const base64 = await blobToBase64(blob)
    const saved: string = await invoke('save_cover_art', { itemId: props.itemId, imageBase64: base64 })
    emit('updated', saved)
  } catch (e: unknown) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

async function fetchFromMB(mbid: string) {
  loading.value = true
  error.value = null
  try {
    const saved: string = await invoke('fetch_caa_cover', { itemId: props.itemId, mbid })
    emit('updated', saved)
  } catch (e: unknown) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

function blobToBase64(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(reader.result as string)
    reader.onerror = reject
    reader.readAsDataURL(blob)
  })
}

defineExpose({ fetchFromMB })
</script>

<template>
  <div style="display: flex; flex-direction: column; gap: 8px; align-items: flex-start;">
    <div
      style="width: 180px; height: 180px; border-radius: var(--radius-md); overflow: hidden;
             border: 1px solid var(--color-border); background: var(--color-bg-tertiary);
             display: flex; align-items: center; justify-content: center; font-size: 48px;"
    >
      <img v-if="currentSrc()" :src="currentSrc()!" alt="Cover" style="width: 100%; height: 100%; object-fit: cover;" />
      <span v-else>💿</span>
    </div>

    <button class="btn btn-ghost" style="font-size: 12px; padding: 4px 10px;" :disabled="loading" @click="pickFile">
      {{ loading ? '…' : '📁 Choose image' }}
    </button>

    <p v-if="error" style="font-size: 12px; color: var(--color-danger);">{{ error }}</p>
  </div>
</template>
