<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import ItemForm from '../components/item/ItemForm.vue'
import CoverArtPicker from '../components/item/CoverArtPicker.vue'
import MbLookupModal from '../components/musicbrainz/MbLookupModal.vue'
import { useUiStore } from '../stores/ui'
import type { ItemWithArtists, MbRelease } from '../types'

const route  = useRoute()
const router = useRouter()
const { t }  = useI18n()
const ui     = useUiStore()

const item         = ref<ItemWithArtists | null>(null)
const loading      = ref(true)
const editing      = ref(false)
const showMbModal  = ref(false)
const error        = ref<string | null>(null)
const coverPicker  = ref<InstanceType<typeof CoverArtPicker> | null>(null)

onMounted(load)

async function load() {
  loading.value = true
  error.value = null
  try {
    item.value = await invoke<ItemWithArtists>('get_item', { id: Number(route.params.id) })
  } catch (e: unknown) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

function onSaved(saved: ItemWithArtists) {
  item.value = saved
  editing.value = false
}

function onCoverUpdated(path: string) {
  if (item.value) item.value = { ...item.value, cover_art_path: path }
}

async function applyMbRelease(release: MbRelease) {
  if (!item.value) return
  showMbModal.value = false
  const updated: ItemWithArtists = await invoke('update_item', {
    id: item.value.id,
    input: {
      title: release.title,
      label: release.label ?? null,
      catalogue_number: release.catalogue ?? null,
      year: release.date ? Number(release.date.slice(0, 4)) : null,
      musicbrainz_id: release.id,
    },
  })
  item.value = updated
  // Try to fetch cover from CAA
  if (coverPicker.value) {
    coverPicker.value.fetchFromMB(release.id)
  }
}

function requestDelete() {
  if (item.value) {
    ui.requestDelete(item.value.id)
    router.push({ name: 'collection' })
  }
}

function formatDuration(secs: number | null): string {
  if (secs == null) return ''
  const m = Math.floor(secs / 60)
  const s = String(secs % 60).padStart(2, '0')
  return `${m}:${s}`
}


</script>

<template>
  <div>
    <div class="flex items-center gap-2" style="margin-bottom: 16px;">
      <button class="btn btn-ghost" @click="router.push({ name: 'collection' })">← Back</button>
      <template v-if="item && !editing">
        <button class="btn btn-secondary" @click="editing = true">{{ t('item.edit') }}</button>
        <button class="btn btn-secondary" @click="showMbModal = true">{{ t('item.lookupMusicBrainz') }}</button>
        <button class="btn btn-danger" style="margin-left: auto;" @click="requestDelete">{{ t('item.delete') }}</button>
      </template>
    </div>

    <div v-if="loading" class="text-muted">{{ t('common.loading') }}</div>
    <div v-else-if="error" style="color: var(--color-danger);">{{ error }}</div>

    <template v-else-if="item">
      <!-- Edit mode -->
      <template v-if="editing">
        <h2 style="margin: 0 0 16px; font-size: 18px;">{{ t('item.edit') }}</h2>
        <ItemForm :item="item" @saved="onSaved" @cancel="editing = false" />
      </template>

      <!-- View mode -->
      <template v-else>
        <div style="display: flex; gap: 24px; align-items: flex-start;">
          <!-- Cover art picker -->
          <div style="flex-shrink: 0;">
            <CoverArtPicker
              ref="coverPicker"
              :item-id="item.id"
              :cover-path="item.cover_art_path"
              @updated="onCoverUpdated"
            />
          </div>

          <!-- Main info -->
          <div style="flex: 1; min-width: 0;">
            <h1 style="margin: 0 0 4px; font-size: 22px;">{{ item.title }}</h1>
            <div v-if="item.artists.length" style="margin-bottom: 8px; color: var(--color-text-muted);">
              <RouterLink
                v-for="a in item.artists"
                :key="a.id"
                :to="{ name: 'artist-detail', params: { id: a.id } }"
                style="color: inherit; text-decoration: none; margin-right: 6px;"
                @click.stop
              >{{ a.name }}</RouterLink>
            </div>
            <div style="display: flex; flex-wrap: wrap; gap: 8px; margin-bottom: 12px;">
              <span class="chip">{{ item.format }}</span>
              <span v-if="item.year" class="chip">{{ item.year }}</span>
              <span v-if="item.condition" class="chip">{{ item.condition }}</span>
              <span v-for="g in item.genres" :key="g.id" class="chip">{{ g.name }}</span>
            </div>
            <table style="font-size: 13px; border-collapse: collapse;">
              <tr v-if="item.label">
                <td style="padding: 2px 12px 2px 0; color: var(--color-text-muted);">{{ t('item.label') }}</td>
                <td>{{ item.label }}</td>
              </tr>
              <tr v-if="item.publisher">
                <td style="padding: 2px 12px 2px 0; color: var(--color-text-muted);">{{ t('item.publisher') }}</td>
                <td>{{ item.publisher }}</td>
              </tr>
              <tr v-if="item.catalogue_number">
                <td style="padding: 2px 12px 2px 0; color: var(--color-text-muted);">{{ t('item.catalogueNumber') }}</td>
                <td>{{ item.catalogue_number }}</td>
              </tr>
              <tr v-if="item.disc_id">
                <td style="padding: 2px 12px 2px 0; color: var(--color-text-muted);">{{ t('item.discId') }}</td>
                <td>{{ item.disc_id }}</td>
              </tr>
              <tr v-if="item.musicbrainz_id">
                <td style="padding: 2px 12px 2px 0; color: var(--color-text-muted);">MusicBrainz</td>
                <td><span class="text-faint text-sm">{{ item.musicbrainz_id }}</span></td>
              </tr>
              <tr>
                <td style="padding: 2px 12px 2px 0; color: var(--color-text-muted);">{{ t('item.dateAdded') }}</td>
                <td class="text-faint text-sm">{{ item.date_added.slice(0, 10) }}</td>
              </tr>
            </table>
            <p v-if="item.notes" style="margin: 12px 0 0; font-size: 13px; color: var(--color-text-muted);">
              {{ item.notes }}
            </p>
          </div>
        </div>

        <!-- Tracks -->
        <div v-if="item.tracks.length" style="margin-top: 24px;">
          <h3 style="margin: 0 0 8px; font-size: 15px;">{{ t('item.tracks') }} ({{ item.tracks.length }})</h3>
          <table class="data-table">
            <thead>
              <tr>
                <th style="width: 48px;">#</th>
                <th>{{ t('item.title') }}</th>
                <th>{{ t('item.artist') }}</th>
                <th>{{ t('item.version') }}</th>
                <th style="text-align: right;">{{ t('item.duration') }}</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="track in item.tracks" :key="track.id">
                <td class="text-faint text-sm">{{ track.track_number }}</td>
                <td>{{ track.title }}</td>
                <td class="text-muted text-sm">
                  {{ track.artists.filter(a => a.role === 'artist').map(a => a.name).join(', ') }}
                </td>
                <td class="text-faint text-sm">{{ track.version }}</td>
                <td style="text-align: right;" class="text-faint text-sm">
                  {{ formatDuration(track.duration_secs) }}
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </template>
    </template>

    <!-- MusicBrainz modal -->
    <MbLookupModal
      v-if="item && showMbModal"
      :initial-title="item.title"
      :initial-artist="item.artists[0]?.name ?? ''"
      @selected="applyMbRelease"
      @close="showMbModal = false"
    />
  </div>
</template>
