import { defineStore } from 'pinia'
import { db } from '../db'
import { queueUpsert } from '../services/sync'
import { useSessionStore } from './session'

const nowIso = () => new Date().toISOString()

async function nextPosition(uid, listId) {
  const items = await db.tasks.where('uid').equals(uid).toArray()
  const listItems = items.filter((item) => item.list_id === listId)
  if (!listItems.length) return 1000
  const last = listItems.sort((a, b) => a.position - b.position).at(-1)
  return last?.position ? last.position + 1000 : 1000
}

const isVisible = (task) => !task.deleted_at

export const useTasksStore = defineStore('tasks', {
  state: () => ({
    tasks: [],
  }),
  getters: {
    activeTasks: (state) => (listId) =>
      state.tasks.filter((t) => isVisible(t) && t.status === 'active' && t.list_id === listId),
    doneTasks: (state) => (listId) =>
      state.tasks.filter((t) => isVisible(t) && t.status === 'done' && t.list_id === listId),
    archivedTasks: (state) => (listId) =>
      state.tasks.filter((t) => isVisible(t) && t.status === 'archived' && t.list_id === listId),
  },
  actions: {
    async refresh() {
      const session = useSessionStore()
      if (!session.uid) return

      const items = await db.tasks.where('uid').equals(session.uid).toArray()
      this.tasks = items.sort((a, b) => b.position - a.position)
    },
    async addTask(title, listId) {
      const session = useSessionStore()
      if (!session.uid) return

      if (!listId) return

      const trimmed = title.trim()
      if (!trimmed) return

      const task = {
        id: crypto.randomUUID(),
        uid: session.uid,
        list_id: listId,
        title: trimmed,
        status: 'active',
        position: await nextPosition(session.uid, listId),
        done_at: null,
        archived_at: null,
        deleted_at: null,
        revision: 0,
        created_at: nowIso(),
        updated_at: nowIso(),
      }

      await db.tasks.put(task)
      await queueUpsert('task', task)
      this.tasks = [task, ...this.tasks]
    },
    async updateTask(task, updates) {
      const updated = {
        ...task,
        ...updates,
        updated_at: nowIso(),
      }

      await db.tasks.put(updated)
      await queueUpsert('task', updated)
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
      await Promise.all(updated.map((task) => queueUpsert('task', task)))
      const updatedMap = new Map(updated.map((task) => [task.id, task]))
      this.tasks = this.tasks.map((task) => updatedMap.get(task.id) || task)
    },
    async deleteTasksByList(listId) {
      const session = useSessionStore()
      if (!session.uid) return

      const now = nowIso()
      const items = this.tasks.filter((task) => task.list_id === listId && !task.deleted_at)
      if (!items.length) return

      const updated = items.map((task) => ({
        ...task,
        deleted_at: now,
        updated_at: now,
      }))

      await db.tasks.bulkPut(updated)
      await Promise.all(updated.map((task) => queueUpsert('task', task)))
      const updatedMap = new Map(updated.map((task) => [task.id, task]))
      this.tasks = this.tasks.map((task) => updatedMap.get(task.id) || task)
    },
  },
})
