<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useUiStore } from '../../stores/ui'
import { useCollectionStore } from '../../stores/collection'

const { t } = useI18n()
const ui = useUiStore()
const collection = useCollectionStore()

async function handleUndo() {
  await collection.undoDelete()
  ui.hideUndo()
}
</script>

<template>
  <Teleport to="body">
    <Transition name="toast">
      <div v-if="ui.showUndoToast" class="toast">
        <span>{{ t('item.deleted') }}</span>
        <button class="btn btn-secondary" style="padding: 4px 12px; font-size: 13px;" @click="handleUndo()">
          {{ t('item.undo') }}
        </button>
        <button class="btn btn-ghost" style="padding: 4px 8px; font-size: 13px;" @click="ui.hideUndo()">
          ✕
        </button>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.toast-enter-active, .toast-leave-active { transition: opacity 0.2s, transform 0.2s; }
.toast-enter-from, .toast-leave-to { opacity: 0; transform: translateX(-50%) translateY(12px); }
</style>
