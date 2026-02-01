# GravityTasks

GravityTasks is a local-first todo app where priorities orbit a gravity well.
Top = highest priority, lower items fade and blur toward the bottom. 

Disclaimer: This is 100% vibe coded, I didn't review a single line of code. :bomb:

## Concept
- Local-first PWA: tasks live in IndexedDB and sync to a Rails API.
- Gravity metaphor: reorder by drag, items dissolve toward the well.
- Offline-ready: Dexie outbox + batch sync.
- Minimal onboarding: User ID only.

## Stack
- Frontend: Vue 3 + Vite + Tailwind + Pinia + Dexie
- Backend: Rails API + SQLite
- Dev: Docker Compose

## Development (Docker)
```bash
docker compose up
```

Frontend: http://localhost:5173
Backend API: http://localhost:3000/api

Override API URL via `.env`:
```
VITE_API_URL=http://localhost:3000/api
```

## Production (Docker Compose)
Requires a Rails master key and a persistent volume for the SQLite DB.

```bash
cp .env.example .env
# set RAILS_MASTER_KEY and VITE_API_URL
docker compose -f docker-compose.prod.yml up --build
```

Notes:
- Backend runs on port 3000 (container port 80).
- Frontend is served via `vite preview` on port 5173.
- DB persists in the `backend_storage` volume. It survives container removal
  but is deleted by `docker compose down -v`.

