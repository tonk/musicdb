import { onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'

export function useKeyboard() {
  const router = useRouter()

  function onKeyDown(e: KeyboardEvent) {
    // Skip when typing in an input, textarea, or select
    const tag = (e.target as HTMLElement).tagName
    if (['INPUT', 'TEXTAREA', 'SELECT'].includes(tag)) return
    if ((e.target as HTMLElement).isContentEditable) return

    if (e.key === 'n' || e.key === 'N') {
      e.preventDefault()
      router.push({ name: 'add-item' })
    }
    if (e.key === 'f' || e.key === 'F') {
      e.preventDefault()
      router.push({ name: 'search' })
    }
  }

  onMounted(() => document.addEventListener('keydown', onKeyDown))
  onUnmounted(() => document.removeEventListener('keydown', onKeyDown))
}
