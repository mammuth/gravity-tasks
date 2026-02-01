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
- [x] User ID format: Crockford Base32
- [x] Sync: REST-only
- [x] Scaffold Rails API app
- [x] Add base docker-compose.yml
- [x] Task model + migrations
- [x] Tasks REST endpoints scaffolded
- [x] Scaffold Vue app + Tailwind + Pinia + Dexie
- [x] Dexie database + sync service scaffold
- [x] Pinia stores scaffolded (session/tasks/sync)
- [x] App shell + onboarding + gravity list placeholders
- [x] Dev CORS enabled for frontend
- [x] Rack CORS added to backend
- [x] Docker-compose services point to app dirs
- [x] Vite API URL wired in docker-compose
- [x] Outbox push/pull sync loop wired
- [x] Done/Archive + Today/Weekly toggle
- [x] Gravity UI + drag-and-drop
- [x] Document REST endpoints + payloads
- [x] Docker-compose dev workflow verified
- [x] Fix Dexie ordering query for nextPosition
- [x] Fix batch upsert params handling
- [x] Improve action button hit areas
- [x] Ensure gravity overlay doesn't block clicks
- [x] Fix batch upsert id handling
- [x] Update done action label to Undo
- [x] Frontend copy moved to i18n
- [x] Switch to vue-i18n integration
- [x] Fix batch upsert unique id conflict handling
- [x] Change tasks primary key to string ids
- [x] Move frontend locales to JSON
- [x] Constrain layout to full-height with scrollable gravity well
- [x] Style app scrollbars
- [x] Trim header and hide sync status unless error
- [x] Restyle scrollbar with gravity accents
- [x] Scope gravity scrollbar styling to list
- [x] Enhance gravity well animation and scrollbar buttons
- [x] Add starfield sparkles animation
- [x] Refresh done section layout and achievement copy
- [x] Replace sparkles with gravity well drift
- [x] Tighten done section header and remove subline
- [x] Darken gravity well center and boost sparkle visibility
- [x] Intensify gravity well palette with purple glow
- [x] Move sparkles above content and enlarge
- [x] Speed up sparkle drift and remove pop-in
- [x] Subdue sparkles and stagger visibility
- [x] Add well shell container for outer sparkles
- [x] Move sparkles to full-screen starfield
- [x] Boost bottom-center sparkle visibility and speed
- [x] Retarget starfield drift to bottom-center
- [x] Increase starfield density and visibility
- [x] Reduce black hole pulse and densify task rows
- [x] Add done strike-through animation and hover-only undo
- [x] Fill .gitignore for Rails + Vue outputs
- [x] Add favicon + meta title
- [x] Densify done item row height
- [x] Add blur toggle button in gravity header
- [x] Fix blur toggle logic to re-enable blur
- [x] Swap blur icon to eye and archive button to trash icon
