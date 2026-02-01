import { db } from '../db'
import { apiFetch } from './api'

const lastSyncKey = (uid) => `last_sync_at:${uid}`

export async function queueUpsert(task) {
  await db.outbox.add({
    uid: task.uid,
    op: 'upsert',
    task_id: task.id,
    payload: task,
    created_at: new Date().toISOString(),
    retry_count: 0,
  })
}

export async function queueDelete(task) {
  await db.outbox.add({
    uid: task.uid,
    op: 'delete',
    task_id: task.id,
    payload: task,
    created_at: new Date().toISOString(),
    retry_count: 0,
  })
}

export async function syncNow(uid) {
  if (!uid) return

  const outboxItems = await db.outbox.where('uid').equals(uid).toArray()
  if (outboxItems.length) {
    const tasks = outboxItems.map((item) => item.payload)
    await apiFetch('/tasks/batch', { method: 'POST', uid, body: { tasks } })
    await db.outbox.bulkDelete(outboxItems.map((item) => item.id))
  }

  const lastSyncRecord = await db.meta.get(lastSyncKey(uid))
  const since = lastSyncRecord?.value
  const query = since ? `?since=${encodeURIComponent(since)}` : ''

  const remoteTasks = await apiFetch(`/tasks${query}`, { uid })
  if (Array.isArray(remoteTasks) && remoteTasks.length) {
    await db.tasks.bulkPut(remoteTasks)
  }

  await db.meta.put({ key: lastSyncKey(uid), value: new Date().toISOString() })
}
