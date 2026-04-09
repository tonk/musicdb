<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useUiStore } from '../../stores/ui'
import { useCollectionStore } from '../../stores/collection'

const { t } = useI18n()
const ui = useUiStore()
const collection = useCollectionStore()

async function confirm() {
  if (ui.pendingDeleteId !== null) {
    await collection.deleteItem(ui.pendingDeleteId)
    ui.cancelDelete()
    ui.showUndo()
  }
}
</script>

<template>
  <Teleport to="body">
    <div v-if="ui.showConfirmDelete" class="modal-overlay" @click.self="ui.cancelDelete()">
      <div class="modal">
        <h3 style="margin: 0 0 12px; font-size: 16px;">{{ t('item.confirmDelete') }}</h3>
        <p style="margin: 0 0 20px; color: var(--color-text-muted);">{{ t('item.confirmDeleteBody') }}</p>
        <div class="flex gap-2" style="justify-content: flex-end;">
          <button class="btn btn-secondary" @click="ui.cancelDelete()">{{ t('item.cancel') }}</button>
          <button class="btn btn-danger" @click="confirm()">{{ t('item.delete') }}</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
