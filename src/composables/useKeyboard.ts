import { onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'

// ── Zoom ─────────────────────────────────────────────────────────────────────
const ZOOM_STEP = 0.1
const ZOOM_MIN  = 0.5
const ZOOM_MAX  = 3.0
let currentZoom = 1.0

function applyZoom(level: number) {
  currentZoom = Math.round(Math.min(Math.max(level, ZOOM_MIN), ZOOM_MAX) * 10) / 10
  document.documentElement.style.zoom = String(currentZoom)
}

// ─────────────────────────────────────────────────────────────────────────────

export function useKeyboard() {
  const router = useRouter()

  function onKeyDown(e: KeyboardEvent) {
    // Zoom shortcuts work regardless of focused element
    if (e.ctrlKey && !e.altKey && !e.metaKey) {
      if (e.key === '+' || e.key === '=') { e.preventDefault(); applyZoom(currentZoom + ZOOM_STEP); return }
      if (e.key === '-' || e.key === '_') { e.preventDefault(); applyZoom(currentZoom - ZOOM_STEP); return }
      if (e.key === '0')                  { e.preventDefault(); applyZoom(1.0);                     return }
    }

    // Skip navigation shortcuts when typing in an input, textarea, or select
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
