import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useUiStore = defineStore('ui', () => {
  const showConfirmDelete = ref(false)
  const pendingDeleteId = ref<number | null>(null)
  const showUndoToast = ref(false)
  const undoTimer = ref<ReturnType<typeof setTimeout> | null>(null)

  function requestDelete(id: number) {
    pendingDeleteId.value = id
    showConfirmDelete.value = true
  }

  function cancelDelete() {
    pendingDeleteId.value = null
    showConfirmDelete.value = false
  }

  function showUndo() {
    showUndoToast.value = true
    if (undoTimer.value) clearTimeout(undoTimer.value)
    undoTimer.value = setTimeout(() => {
      showUndoToast.value = false
    }, 5000)
  }

  function hideUndo() {
    showUndoToast.value = false
    if (undoTimer.value) clearTimeout(undoTimer.value)
  }

  return { showConfirmDelete, pendingDeleteId, showUndoToast, requestDelete, cancelDelete, showUndo, hideUndo }
})
