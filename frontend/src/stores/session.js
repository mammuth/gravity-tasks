import { defineStore } from 'pinia'

const UID_KEY = 'gravity_uid'

export const useSessionStore = defineStore('session', {
  state: () => ({
    uid: '',
  }),
  actions: {
    loadUid() {
      this.uid = localStorage.getItem(UID_KEY) || ''
    },
    setUid(uid) {
      this.uid = uid
      localStorage.setItem(UID_KEY, uid)
    },
    clearUid() {
      this.uid = ''
      localStorage.removeItem(UID_KEY)
    },
  },
})
