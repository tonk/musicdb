import { createRouter, createWebHashHistory } from 'vue-router'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/',             name: 'collection',    component: () => import('../views/CollectionView.vue') },
    { path: '/item/new',     name: 'add-item',      component: () => import('../views/AddItemView.vue') },
    { path: '/item/:id',     name: 'item-detail',   component: () => import('../views/ItemDetailView.vue') },
    { path: '/search',       name: 'search',        component: () => import('../views/SearchView.vue') },
    { path: '/artist/:id',   name: 'artist-detail', component: () => import('../views/ArtistDetailView.vue') },
    { path: '/statistics',   name: 'statistics',    component: () => import('../views/StatisticsView.vue') },
    { path: '/import',       name: 'import',        component: () => import('../views/ImportView.vue') },
    { path: '/settings',     name: 'settings',      component: () => import('../views/SettingsView.vue') },
  ],
})

export default router
