import Dexie from 'dexie'

export const db = new Dexie('gravity_tasks')

db.version(1).stores({
  tasks: 'id, uid, status, updated_at, deleted_at, position',
  outbox: '++id, uid, op, task_id, created_at',
  meta: 'key',
})
