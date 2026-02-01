# GravityTasks API (REST)

Base URL: `http://localhost:3000/api`
Header: `X-UID: <user-id>`

## GET /tasks
Query params:
- `since` (optional, ISO 8601): return tasks updated or deleted after this timestamp

Response: `200 OK`
```json
[
  {
    "id": "uuid",
    "uid": "ABCD-EFGH-IJKL",
    "title": "Task title",
    "status": "active",
    "position": 3000,
    "done_at": null,
    "archived_at": null,
    "deleted_at": null,
    "revision": 1,
    "created_at": "2026-02-01T16:00:00Z",
    "updated_at": "2026-02-01T16:10:00Z"
  }
]
```

## POST /tasks
Body:
```json
{
  "task": {
    "title": "New task",
    "status": "active",
    "position": 1000
  }
}
```

Response: `201 Created`

## PATCH /tasks/:id
Body:
```json
{
  "task": {
    "title": "Updated title",
    "status": "done",
    "done_at": "2026-02-01T16:30:00Z"
  }
}
```

Response: `200 OK`

## POST /tasks/batch
Body:
```json
{
  "tasks": [
    {
      "id": "uuid",
      "title": "Task title",
      "status": "active",
      "position": 2000,
      "done_at": null,
      "archived_at": null,
      "deleted_at": null
    }
  ]
}
```

Response: `200 OK`
