export interface Artist {
  id: number
  name: string
  sort_name: string
  created_at: string
}

export interface ArtistWithRole extends Artist {
  role: string
  sort_order: number
}

export interface Genre {
  id: number
  name: string
}

export interface Track {
  id: number
  item_id: number
  disc_id: string
  track_number: string
  title: string
  duration_secs: number | null
  version: string | null
  sort_order: number
  artists: ArtistWithRole[]
}

export interface ItemSummary {
  id: number
  title: string
  format: string
  year: number | null
  label: string | null
  catalogue_number: string | null
  cover_art_path: string | null
  date_added: string
  artist_names: string
}

export interface ItemWithArtists extends ItemSummary {
  publisher: string | null
  condition: string | null
  notes: string | null
  disc_id: string | null
  source_category: string | null
  musicbrainz_id: string | null
  updated_at: string
  artists: ArtistWithRole[]
  genres: Genre[]
  tracks: Track[]
}

export interface ItemsPage {
  items: ItemSummary[]
  total: number
  page: number
  page_size: number
}

export interface ListItemsParams {
  page?: number
  page_size?: number
  sort_field?: string
  sort_dir?: 'asc' | 'desc'
  format?: string
  condition?: string
  year_from?: number
  year_to?: number
  genre_id?: number
}

export interface CreateItemInput {
  title: string
  format: string
  year?: number | null
  label?: string | null
  publisher?: string | null
  catalogue_number?: string | null
  condition?: string | null
  notes?: string | null
  musicbrainz_id?: string | null
  artist_ids: { artist_id: number; role: string }[]
  genre_ids: number[]
}

export interface Statistics {
  total_items: number
  by_format: CountEntry[]
  by_genre: CountEntry[]
  by_year: CountEntry[]
}

export interface CountEntry {
  label: string
  count: number
}

export interface ImportSummary {
  total: number
  imported: number
  skipped: number
}

export interface MbRelease {
  id: string
  title: string
  date: string | null
  label: string | null
  catalogue: string | null
  genres: string[]
  artist: string
}

export const FORMATS = ['CD', 'LP', 'EP', 'Single', 'Cassette', 'Music Book', 'Sheet Music', 'Other'] as const
export const CONDITIONS = ['Mint', 'Near Mint', 'VG+', 'VG', 'Good', 'Fair', 'Poor'] as const
