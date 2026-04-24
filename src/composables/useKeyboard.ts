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

  function isCtrlZoomShortcut(e: KeyboardEvent): 'in' | 'out' | 'reset' | null {
    if (!e.ctrlKey || e.altKey || e.metaKey) return null
    if (e.key === '0') return 'reset'
    if (e.key === '+' || e.key === '=' || e.code === 'NumpadAdd') return 'in'
    if (e.key === '-' || e.key === '_' || e.code === 'NumpadSubtract') return 'out'
    return null
  }

  function onKeyDown(e: KeyboardEvent) {
    // Zoom shortcuts work regardless of focused element
    const zoomAction = isCtrlZoomShortcut(e)
    if (zoomAction) {
      e.preventDefault()
      if (zoomAction === 'in') applyZoom(currentZoom + ZOOM_STEP)
      if (zoomAction === 'out') applyZoom(currentZoom - ZOOM_STEP)
      if (zoomAction === 'reset') applyZoom(1.0)
      return
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

  function onWheel(e: WheelEvent) {
    if (!e.ctrlKey) return
    e.preventDefault()
    if (e.deltaY < 0) applyZoom(currentZoom + ZOOM_STEP)
    if (e.deltaY > 0) applyZoom(currentZoom - ZOOM_STEP)
  }

  onMounted(() => {
    document.addEventListener('keydown', onKeyDown)
    // passive:false is required so preventDefault() can stop browser/page zoom.
    document.addEventListener('wheel', onWheel, { passive: false })
  })
  onUnmounted(() => {
    document.removeEventListener('keydown', onKeyDown)
    document.removeEventListener('wheel', onWheel)
  })
}
