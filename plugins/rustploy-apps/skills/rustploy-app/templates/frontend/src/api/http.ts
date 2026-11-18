// Cliente HTTP único. Sempre caminho RELATIVO (/api/...): mesma origem, sem CORS,
// sem URL de servidor no código. Nunca coloque segredo aqui: tudo no frontend é público.
const TOKEN_KEY = 'auth_token'

export class ApiError extends Error {
  status: number
  constructor(status: number, message: string) {
    super(message)
    this.status = status
  }
}

export const tokenStorage = {
  get: () => localStorage.getItem(TOKEN_KEY),
  set: (token: string) => localStorage.setItem(TOKEN_KEY, token),
  clear: () => localStorage.removeItem(TOKEN_KEY),
}

export async function api<T>(path: string, options: RequestInit = {}): Promise<T> {
  const headers = new Headers(options.headers)
  if (options.body && !headers.has('Content-Type')) headers.set('Content-Type', 'application/json')
  const token = tokenStorage.get()
  if (token) headers.set('Authorization', `Bearer ${token}`)

  const response = await fetch(`/api${path}`, { ...options, headers })
  if (response.status === 401) {
    tokenStorage.clear()
    if (!location.pathname.startsWith('/login')) location.assign('/login')
  }
  if (!response.ok) {
    const body = await response.json().catch(() => ({}))
    throw new ApiError(response.status, body.error ?? 'Erro inesperado')
  }
  return response.status === 204 ? (undefined as T) : response.json()
}
