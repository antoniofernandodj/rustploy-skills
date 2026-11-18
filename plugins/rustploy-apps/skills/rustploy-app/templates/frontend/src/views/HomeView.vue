<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const auth = useAuthStore()
const router = useRouter()

function logout() {
  auth.logout()
  router.push('/login')
}
</script>

<template>
  <main style="padding: 24px; font-family: system-ui, sans-serif">
    <h1>Olá, {{ auth.user?.name }}</h1>
    <button @click="logout">Sair</button>
  </main>
</template>
