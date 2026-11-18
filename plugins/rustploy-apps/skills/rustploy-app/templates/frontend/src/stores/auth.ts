import { defineStore } from 'pinia'
import { api, tokenStorage } from '../api/http'

export interface User {
  id: string
  email: string
  name: string
  role: 'admin' | 'user'
}

export const useAuthStore = defineStore('auth', {
  state: () => ({ user: null as User | null }),
  getters: {
    isAuthenticated: (state) => state.user !== null,
  },
  actions: {
    async login(email: string, password: string) {
      const response = await api<{ token: string; user: User }>('/auth/login', {
        method: 'POST',
        body: JSON.stringify({ email, password }),
      })
      tokenStorage.set(response.token)
      this.user = response.user
    },
    async loadCurrentUser() {
      if (!tokenStorage.get()) return
      try {
        this.user = await api<User>('/auth/me')
      } catch {
        this.logout()
      }
    },
    logout() {
      tokenStorage.clear()
      this.user = null
    },
  },
})
