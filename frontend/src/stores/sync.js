import { defineStore } from 'pinia'
import { syncNow } from '../services/sync'

export const useSyncStore = defineStore('sync', {
  state: () => ({
    isSyncing: false,
    lastError: '',
  }),
  actions: {
    async sync(uid) {
      if (!uid) return

      this.isSyncing = true
      this.lastError = ''

      try {
        await syncNow(uid)
      } catch (error) {
        this.lastError = error?.message || 'Sync failed'
      } finally {
        this.isSyncing = false
      }
    },
  },
})
