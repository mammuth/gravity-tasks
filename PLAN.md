# GravityTasks Plan

## Vision
Local-first todo app with a gravity metaphor. Top = highest priority; bottom dissolves into the well.

## Stack
- Backend: Rails API, SQLite
- Frontend: Vue 3 + Vite + Tailwind CSS + Pinia + Dexie (IndexedDB)
- Sync: REST only (push outbox + pull since last_sync_at)
- Dev: docker-compose

## UX Flow
1) Onboarding: generate User ID -> copy -> re-enter to confirm
2) Main: Done strip (Today/Weekly toggle) -> input -> gravity list -> archive

## Data Model
Task: id (uuid), uid, title, status (active|done|archived), position (float), done_at, archived_at,
deleted_at, created_at, updated_at, revision

## Sync Rules
- Client local DB is source of truth
- Outbox push; pull deltas by `since`
- Conflict: per-field LWW by `updated_at`
- Soft delete with tombstones

## Ordering
Float position values; insert between neighbors. Rebalance if gaps shrink.

## Visual Theme
Dark palette, gravity well halo, subtle motion, fade/blur near bottom.

## Milestones
M1: Rails API + Task endpoints
M2: Vue app + Tailwind + Dexie schema + offline CRUD
M3: Sync loop + conflict merge
M4: Gravity UI + drag-and-drop
M5: Done/Archive + Today/Weekly toggle

## Still To Do
- [ ] Add /session endpoint (optional)

## Done
- [x] Architecture + UX decisions locked
- [x] Rails API + Task endpoints with SQLite storage
- [x] Vue app + Tailwind + Pinia + Dexie offline CRUD
- [x] Sync loop + conflict handling
- [x] Gravity UI + drag-and-drop ordering
- [x] Done/Archive flow + onboarding User ID
- [x] PWA manifest + service worker
- [x] Docker dev + production deployment files
- [x] Project README with concept + deployment
- [x] Inline task title editing with debounced autosave
- [x] Multi-list support with list CRUD + list-aware tasks
