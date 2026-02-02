import Dexie from 'dexie'

export const db = new Dexie('gravity_tasks')

db.version(1).stores({
  tasks: 'id, uid, status, updated_at, deleted_at, position',
  outbox: '++id, uid, op, task_id, created_at',
  meta: 'key',
})

db.version(2)
  .stores({
    tasks: 'id, uid, list_id, status, updated_at, deleted_at, position',
    lists: 'id, uid, updated_at, deleted_at, position',
    outbox: '++id, uid, entity, op, task_id, created_at',
    meta: 'key',
  })
  .upgrade(async (tx) => {
    await tx.table('outbox').toCollection().modify((item) => {
      if (!item.entity) {
        item.entity = 'task'
      }
    })

    const tasks = await tx.table('tasks').toArray()
    const listsTable = tx.table('lists')
    const now = new Date().toISOString()
    const defaultLists = new Map()

    tasks.forEach((task) => {
      if (task.list_id) return
      const uid = task.uid
      if (!uid) return
      const listId = `inbox-${uid}`

      if (!defaultLists.has(uid)) {
        defaultLists.set(uid, {
          id: listId,
          uid,
          name: 'Inbox',
          position: 1000,
          revision: 0,
          deleted_at: null,
          created_at: now,
          updated_at: now,
        })
      }

      task.list_id = listId
    })

    if (defaultLists.size) {
      await listsTable.bulkPut(Array.from(defaultLists.values()))
      await tx.table('tasks').bulkPut(tasks)
    }
  })
