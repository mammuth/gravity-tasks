import { defineStore } from 'pinia'
import { db } from '../db'
import { queueUpsert } from '../services/sync'
import { useSessionStore } from './session'
import { useTasksStore } from './tasks'

const nowIso = () => new Date().toISOString()
const activeListKey = (uid) => `active_list_id:${uid}`
const defaultListId = (uid) => `inbox-${uid}`

async function nextPosition(uid) {
  const items = await db.lists.where('uid').equals(uid).toArray()
  if (!items.length) return 1000
  const last = items.sort((a, b) => a.position - b.position).at(-1)
  return last?.position ? last.position + 1000 : 1000
}

export const useListsStore = defineStore('lists', {
  state: () => ({
    lists: [],
    activeListId: '',
  }),
  getters: {
    activeList: (state) => state.lists.find((list) => list.id === state.activeListId),
  },
  actions: {
    async refresh() {
      const session = useSessionStore()
      if (!session.uid) return

      const items = await db.lists.where('uid').equals(session.uid).toArray()
      const visible = items.filter((list) => !list.deleted_at)
      this.lists = visible.sort((a, b) => b.position - a.position)

      const stored = await db.meta.get(activeListKey(session.uid))
      const storedId = stored?.value
      const hasStored = storedId && this.lists.some((list) => list.id === storedId)
      const nextActive = hasStored ? storedId : this.lists[0]?.id || ''

      if (nextActive && nextActive !== this.activeListId) {
        this.activeListId = nextActive
      }
    },
    async ensureDefaultList() {
      const session = useSessionStore()
      if (!session.uid) return

      const items = await db.lists.where('uid').equals(session.uid).toArray()
      const visible = items.filter((list) => !list.deleted_at)
      if (visible.length) {
        await this.refresh()
        return
      }

      const list = {
        id: defaultListId(session.uid),
        uid: session.uid,
        name: 'Inbox',
        position: await nextPosition(session.uid),
        deleted_at: null,
        revision: 0,
        created_at: nowIso(),
        updated_at: nowIso(),
      }

      await db.lists.put(list)
      await queueUpsert('list', list)
      this.lists = [list]
      this.activeListId = list.id
      await db.meta.put({ key: activeListKey(session.uid), value: list.id })
    },
    async setActiveList(listId) {
      const session = useSessionStore()
      if (!session.uid) return

      this.activeListId = listId
      await db.meta.put({ key: activeListKey(session.uid), value: listId })
    },
    async addList(name) {
      const session = useSessionStore()
      if (!session.uid) return

      const trimmed = name.trim()
      if (!trimmed) return

      const list = {
        id: crypto.randomUUID(),
        uid: session.uid,
        name: trimmed,
        position: await nextPosition(session.uid),
        deleted_at: null,
        revision: 0,
        created_at: nowIso(),
        updated_at: nowIso(),
      }

      await db.lists.put(list)
      await queueUpsert('list', list)
      this.lists = [list, ...this.lists]
      await this.setActiveList(list.id)
    },
    async updateList(list, updates) {
      const updated = {
        ...list,
        ...updates,
        updated_at: nowIso(),
      }

      await db.lists.put(updated)
      await queueUpsert('list', updated)
      this.lists = this.lists.map((item) => (item.id === list.id ? updated : item))
    },
    async deleteList(list) {
      const session = useSessionStore()
      if (!session.uid) return

      await this.updateList(list, { deleted_at: nowIso() })
      const tasksStore = useTasksStore()
      await tasksStore.deleteTasksByList(list.id)

      this.lists = this.lists.filter((item) => item.id !== list.id)
      if (this.activeListId === list.id) {
        const fallback = this.lists[0]?.id || ''
        if (fallback) {
          await this.setActiveList(fallback)
        } else {
          this.activeListId = ''
          await db.meta.delete(activeListKey(session.uid))
        }
      }
    },
  },
})
