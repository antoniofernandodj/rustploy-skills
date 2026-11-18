<script setup lang="ts">
import { ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const auth = useAuthStore()
const router = useRouter()
const route = useRoute()
const email = ref('')
const password = ref('')
const errorMessage = ref('')
const isSubmitting = ref(false)

async function submit() {
  errorMessage.value = ''
  isSubmitting.value = true
  try {
    await auth.login(email.value, password.value)
    await router.push((route.query.redirect as string) || '/')
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : 'Não foi possível entrar'
  } finally {
    isSubmitting.value = false
  }
}
</script>

<template>
  <main class="login">
    <form @submit.prevent="submit">
      <h1>Entrar</h1>
      <label>E-mail <input v-model="email" type="email" autocomplete="username" required /></label>
      <label>Senha <input v-model="password" type="password" autocomplete="current-password" required /></label>
      <p v-if="errorMessage" class="error">{{ errorMessage }}</p>
      <button type="submit" :disabled="isSubmitting">{{ isSubmitting ? 'Entrando…' : 'Entrar' }}</button>
    </form>
  </main>
</template>

<style scoped>
.login { min-height: 100vh; display: grid; place-items: center; font-family: system-ui, sans-serif; }
form { display: grid; gap: 12px; width: min(360px, 90vw); }
input { display: block; width: 100%; padding: 8px; margin-top: 4px; box-sizing: border-box; }
button { padding: 10px; cursor: pointer; }
.error { color: #b00020; margin: 0; }
</style>
