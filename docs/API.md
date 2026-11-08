# GravityTasks API (REST)

Base URL: `/api` (same origin as the app; `http://localhost:3000/api` in dev)
Header: `X-UID: <user-id>` (or a `uid` query/body param)

Sync model: the client pushes its outbox via the batch endpoints, then pulls deltas with `since`.

## GET /lists, GET /tasks
Query params:
- `since` (optional, ISO 8601): only records updated or deleted after this timestamp

Response: `200 OK`
```json
[
  {
    "id": "uuid",
    "uid": "ABCD-EFGH-IJKL",
    "title": "Task title",
    "status": "active",
    "position": 3000.0,
    "done_at": null,
    "archived_at": null,
    "deleted_at": null,
    "revision": 1,
    "created_at": "2026-02-01T16:00:00.000Z",
    "updated_at": "2026-02-01T16:10:00.000Z",
    "list_id": "inbox-ABCD-EFGH-IJKL",
    "description": "Multiline\nnotes"
  }
]
```
Lists have `id, uid, name, position, deleted_at, revision, created_at, updated_at`.

## POST /lists/batch, POST /tasks/batch
Upserts by `id`. Body: `{ "lists": [...] }` / `{ "tasks": [...] }`.

Writable fields:
- lists: `name`, `position`, `deleted_at` (setting it soft-deletes the list's tasks too)
- tasks: `title`, `description`, `status` (`active|done|archived`), `position`, `done_at`, `archived_at`,
  `deleted_at`, `list_id` (blank or missing → the user's `inbox-<uid>` list)

Other fields are ignored. `revision` increments on every update; timestamps are set by the server.

```json
{
  "tasks": [
    { "id": "uuid", "title": "Task title", "status": "active", "position": 2000, "list_id": "inbox-ABCD-EFGH-IJKL" }
  ]
}
```

Response: `200 OK` with the saved records. Items that fail validation or belong to another user are
skipped (and logged server-side), so one bad item never blocks the rest of a sync.

Errors: `401` without a uid, `400` for malformed JSON or a missing/empty array.
