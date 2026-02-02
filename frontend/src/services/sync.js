import { db } from '../db'
import { apiFetch } from './api'

const lastSyncKey = (uid) => `last_sync_at:${uid}`

export async function queueUpsert(entity, item) {
  await db.outbox.add({
    uid: item.uid,
    entity,
    op: 'upsert',
    task_id: item.id,
    payload: item,
    created_at: new Date().toISOString(),
    retry_count: 0,
  })
}

export async function queueDelete(entity, item) {
  await db.outbox.add({
    uid: item.uid,
    entity,
    op: 'delete',
    task_id: item.id,
    payload: item,
    created_at: new Date().toISOString(),
    retry_count: 0,
  })
}

export async function syncNow(uid) {
  if (!uid) return

  const outboxItems = await db.outbox.where('uid').equals(uid).toArray()
  if (outboxItems.length) {
    const listItems = outboxItems.filter((item) => item.entity === 'list').map((item) => item.payload)
    const taskItems = outboxItems.filter((item) => item.entity === 'task').map((item) => item.payload)

    if (listItems.length) {
      await apiFetch('/lists/batch', { method: 'POST', uid, body: { lists: listItems } })
    }
    if (taskItems.length) {
      await apiFetch('/tasks/batch', { method: 'POST', uid, body: { tasks: taskItems } })
    }

    await db.outbox.bulkDelete(outboxItems.map((item) => item.id))
  }

  const lastSyncRecord = await db.meta.get(lastSyncKey(uid))
  const since = lastSyncRecord?.value
  const query = since ? `?since=${encodeURIComponent(since)}` : ''

  const [remoteLists, remoteTasks] = await Promise.all([
    apiFetch(`/lists${query}`, { uid }),
    apiFetch(`/tasks${query}`, { uid }),
  ])

  if (Array.isArray(remoteLists) && remoteLists.length) {
    await db.lists.bulkPut(remoteLists)
  }
  if (Array.isArray(remoteTasks) && remoteTasks.length) {
    await db.tasks.bulkPut(remoteTasks)
  }

  await db.meta.put({ key: lastSyncKey(uid), value: new Date().toISOString() })
}
