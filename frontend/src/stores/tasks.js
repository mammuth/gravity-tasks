import { defineStore } from 'pinia'
import { db } from '../db'
import { queueUpsert } from '../services/sync'
import { useSessionStore } from './session'

const nowIso = () => new Date().toISOString()

async function nextPosition(uid) {
  const items = await db.tasks.where('uid').equals(uid).toArray()
  if (!items.length) return 1000
  const last = items.sort((a, b) => a.position - b.position).at(-1)
  return last?.position ? last.position + 1000 : 1000
}

export const useTasksStore = defineStore('tasks', {
  state: () => ({
    tasks: [],
  }),
  getters: {
    activeTasks: (state) => state.tasks.filter((t) => t.status === 'active'),
    doneTasks: (state) => state.tasks.filter((t) => t.status === 'done'),
    archivedTasks: (state) => state.tasks.filter((t) => t.status === 'archived'),
  },
  actions: {
    async refresh() {
      const session = useSessionStore()
      if (!session.uid) return

      const items = await db.tasks.where('uid').equals(session.uid).toArray()
      this.tasks = items.sort((a, b) => b.position - a.position)
    },
    async addTask(title) {
      const session = useSessionStore()
      if (!session.uid) return

      const trimmed = title.trim()
      if (!trimmed) return

      const task = {
        id: crypto.randomUUID(),
        uid: session.uid,
        title: trimmed,
        status: 'active',
        position: await nextPosition(session.uid),
        done_at: null,
        archived_at: null,
        deleted_at: null,
        revision: 0,
        created_at: nowIso(),
        updated_at: nowIso(),
      }

      await db.tasks.put(task)
      await queueUpsert(task)
      this.tasks = [task, ...this.tasks]
    },
    async updateTask(task, updates) {
      const updated = {
        ...task,
        ...updates,
        updated_at: nowIso(),
      }

      await db.tasks.put(updated)
      await queueUpsert(updated)
      this.tasks = this.tasks.map((item) => (item.id === task.id ? updated : item))
    },
    async markDone(task) {
      await this.updateTask(task, {
        status: 'done',
        done_at: nowIso(),
      })
    },
    async markActive(task) {
      await this.updateTask(task, {
        status: 'active',
        done_at: null,
        archived_at: null,
      })
    },
    async archiveTask(task) {
      await this.updateTask(task, {
        status: 'archived',
        archived_at: nowIso(),
      })
    },
    async deleteTask(task) {
      await this.updateTask(task, {
        deleted_at: nowIso(),
      })
    },
    async reorderActive(ordered) {
      const total = ordered.length
      const updated = ordered.map((task, index) => ({
        ...task,
        position: (total - index) * 1000,
        updated_at: nowIso(),
      }))

      await db.tasks.bulkPut(updated)
      await Promise.all(updated.map((task) => queueUpsert(task)))
      const rest = this.tasks.filter((task) => task.status !== 'active')
      this.tasks = [...updated, ...rest]
    },
  },
})
