<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import draggable from 'vuedraggable'
import { useI18n } from 'vue-i18n'
import { useSessionStore } from './stores/session'
import { useTasksStore } from './stores/tasks'
import { useSyncStore } from './stores/sync'
import { generateUid } from './utils/uid'

const session = useSessionStore()
const tasksStore = useTasksStore()
const syncStore = useSyncStore()
const { t } = useI18n()

const generatedUid = ref('')
const confirmUid = ref('')
const newTask = ref('')
const doneRange = ref('today')
const onboardingStep = ref('generate')
const blurEnabled = ref(true)

const showOnboarding = computed(() => !session.uid)
const canConfirm = computed(() => {
  if (onboardingStep.value === 'confirm') {
    return generatedUid.value && confirmUid.value.trim().toUpperCase() === generatedUid.value
  }
  return confirmUid.value.trim().length > 0
})

const activeTasks = computed(() => tasksStore.activeTasks)
const archivedTasks = computed(() => tasksStore.archivedTasks)
const activeList = computed({
  get: () => activeTasks.value,
  set: async (value) => {
    await tasksStore.reorderActive(value)
    await syncStore.sync(session.uid)
  },
})

const doneTasks = computed(() => {
  const now = new Date()
  return tasksStore.doneTasks.filter((task) => {
    if (!task.done_at) return false
    const doneAt = new Date(task.done_at)
    if (doneRange.value === 'today') {
      return isSameDay(doneAt, now)
    }
    return isSameWeek(doneAt, now)
  })
})


const syncStatus = computed(() => {
  if (syncStore.isSyncing) return t('sync.syncing')
  if (syncStore.lastError) return t('sync.offline')
  return t('sync.synced')
})

let syncTimer

const initForUid = async (uid) => {
  if (!uid) return
  await tasksStore.refresh()
  await syncStore.sync(uid)
  await tasksStore.refresh()
}

const handleGenerate = () => {
  generatedUid.value = generateUid()
  confirmUid.value = ''
  onboardingStep.value = 'generate'
}

const handleCopy = async () => {
  if (!generatedUid.value) return
  await navigator.clipboard?.writeText(generatedUid.value)
}

const handleConfirm = async () => {
  if (!confirmUid.value.trim()) return
  if (onboardingStep.value === 'confirm') {
    if (!canConfirm.value) return
    session.setUid(generatedUid.value)
    return
  }
  session.setUid(confirmUid.value.trim().toUpperCase())
}

const handleContinue = () => {
  if (!generatedUid.value) return
  onboardingStep.value = 'confirm'
}

const handleLogin = () => {
  generatedUid.value = ''
  confirmUid.value = ''
  onboardingStep.value = 'login'
}

const handleAddTask = async () => {
  await tasksStore.addTask(newTask.value)
  newTask.value = ''
  await syncStore.sync(session.uid)
}

const handleDone = async (task) => {
  await tasksStore.markDone(task)
  await syncStore.sync(session.uid)
}

const handleArchive = async (task) => {
  await tasksStore.archiveTask(task)
  await syncStore.sync(session.uid)
}

const handleRestore = async (task) => {
  await tasksStore.markActive(task)
  await syncStore.sync(session.uid)
}

const handleSync = () => syncStore.sync(session.uid)

onMounted(async () => {
  session.loadUid()
  await initForUid(session.uid)

  syncTimer = window.setInterval(handleSync, 30000)
  window.addEventListener('online', handleSync)
  window.addEventListener('focus', handleSync)
})

onBeforeUnmount(() => {
  window.clearInterval(syncTimer)
  window.removeEventListener('online', handleSync)
  window.removeEventListener('focus', handleSync)
})

watch(
  () => session.uid,
  async (uid) => {
    await initForUid(uid)
  }
)

function isSameDay(a, b) {
  return (
    a.getFullYear() === b.getFullYear() &&
    a.getMonth() === b.getMonth() &&
    a.getDate() === b.getDate()
  )
}

function startOfWeek(date) {
  const day = date.getDay()
  const diff = (day === 0 ? -6 : 1) - day
  const start = new Date(date)
  start.setDate(date.getDate() + diff)
  start.setHours(0, 0, 0, 0)
  return start
}

function isSameWeek(a, b) {
  const start = startOfWeek(b)
  const end = new Date(start)
  end.setDate(start.getDate() + 7)
  return a >= start && a < end
}
</script>

<template>
  <div class="h-screen overflow-hidden">
    <div class="starfield" aria-hidden="true">
      <span class="star-sparkle" style="--x: 6vw; --y: 12vh; --delay: -0.6s; --duration: 2.0s; --size: 1.7px"></span>
      <span class="star-sparkle" style="--x: 14vw; --y: 42vh; --delay: -1.2s; --duration: 2.2s; --size: 1.5px"></span>
      <span class="star-sparkle" style="--x: 24vw; --y: 8vh; --delay: -0.9s; --duration: 2.1s; --size: 1.9px"></span>
      <span class="star-sparkle" style="--x: 36vw; --y: 22vh; --delay: -1.4s; --duration: 2.3s; --size: 1.4px"></span>
      <span class="star-sparkle" style="--x: 52vw; --y: 6vh; --delay: -0.7s; --duration: 1.9s; --size: 1.6px"></span>
      <span class="star-sparkle" style="--x: 68vw; --y: 18vh; --delay: -1.1s; --duration: 2.1s; --size: 1.7px"></span>
      <span class="star-sparkle" style="--x: 82vw; --y: 10vh; --delay: -0.8s; --duration: 2.0s; --size: 1.8px"></span>
      <span class="star-sparkle" style="--x: 92vw; --y: 30vh; --delay: -1.3s; --duration: 2.2s; --size: 1.5px"></span>
      <span class="star-sparkle" style="--x: 8vw; --y: 70vh; --delay: -1.6s; --duration: 2.4s; --size: 1.4px"></span>
      <span class="star-sparkle" style="--x: 20vw; --y: 82vh; --delay: -1.0s; --duration: 2.1s; --size: 1.6px"></span>
      <span class="star-sparkle" style="--x: 38vw; --y: 78vh; --delay: -1.5s; --duration: 2.3s; --size: 1.4px"></span>
      <span class="star-sparkle" style="--x: 58vw; --y: 80vh; --delay: -0.9s; --duration: 2.0s; --size: 1.5px"></span>
      <span class="star-sparkle" style="--x: 74vw; --y: 76vh; --delay: -1.2s; --duration: 2.1s; --size: 1.6px"></span>
      <span class="star-sparkle" style="--x: 90vw; --y: 72vh; --delay: -1.8s; --duration: 2.4s; --size: 1.4px"></span>
      <span class="star-sparkle" style="--x: -6vw; --y: 26vh; --delay: -0.5s; --duration: 1.9s; --size: 1.7px"></span>
      <span class="star-sparkle" style="--x: 106vw; --y: 28vh; --delay: -1.7s; --duration: 2.2s; --size: 1.5px"></span>
      <span class="star-sparkle" style="--x: -4vw; --y: 64vh; --delay: -1.4s; --duration: 2.3s; --size: 1.4px"></span>
      <span class="star-sparkle" style="--x: 108vw; --y: 62vh; --delay: -1.9s; --duration: 2.5s; --size: 1.4px"></span>
      <span class="star-sparkle" style="--x: 12vw; --y: 54vh; --delay: -1.1s; --duration: 2.1s; --size: 1.5px"></span>
      <span class="star-sparkle" style="--x: 28vw; --y: 60vh; --delay: -1.6s; --duration: 2.2s; --size: 1.4px"></span>
      <span class="star-sparkle" style="--x: 44vw; --y: 64vh; --delay: -0.9s; --duration: 2.0s; --size: 1.6px"></span>
      <span class="star-sparkle" style="--x: 60vw; --y: 66vh; --delay: -1.3s; --duration: 2.2s; --size: 1.5px"></span>
      <span class="star-sparkle" style="--x: 76vw; --y: 60vh; --delay: -1.7s; --duration: 2.3s; --size: 1.4px"></span>
      <span class="star-sparkle" style="--x: 94vw; --y: 56vh; --delay: -1.0s; --duration: 2.1s; --size: 1.5px"></span>
      <span class="star-sparkle core-sparkle" style="--x: 46vw; --y: 72vh; --delay: -0.7s; --duration: 1.8s; --size: 2.4px"></span>
      <span class="star-sparkle core-sparkle" style="--x: 52vw; --y: 78vh; --delay: -1.1s; --duration: 1.9s; --size: 2.2px"></span>
      <span class="star-sparkle core-sparkle" style="--x: 48vw; --y: 84vh; --delay: -0.9s; --duration: 1.7s; --size: 2.6px"></span>
      <span class="star-sparkle core-sparkle" style="--x: 54vw; --y: 88vh; --delay: -1.3s; --duration: 2.0s; --size: 2.3px"></span>
    </div>
    <main class="mx-auto flex h-full max-w-md flex-col gap-4 px-5 py-6 relative z-10">
      <header class="flex items-start justify-between">
        <div>
          <p class="text-xs uppercase tracking-[0.2em] text-[var(--muted)]">{{ t('app.name') }}</p>
        </div>
        <div class="text-right text-xs text-[var(--muted)]">
          <p v-if="session.uid">{{ t('auth.userId') }}</p>
          <p v-if="session.uid" class="font-medium text-[var(--text)]">{{ session.uid }}</p>
          <p v-if="syncStore.lastError" class="mt-1 text-[var(--accent-warm)]">{{ t('sync.offline') }}</p>
        </div>
      </header>

      <section
        v-if="showOnboarding"
        class="rounded-3xl border border-[var(--panel-border)] bg-[var(--panel)]/80 p-5"
      >
        <p class="text-sm text-[var(--muted)]">{{ t('auth.generateCopyHint') }}</p>
        <div class="mt-4 flex flex-col gap-3">
          <template v-if="onboardingStep === 'generate'">
            <button
              v-if="!generatedUid"
              class="rounded-full bg-[var(--accent)] px-4 py-2 text-sm font-semibold text-black"
              type="button"
              @click="handleGenerate"
            >
              {{ t('auth.generate') }}
            </button>

            <div v-if="generatedUid" class="rounded-2xl border border-[var(--panel-border)] p-4">
              <p class="text-sm text-[var(--muted)]">{{ t('auth.yourUserId') }}</p>
              <div class="mt-2 flex items-center justify-between gap-3">
                <p class="text-lg font-semibold tracking-[0.2em]">{{ generatedUid }}</p>
                <button
                  class="inline-flex h-9 w-9 items-center justify-center rounded-full border border-[var(--panel-border)] text-sm text-[var(--muted)]"
                  type="button"
                  :aria-label="t('auth.copyUserId')"
                  :title="t('auth.copyUserId')"
                  @click="handleCopy"
                >
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.6"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    class="h-4 w-4"
                  >
                    <rect x="9" y="9" width="10" height="10" rx="2" />
                    <path d="M5 15V5a2 2 0 0 1 2-2h10" />
                  </svg>
                </button>
              </div>
            </div>

            <button
              v-if="generatedUid"
              class="rounded-full bg-[var(--accent)] px-4 py-2 text-sm font-semibold text-black"
              type="button"
              @click="handleContinue"
            >
              {{ t('auth.continue') }}
            </button>

            <button
              class="rounded-full border border-[var(--panel-border)] px-4 py-2 text-sm font-semibold text-[var(--muted)] transition hover:border-white/30 hover:text-white"
              type="button"
              @click="handleLogin"
            >
              {{ t('auth.signIn') }}
            </button>
          </template>

          <template v-else>
            <p class="text-sm text-[var(--muted)]">{{ t('auth.signInHint') }}</p>
            <div class="flex flex-col gap-2">
              <label class="text-xs uppercase tracking-[0.2em] text-[var(--muted)]">
                {{ t('auth.userId') }}
              </label>
              <input
                v-model="confirmUid"
                class="w-full rounded-2xl border border-[var(--panel-border)] bg-transparent px-4 py-3 text-sm"
                :placeholder="t('auth.userIdPlaceholder')"
                type="text"
              />
            </div>
            <button
              class="rounded-full px-4 py-2 text-sm font-semibold"
              :class="
                canConfirm
                  ? 'bg-[var(--accent)] text-black'
                  : 'border border-[var(--panel-border)] text-[var(--muted)]'
              "
              type="button"
              @click="handleConfirm"
            >
              {{ t('auth.signIn') }}
            </button>
          </template>
        </div>
      </section>

      <section v-else class="flex min-h-0 flex-1 flex-col gap-4">
        <div
          class="rounded-3xl border border-[var(--panel-border)] bg-[var(--panel)]/80 p-5"
        >
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
              <p class="text-sm font-semibold">{{ t('done.title') }}</p>
              <span
                class="rounded-full border border-[var(--panel-border)] bg-white/5 px-2.5 py-0.5 text-xs text-[var(--muted)]"
              >
                {{ t('done.badge', { count: doneTasks.length }) }}
              </span>
            </div>
            <div class="flex gap-2 text-xs">
              <button
                class="rounded-full border border-[var(--panel-border)] px-3 py-1"
                :class="doneRange === 'today' ? 'bg-white/10 text-white' : 'text-[var(--muted)]'"
                type="button"
                @click="doneRange = 'today'"
              >
                {{ t('done.today') }}
              </button>
              <button
                class="rounded-full border border-[var(--panel-border)] px-3 py-1"
                :class="doneRange === 'week' ? 'bg-white/10 text-white' : 'text-[var(--muted)]'"
                type="button"
                @click="doneRange = 'week'"
              >
                {{ t('done.weekly') }}
              </button>
            </div>
          </div>
          <div class="mt-3 flex flex-col gap-1.5">
            <div
              v-for="task in doneTasks"
              :key="task.id"
              class="group flex items-center justify-between rounded-xl border border-dashed border-[var(--panel-border)] px-3 py-1 text-xs"
            >
              <span class="done-title">{{ task.title }}</span>
              <button
                class="h-7 rounded-full border border-[var(--panel-border)] px-2.5 text-[11px] font-medium text-[var(--muted)] opacity-100 transition hover:border-white/30 hover:bg-white/10 hover:text-white md:pointer-events-none md:opacity-0 md:group-hover:pointer-events-auto md:group-hover:opacity-100"
                type="button"
                @click="handleRestore(task)"
              >
                {{ t('done.undo') }}
              </button>
            </div>
            <p v-if="!doneTasks.length" class="text-xs text-[var(--muted)]">
              {{ t('done.empty') }}
            </p>
          </div>
        </div>

        <form
          class="rounded-3xl border border-[var(--panel-border)] bg-[var(--panel)]/80 p-4"
          @submit.prevent="handleAddTask"
        >
          <div class="flex gap-2">
            <input
              v-model="newTask"
              class="flex-1 rounded-2xl border border-[var(--panel-border)] bg-transparent px-4 py-3 text-sm"
              :placeholder="t('tasks.placeholder')"
              type="text"
            />
            <button
              class="rounded-2xl bg-[var(--accent)] px-4 py-3 text-sm font-semibold text-black"
              type="submit"
            >
              {{ t('tasks.add') }}
            </button>
          </div>
        </form>

        <div class="well-shell flex min-h-0 flex-1">
          <section
            class="gravity-well flex min-h-0 flex-1 flex-col rounded-[2.5rem] border border-[var(--panel-border)] bg-[var(--panel)]/60 p-5"
          >
            <div class="flex items-center justify-between">
              <p class="text-sm font-semibold">{{ t('gravity.title') }}</p>
              <div class="flex items-center gap-2 text-xs text-[var(--muted)]">
                <p>{{ t('gravity.hint') }}</p>
                <button
                  class="rounded-full border border-[var(--panel-border)] p-2 text-[var(--muted)] transition hover:border-white/30 hover:bg-white/10 hover:text-white"
                  type="button"
                  aria-label="Toggle blur"
                  title="Toggle blur"
                  @click="blurEnabled = !blurEnabled"
                >
                  <svg
                    v-if="blurEnabled"
                    class="h-3.5 w-3.5"
                    viewBox="0 0 20 20"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.6"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    aria-hidden="true"
                  >
                    <path d="M2 10s3.2-5 8-5 8 5 8 5-3.2 5-8 5-8-5-8-5z" />
                    <circle cx="10" cy="10" r="2.5" />
                  </svg>
                  <svg
                    v-else
                    class="h-3.5 w-3.5"
                    viewBox="0 0 20 20"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.6"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    aria-hidden="true"
                  >
                    <path d="M2 10s3.2-5 8-5 8 5 8 5-3.2 5-8 5-8-5-8-5z" />
                    <path d="M4 4l12 12" />
                  </svg>
                </button>
              </div>
            </div>
            <div class="gravity-scrollbar mt-4 flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto pr-1">
              <draggable
                v-model="activeList"
                item-key="id"
                handle=".drag-handle"
                :animation="200"
                class="flex flex-col gap-3"
              >
                <template #item="{ element, index }">
                  <div
                    class="flex items-center justify-between gap-3 rounded-2xl border border-[var(--panel-border)] bg-black/20 px-3 py-2 text-sm"
                    :style="{
                      opacity: 1 - (index / Math.max(1, activeList.length)) * 0.6,
                      filter: blurEnabled
                        ? index > 2
                          ? `blur(${index * 0.2}px)`
                          : 'none'
                        : 'none',
                    }"
                  >
                    <div class="flex flex-1 items-center gap-3">
                      <span class="drag-handle cursor-grab text-[var(--muted)]">⋮⋮</span>
                      <span>{{ element.title }}</span>
                    </div>
                    <div class="flex gap-2">
                      <button
                        class="pointer-events-auto relative z-10 min-h-8 rounded-full border border-[var(--panel-border)] px-2.5 py-1.5 text-xs font-medium text-[var(--muted)] transition hover:border-white/30 hover:bg-white/10 hover:text-white"
                        type="button"
                        @click="handleDone(element)"
                      >
                        {{ t('tasks.done') }}
                      </button>
                      <button
                        class="pointer-events-auto relative z-10 flex h-8 w-8 items-center justify-center rounded-full border border-[var(--panel-border)] text-[var(--muted)] transition hover:border-white/30 hover:bg-white/10 hover:text-white"
                        type="button"
                        :aria-label="t('tasks.archive')"
                        :title="t('tasks.archive')"
                        @click="handleArchive(element)"
                      >
                        <svg
                          class="h-4 w-4"
                          viewBox="0 0 20 20"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="1.6"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          aria-hidden="true"
                        >
                          <path d="M4 6h12" />
                          <path d="M7 6V4h6v2" />
                          <path d="M6.5 6l.6 10h5.8l.6-10" />
                          <path d="M9 9.5v4" />
                          <path d="M11 9.5v4" />
                        </svg>
                      </button>
                    </div>
                  </div>
                </template>
              </draggable>
              <p v-if="!activeTasks.length" class="text-xs text-[var(--muted)]">
                {{ t('tasks.empty') }}
              </p>
            </div>
          </section>
        </div>

        <details
          class="rounded-3xl border border-[var(--panel-border)] bg-[var(--panel)]/40 px-5 py-4"
        >
          <summary class="cursor-pointer text-sm font-semibold">{{ t('archive.title') }}</summary>
          <div class="mt-4 flex flex-col gap-2">
            <div
              v-for="task in archivedTasks"
              :key="task.id"
              class="flex items-center justify-between rounded-2xl border border-[var(--panel-border)] px-3 py-1.5 text-sm text-[var(--muted)]"
            >
              <span>{{ task.title }}</span>
              <button
                class="min-h-8 rounded-full border border-[var(--panel-border)] px-2.5 py-1.5 text-xs font-medium transition hover:border-white/30 hover:bg-white/10 hover:text-white"
                type="button"
                @click="handleRestore(task)"
              >
                {{ t('archive.restore') }}
              </button>
            </div>
            <p v-if="!archivedTasks.length" class="text-xs text-[var(--muted)]">
              {{ t('archive.empty') }}
            </p>
          </div>
        </details>
      </section>
    </main>
  </div>
</template>
